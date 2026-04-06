// src-rust/audit.rs
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::fs::OpenOptions;
use std::io::Write;
use chrono::Utc;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditLog {
    pub event: String,
    pub timestamp: u64,
    pub user: String,
    pub agent: String,
    pub input: serde_json::Value,
    pub output: serde_json::Value,
    pub duration_ms: u64,
    pub trace_id: String,
}

impl AuditLog {
    pub fn new() -> Self {
        AuditLog {
            event: String::new(),
            timestamp: 0,
            user: String::new(),
            agent: String::new(),
            input: serde_json::json!({}),
            output: serde_json::json!({}),
            duration_ms: 0,
            trace_id: String::new(),
        }
    }

    pub fn with_event(event: &str) -> Self {
        AuditLog {
            event: event.to_string(),
            timestamp: Utc::now().timestamp_millis() as u64,
            user: "local".to_string(),
            agent: "Orchestrator".to_string(),
            input: serde_json::json!({}),
            output: serde_json::json!({}),
            duration_ms: 0,
            trace_id: uuid::Uuid::new_v4().to_string(),
        }
    }

    pub fn with_trace_id(mut self, trace_id: &str) -> Self {
        self.trace_id = trace_id.to_string();
        self
    }

    pub fn with_input(mut self, input: serde_json::Value) -> Self {
        self.input = input;
        self
    }

    pub fn with_output(mut self, output: serde_json::Value) -> Self {
        self.output = output;
        self
    }

    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.duration_ms = duration_ms;
        self
    }

    pub fn with_agent(mut self, agent: &str) -> Self {
        self.agent = agent.to_string();
        self
    }

    pub fn write_to_file(&self, path: &Path) -> Result<(), String> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|e| e.to_string())?;

        let json = serde_json::to_string(self).map_err(|e| e.to_string())?;
        writeln!(file, "{}", json).map_err(|e| e.to_string())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_audit_log_new() {
        let log = AuditLog::new();
        
        assert_eq!(log.event, "");
        assert_eq!(log.timestamp, 0);
        assert_eq!(log.user, "");
        assert_eq!(log.agent, "");
        assert_eq!(log.duration_ms, 0);
        assert_eq!(log.trace_id, "");
    }

    #[test]
    fn test_audit_log_with_event() {
        let log = AuditLog::with_event("swarm_started");
        
        assert_eq!(log.event, "swarm_started");
        assert!(log.timestamp > 0);
        assert_eq!(log.user, "local");
        assert_eq!(log.agent, "Orchestrator");
        assert!(!log.trace_id.is_empty());
    }

    #[test]
    fn test_audit_log_builder_pattern() {
        let log = AuditLog::with_event("researcher_done")
            .with_trace_id("test-trace-123")
            .with_input(serde_json::json!({"prompt": "test"}))
            .with_output(serde_json::json!({"docs": 3}))
            .with_duration(1500)
            .with_agent("Researcher");
        
        assert_eq!(log.event, "researcher_done");
        assert_eq!(log.trace_id, "test-trace-123");
        assert_eq!(log.input["prompt"], "test");
        assert_eq!(log.output["docs"], 3);
        assert_eq!(log.duration_ms, 1500);
        assert_eq!(log.agent, "Researcher");
    }

    #[test]
    fn test_audit_log_write_to_file() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_audit.jsonl");
        
        // Clean up if exists
        if test_file.exists() {
            fs::remove_file(&test_file).unwrap();
        }
        
        let log = AuditLog::with_event("test_event")
            .with_trace_id("test-trace");
        
        let result = log.write_to_file(&test_file);
        assert!(result.is_ok());
        
        // Verify file was created
        assert!(test_file.exists());
        
        // Read and verify content
        let content = fs::read_to_string(&test_file).unwrap();
        assert!(content.contains("test_event"));
        assert!(content.contains("test-trace"));
        
        // Clean up
        fs::remove_file(&test_file).unwrap();
    }

    #[test]
    fn test_audit_log_append_multiple() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_audit_append.jsonl");
        
        if test_file.exists() {
            fs::remove_file(&test_file).unwrap();
        }
        
        let log1 = AuditLog::with_event("event_1");
        let log2 = AuditLog::with_event("event_2");
        
        log1.write_to_file(&test_file).unwrap();
        log2.write_to_file(&test_file).unwrap();
        
        let content = fs::read_to_string(&test_file).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("event_1"));
        assert!(lines[1].contains("event_2"));
        
        fs::remove_file(&test_file).unwrap();
    }

    #[test]
    fn test_audit_log_serialization() {
        let log = AuditLog::with_event("serialize_test")
            .with_trace_id("trace-456")
            .with_input(serde_json::json!({"key": "value"}));
        
        let json = serde_json::to_string(&log).unwrap();
        
        assert!(json.contains("serialize_test"));
        assert!(json.contains("trace-456"));
        assert!(json.contains("key"));
        
        let deserialized: AuditLog = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.event, "serialize_test");
        assert_eq!(deserialized.trace_id, "trace-456");
    }

    #[test]
    fn test_audit_log_timestamp_generation() {
        let log1 = AuditLog::with_event("test1");
        std::thread::sleep(std::time::Duration::from_millis(10));
        let log2 = AuditLog::with_event("test2");
        
        // Timestamps should be different (or same if very fast)
        assert!(log1.timestamp > 0);
        assert!(log2.timestamp > 0);
    }
}
