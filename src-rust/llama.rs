// src-rust/llama.rs
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::{error, info, instrument, warn};
use uuid::Uuid;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LlamaError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("JSON serialization/deserialization failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error("LLM server unhealthy: {0}")]
    Unhealthy(String),
    #[error("LLM pool exhausted")]
    PoolExhausted,
    #[error("Invalid prompt: {0}")]
    InvalidPrompt(String),
    #[error("Timeout after {0}s")]
    Timeout(u64),
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
    client: Client,
    base_url: String,
    slots: Arc<Mutex<[bool; 4]>>,
    max_tokens: usize,
}

impl LlamaPool {
    pub fn new() -> Self {
        Self::with_config("http://127.0.0.1:8080", 120, 4096)
    }

    pub fn with_config(base_url: &str, timeout_secs: u64, max_tokens: usize) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            .expect("Failed to build reqwest client");

        LlamaPool {
            client,
            base_url: base_url.to_string(),
            slots: Arc::new(Mutex::new([true; 4])),
            max_tokens,
        }
    }

    #[instrument(name = "llama_health_check", skip(self), fields(trace_id = %Uuid::new_v4()))]
    pub async fn health_check(&self) -> Result<bool, LlamaError> {
        let url = format!("{}/health", self.base_url);
        match self.client.get(&url).send().await {
            Ok(resp) => {
                let status = resp.status();
                if status.is_success() {
                    info!("llama.cpp server is healthy");
                    Ok(true)
                } else {
                    warn!("llama.cpp server returned {}", status);
                    Err(LlamaError::Unhealthy(format!("HTTP {}", status)))
                }
            }
            Err(e) => {
                error!("Failed to connect to llama.cpp server: {}", e);
                Err(LlamaError::Unhealthy(e.to_string()))
            }
        }
    }

    async fn acquire_slot(&self) -> Result<usize, LlamaError> {
        let mut guard = self.slots.lock().await;
        for (i, available) in guard.iter_mut().enumerate() {
            if *available {
                *available = false;
                info!("Acquired slot {}", i);
                return Ok(i);
            }
        }
        Err(LlamaError::PoolExhausted)
    }

    async fn release_slot(&self, slot: usize) {
        if slot < 4 {
            let mut guard = self.slots.lock().await;
            guard[slot] = true;
            info!("Released slot {}", slot);
        }
    }

    #[instrument(name = "llama_complete", skip(self), fields(trace_id = %Uuid::new_v4()))]
    pub async fn complete(&self, prompt: &str) -> Result<String, LlamaError> {
        if prompt.trim().is_empty() {
            return Err(LlamaError::InvalidPrompt("Prompt is empty".into()));
        }
        if prompt.len() > 32768 {
            return Err(LlamaError::InvalidPrompt("Prompt exceeds 32768 characters".into()));
        }

        let slot = self.acquire_slot().await?;

        let req = CompletionRequest {
            prompt: prompt.to_string(),
            n_predict: self.max_tokens,
            temperature: 0.7,
            top_k: 40,
            top_p: 0.95,
            repeat_penalty: 1.1,
            stream: false,
            stop: vec!["</s>".to_string()],
        };

        let url = format!("{}/completion", self.base_url);
        let resp: Response = self.client.post(&url).json(&req).send().await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            error!(status = %status, body = %body, "LLM completion failed");
            self.release_slot(slot).await;
            return Err(LlamaError::Unhealthy(format!("HTTP {}: {}", status, body)));
        }

        let data: CompletionResponse = resp.json().await?;
        self.release_slot(slot).await;
        info!("LLM completion successful ({} chars)", data.content.len());
        Ok(data.content)
    }

    #[instrument(name = "llama_embed", skip(self), fields(trace_id = %Uuid::new_v4()))]
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>, LlamaError> {
        if text.trim().is_empty() {
            return Err(LlamaError::InvalidPrompt("Text is empty".into()));
        }

        let slot = self.acquire_slot().await?;

        let req = EmbeddingRequest {
            content: text.to_string(),
        };

        let url = format!("{}/embedding", self.base_url);
        let resp: Response = self.client.post(&url).json(&req).send().await?;

        if !resp.status().is_success() {
            let status = resp.status();
            error!(status = %status, "LLM embedding failed");
            self.release_slot(slot).await;
            return Err(LlamaError::Unhealthy(format!("HTTP {}", status)));
        }

        let data: EmbeddingResponse = resp.json().await?;
        self.release_slot(slot).await;
        info!("LLM embedding successful ({} dimensions)", data.embedding.len());
        Ok(data.embedding)
    }

    /// Batch embed multiple texts
    pub async fn embed_batch(&self, texts: Vec<&str>) -> Result<Vec<Vec<f32>>, LlamaError> {
        let mut embeddings = Vec::with_capacity(texts.len());
        for text in texts {
            let emb = self.embed(text).await?;
            embeddings.push(emb);
        }
        Ok(embeddings)
    }

    /// Get server info
    pub async fn get_server_info(&self) -> Result<serde_json::Value, LlamaError> {
        let url = format!("{}/props", self.base_url);
        let resp = self.client.get(&url).send().await?;
        let info: serde_json::Value = resp.json().await?;
        Ok(info)
    }
}

impl Default for LlamaPool {
    fn default() -> Self {
        Self::new()
    }
}