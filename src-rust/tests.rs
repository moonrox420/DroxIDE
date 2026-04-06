// src-rust/tests.rs - Comprehensive test suite
// This module contains integration tests for the DroxIDE Rust core

#[cfg(test)]
mod tests {
    use crate::agent::*;
    use crate::orchestrator::OrchestratorState;
    use crate::metrics::Metrics;
    use serde_json;

    // ========== Agent Tests ==========

    #[test]
    fn test_agent_id_serialization() {
        let ids = vec![
            AgentId::Orchestrator,
            AgentId::Researcher,
            AgentId::Architect,
            AgentId::Coder,
            AgentId::Reviewer,
            AgentId::Tester,
            AgentId::Janitor,
        ];

        for id in ids {
            let json = serde_json::to_string(&id).unwrap();
            let deserialized: AgentId = serde_json::from_str(&json).unwrap();
            assert_eq!(id, deserialized);
        }
    }

    #[test]
    fn test_agent_state_transitions() {
        let states = vec![
            AgentState::Idle,
            AgentState::Processing,
            AgentState::Done,
            AgentState::Error,
        ];

        for state in states {
            let json = serde_json::to_string(&state).unwrap();
            assert!(json.contains(&format!("{:?}", state)));
        }
    }

    #[test]
    fn test_agent_message_creation() {
        let msg = AgentMessage {
            agent_id: "researcher".to_string(),
            state: "processing".to_string(),
            step: "Querying RAG".to_string(),
            progress: 0.5,
            payload: serde_json::json!({"docs": 3}),
            timestamp: 1701234567890,
            trace_id: uuid::Uuid::new_v4().to_string(),
        };

        assert_eq!(msg.agent_id, "researcher");
        assert_eq!(msg.progress, 0.5);
        assert!(msg.payload["docs"].is_number());
    }

