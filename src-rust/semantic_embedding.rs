// src-rust/semantic_embedding.rs
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{instrument, info};
use uuid::Uuid;
use thiserror::Error;
use crate::llama::LlamaPool;

/// Production-grade semantic code embedding strategy for DroxIDE
/// - Normalizes embeddings (L2)
/// - Supports batch embedding
/// - Hybrid scoring (structural + semantic cosine)
/// - Used by RAGPipeline, AstSearchEngine, ResearcherAgent

#[derive(Error, Debug)]
pub enum EmbeddingError {
    #[error("Llama embedding failed: {0}")]
    Llama(#[from] crate::llama::LlamaError),
    #[error("Normalization error")]
    Normalization,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SemanticEmbedding {
    pub vector: Vec<f32>,
    pub norm: f32,
    pub token_count: usize,
}

impl SemanticEmbedding {
    pub fn new(vector: Vec<f32>, token_count: usize) -> Self {
        let norm = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        SemanticEmbedding { vector, norm: if norm > 0.0 { norm } else { 1.0 }, token_count }
    }

    pub fn cosine_similarity(&self, other: &SemanticEmbedding) -> f32 {
        if self.norm == 0.0 || other.norm == 0.0 {
            return 0.0;
        }
        let dot: f32 = self.vector.iter().zip(other.vector.iter()).map(|(a, b)| a * b).sum();
        dot / (self.norm * other.norm)
    }
}

#[derive(Clone, Debug)]
pub struct SemanticEmbedder {
    llm_pool: Arc<LlamaPool>,
}

impl SemanticEmbedder {
    pub fn new(llm_pool: Arc<LlamaPool>) -> Self {
        SemanticEmbedder { llm_pool }
    }

    #[instrument(name = "semantic_embed_batch", skip(self), fields(trace_id = %Uuid::new_v4()))]
    pub async fn embed_batch(&self, texts: Vec<&str>) -> Result<Vec<SemanticEmbedding>, EmbeddingError> {
        info!("Embedding batch of {} code chunks", texts.len());
        
        let mut embeddings = Vec::with_capacity(texts.len());
        
        // Production: use llama.cpp batch embedding endpoint when available
        for text in texts {
            let raw_vec = self.llm_pool.embed(text).await?;
            let token_count = text.split_whitespace().count();
            embeddings.push(SemanticEmbedding::new(raw_vec, token_count));
        }
        
        info!("Completed batch embedding");
        Ok(embeddings)
    }

    #[instrument(name = "semantic_embed_single", skip(self), fields(trace_id = %Uuid::new_v4()))]
    pub async fn embed_single(&self, text: &str) -> Result<SemanticEmbedding, EmbeddingError> {
        let raw_vec = self.llm_pool.embed(text).await?;
        let token_count = text.split_whitespace().count();
        Ok(SemanticEmbedding::new(raw_vec, token_count))
    }

    /// Hybrid score: structural confidence (0-1) + semantic cosine
    pub fn hybrid_score(structural_confidence: f32, semantic_sim: f32) -> f32 {
        (structural_confidence * 0.6) + (semantic_sim * 0.4)
    }
}