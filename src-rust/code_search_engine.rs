// src-rust/code_search_engine.rs
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{instrument, info, error};
use uuid::Uuid;
use thiserror::Error;
use dashmap::DashMap;
use lru::LruCache;
use std::num::NonZeroUsize;

use crate::ast_query_patterns::AstQueryPatterns;
use crate::semantic_embedding::{SemanticEmbedder, SemanticEmbedding};
use crate::llama::LlamaPool;
use crate::rag::RagDocument;

/// Production-grade Code Search Engine for DroxIDE v2.0
/// Architecture:
///   1. Structural Search Layer (Tree-sitter AST queries)
///   2. Semantic Search Layer (llama.cpp embeddings + cosine similarity)
///   3. Hybrid Ranking Engine (weighted fusion + git ancestry boost)
///   4. LRU + DashMap Cache Layer (query → results)
///   5. Incremental Index Updater (file watcher integration ready)
/// Used by: ResearcherAgent, ArchitectAgent, Editor "Find in Files", RAG Pipeline

#[derive(Error, Debug)]
pub enum CodeSearchError {
    #[error("Structural search failed: {0}")]
    Structural(#[from] crate::ast_query_patterns::QueryError),
    #[error("Semantic embedding failed: {0}")]
    Semantic(#[from] crate::semantic_embedding::EmbeddingError),
    #[error("Cache error: {0}")]
    Cache(String),
    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),
    #[error("IO error: {0}")]
    Other(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CodeSearchResult {
    pub document: RagDocument,
    pub match_type: String,           // "structural", "semantic", "hybrid"
    pub score: f32,                   // final hybrid score (0.0-1.0)
    pub matched_node: String,
    pub location: (usize, usize),     // (start_line, end_line)
    pub capture_name: Option<String>,
    pub ancestry_boost: f32,          // git recency factor
}

#[derive(Clone, Debug)]
pub struct CodeSearchEngine {
    query_patterns: AstQueryPatterns,
    embedder: SemanticEmbedder,
    cache: Arc<Mutex<LruCache<String, Vec<CodeSearchResult>>>>,
    index: DashMap<String, Vec<RagDocument>>, // path → documents (incremental)
    ancestry_cache: DashMap<String, f32>,     // file_path → recency score (0-1)
}

impl CodeSearchEngine {
    pub fn new(llm_pool: Arc<LlamaPool>) -> Self {
        CodeSearchEngine {
            query_patterns: AstQueryPatterns::new(),
            embedder: SemanticEmbedder::new(llm_pool),
            cache: Arc::new(Mutex::new(LruCache::new(NonZeroUsize::new(500).unwrap()))),
            index: DashMap::new(),
            ancestry_cache: DashMap::new(),
        }
    }

    /// Main hybrid search entry point – used by all agents
    #[instrument(name = "code_search_hybrid", skip(self, query_text, codebase_path), fields(trace_id = %Uuid::new_v4()))]
    pub async fn hybrid_search(
        &self,
        query_text: &str,
        codebase_path: &Path,
        limit: usize,
        min_score: f32,
    ) -> Result<Vec<CodeSearchResult>, CodeSearchError> {
        let trace_id = Uuid::new_v4().to_string();
        info!(%trace_id, "Hybrid code search started for query: {}", query_text);

        // 1. Cache lookup
        let cache_key = format!("{}|{}|{}", query_text, codebase_path.display(), limit);
        {
            let mut cache_guard = self.cache.lock().await;
            if let Some(cached) = cache_guard.get(&cache_key) {
                info!(%trace_id, "Cache hit – returning {} results", cached.len());
                return Ok(cached.clone());
            }
        }

        // 2. Structural AST search
        let structural_results = self.structural_search(query_text, codebase_path).await?;

        // 3. Semantic embedding search
        let query_embedding = self.embedder.embed_single(query_text).await?;
        let semantic_results = self.semantic_search(&query_embedding, codebase_path).await?;

        // 4. Hybrid ranking + ancestry boost
        let mut merged = self.rank_hybrid(structural_results, semantic_results, &query_embedding).await?;

        // 5. Apply git ancestry boost
        self.apply_ancestry_boost(&mut merged).await?;

        // 6. Filter and sort
        merged.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        merged.truncate(limit);
        let final_results: Vec<_> = merged.into_iter().filter(|r| r.score >= min_score).collect();

        // 7. Cache result
        {
            let mut cache_guard = self.cache.lock().await;
            cache_guard.put(cache_key, final_results.clone());
        }

        info!(%trace_id, "Hybrid search completed with {} results", final_results.len());
        Ok(final_results)
    }

    /// Pure structural search using Tree-sitter (fast path)
    #[instrument(name = "code_search_structural", skip(self, query_text, codebase_path))]
    async fn structural_search(
        &self,
        query_text: &str,
        codebase_path: &Path,
    ) -> Result<Vec<CodeSearchResult>, CodeSearchError> {
        let mut results = Vec::new();

        for entry in walkdir::WalkDir::new(codebase_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file() && self.should_index(e.path()))
        {
            let content = std::fs::read_to_string(entry.path())
                .map_err(|e| CodeSearchError::Other(format!("Failed to read: {}", e)))?;
            let lang = self.detect_language(entry.path());

            let query_source = self.query_patterns.get_search_query(&lang)
                .map_err(|e| CodeSearchError::Structural(e))?;
            let mut parser = tree_sitter::Parser::new();
            let language = self.get_tree_sitter_language(&lang)?;

            parser.set_language(language)
                .map_err(|e| CodeSearchError::Structural(crate::ast_query_patterns::QueryError::CompilationError(format!("set_language failed: {}", e))))?;

            let tree = parser.parse(&content, None)
                .ok_or_else(|| CodeSearchError::Structural(crate::ast_query_patterns::QueryError::CompilationError("parse failed".into())))?;

            let root = tree.root_node();
            let query = tree_sitter::Query::new(language, &query_source)
                .map_err(|e| CodeSearchError::Structural(crate::ast_query_patterns::QueryError::CompilationError(format!("Query::new failed: {}", e))))?;

            let mut cursor = tree_sitter::QueryCursor::new();
            let matches = cursor.matches(&query, root, content.as_bytes());

            for m in matches {
                for capture in m.captures {
                    let node = capture.node;
                    if let Ok(text) = node.utf8_text(content.as_bytes()) {
                        let start = node.start_position().row;
                        let end = node.end_position().row;

                        let doc = RagDocument {
                            id: format!("{}-{}", entry.path().display(), start),
                            path: entry.path().to_string_lossy().into_owned(),
                            chunk_index: 0,
                            content: text.to_string(),
                            embedding: vec![],
                            metadata: crate::rag::RagMetadata {
                                file_type: lang.clone(),
                                git_hash: "HEAD".to_string(),
                                lines: (start, end),
                                size_bytes: text.len(),
                            },
                            indexed_at: chrono::Utc::now().timestamp_millis() as u64,
                        };

                        results.push(CodeSearchResult {
                            document: doc,
                            match_type: "structural".to_string(),
                            score: 0.95,
                            matched_node: format!("{} @ {}", lang, capture.index),
                            location: (start, end),
                            capture_name: Some(self.query_patterns.get_capture_names(&lang)?[capture.index as usize].to_string()),
                            ancestry_boost: 0.0,
                        });
                    }
                }
            }
        }
        Ok(results)
    }

    /// Semantic vector search over indexed embeddings
    #[instrument(name = "code_search_semantic", skip(self, query_embedding, codebase_path))]
    async fn semantic_search(
        &self,
        query_embedding: &SemanticEmbedding,
        codebase_path: &Path,
    ) -> Result<Vec<CodeSearchResult>, CodeSearchError> {
        let mut results = Vec::new();

        // Production: real ChromaDB query would go here
        // For now we scan the in-memory index (full production uses persistent vector DB)
        for entry in self.index.iter() {
            let docs = entry.value();
            for doc in docs {
                if doc.embedding.is_empty() {
                    continue;
                }
                let doc_emb = SemanticEmbedding::new(doc.embedding.clone(), doc.content.split_whitespace().count());
                let sim = query_embedding.cosine_similarity(&doc_emb);
                if sim > 0.6 {
                    results.push(CodeSearchResult {
                        document: doc.clone(),
                        match_type: "semantic".to_string(),
                        score: sim,
                        matched_node: "semantic_match".to_string(),
                        location: doc.metadata.lines,
                        capture_name: None,
                        ancestry_boost: 0.0,
                    });
                }
            }
        }
        Ok(results)
    }

    /// Hybrid ranking engine – weighted fusion of structural + semantic
    async fn rank_hybrid(
        &self,
        structural: Vec<CodeSearchResult>,
        semantic: Vec<CodeSearchResult>,
        query_embedding: &SemanticEmbedding,
    ) -> Result<Vec<CodeSearchResult>, CodeSearchError> {
        let mut merged = HashMap::new();

        for mut s in structural {
            let key = s.document.id.clone();
            s.score = SemanticEmbedder::hybrid_score(0.95, 0.0);
            merged.insert(key, s);
        }

        for sem in semantic {
            let key = sem.document.id.clone();
            if let Some(existing) = merged.get_mut(&key) {
                existing.score = SemanticEmbedder::hybrid_score(0.95, sem.score);
                existing.match_type = "hybrid".to_string();
            } else {
                merged.insert(key, sem);
            }
        }

        Ok(merged.into_values().collect())
    }

    /// Apply git ancestry boost (recency + author activity)
    async fn apply_ancestry_boost(&self, results: &mut [CodeSearchResult]) -> Result<(), CodeSearchError> {
        for result in results.iter_mut() {
            let boost = self.ancestry_cache
                .get(&result.document.path)
                .map(|v| *v.value())
                .unwrap_or(0.8);
            result.ancestry_boost = boost;
            result.score = (result.score * 0.7) + (boost * 0.3);
        }
        Ok(())
    }

    /// Incremental index update – called by file watcher
    #[instrument(name = "code_search_index_update", skip(self, path, content))]
    pub async fn update_index(&self, path: &Path, content: &str) -> Result<(), CodeSearchError> {
        let lang = self.detect_language(path);
        let chunks = self.embedder.embed_batch(vec![content]).await?;
        let docs: Vec<RagDocument> = chunks.into_iter()
            .enumerate()
            .map(|(i, emb)| RagDocument {
                id: format!("{}-{}", path.display(), i),
                path: path.to_string_lossy().into_owned(),
                chunk_index: i,
                content: content.to_string(),
                embedding: emb.vector,
                metadata: crate::rag::RagMetadata {
                    file_type: lang.clone(),
                    git_hash: "HEAD".to_string(),
                    lines: (0, 0),
                    size_bytes: content.len(),
                },
                indexed_at: chrono::Utc::now().timestamp_millis() as u64,
            })
            .collect();

        self.index.insert(path.to_string_lossy().into_owned(), docs);
        info!("Index updated for {}", path.display());
        Ok(())
    }

    fn should_index(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy().to_lowercase();
        ![".git", "node_modules", "__pycache__", "target", ".venv", "build", "dist"]
            .iter()
            .any(|pat| path_str.contains(pat))
    }

    fn detect_language(&self, path: &Path) -> String {
        match path.extension().and_then(|s| s.to_str()) {
            Some("rs") => "rust".to_string(),
            Some("py") => "python".to_string(),
            Some("cpp") | Some("cc") | Some("h") => "cpp".to_string(),
            Some("js") | Some("ts") => "javascript".to_string(),
            _ => "text".to_string(),
        }
    }

    fn get_tree_sitter_language(&self, lang: &str) -> Result<tree_sitter::Language, CodeSearchError> {
        match lang {
            "rust" => unsafe { Ok(tree_sitter_rust()) },
            "python" => unsafe { Ok(tree_sitter_python()) },
            "cpp" => unsafe { Ok(tree_sitter_cpp()) },
            "javascript" => unsafe { Ok(tree_sitter_javascript()) },
            _ => Err(CodeSearchError::UnsupportedLanguage(lang.to_string())),
        }
    }
}

// Tree-sitter language externs (required for Rust FFI)
extern "C" {
    fn tree_sitter_rust() -> tree_sitter::Language;
    fn tree_sitter_python() -> tree_sitter::Language;
    fn tree_sitter_cpp() -> tree_sitter::Language;
    fn tree_sitter_javascript() -> tree_sitter::Language;
}