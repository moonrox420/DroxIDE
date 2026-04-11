// src-rust/llama_stub.rs
// Stub implementation when llama_cpp feature is disabled

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LlamaError {
    #[error("llama.cpp not available - enable 'llama' feature")]
    NotAvailable,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub prompt: String,
    pub n_predict: usize,
    pub temperature: f32,
    pub top_k: usize,
    pub top_p: f32,
    pub repeat_penalty: f32,
    pub stream: bool,
    pub stop: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompletionResponse {
    pub content: String,
    pub stop: bool,
    pub generation_settings: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmbeddingRequest {
    pub content: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    pub embedding: Vec<f32>,
}

#[derive(Debug)]
pub struct LlamaPool {
    #[allow(dead_code)]
    available: bool,
}

impl LlamaPool {
    pub fn new() -> Self {
        LlamaPool { available: false }
    }

    pub fn with_config(_base_url: &str, _timeout_secs: u64, _max_tokens: usize) -> Self {
        LlamaPool { available: false }
    }

    pub async fn health_check(&self) -> Result<bool, LlamaError> {
        Ok(false)
    }

    pub async fn complete(&self, _prompt: &str) -> Result<String, LlamaError> {
        Err(LlamaError::NotAvailable)
    }

    pub async fn embed(&self, _text: &str) -> Result<Vec<f32>, LlamaError> {
        Err(LlamaError::NotAvailable)
    }

    pub async fn embed_batch(&self, _texts: Vec<&str>) -> Result<Vec<Vec<f32>>, LlamaError> {
        Err(LlamaError::NotAvailable)
    }

    pub async fn get_server_info(&self) -> Result<serde_json::Value, LlamaError> {
        Err(LlamaError::NotAvailable)
    }
}

impl Default for LlamaPool {
    fn default() -> Self {
        Self::new()
    }
}
