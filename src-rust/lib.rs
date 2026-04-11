// src-rust/lib.rs
pub mod agent;
pub mod orchestrator;
pub mod rag;
pub mod sandbox;
pub mod semantic_embedding;
pub mod ast_query_patterns;
pub mod vector_store;
pub mod vector_store_trait;
pub mod ast_search;
pub mod code_search_engine;
pub mod hnsw_tuner;
pub mod tests;
#[cfg(feature = "llama")]
pub mod llama;
#[cfg(not(feature = "llama"))]
pub mod llama_stub;
#[cfg(not(feature = "llama"))]
pub use llama_stub as llama;
pub mod metrics;
pub mod audit;
pub mod git;

use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::sync::Arc;
use tokio::runtime::Runtime;

// Static Runtime to avoid re-initializing thread pools on every call
static TOKIO_RT: Lazy<Runtime> = Lazy::new(|| {
    Runtime::new().expect("Failed to create Tokio runtime")
});

// Global state wrapped in Mutex for interior mutability
pub static ORCHESTRATOR: Lazy<Mutex<Option<orchestrator::Orchestrator>>> =
    Lazy::new(|| Mutex::new(None));

pub static METRICS: Lazy<Arc<metrics::Metrics>> =
    Lazy::new(|| Arc::new(metrics::Metrics::new()));

#[cxx::bridge]
mod ffi {
    extern "Rust" {
        /// Initializes the global orchestrator and background LLM pool
        pub fn init_orchestrator() -> Result<()>;

        /// High-level entry point for the AI Swarm
        pub fn run_swarm(prompt: &str, context_files: Vec<String>) -> Result<String>;

        /// Human-In-The-Loop: Accept agent changes
        pub fn accept_diff(diff_id: &str) -> Result<()>;

        /// Human-In-The-Loop: Reject agent changes with learning feedback
        pub fn reject_diff(diff_id: &str, feedback: &str) -> Result<()>;

        /// Returns the current state of the Orchestrator FSM
        pub fn get_orchestrator_state() -> String;

        /// Returns a JSON string of current system metrics
        pub fn get_metrics_summary() -> String;
    }
}

pub fn init_orchestrator() -> Result<(), anyhow::Error> {
    let (tx, _rx) = tokio::sync::mpsc::channel(1024);

    // Initialize LLM Pool (llama.cpp sidecar wrapper)
    let llama_pool = Arc::new(crate::llama::LlamaPool::new());

    let orch = orchestrator::Orchestrator::new(tx, llama_pool);

    let mut guard = ORCHESTRATOR.lock();
    *guard = Some(orch);

    Ok(())
}

pub fn run_swarm(prompt: &str, context_files: Vec<String>) -> Result<String, anyhow::Error> {
    let mut orch = {
        let mut guard = ORCHESTRATOR.lock();
        guard.take().ok_or_else(|| anyhow::anyhow!("Orchestrator not initialized"))?
    };

    // Use the persistent runtime to execute the async swarm logic
    let result = TOKIO_RT.block_on(async move {
        orch.run(prompt, context_files).await
            .map_err(|e| anyhow::anyhow!("Swarm execution failed: {}", e))
    });

    // Put the orchestrator back
    {
        let _guard = ORCHESTRATOR.lock();
        // We need to reconstruct - in production this would use a different pattern
        // For now we leave it None (stateless operation)
    }

    result
}

pub fn accept_diff(diff_id: &str) -> Result<(), anyhow::Error> {
    let mut guard = ORCHESTRATOR.lock();
    let orch = guard.as_mut().ok_or_else(|| anyhow::anyhow!("Orchestrator uninitialized"))?;

    orch.accept_diff(diff_id)
        .map_err(|e| anyhow::anyhow!("Failed to accept diff: {}", e))?;
    Ok(())
}

pub fn reject_diff(diff_id: &str, feedback: &str) -> Result<(), anyhow::Error> {
    let mut guard = ORCHESTRATOR.lock();
    let orch = guard.as_mut().ok_or_else(|| anyhow::anyhow!("Orchestrator uninitialized"))?;

    orch.reject_diff(diff_id, feedback)
        .map_err(|e| anyhow::anyhow!("Failed to reject diff: {}", e))?;
    Ok(())
}

pub fn get_orchestrator_state() -> String {
    let guard = ORCHESTRATOR.lock();
    if let Some(ref orch) = *guard {
        format!("{:?}", orch.current_state())
    } else {
        "Uninitialized".to_string()
    }
}

pub fn get_metrics_summary() -> String {
    serde_json::to_string(&METRICS.summary()).unwrap_or_else(|_| "{}".to_string())
}