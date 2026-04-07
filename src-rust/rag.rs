// src-rust/rag.rs
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use tree_sitter::{Parser, Language, Query, QueryCursor};
use tracing::{instrument, info, error};
use uuid::Uuid;
use chrono::Utc;
use std::sync::Arc;
use thiserror::Error;

use crate::llama::LlamaPool;
use crate::semantic_embedding::{SemanticEmbedder, SemanticEmbedding};
use crate::ast_query_patterns::AstQueryPatterns;

#[derive(Error, Debug)]
pub enum RagError {
    #[error("Tree-sitter parse error: {0}")]
    Parse(String),
    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),
    #[error("Embedding failed: {0}")]
    Embedding(#[from] crate::llama::LlamaError),
    #[error("Semantic embedding failed: {0}")]
    Embedding(#[from] crate::semantic_embedding::EmbeddingError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Query pattern error: {0}")]
    Query(#[from] crate::ast_query_patterns::QueryError),
    #[error("Query compilation error: {0}")]
    QueryCompilation(String),
}

impl From<tree_sitter::QueryError> for RagError {
    fn from(e: tree_sitter::QueryError) -> Self {
        RagError::QueryCompilation(format!("Query compilation failed: {:?}", e))
    }
}
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RagDocument {
    pub id: String,
    pub path: String,
    pub chunk_index: usize,
    pub content: String,
    pub embedding: Vec<f32>,
    pub metadata: RagMetadata,
    pub indexed_at: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RagMetadata {
    pub file_type: String,
    pub git_hash: String,
    pub lines: (usize, usize),
    pub size_bytes: usize,
}

pub struct RagPipeline {
    vector_store: Arc<dyn crate::vector_store_trait::VectorStore>,
    chunker: OptimizedChunker,
}

impl RagPipeline {
    pub fn new(llm_pool: Arc<LlamaPool>) -> Self {
        let vector_store: Arc<dyn crate::vector_store_trait::VectorStore> =
            Arc::new(crate::vector_store::ChromaVectorStore::new(llm_pool.clone()));
        RagPipeline {
            vector_store,
            chunker: OptimizedChunker::new(llm_pool),
        }
    }

    #[instrument(name = "rag_ingest_folder", skip(self), fields(trace_id = %Uuid::new_v4()))]
    pub async fn ingest_folder(&self, path: &Path) -> Result<usize, String> {
        info!("Starting full semantic RAG ingestion for folder: {}", path.display());
        self.vector_store.ensure_index().await.map_err(|e| e.to_string())?;

        let mut count = 0;
        let mut batch = Vec::new();

        for entry in WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file() && self.should_ingest(e.path()))
        {
            let content = std::fs::read_to_string(entry.path())
                .map_err(|e| format!("Failed to read {}: {}", entry.path().display(), e))?;
            let lang = self.detect_language(entry.path());

            let chunks = self.chunker.chunk(&content, &lang).await
                .map_err(|e| format!("Chunking failed for {}: {}", entry.path().display(), e))?;

            for (idx, (chunk_text, embedding)) in chunks.into_iter().enumerate() {
                let doc = RagDocument {
                    id: format!("{}-{}", entry.path().display(), idx),
                    path: entry.path().to_string_lossy().into_owned(),
                    chunk_index: idx,
                    content: chunk_text,
                    embedding: embedding.vector,
                    metadata: RagMetadata {
                        file_type: lang.clone(),
                        git_hash: "HEAD".to_string(),
                        lines: (0, 0),
                        size_bytes: content.len(),
                    },
                    indexed_at: Utc::now().timestamp_millis() as u64,
                };
                batch.push(doc);
                count += 1;

                if batch.len() >= 128 {
                    self.vector_store.batch_upsert(batch.clone()).await.map_err(|e| e.to_string())?;
                    batch.clear();
                }
            }
        }

        if !batch.is_empty() {
            let _ = self.vector_store.batch_upsert(batch).await;
        }

        info!("RAG ingestion complete: {} documents indexed", count);
        Ok(count)
    }

    #[instrument(name = "rag_query", skip(self), fields(trace_id = %Uuid::new_v4()))]
    pub async fn query(&self, prompt: &str, limit: usize) -> Result<Vec<RagDocument>, String> {
        self.vector_store.query(prompt, limit, None).await.map_err(|e| e.to_string())
    }

    fn should_ingest(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy().to_lowercase();
        let ignore = [".git", "node_modules", "__pycache__", "target", ".venv"];
        !ignore.iter().any(|ig| path_str.contains(ig)) &&
        (path_str.ends_with(".rs") || path_str.ends_with(".py") || path_str.ends_with(".cpp") ||
         path_str.ends_with(".h") || path_str.ends_with(".js") || path_str.ends_with(".ts"))
    }

    fn detect_language(&self, path: &Path) -> String {
        match path.extension().and_then(|s| s.to_str()) {
            Some("rs") => "rust".to_string(),
            Some("py") => "python".to_string(),
            Some("cpp") | Some("cc") | Some("h") => "cpp".to_string(),
            Some("js") | Some("ts") => "javascript".to_string(),
            _ => "text".to_string(),
        }
    }
}

pub struct OptimizedChunker {
    query_patterns: AstQueryPatterns,
    embedder: SemanticEmbedder,
}

impl OptimizedChunker {
    pub fn new(llm_pool: Arc<LlamaPool>) -> Self {
        OptimizedChunker {
            query_patterns: AstQueryPatterns::new(),
            embedder: SemanticEmbedder::new(llm_pool),
        }
    }

    #[instrument(name = "chunker_chunk", skip(self), fields(trace_id = %Uuid::new_v4()))]
    pub async fn chunk(&self, code: &str, language: &str) -> Result<Vec<(String, SemanticEmbedding)>, RagError> {
        info!("Performing semantic AST chunking for language: {}", language);

        let mut parser = Parser::new();
        let lang: Language = match language {
            "rust" => tree_sitter_rust::language(),
            "python" => tree_sitter_python::language(),
            "cpp" => tree_sitter_cpp::language(),
            "javascript" => tree_sitter_javascript::language(),
            _ => return Err(RagError::UnsupportedLanguage(language.to_string())),
        };

        parser.set_language(lang).map_err(|e| RagError::Parse(format!("set_language failed: {}", e)))?;

        let tree = parser.parse(code, None)
            .ok_or_else(|| RagError::Parse("Failed to parse source".into()))?;

        let root = tree.root_node();
        let query_source = self.query_patterns.get_chunk_query(language)?;
        let query = Query::new(lang, query_source)?;

        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&query, root, code.as_bytes());

        let mut chunks = Vec::new();
        let chunk_size_target = 50;
        let overlap = 10;

        for m in matches {
            for capture in m.captures {
                let node = capture.node;
                if let Ok(text) = node.utf8_text(code.as_bytes()) {
                    chunks.push(text.to_string());
                }
            }
        }

        // Fallback to line-based if AST produced nothing
        if chunks.is_empty() {
            chunks = code.lines()
                .collect::<Vec<_>>()
                .chunks(chunk_size_target)
                .map(|c| c.join("\n"))
                .collect();
        }

        // Embed all chunks in batch
        let texts: Vec<&str> = chunks.iter().map(|s| s.as_str()).collect();
        let embeddings = self.embedder.embed_batch(texts).await?;

        let result: Vec<(String, SemanticEmbedding)> = chunks.into_iter().zip(embeddings).collect();

        info!("Produced {} semantic chunks", result.len());
        Ok(result)
    }
}