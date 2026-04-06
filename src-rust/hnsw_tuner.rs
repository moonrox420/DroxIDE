// src-rust/hnsw_tuner.rs
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{instrument, info};
use uuid::Uuid;
use thiserror::Error;
use std::num::NonZeroUsize;
use lru::LruCache;

#[derive(Error, Debug)]
pub enum HnswTunerError {
    #[error("Hardware detection failed: {0}")]
    Hardware(String),
    #[error("Invalid corpus profile")]
    InvalidProfile,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HnswParams {
    pub m: usize,
    pub ef_construction: usize,
    pub ef_search: usize,
    pub num_threads: usize,
    pub recall_target: f32,
    pub profile: String,
}

#[derive(Clone, Debug)]
pub struct CorpusProfile {
    pub vector_count: usize,
    pub dim: usize,
    pub avg_chunk_size_bytes: usize,
    pub hardware_cores: usize,
    pub available_ram_mb: usize,
    pub workload: WorkloadType,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkloadType {
    Research,
    Editor,
    Background,
}

#[derive(Clone)]
pub struct HnswTuner {
    cache: Arc<Mutex<LruCache<String, HnswParams>>>,
    last_profile: Arc<parking_lot::Mutex<Option<CorpusProfile>>>,
}

impl HnswTuner {
    pub fn new() -> Self {
        HnswTuner {
            cache: Arc::new(Mutex::new(LruCache::new(NonZeroUsize::new(50).unwrap()))),
            last_profile: Arc::new(parking_lot::Mutex::new(None)),
        }
    }

    #[instrument(name = "hnsw_tune_parameters", skip(self), fields(trace_id = %Uuid::new_v4()))]
    pub async fn tune_for_corpus(
        &self,
        vector_count: usize,
        dim: usize,
        avg_chunk_bytes: usize,
        workload: WorkloadType,
    ) -> Result<HnswParams, HnswTunerError> {
        let trace_id = Uuid::new_v4().to_string();
        info!(%trace_id, "HNSW adaptive tuning started for {} vectors", vector_count);

        let profile = self.build_corpus_profile(vector_count, dim, avg_chunk_bytes, workload).await?;
        let cache_key = format!("{}-{}-{}", profile.vector_count / 1000, profile.dim, profile.workload as usize);

        {
            let mut cache_guard = self.cache.lock().await;
            if let Some(cached) = cache_guard.get(&cache_key) {
                info!(%trace_id, "HNSW cache hit");
                return Ok(cached.clone());
            }
        }

        let params = self.compute_optimal_params(&profile).await?;

        {
            let mut cache_guard = self.cache.lock().await;
            cache_guard.put(cache_key, params.clone());
        }

        info!(%trace_id, "HNSW tuning complete → M={}, ef_construction={}, ef_search={}", params.m, params.ef_construction, params.ef_search);
        Ok(params)
    }

    async fn build_corpus_profile(
        &self,
        vector_count: usize,
        dim: usize,
        avg_chunk_bytes: usize,
        workload: WorkloadType,
    ) -> Result<CorpusProfile, HnswTunerError> {
        let cores = num_cpus::get();
        let ram_mb = 16384; // Production: use sysinfo crate

        let profile = CorpusProfile {
            vector_count,
            dim,
            avg_chunk_size_bytes: avg_chunk_bytes,
            hardware_cores: cores,
            available_ram_mb: ram_mb,
            workload,
        };

        *self.last_profile.lock() = Some(profile.clone());
        Ok(profile)
    }

    async fn compute_optimal_params(&self, profile: &CorpusProfile) -> Result<HnswParams, HnswTunerError> {
        let size_category = match profile.vector_count {
            0..=10_000 => "small",
            10_001..=100_000 => "medium",
            100_001..=1_000_000 => "large",
            _ => "massive",
        };

        let (m, ef_construction, ef_search, recall_target) = match (size_category, profile.workload) {
            ("small", WorkloadType::Editor) => (8, 40, 20, 0.92),
            ("small", WorkloadType::Research) => (12, 80, 50, 0.97),
            ("medium", WorkloadType::Editor) => (16, 80, 40, 0.94),
            ("medium", WorkloadType::Research) => (24, 150, 80, 0.98),
            ("large", WorkloadType::Editor) => (32, 120, 60, 0.95),
            ("large", WorkloadType::Research) => (48, 250, 120, 0.99),
            ("massive", WorkloadType::Editor) => (64, 200, 100, 0.96),
            ("massive", WorkloadType::Research) => (96, 400, 200, 0.995),
            _ => (32, 120, 60, 0.95),
        };

        let num_threads = (profile.hardware_cores / 2).clamp(4, 16);

        Ok(HnswParams {
            m,
            ef_construction,
            ef_search,
            num_threads,
            recall_target,
            profile: size_category.to_string(),
        })
    }

    pub fn to_chroma_metadata(&self, params: &HnswParams) -> serde_json::Value {
        serde_json::json!({
            "hnsw:space": "cosine",
            "hnsw:construction_ef": params.ef_construction,
            "hnsw:search_ef": params.ef_search,
            "hnsw:M": params.m,
            "hnsw:num_threads": params.num_threads,
            "droxide:tuned_for": params.profile,
            "droxide:recall_target": params.recall_target,
        })
    }
}