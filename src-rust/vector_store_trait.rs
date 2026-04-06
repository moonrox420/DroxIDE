// src-rust/vector_store_trait.rs
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::rag::RagDocument;

#[derive(Debug)]
pub enum VectorStoreError {
    Connection(String),
    Operation(String),
    Query(String),
    PartialUpsert(usize),
}

impl std::fmt::Display for VectorStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            VectorStoreError::Connection(s) => write!(f, "Connection error: {}", s),
            VectorStoreError::Operation(s) => write!(f, "Operation error: {}", s),
            VectorStoreError::Query(s) => write!(f, "Query error: {}", s),
            VectorStoreError::PartialUpsert(n) => write!(f, "Partial upsert: {} failed", n),
        }
    }
}

impl std::error::Error for VectorStoreError {}

#[async_trait]
pub trait VectorStore: Send + Sync {
    async fn batch_upsert(&self, documents: Vec<RagDocument>) -> Result<(), VectorStoreError>;
    async fn query(&self, prompt: &str, limit: usize, filter: Option<serde_json::Value>) -> Result<Vec<RagDocument>, VectorStoreError>;
    async fn delete(&self, id: String) -> Result<(), VectorStoreError>;
    async fn ensure_index(&self) -> Result<(), VectorStoreError>;
    async fn delete_index(&self) -> Result<(), VectorStoreError>;
    async fn health_check(&self) -> Result<bool, VectorStoreError>;
    async fn update_single(&self, document: RagDocument) -> Result<(), VectorStoreError>;
}
