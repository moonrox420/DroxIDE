// src-rust/metrics.rs
use std::sync::atomic::{AtomicU64, Ordering};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MetricsSummary {
    pub prompts_total: u64,
    pub hallucinations: u64,
    pub accepted: u64,
    pub rejected: u64,
    pub time_saved_minutes: u64,
    pub avg_latency_ms: u64,
}

pub struct Metrics {
    pub prompts_total: AtomicU64,
    pub hallucinations: AtomicU64,
    pub accepted: AtomicU64,
    pub rejected: AtomicU64,
    pub time_saved_minutes: AtomicU64,
    pub avg_latency_ms: AtomicU64,
}

impl Metrics {
    pub fn new() -> Self {
        Metrics {
            prompts_total: AtomicU64::new(0),
            hallucinations: AtomicU64::new(0),
            accepted: AtomicU64::new(0),
            rejected: AtomicU64::new(0),
            time_saved_minutes: AtomicU64::new(0),
            avg_latency_ms: AtomicU64::new(0),
        }
    }

    pub fn increment_prompts(&self) {
        self.prompts_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_hallucinations(&self) {
        self.hallucinations.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_accepted(&self) {
        self.accepted.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_rejected(&self) {
        self.rejected.fetch_add(1, Ordering::Relaxed);
    }

    pub fn add_time_saved(&self, minutes: u64) {
        self.time_saved_minutes.fetch_add(minutes, Ordering::Relaxed);
    }

    pub fn update_avg_latency(&self, latency_ms: u64) {
        self.avg_latency_ms.store(latency_ms, Ordering::Relaxed);
    }

    pub fn summary(&self) -> MetricsSummary {
        MetricsSummary {
            prompts_total: self.prompts_total.load(Ordering::Relaxed),
            hallucinations: self.hallucinations.load(Ordering::Relaxed),
            accepted: self.accepted.load(Ordering::Relaxed),
            rejected: self.rejected.load(Ordering::Relaxed),
            time_saved_minutes: self.time_saved_minutes.load(Ordering::Relaxed),
            avg_latency_ms: self.avg_latency_ms.load(Ordering::Relaxed),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_initial_state() {
        let metrics = Metrics::new();
        let summary = metrics.summary();
        
        assert_eq!(summary.prompts_total, 0);
        assert_eq!(summary.hallucinations, 0);
        assert_eq!(summary.accepted, 0);
        assert_eq!(summary.rejected, 0);
        assert_eq!(summary.time_saved_minutes, 0);
        assert_eq!(summary.avg_latency_ms, 0);
    }

    #[test]
    fn test_increment_prompts() {
        let metrics = Metrics::new();
        
        metrics.increment_prompts();
        metrics.increment_prompts();
        metrics.increment_prompts();
        
        assert_eq!(metrics.prompts_total.load(Ordering::Relaxed), 3);
    }

    #[test]
    fn test_increment_accepted() {
        let metrics = Metrics::new();
        
        metrics.increment_accepted();
        metrics.increment_accepted();
        
        assert_eq!(metrics.accepted.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn test_increment_rejected() {
        let metrics = Metrics::new();
        
        metrics.increment_rejected();
        
        assert_eq!(metrics.rejected.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_increment_hallucinations() {
        let metrics = Metrics::new();
        
        metrics.increment_hallucinations();
        metrics.increment_hallucinations();
        
        assert_eq!(metrics.hallucinations.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn test_add_time_saved() {
        let metrics = Metrics::new();
        
        metrics.add_time_saved(10);
        metrics.add_time_saved(20);
        
        assert_eq!(metrics.time_saved_minutes.load(Ordering::Relaxed), 30);
    }

    #[test]
    fn test_update_avg_latency() {
        let metrics = Metrics::new();
        
        metrics.update_avg_latency(1500);
        
        assert_eq!(metrics.avg_latency_ms.load(Ordering::Relaxed), 1500);
    }

    #[test]
    fn test_summary_snapshot() {
        let metrics = Metrics::new();
        
        metrics.increment_prompts();
        metrics.increment_accepted();
        metrics.add_time_saved(5);
        metrics.update_avg_latency(800);
        
        let summary = metrics.summary();
        
        assert_eq!(summary.prompts_total, 1);
        assert_eq!(summary.accepted, 1);
        assert_eq!(summary.time_saved_minutes, 5);
        assert_eq!(summary.avg_latency_ms, 800);
    }

    #[test]
    fn test_concurrent_access() {
        use std::sync::Arc;
        use std::thread;
        
        let metrics = Arc::new(Metrics::new());
        let mut handles = vec![];
        
        // Spawn 10 threads, each incrementing 100 times
        for _ in 0..10 {
            let metrics_clone = metrics.clone();
            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    metrics_clone.increment_prompts();
                }
            });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.join().unwrap();
        }
        
        assert_eq!(metrics.prompts_total.load(Ordering::Relaxed), 1000);
    }

    #[test]
    fn test_metrics_summary_serialization() {
        let metrics = Metrics::new();
        metrics.increment_prompts();
        metrics.increment_accepted();
        
        let summary = metrics.summary();
        let json = serde_json::to_string(&summary).unwrap();
        
        assert!(json.contains("prompts_total"));
        assert!(json.contains("1"));
        
        let deserialized: MetricsSummary = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.prompts_total, 1);
    }
}
