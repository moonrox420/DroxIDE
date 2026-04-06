// src-rust/vector_store.rs
use crate::rag::RagDocument;
use crate::vector_store_trait::{VectorStore, VectorStoreError};
use crate::hnsw_tuner::{HnswTuner, HnswParams, WorkloadType};
use crate::llama::LlamaPool;
use async_trait::async_trait;
use backoff::future::retry;
use backoff::ExponentialBackoff;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{instrument, info, error};
use uuid::Uuid;
use chrono::Utc;

/// Production-grade ChromaDB Vector Store with HNSW tuning
#[derive(Clone)]
pub struct ChromaVectorStore {
    client: Client,
    base_url: String,
    collection_name: String,
    llm_pool: Arc<LlamaPool>,
    hnsw_tuner: Arc<HnswTuner>,
    batch_size: usize,
}

impl ChromaVectorStore {
    pub fn new(llm_pool: Arc<LlamaPool>) -> Self {
        ChromaVectorStore {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(60))
                .build()
                .unwrap(),
            base_url: "http://127.0.0.1:8000".to_string(),
            collection_name: "droxide_codebase".to_string(),
            llm_pool: llm_pool.clone(),
            hnsw_tuner: Arc::new(HnswTuner::new()),
            batch_size: 128,
        }
    }

    async fn create_optimized_collection(&self) -> Result<(), VectorStoreError> {
        // Auto-tune HNSW parameters based on current corpus
        let params = self.hnsw_tuner.tune_for_corpus(0, 768, 512, WorkloadType::Research)
            .await
            .map_err(|e| VectorStoreError::Operation(e.to_string()))?;

        let payload = serde_json::json!({
            "name": self.collection_name,
            "metadata": self.hnsw_tuner.to_chroma_metadata(&params)
        });

        let resp = self.client
            .post(&format!("{}/api/v1/collections", self.base_url))
            .json(&payload)
            .send()
            .await
            .map_err(|e| VectorStoreError::Connection(e.to_string()))?;

        match resp.status() {
            StatusCode::OK | StatusCode::CREATED | StatusCode::CONFLICT => Ok(()),
            _ => Err(VectorStoreError::Operation(format!("Collection creation failed: {}", resp.status()))),
        }
    }
}

#[async_trait]
impl VectorStore for ChromaVectorStore {
    #[instrument(name = "chroma_ensure_index", skip(self), fields(trace_id = %Uuid::new_v4()))]
    async fn ensure_index(&self) -> Result<(), VectorStoreError> {
        info!("Ensuring ChromaDB collection with optimized HNSW parameters");
        self.create_optimized_collection().await
    }

    #[instrument(name = "chroma_batch_upsert", skip(self, documents), fields(trace_id = %Uuid::new_v4()))]
    async fn batch_upsert(&self, documents: Vec<RagDocument>) -> Result<(), VectorStoreError> {
        if documents.is_empty() {
            return Ok(());
        }

        let total = documents.len();
        let mut failed = 0;

        for chunk in documents.chunks(self.batch_size) {
            let mut ids = Vec::with_capacity(chunk.len());
            let mut embeddings = Vec::with_capacity(chunk.len());
            let mut metadatas = Vec::with_capacity(chunk.len());
            let mut docs_content = Vec::with_capacity(chunk.len());

            for doc in chunk {
                ids.push(doc.id.clone());
                embeddings.push(doc.embedding.clone());
                metadatas.push(serde_json::json!({
                    "path": doc.path,
                    "file_type": doc.metadata.file_type,
                    "git_hash": doc.metadata.git_hash,
                    "lines_start": doc.metadata.lines.0,
                    "lines_end": doc.metadata.lines.1,
                    "size_bytes": doc.metadata.size_bytes,
                    "indexed_at": doc.indexed_at,
                }));
                docs_content.push(Some(doc.content.clone()));
            }

            let payload = serde_json::json!({
                "ids": ids,
                "embeddings": embeddings,
                "metadatas": metadatas,
                "documents": docs_content
            });

            let op = || async {
                let resp = self.client
                    .post(&format!("{}/api/v1/collections/{}/upsert", self.base_url, self.collection_name))
                    .json(&payload)
                    .send()
                    .await
                    .map_err(|e| backoff::Error::transient(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        e.to_string(),
                    )))?;

                if !resp.status().is_success() {
                    return Err(backoff::Error::transient(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("HTTP {}", resp.status()),
                    )));
                }
                Ok(())
            };

