// src-rust/ast_query_patterns.rs
use std::collections::HashMap;
use tracing::{instrument, info};
use uuid::Uuid;
use thiserror::Error;

/// Production-grade centralized Tree-sitter query registry for DroxIDE
/// Supports Rust, Python, C++, JavaScript/TypeScript
/// All queries are optimized for semantic chunking + structural search
/// Used by OptimizedChunker, AstSearchEngine, ResearcherAgent, ArchitectAgent

#[derive(Error, Debug)]
pub enum QueryError {
    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),
    #[error("Query compilation failed: {0}")]
    CompilationError(String),
}

#[derive(Clone, Debug)]
pub struct AstQueryPatterns {
    patterns: HashMap<String, LanguagePatterns>,
}

#[derive(Clone, Debug)]
struct LanguagePatterns {
    chunk_query: String,      // For semantic chunking (top-level declarations)
    search_query: String,     // For structural search (functions, structs, calls, etc.)
    capture_names: Vec<&'static str>,
}

impl AstQueryPatterns {
    pub fn new() -> Self {
        let mut patterns = HashMap::new();

        // Rust patterns
        patterns.insert(
            "rust".to_string(),
            LanguagePatterns {
                chunk_query: r#"
                    (function_item) @fn
                    (struct_item) @struct
                    (enum_item) @enum
                    (impl_item) @impl
                    (trait_item) @trait
                    (mod_item) @mod
                    (macro_definition) @macro
                "#.to_string(),
                search_query: r#"
                    (function_item name: (identifier) @name) @fn
                    (struct_item name: (type_identifier) @name) @struct
                    (enum_item name: (type_identifier) @name) @enum
                    (impl_item) @impl
                    (call_expression function: (identifier) @call) @call_expr
                    (attribute_item) @attr
                "#.to_string(),
                capture_names: vec!["fn", "struct", "enum", "impl", "trait", "mod", "macro", "name", "call", "call_expr", "attr"],
            },
        );

        // Python patterns
        patterns.insert(
            "python".to_string(),
            LanguagePatterns {
                chunk_query: r#"
                    (function_definition) @fn
                    (class_definition) @class
                    (decorated_definition) @decorated
                "#.to_string(),
                search_query: r#"
                    (function_definition name: (identifier) @name) @fn
                    (class_definition name: (identifier) @name) @class
                    (call) @call
                "#.to_string(),
                capture_names: vec!["fn", "class", "decorated", "name", "call"],
            },
        );

        // C++ patterns
        patterns.insert(
            "cpp".to_string(),
            LanguagePatterns {
                chunk_query: r#"
                    (function_definition) @fn
                    (class_specifier) @class
                    (struct_specifier) @struct
                    (namespace_definition) @ns
                "#.to_string(),
                search_query: r#"
                    (function_definition declarator: (function_declarator declarator: (identifier) @name)) @fn
                    (class_specifier name: (type_identifier) @name) @class
                    (call_expression) @call
                "#.to_string(),
                capture_names: vec!["fn", "class", "struct", "ns", "name", "call"],
            },
        );

        // JavaScript / TypeScript patterns
        patterns.insert(
            "javascript".to_string(),
            LanguagePatterns {
                chunk_query: r#"
                    (function_declaration) @fn
                    (class_declaration) @class
                    (method_definition) @method
                    (arrow_function) @arrow
                "#.to_string(),
                search_query: r#"
                    (function_declaration name: (identifier) @name) @fn
                    (class_declaration name: (identifier) @name) @class
                    (call_expression) @call
                    (method_definition name: (property_identifier) @name) @method
                "#.to_string(),
                capture_names: vec!["fn", "class", "method", "arrow", "name", "call"],
            },
        );

        AstQueryPatterns { patterns }
    }

    #[instrument(name = "query_patterns_get_chunk_query", skip(self), fields(trace_id = %Uuid::new_v4()))]
    pub fn get_chunk_query(&self, language: &str) -> Result<&str, QueryError> {
        self.patterns.get(language)
            .map(|p| p.chunk_query.as_str())
            .ok_or_else(|| QueryError::UnsupportedLanguage(language.to_string()))
    }

    #[instrument(name = "query_patterns_get_search_query", skip(self), fields(trace_id = %Uuid::new_v4()))]
    pub fn get_search_query(&self, language: &str) -> Result<&str, QueryError> {
        self.patterns.get(language)
            .map(|p| p.search_query.as_str())
            .ok_or_else(|| QueryError::UnsupportedLanguage(language.to_string()))
    }

    pub fn get_capture_names(&self, language: &str) -> Result<&[&'static str], QueryError> {
        self.patterns.get(language)
            .map(|p| p.capture_names.as_slice())
            .ok_or_else(|| QueryError::UnsupportedLanguage(language.to_string()))
    }

    #[instrument(name = "query_patterns_supported_languages", skip(self), fields(trace_id = %Uuid::new_v4()))]
    pub fn supported_languages(&self) -> Vec<String> {
        self.patterns.keys().cloned().collect()
    }
}