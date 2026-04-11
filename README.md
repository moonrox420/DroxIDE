# DroxIDE

**A native desktop AI-powered IDE with a local swarm orchestrator, RAG, and sandboxing.**

---

## Overview

DroxIDE is a **native Qt 6 desktop application** (not a web app) with an embedded **Rust core** that powers:

- **Swarm Orchestration** — 7 AI agents coordinated via a 9-state FSM
- **Local RAG** — Tree-sitter AST chunking, embedding, ChromaDB persistence
- **Docker Sandbox** — Isolated code execution with seccomp and resource limits
- **HITL Checkpoint** — Human-in-the-loop accept/reject for every code change
- **Audit & Metrics** — JSONL logging, atomic counters, full traceability

---

## Quick Start

### Prerequisites

- **Rust** (1.75+) — `rustup install stable`
- **CMake** (3.20+) — for full Qt build
- **Qt 6** (optional) — for desktop UI build
- **Docker** (optional) — for sandbox testing

### Build Rust Core

```powershell
cargo build --lib       # Debug
cargo build --release   # Release
cargo test --lib        # 37 tests
cargo clippy --lib -- -D warnings  # Lint check
```

### Build Full Desktop App (requires Qt 6)

```powershell
.\build-windows.ps1
```

---

## Architecture

```
┌──────────────────────────────────────────────┐
│              Qt 6 UI (C++)                    │
│  Menus · Editor · Terminal · Explorer · Panels│
├──────────────────────────────────────────────┤
│              cxx-qt FFI Bridge                │
├──────────────────────────────────────────────┤
│              Rust Core                        │
│  ┌─────────────┐  ┌──────┐  ┌─────────────┐  │
│  │ Orchestrator│  │ RAG  │  │  Sandbox    │  │
│  │   (FSM)     │  │Pipeline│  │  (Docker)  │  │
│  └─────────────┘  └──────┘  └─────────────┘  │
│  ┌─────────────┐  ┌──────┐  ┌─────────────┐  │
│  │  7 Agents   │  │Metrics│  │   Git       │  │
│  │             │  │ Audit │  │  (git2-rs)  │  │
│  └─────────────┘  └──────┘  └─────────────┘  │
├──────────────────────────────────────────────┤
│         llama.cpp · Docker Engine             │
└──────────────────────────────────────────────┘
```

### Swarm Workflow

```
User Prompt → Researcher → Architect → Pruning → Shadow →
Coder → Reviewer → [HITL: Accept/Reject] → Apply → Done
```

---

## Project Structure

```
DroxIDE/
├── src-rust/            Rust core library
│   ├── lib.rs           FFI bridge for Qt integration
│   ├── orchestrator.rs  9-state FSM controller
│   ├── agent.rs         7 agent types with contracts
│   ├── rag.rs           RAG pipeline (tree-sitter + embeddings)
│   ├── sandbox.rs       Docker sandbox (bollard)
│   ├── llama.rs         llama.cpp inference pool
│   ├── metrics.rs       Atomic counters
│   ├── audit.rs         JSONL audit logging
│   ├── git.rs           Git operations
│   ├── vector_store.rs  ChromaDB vector store
│   ├── ast_search.rs    AST structural search
│   ├── code_search_engine.rs  Hybrid code search
│   └── hnsw_tuner.rs    HNSW parameter auto-tuning
├── src/                 Qt 6 C++ UI layer
│   ├── mainwindow.h/cpp Main window with full menu bar
│   ├── editor/          Multi-tab editor + syntax highlighting
│   ├── terminal/        Real terminal tabs (PTY)
│   ├── explorer/        File tree with watcher
│   ├── panels/          Agent trace + RAG heatmap
│   ├── dialogs/         Run Swarm, Commit, Preferences
│   └── git/             Git manager (C++ side)
├── docker/              Sandbox Dockerfiles
│   ├── Dockerfile.py3.11
│   ├── Dockerfile.rust1.75
│   └── Dockerfile.node20
└── docs/
    ├── README.md             This file
    ├── BUILD_STATUS.md       Current build status
    ├── PDR_v2_PRODUCTION.md  Full architecture document
    ├── BUILD.md              Build & deployment guide
    └── QUICKSTART.md         Integration guide
```

---

## Build Status

| Check | Status |
|-------|--------|
| `cargo build --lib` | ✅ Zero errors, zero warnings |
| `cargo test --lib` | ✅ 37/37 passing |
| `cargo clippy -- -D warnings` | ✅ Zero lint warnings |
| `cargo build --release` | ✅ Optimized release build |

See [BUILD_STATUS.md](BUILD_STATUS.md) for details.

---

## Key Features

### Native Desktop
- Full menu bar (File, Edit, View, Terminal, Git, Tools, Help)
- Real terminal tabs with PTY (not fake JS shells)
- True file explorer with filesystem watcher
- Syntax highlighting for Rust, Python, C++, JavaScript/TypeScript

### Swarm Orchestration
- **Researcher** — RAG query, doc scoring, ancestry boost
- **Architect** — Codebase analysis, pattern inference
- **Coder** — LLM generation, diff production
- **Reviewer** — LSP validation, risk scoring
- **Tester** — Sandbox execution, test results
- **Janitor** — Re-embedding, RAG maintenance

### Enterprise Guardrails
- HITL checkpoint: every code change requires user approval
- Audit logging: every event in immutable JSONL format
- Docker sandbox: seccomp, read-only FS, resource limits
- Metrics: 6 atomic counters (prompts, hallucinations, accepted, rejected, time saved, latency)

---

## License

See [LICENSE](LICENSE) for details.

---

*DroxIDE: A real native IDE. Not a web hack. Built right.*
