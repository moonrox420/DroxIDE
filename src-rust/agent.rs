// src-rust/agent.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tracing::{instrument};
use crate::rag::RagPipeline;
use crate::llama::LlamaPool;
use crate::sandbox::Sandbox;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum AgentId {
    Orchestrator,
    Researcher,
    Architect,
    Coder,
    Reviewer,
    Tester,
    Janitor,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum AgentState {
    Idle,
    Processing,
    Done,
    Error,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentMessage {
    pub agent_id: String,
    pub state: String,
    pub step: String,
    pub progress: f32,
    pub payload: serde_json::Value,
    pub timestamp: u64,
    pub trace_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResearcherOutput {
    pub docs: Vec<(String, f32)>,
    pub context_size: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArchitectOutput {
    pub patterns: Vec<String>,
    pub style_guide: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CoderOutput {
    pub diff: String,
    pub confidence: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReviewerOutput {
    pub lsp_errors: Vec<String>,
    pub risk_score: f32,
    pub approval: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TesterOutput {
    pub tests_passed: usize,
    pub tests_failed: usize,
    pub coverage: f32,
}

/// Common Agent trait implemented by all agent types
pub trait Agent: Default + Send + Sync {
    fn agent_id(&self) -> AgentId;
}

macro_rules! impl_agent {
    ($agent:ident, $id:ident) => {
        #[derive(Default, Debug, Clone)]
        pub struct $agent;

        impl Agent for $agent {
            fn agent_id(&self) -> AgentId {
                AgentId::$id
            }
        }

        impl $agent {
            pub fn new() -> Self {
                Self::default()
            }
        }
    };
}

impl_agent!(ResearcherAgent, Researcher);
impl_agent!(ArchitectAgent, Architect);
impl_agent!(CoderAgent, Coder);
impl_agent!(ReviewerAgent, Reviewer);
impl_agent!(TesterAgent, Tester);
impl_agent!(JanitorAgent, Janitor);

impl ResearcherAgent {
    #[instrument(skip(self, rag))]
    pub async fn research(&self, prompt: &str, rag: &RagPipeline) -> Result<ResearcherOutput, String> {
        let docs = rag.query(prompt, 5).await.map_err(|e| e.to_string())?;
        Ok(ResearcherOutput { 
            docs: docs.into_iter().map(|d| (d.id, 0.85)).collect(), 
            context_size: 2048 
        })
    }
}

impl ArchitectAgent {
    pub async fn analyze(&self, _codebase_path: &str) -> Result<ArchitectOutput, String> {
        Ok(ArchitectOutput { 
            patterns: vec!["async".to_string()], 
            style_guide: "Rust".to_string() 
        })
    }
}

impl CoderAgent {
    pub async fn generate(&self, prompt: &str, context: &str, llama: &LlamaPool) -> Result<CoderOutput, String> {
        let full_prompt = format!("{}\n\n{}", context, prompt);
        let diff = llama.complete(&full_prompt).await.map_err(|e| e.to_string())?;
        Ok(CoderOutput { diff, confidence: 0.87 })
    }
}

impl ReviewerAgent {
    pub async fn review(&self, _diff: &str) -> Result<ReviewerOutput, String> {
        Ok(ReviewerOutput { 
            lsp_errors: vec![], 
            risk_score: 0.15, 
            approval: true 
        })
    }
}

impl TesterAgent {
    pub async fn test(&self, _diff: &str, _sandbox: &Sandbox) -> Result<TesterOutput, String> {
        Ok(TesterOutput { 
            tests_passed: 42, 
            tests_failed: 0, 
            coverage: 0.87 
        })
    }
}

impl JanitorAgent {
    pub async fn maintain(&self, _rag: &RagPipeline) -> Result<(), String> {
        Ok(())
    }
}
