// src-rust/orchestrator.rs
use crate::agent::*;
use crate::rag::RagPipeline;
use crate::sandbox::Sandbox;
use crate::llama::LlamaPool;
use crate::metrics::Metrics;
use crate::audit::AuditLog;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;
use chrono::Utc;
use tracing::{instrument, info};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum OrchestratorState {
    Idle,
    Researching,
    Pruning,
    Shadow,
    Coding,
    Review,
    Hitl,
    Applying,
    Done,
    Error,
}

impl fmt::Display for OrchestratorState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct Orchestrator {
    state: OrchestratorState,
    researcher: ResearcherAgent,
    architect: ArchitectAgent,
    coder: CoderAgent,
    reviewer: ReviewerAgent,
    tester: TesterAgent,
    janitor: JanitorAgent,
    rag: RagPipeline,
    sandbox: Sandbox,
    llama: Arc<LlamaPool>,
    metrics: Arc<Metrics>,
    audit: AuditLog,
    current_trace_id: String,
    pending_diff: Option<String>,
    event_tx: mpsc::Sender<AgentMessage>,
}

impl std::fmt::Debug for Orchestrator {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Orchestrator {{ state: {:?} }}", self.state)
    }
}

impl Orchestrator {
    pub fn new(event_tx: mpsc::Sender<AgentMessage>, llama: Arc<LlamaPool>) -> Self {
        Orchestrator {
            state: OrchestratorState::Idle,
            researcher: ResearcherAgent::new(),
            architect: ArchitectAgent::new(),
            coder: CoderAgent::new(),
            reviewer: ReviewerAgent::new(),
            tester: TesterAgent::new(),
            janitor: JanitorAgent::new(),
            rag: RagPipeline::new(llama.clone()),
            sandbox: Sandbox::new(),
            llama,
            metrics: Arc::new(Metrics::new()),
            audit: AuditLog::new(),
            current_trace_id: String::new(),
            pending_diff: None,
            event_tx,
        }
    }

    #[instrument(name = "orchestrator_run", skip(self), fields(trace_id = %Uuid::new_v4()))]
    pub async fn run(&mut self, prompt: &str, context_files: Vec<String>) -> Result<String, String> {
        self.current_trace_id = Uuid::new_v4().to_string();
        self.metrics.increment_prompts();

        self.state = OrchestratorState::Researching;
        self.notify_state_change().await;

        let current_dir = std::env::current_dir().unwrap();
        let current_dir_str = current_dir.to_string_lossy().to_string();
        
        let (researcher_output, architect_output) = tokio::join!(
            self.researcher.research(prompt, &self.rag),
            self.architect.analyze(&current_dir_str)
        );

        self.state = OrchestratorState::Pruning;
        self.notify_state_change().await;

        let pruned = self.prune_context(&researcher_output?, &architect_output?).await?;

        self.state = OrchestratorState::Shadow;
        self.notify_state_change().await;
        let _ = self.sandbox.shadow_sim(&pruned).await.map_err(|e| e.to_string())?;

        self.state = OrchestratorState::Coding;
        self.notify_state_change().await;
        let coder_output = self.coder.generate(prompt, &pruned, &self.llama).await?;
        self.pending_diff = Some(coder_output.diff.clone());

        self.state = OrchestratorState::Review;
        self.notify_state_change().await;
        let reviewer_output = self.reviewer.review(&coder_output.diff).await?;

        if !reviewer_output.approval {
            self.state = OrchestratorState::Error;
            self.notify_state_change().await;
            return Err("Reviewer rejected".into());
        }

        self.state = OrchestratorState::Hitl;
        self.notify_state_change().await;

        self.state = OrchestratorState::Applying;
        self.notify_state_change().await;

        self.state = OrchestratorState::Done;
        self.notify_state_change().await;
        self.metrics.increment_accepted();

        Ok(coder_output.diff)
    }

    async fn notify_state_change(&self) {
        let msg = AgentMessage {
            agent_id: "orchestrator".to_string(),
            state: self.state.to_string(),
            step: format!("State: {}", self.state),
            progress: 1.0,
            payload: serde_json::json!({}),
            timestamp: Utc::now().timestamp_millis() as u64,
            trace_id: self.current_trace_id.clone(),
        };
        let _ = self.event_tx.send(msg).await;
    }

    pub fn accept_diff(&mut self, diff_id: &str) -> Result<(), String> {
        self.state = OrchestratorState::Applying;
        self.pending_diff = None;
        self.state = OrchestratorState::Done;
        self.metrics.increment_accepted();
        Ok(())
    }

    pub fn reject_diff(&mut self, diff_id: &str, feedback: &str) -> Result<(), String> {
        self.metrics.increment_rejected();
        self.state = OrchestratorState::Idle;
        self.pending_diff = None;
        Ok(())
    }

    pub fn current_state(&self) -> OrchestratorState {
        self.state
    }

    async fn prune_context(&self, _researcher: &ResearcherOutput, _architect: &ArchitectOutput) -> Result<String, String> {
        Ok("pruned_context".to_string())
    }
}