// src-rust/lib.rs
pub mod agent;
pub mod orchestrator;
pub mod rag;
pub mod sandbox;
#[cfg(feature = "llama")]
pub mod llama;
#[cfg(not(feature = "llama"))]
pub mod llama_stub;
#[cfg(not(feature = "llama"))]
pub use llama_stub as llama;
pub mod metrics;
pub mod audit;
pub mod git;
pub mod vector_store;
pub mod vector_store_trait;
pub mod semantic_embedding;
pub mod ast_query_patterns;
pub mod ast_search;
pub mod code_search_engine;
pub mod hnsw_tuner;

pub use agent::{Agent, AgentId, AgentState};
pub use orchestrator::Orchestrator;
pub use rag::RagPipeline;
pub use sandbox::Sandbox;
pub use metrics::Metrics;

use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::sync::Arc;

// Test module
#[cfg(test)]
mod tests;

// Global state (shared with Qt via cxx-qt)
pub static ORCHESTRATOR: Lazy<Arc<Mutex<Option<Orchestrator>>>> =
    Lazy::new(|| Arc::new(Mutex::new(None)));

pub static METRICS: Lazy<Arc<Metrics>> =
    Lazy::new(|| Arc::new(Metrics::new()));

#[cxx::bridge]
mod ffi {
    extern "Rust" {
        // Orchestrator API
        pub fn init_orchestrator() -> Result<()>;
        pub fn run_swarm(prompt: &str, context_files: Vec<String>) -> Result<String>;
        pub fn accept_diff(diff_id: &str) -> Result<()>;
        pub fn reject_diff(diff_id: &str, feedback: &str) -> Result<()>;
        pub fn get_orchestrator_state() -> String;
        pub fn get_metrics_summary() -> String;
    }
}

pub fn init_orchestrator() -> Result<(), String> {
    let (tx, _rx) = tokio::sync::mpsc::channel(100);
    let llama = Arc::new(crate::llama::LlamaPool::new());
    let orchestrator = Orchestrator::new(tx, llama);
    *ORCHESTRATOR.lock() = Some(orchestrator);
    Ok(())
}

pub fn run_swarm(prompt: &str, context_files: Vec<String>) -> Result<String, String> {
    let mut guard = ORCHESTRATOR.lock();
    if let Some(ref mut orch) = *guard {
        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(orch.run(prompt, context_files))?;
        Ok(result)
    } else {
        Err("Orchestrator not initialized".into())
    }
}

pub fn accept_diff(diff_id: &str) -> Result<(), String> {
    let mut guard = ORCHESTRATOR.lock();
    if let Some(ref mut orch) = *guard {
        orch.accept_diff(diff_id)?;
        Ok(())
    } else {
        Err("Orchestrator not initialized".into())
    }
}

pub fn reject_diff(diff_id: &str, feedback: &str) -> Result<(), String> {
    let mut guard = ORCHESTRATOR.lock();
    if let Some(ref mut orch) = *guard {
        orch.reject_diff(diff_id, feedback)?;
        Ok(())
    } else {
        Err("Orchestrator not initialized".into())
    }
}

pub fn get_orchestrator_state() -> String {
    let guard = ORCHESTRATOR.lock();
    if let Some(ref orch) = *guard {
        format!("{:?}", orch.current_state())
    } else {
        "Uninitialized".into()
    }
}

pub fn get_metrics_summary() -> String {
    serde_json::to_string(&METRICS.summary()).unwrap_or_default()
}