    #[test]
    fn test_agent_message_serialization() {
        let msg = AgentMessage {
            agent_id: "coder".to_string(),
            state: "done".to_string(),
            step: "Generated diff".to_string(),
            progress: 1.0,
            payload: serde_json::json!({"diff_size": 150}),
            timestamp: 1701234567890,
            trace_id: "test-trace".to_string(),
        };

        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: AgentMessage = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.agent_id, "coder");
        assert_eq!(deserialized.progress, 1.0);
    }

    #[test]
    fn test_researcher_output_structure() {
        let output = ResearcherOutput {
            docs: vec![
                ("doc1.rs".to_string(), 0.95),
                ("doc2.rs".to_string(), 0.87),
            ],
            context_size: 2048,
        };

        assert_eq!(output.docs.len(), 2);
        assert_eq!(output.context_size, 2048);
        assert!(output.docs[0].1 > 0.8);
    }

    #[test]
    fn test_architect_output_structure() {
        let output = ArchitectOutput {
            patterns: vec!["async".to_string(), "modular".to_string()],
            style_guide: "Rust".to_string(),
        };

        assert_eq!(output.patterns.len(), 2);
        assert_eq!(output.style_guide, "Rust");
    }

    #[test]
    fn test_coder_output_structure() {
        let output = CoderOutput {
            diff: "+ fn main() {}".to_string(),
            confidence: 0.87,
        };

        assert!(!output.diff.is_empty());
        assert!(output.confidence > 0.0 && output.confidence <= 1.0);
    }

    #[test]
    fn test_reviewer_output_approval() {
        let output = ReviewerOutput {
            lsp_errors: vec![],
            risk_score: 0.15,
            approval: true,
        };

        assert!(output.approval);
        assert!(output.risk_score < 0.5);
        assert!(output.lsp_errors.is_empty());
    }

    #[test]
    fn test_tester_output_structure() {
        let output = TesterOutput {
            tests_passed: 42,
            tests_failed: 0,
            coverage: 0.87,
        };

        assert_eq!(output.tests_passed, 42);
        assert_eq!(output.tests_failed, 0);
        assert!(output.coverage > 0.8);
    }

    // ========== Orchestrator State Tests ==========

    #[test]
    fn test_orchestrator_state_display() {
        let states = vec![
            OrchestratorState::Idle,
            OrchestratorState::Researching,
            OrchestratorState::Pruning,
            OrchestratorState::Shadow,
            OrchestratorState::Coding,
            OrchestratorState::Review,
            OrchestratorState::Hitl,
            OrchestratorState::Applying,
            OrchestratorState::Done,
            OrchestratorState::Error,
        ];

        for state in states {
            let display = format!("{}", state);
            assert!(!display.is_empty());
        }
    }

    #[test]
    fn test_orchestrator_state_serialization() {
        let state = OrchestratorState::Researching;
        let json = serde_json::to_string(&state).unwrap();
        assert!(json.contains("Researching"));

        let deserialized: OrchestratorState = serde_json::from_str(&json).unwrap();
        assert_eq!(state, deserialized);
    }

    #[test]
    fn test_orchestrator_state_equality() {
        assert_eq!(OrchestratorState::Idle, OrchestratorState::Idle);
        assert_ne!(OrchestratorState::Idle, OrchestratorState::Done);
    }

    // ========== Integration Tests ==========

    #[test]
    fn test_metrics_with_audit_log() {
        let metrics = Metrics::new();
        
        metrics.increment_prompts();
        metrics.increment_accepted();
        
        let summary = metrics.summary();
        
        assert_eq!(summary.prompts_total, 1);
        assert_eq!(summary.accepted, 1);
    }

    #[test]
    fn test_agent_message_with_metrics() {
        let metrics = Metrics::new();
        metrics.increment_prompts();

        let msg = AgentMessage {
            agent_id: "orchestrator".to_string(),
            state: "Researching".to_string(),
            step: "Starting research".to_string(),
            progress: 0.1,
            payload: serde_json::json!({"prompts_total": metrics.prompts_total.load(std::sync::atomic::Ordering::Relaxed)}),
            timestamp: 1701234567890,
            trace_id: uuid::Uuid::new_v4().to_string(),
        };

        assert_eq!(msg.payload["prompts_total"], 1);
    }

    #[test]
    fn test_full_swarm_workflow_simulation() {
        // Simulate a complete swarm workflow without actual LLM calls
        let metrics = Metrics::new();
        
        // Step 1: User submits prompt
        metrics.increment_prompts();
        
        // Step 2: Researcher queries RAG
        let researcher_msg = AgentMessage {
            agent_id: "researcher".to_string(),
            state: "done".to_string(),
            step: "Found 5 relevant docs".to_string(),
            progress: 1.0,
            payload: serde_json::json!({"docs": 5}),
            timestamp: 1701234567890,
            trace_id: uuid::Uuid::new_v4().to_string(),
        };
        assert_eq!(researcher_msg.payload["docs"], 5);
        
        // Step 3: Coder generates diff
        let coder_output = CoderOutput {
            diff: "+ async fn main() {}".to_string(),
            confidence: 0.87,
        };
        assert!(coder_output.confidence > 0.8);
        
        // Step 4: Reviewer approves
        let reviewer_output = ReviewerOutput {
            lsp_errors: vec![],
            risk_score: 0.15,
            approval: true,
        };
        assert!(reviewer_output.approval);
        
        // Step 5: User accepts
        metrics.increment_accepted();
        
        // Verify metrics
        let summary = metrics.summary();
        assert_eq!(summary.prompts_total, 1);
        assert_eq!(summary.accepted, 1);
        assert_eq!(summary.hallucinations, 0);
    }

    // ========== Edge Cases ==========

    #[test]
    fn test_empty_agent_message_payload() {
        let msg = AgentMessage {
            agent_id: "test".to_string(),
            state: "idle".to_string(),
            step: "".to_string(),
            progress: 0.0,
            payload: serde_json::json!({}),
            timestamp: 0,
            trace_id: "".to_string(),
        };

        assert!(msg.payload.as_object().unwrap().is_empty());
        assert_eq!(msg.progress, 0.0);
    }

    #[test]
    fn test_large_context_size() {
        let output = ResearcherOutput {
            docs: vec![],
            context_size: 32768,
        };

        assert_eq!(output.context_size, 32768);
    }

    #[test]
    fn test_zero_confidence_diff() {
        let output = CoderOutput {
            diff: "".to_string(),
            confidence: 0.0,
        };

        assert_eq!(output.confidence, 0.0);
        assert!(output.diff.is_empty());
    }

    #[test]
    fn test_max_risk_score() {
        let output = ReviewerOutput {
            lsp_errors: vec!["error".to_string()],
            risk_score: 1.0,
            approval: false,
        };

        assert_eq!(output.risk_score, 1.0);
        assert!(!output.approval);
    }

    // ========== Performance Tests ==========

    #[test]
    fn test_metrics_atomic_operations_performance() {
        let metrics = Metrics::new();
        let iterations = 100_000;

        let start = std::time::Instant::now();
        for _ in 0..iterations {
            metrics.increment_prompts();
        }
        let duration = start.elapsed();

        assert_eq!(metrics.prompts_total.load(std::sync::atomic::Ordering::Relaxed), iterations);
        println!("{} increments completed in {:?}", iterations, duration);
    }
}