            match retry(ExponentialBackoff::default(), op).await {
                Ok(_) => info!("Batch of {} documents upserted", chunk.len()),
                Err(_) => {
                    failed += chunk.len();
                    error!("Batch upsert failed for {} documents", chunk.len());
                }
            }
        }

        if failed > 0 {
            Err(VectorStoreError::PartialUpsert(failed))
        } else {
            info!("Successfully upserted {} documents", total);
            Ok(())
        }
    }

    #[instrument(name = "chroma_query", skip(self), fields(trace_id = %Uuid::new_v4()))]
    async fn query(
        &self,
        query_text: &str,
        limit: usize,
        where_filter: Option<serde_json::Value>,
    ) -> Result<Vec<RagDocument>, VectorStoreError> {
        let embedding = self.llm_pool.embed(query_text).await
            .map_err(|e| VectorStoreError::Operation(e.to_string()))?;

        let payload = serde_json::json!({
            "query_embeddings": [embedding],
            "n_results": limit,
            "where": where_filter,
            "include": ["documents", "metadatas", "distances"]
        });

        let resp = self.client
            .post(&format!("{}/api/v1/collections/{}/query", self.base_url, self.collection_name))
            .json(&payload)
            .send()
            .await
            .map_err(|e| VectorStoreError::Query(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(VectorStoreError::Query(format!("HTTP {}", resp.status())));
        }

        let result: ChromaQueryResult = resp.json().await
            .map_err(|e| VectorStoreError::Query(e.to_string()))?;

        let mut documents = Vec::new();
        for i in 0..result.ids.len() {
            if let (Some(content), Some(meta)) = (result.documents.get(i).and_then(|o| o.clone()), result.metadatas.get(i).and_then(|o| o.clone())) {
                let metadata: crate::rag::RagMetadata = serde_json::from_value(meta.clone())
                    .unwrap_or_else(|_| crate::rag::RagMetadata {
                        file_type: "unknown".to_string(),
                        git_hash: "HEAD".to_string(),
                        lines: (0, 0),
                        size_bytes: content.len(),
                    });

                documents.push(RagDocument {
                    id: result.ids[i].clone(),
                    path: meta["path"].as_str().unwrap_or("").to_string(),
                    chunk_index: 0,
                    content,
                    embedding: vec![],
                    metadata,
                    indexed_at: meta["indexed_at"].as_u64().unwrap_or(0),
                });
            }
        }
        Ok(documents)
    }

    async fn delete_index(&self) -> Result<(), VectorStoreError> {
        let resp = self.client
            .delete(&format!("{}/api/v1/collections/{}", self.base_url, self.collection_name))
            .send()
            .await
            .map_err(|e| VectorStoreError::Operation(e.to_string()))?;

        if resp.status().is_success() || resp.status() == StatusCode::NOT_FOUND {
            Ok(())
        } else {
            Err(VectorStoreError::Operation(format!("Delete failed: {}", resp.status())))
        }
    }

    async fn health_check(&self) -> Result<bool, VectorStoreError> {
        let resp = self.client.get(&format!("{}/api/v1/heartbeat", self.base_url)).send().await
            .map_err(|e| VectorStoreError::Connection(e.to_string()))?;
        Ok(resp.status().is_success())
    }

    async fn update_single(&self, document: RagDocument) -> Result<(), VectorStoreError> {
        self.batch_upsert(vec![document]).await
    }

    async fn delete(&self, id: String) -> Result<(), VectorStoreError> {
        let payload = serde_json::json!({
            "ids": [id]
        });
        
        let resp = self.client
            .post(&format!("{}/api/v1/collections/{}/delete", self.base_url, self.collection_name))
            .json(&payload)
            .send()
            .await
            .map_err(|e| VectorStoreError::Operation(e.to_string()))?;

        if resp.status().is_success() {
            Ok(())
        } else {
            Err(VectorStoreError::Operation(format!("Delete failed: {}", resp.status())))
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ChromaQueryResult {
    ids: Vec<String>,
    embeddings: Option<Vec<Vec<f32>>>,
    documents: Vec<Option<String>>,
    metadatas: Vec<Option<serde_json::Value>>,
    distances: Vec<f32>,
}