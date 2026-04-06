# Changelog

All notable changes to DroxIDE will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial project structure with Qt 6 UI and Rust core
- 7-agent swarm orchestration system (Researcher, Architect, Coder, Reviewer, Tester, Janitor)
- FSM-based orchestrator with 9 states (Idle → Researching → Pruning → Shadow → Coding → Review → HITL → Applying → Done)
- Semantic RAG pipeline with tree-sitter AST chunking
- Vector store abstraction with ChromaDB integration
- Docker sandbox support for Python 3.11, Rust 1.75, and Node.js 20
- Metrics tracking system with atomic counters (prompts, hallucinations, acceptances, rejections)
- Immutable audit logging (JSONL format)
- Git integration via git2-rs
- Dark theme stylesheet for modern UI
- Qt resource file with icon placeholders
- Comprehensive unit tests for core Rust modules
- `.gitignore` for clean repository management
- MIT License for open-source distribution
- CI/CD pipeline via GitHub Actions

### Changed
- Unified Docker client to bollard (removed duplicate docker-api crate)
- Version alignment between README (v1.0.0) and CMakeLists.txt (v1.0.0)

### Known Issues
- llama.cpp FFI integration requires manual sidecar setup
- LSP client is placeholder; full JSON-RPC integration pending
- Terminal PTY integration incomplete on Windows
- Tree-sitter semantic chunking needs language grammar compilation

## [1.0.0] - 2026-04-05

### Initial Release
- Production skeleton with UI + Core architecture ready
- Qt 6 desktop application with full menu bar, editor, terminal, file explorer
- Rust swarm orchestrator with state machine
- Local RAG pipeline design (no cloud dependencies)
- Enterprise guardrails (HITL checkpoint, audit logging, Docker sandbox)

---

[Unreleased]: https://github.com/droxide/droxide/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/droxide/droxide/releases/tag/v1.0.0
