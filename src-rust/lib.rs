// src-rust/lib.rs
pub mod agent;
pub mod orchestrator;
pub mod rag;
pub mod sandbox;
pub mod semantic_embedding;
pub mod ast_query_patterns;
pub mod vector_store_trait;
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
use parking_lot::RwLock;
use std::sync::Arc;
use tokio::runtime::Runtime;

// Static Runtime to avoid re-initializing thread pools on every call
static TOKIO_RT: Lazy<Runtime> = Lazy::new(|| {
    Runtime::new().expect("Failed to create Tokio runtime")
});

// Global state wrapped in Arc/RwLock for thread-safe access from Qt
pub static ORCHESTRATOR: Lazy<RwLock<Option<Arc<orchestrator::Orchestrator>>>> =
    Lazy::new(|| RwLock::new(None));

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
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    
    // Initialize LLM Pool (llama.cpp sidecar wrapper)
    let llama_pool = Arc::new(crate::llama::LlamaPool::new());
    
    let orch = orchestrator::Orchestrator::new(tx, llama_pool);
    
    let mut guard = ORCHESTRATOR.write();
    *guard = Some(Arc::new(orch));
    
    Ok(())
}

pub fn run_swarm(prompt: &str, context_files: Vec<String>) -> Result<String, anyhow::Error> {
    let orch = {
        let guard = ORCHESTRATOR.read();
        guard.as_ref()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Orchestrator not initialized"))?
    };

    // Use the persistent runtime to execute the async swarm logic
    TOKIO_RT.block_on(async move {
        orch.run(prompt, context_files).await
            .map_err(|e| anyhow::anyhow!("Swarm execution failed: {}", e))
    })
}

pub fn accept_diff(diff_id: &str) -> Result<(), anyhow::Error> {
    let guard = ORCHESTRATOR.read();
    let orch = guard.as_ref().ok_or_else(|| anyhow::anyhow!("Orchestrator uninitialized"))?;
    
    orch.accept_diff(diff_id); // Internal state update
    Ok(())
}

pub fn reject_diff(diff_id: &str, feedback: &str) -> Result<(), anyhow::Error> {
    let guard = ORCHESTRATOR.read();
    let orch = guard.as_ref().ok_or_else(|| anyhow::anyhow!("Orchestrator uninitialized"))?;
    
    orch.reject_diff(diff_id, feedback);
    Ok(())
}

pub fn get_orchestrator_state() -> String {
    let guard = ORCHESTRATOR.read();
    if let Some(ref orch) = *guard {
        format!("{:?}", orch.current_state())
    } else {
        "Uninitialized".to_string()
    }
}

pub fn get_metrics_summary() -> String {
    serde_json::to_string(&METRICS.summary()).unwrap_or_else(|_| "{}".to_string())
}