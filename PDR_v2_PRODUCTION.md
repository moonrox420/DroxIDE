# DroxIDE – Professional Design Review v2 (Production-Grade)
**Project:** DroxIDE – Native Desktop AI-Powered IDE  
**Version:** 2.0 (Production)  
**Prepared For:** Dusti – Legendary Engineer Mode  
**Date:** April 03, 2026  
**Status:** Ready for Phase 1 Implementation

---

## 1. Executive Summary

DroxIDE is a **native Qt 6 desktop IDE** with an embedded Rust swarm orchestrator, local LLM inference (llama.cpp), and enterprise-grade RAG + sandboxing. Single executable (.exe/.app/.deb), zero cloud, zero hallucinations through grounding + HITL.

**Key Claims:**
- **30+ min/day saved** (automation + swarm)
- **<5% hallucinations** (RAG grounding + ancestry)
- **>70% code acceptance** (LSP + sandbox validation)
- **Native feel** (Qt 6 full menu bar, real terminals, drag-drop)
- **Enterprise-ready** (audit logs, HITL, sandbox, metrics)

---

## 2. System Architecture

### 2.1 High-Level Component Model

```
┌──────────────────────────────────────────────────────────────────┐
│                        Qt 6 UI Layer (C++)                        │
├──────────┬──────────┬──────────┬─────────────┬──────────┬─────────┤
│MenuBar   │Editor    │Terminal  │Agent Trace  │RAG Heat  │Settings │
│          │(Scintilla│(QTermW   │Panel        │map       │Dialog   │
│          │+ LSP)    │+ tabs)   │             │          │         │
└──────────┴──────────┴──────────┴─────────────┴──────────┴─────────┘
              │
              │ cxx-qt bridge
              ▼
┌──────────────────────────────────────────────────────────────────┐
│                   Rust Core (Agent Swarm)                         │
├────────────────┬──────────────┬──────────────┬──────────────────┤
│FSM Orchestrator│Agent Spawner │Event Queue   │State Persistence │
│(7 agents)      │(threads)     │(crossbeam)   │(serde)           │
└────────────────┴──────────────┴──────────────┴──────────────────┘
   │                │                │               │
   ├─▶ Researcher   ├─▶ Coder        ├─▶ Reviewer   ├─▶ Tester
   ├─▶ Janitor      ├─▶ Architect    └─▶ Orchestrator
   
         │
         │ FFI
         ▼
┌──────────────────────────────────────────────────────────────────┐
│                 llama.cpp Sidecar (Binary)                        │
├──────────────────────────────────────────────────────────────────┤
│Inference Engine (GGUF pooling, Vulkan/Metal/CPU)                 │
└──────────────────────────────────────────────────────────────────┘

         │
         │ Folder watch / USB
         ▼
┌──────────────────────────────────────────────────────────────────┐
│                    RAG Pipeline (Rust)                            │
├──────────────┬──────────────┬──────────────┬──────────────────────┤
│Folder Watch  │OptimizedChunk│llama.cpp     │ChromaDB Persist     │
│(notify-rs)   │er (Tree-sit) │Embed Pool    │(local .db)          │
└──────────────┴──────────────┴──────────────┴──────────────────────┘

         │
         │ API
         ▼
┌──────────────────────────────────────────────────────────────────┐
│              Docker Engine API (Sandbox + Tests)                  │
├──────────┬──────────────┬──────────────┬──────────────────────────┤
│Shadow Sim│Test Runner   │LSP Sandbox   │Security (seccomp)       │
└──────────┴──────────────┴──────────────┴──────────────────────────┘
```

### 2.2 Data Flow – Swarm Request Lifecycle

```
User Input (Menu / Prompt)
    │
    ▼
┌─────────────────────────────────────────────────────────────┐
│ Orchestrator FSM (Rust)                                     │
│ State: IDLE → RESEARCHING → PRUNING → SHADOW → CODING →   │
│        REVIEW → HITL → APPLYING → DONE                     │
└─────────────────────────────────────────────────────────────┘
    │
    ├─▶ Researcher Agent (Parallel)
    │   ├─ Query RAG (ChromaDB)
    │   ├─ Fetch Git ancestry (git2-rs)
    │   └─ Score context (TF-IDF)
    │
    ├─▶ Architect Agent (Parallel)
    │   ├─ Analyze codebase (Tree-sitter)
    │   ├─ Infer patterns
    │   └─ Draft approach
    │
    ├─▶ Coder Agent (Waits for context)
    │   ├─ Query llama.cpp (streaming)
    │   ├─ Generate diff
    │   └─ Sandbox test (Docker)
    │
    ├─▶ Reviewer Agent (Waits for code)
    │   ├─ Check LSP errors
    │   ├─ Compare ancestry
    │   └─ Risk score
    │
    ├─▶ Tester Agent (Waits for review)
    │   ├─ Run pytest in sandbox
    │   ├─ Coverage report
    │   └─ Pass/fail verdict
    │
    ├─▶ Janitor Agent (Async, background)
    │   ├─ Re-embed misses
    │   └─ Prune old RAG entries
    │
    └─▶ HITL Checkpoint (UI blocks)
        ├─ User accepts → APPLYING
        ├─ User rejects → Janitor learns
        └─ 3+ rejects → Escalate
    │
    ▼
Applied Diff → Git commit (optional) → Metrics log
```

### 2.3 Data Contracts (Protocol Buffers / Serde)

#### Agent Message (Rust → Qt)

```rust
#[derive(Serialize, Deserialize)]
pub struct AgentMessage {
    pub agent_id: String,           // "researcher", "coder", etc.
    pub state: AgentState,          // enum: IDLE, PROCESSING, DONE, ERROR
    pub step: String,               // "Querying RAG", "Generating code", etc.
    pub progress: f32,              // 0.0 - 1.0
    pub payload: serde_json::Value, // Flexible data
    pub timestamp: u64,             // Unix ms
    pub trace_id: String,           // Audit trail
}

pub enum AgentState {
    Idle,
    Processing,
    Done,
    Error(String),
}
```

#### RAG Document (ChromaDB)

```rust
#[derive(Serialize, Deserialize)]
pub struct RagDocument {
    pub id: String,
    pub path: String,
    pub chunk_index: usize,
    pub content: String,
    pub embedding: Vec<f32>,       // 768-dim (Sentence-Transformers)
    pub metadata: RagMetadata,
    pub indexed_at: u64,
}

pub struct RagMetadata {
    pub file_type: String,         // "rust", "python", "md"
    pub git_hash: String,          // Last commit touching this file
    pub lines: (usize, usize),     // Start, end
    pub size_bytes: usize,
}
```

#### Diff & Sandbox Result

```rust
#[derive(Serialize, Deserialize)]
pub struct DiffRequest {
    pub file_path: String,
    pub before: String,
    pub after: String,
    pub metadata: DiffMetadata,
}

pub struct DiffMetadata {
    pub agent: String,
    pub reason: String,
    pub confidence: f32,           // 0.0 - 1.0
    pub lsp_errors_before: usize,
    pub lsp_errors_after: usize,
}

#[derive(Serialize, Deserialize)]
pub struct SandboxResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
    pub container_id: String,
    pub tests_passed: usize,
    pub tests_failed: usize,
}
```

---

## 3. UI Specification (Qt 6)

### 3.1 Menu Bar (100% Functional)

```
File
  ├─ New File (Ctrl+N)
  ├─ New Folder
  ├─ Open Folder (Ctrl+K Ctrl+O)
  ├─ Open Recent (submenu)
  ├─ Save All (Ctrl+Shift+S)
  ├─ Close Tab (Ctrl+W)
  ├─ Preferences (Ctrl+,)
  └─ Exit (Ctrl+Q)

Edit
  ├─ Undo (Ctrl+Z)
  ├─ Redo (Ctrl+Shift+Z)
  ├─ Cut (Ctrl+X)
  ├─ Copy (Ctrl+C)
  ├─ Paste (Ctrl+V)
  ├─ Find (Ctrl+F)
  ├─ Find & Replace (Ctrl+H)
  ├─ Find in Files (Ctrl+Shift+F)
  └─ Refactor (Swarm-aided)

View
  ├─ Explorer (Ctrl+B)
  ├─ Editor (Ctrl+E)
  ├─ Terminal (Ctrl+`)
  ├─ Agent Trace (Ctrl+Shift+A)
  ├─ RAG Heatmap (Ctrl+Shift+R)
  ├─ Toggle Sidebar
  ├─ Toggle Panel
  ├─ Zoom In (Ctrl++)
  └─ Zoom Out (Ctrl+-)

Terminal
  ├─ New Terminal (Ctrl+Shift+`)
  ├─ Shell Type (submenu)
  │   ├─ Git Bash
  │   ├─ PowerShell
  │   ├─ CMD
  │   └─ Custom...
  ├─ Kill Terminal
  └─ Clear

Git
  ├─ Commit (Ctrl+Shift+C)
  ├─ Push (Ctrl+Shift+P)
  ├─ Pull (Ctrl+Shift+L)
  ├─ Branch (Ctrl+Shift+B)
  ├─ Stash
  ├─ Blame (on editor line)
  └─ Status

Tools
  ├─ Run Swarm (Ctrl+Shift+S)
  ├─ Voice Dictate (Ctrl+Shift+V)
  ├─ Sandbox Test (Ctrl+Shift+T)
  ├─ Clear RAG Index
  └─ Audit Logs

Help
  ├─ Documentation
  ├─ Keyboard Shortcuts (Ctrl+K Ctrl+S)
  ├─ Agent Guide
  ├─ Debug Logs
  └─ About DroxIDE
```

### 3.2 Main Layout

```
┌─────────────────────────────────────────────────────────────────────┐
│ File Edit View Terminal Git Tools Help                              │
├──────────────────────────────────────────────────────────────────────┤
│ [Open Folder] [Search] [Run Swarm] [Voice] | [Settings] [User Menu] │
├─────────┬────────────────────────────────────────┬──────────────────┤
│         │ main.rs [x]                            │ RAG Heatmap      │
│ Explorer│ utils.rs [x]                           │ ─────────────    │
│ ─────── │ config.toml [x]                        │ [Refresh] [>]    │
│         │ ────────────────────────────────────── │                  │
│ [+] src │ ┌─────────────────────────────────────┐│ ■ Chunk 1: 89%   │
│ [>] lib │ │ fn main() {                          ││ ■ Chunk 2: 76%   │
│ [>] bin │ │   println!("Hello");                 ││ ■ Chunk 3: 45%   │
│ [>] test│ │   // AGENT CURSOR HERE ◄──┐          ││ □ Chunk 4: 12%   │
│ [+] cfg │ │ }                          │          ││ □ Chunk 5: 5%    │
│         │ │                            │          ││ More...          │
│ .git    │ │ >>> Swarm ready            │          ││                  │
│ .gitign │ │                            │          ││ ─────────────    │
│ Cargo.t │ │ [Run Swarm] [Trace] [HITL]│          ││ Agent Trace      │
│ README  │ │                            │          ││ ─────────────    │
│         │ └─────────────────────────────────────┘│ ┌─────────────┐   │
│         │                                         │ │ Researcher  │   │
│ ──────  │ 50:20 Ln 5 Col 12 | Rust | UTF-8  │   │ ├─────────────┤   │
├─────────┼────────────────────────────────────────┤ │ ✓ RAG query │   │
│ Main    │ ┌─────────────────────────────────────┐│ │ ✓ Ancestry  │   │
│ [x] Git │ │ $ cargo test                         ││ │ ✓ 3 docs    │   │
│ [x] Bash│ │ running 5 tests                      ││ │             │   │
│ [x] Pwsh│ │ test result: ok. 5 passed; 0 failed ││ │ Architect   │   │
│         │ │                                      ││ │ ├─────────────┤   │
│         │ │ $ _                                  ││ │ ⏳ Analyzing    │   │
│ ──────  │ │                                      ││ │             │   │
│ All [✓] │ │                                      ││ │ Coder       │   │
│ Output  │ │                                      ││ │ ├─────────────┤   │
│ Problems│ │                                      ││ │ ⏳ Streaming    │   │
│ Debug   │ │                                      ││ │ 2/150 tokens   │   │
│ Tests   │ │                                      ││ │             │   │
│         │ └─────────────────────────────────────┘│ │ [ACCEPT] [REJECT] │
│         │                                         │ └─────────────┘   │
└─────────┴────────────────────────────────────────┴──────────────────┘
```

### 3.3 Dialogs & Popups

#### Run Swarm Dialog

```
┌──────────────────────────────────────────────┐
│ Run Swarm                                [x] │
├──────────────────────────────────────────────┤
│ Prompt:                                      │
│ ┌──────────────────────────────────────────┐ │
│ │ "Refactor main.rs to use async/await"   │ │
│ └──────────────────────────────────────────┘ │
│                                              │
│ Context:                                     │
│ ☑ Current file (main.rs)                    │
│ ☑ Folder (src/)                             │
│ ☑ Git history (last 10 commits)             │
│ ☑ Dependencies (Cargo.toml)                 │
│                                              │
│ RAG Filters:                                 │
│ [Tag Filter...] [Date Range...]             │
│                                              │
│ HITL:                                        │
│ ☑ Show trace                                │
│ ☑ Block on review                           │
│ ☑ Auto-apply if confidence >90%             │
│                                              │
│ [Cancel]  [Run]                             │
└──────────────────────────────────────────────┘
```

#### Commit Dialog (Git Menu)

```
┌──────────────────────────────────────────────┐
│ Git Commit                                 [x] │
├──────────────────────────────────────────────┤
│ Files to commit: 3                           │
│ ☑ main.rs                                    │
│ ☑ Cargo.toml                                 │
│ ☑ src/lib.rs                                 │
│                                              │
│ Message:                                     │
│ ┌──────────────────────────────────────────┐ │
│ │ Refactor main logic to use async/await  │ │
│ │                                          │ │
│ │ - Rewrote handler to async fn           │ │
│ │ - Added tokio runtime                   │ │
│ │ - Tests pass locally                    │ │
│ └──────────────────────────────────────────┘ │
│                                              │
│ [Cancel]  [Commit]                          │
└──────────────────────────────────────────────┘
```

---

## 4. Rust Core Specification

### 4.1 Agent FSM & States

```rust
pub enum OrchestratorState {
    Idle,
    Researching,    // Researcher + Architect active
    Pruning,        // Analyze context, score docs
    Shadow,         // Prepare Docker image
    Coding,         // Coder generates diff
    Review,         // Reviewer validates
    Hitl,           // User accepts/rejects
    Applying,       // Write to disk, commit
    Done,
    Error(String),
}

pub struct Orchestrator {
    state: Arc<Mutex<OrchestratorState>>,
    agents: HashMap<String, AgentHandle>,
    event_queue: crossbeam::channel::Sender<AgentEvent>,
    metrics: Arc<Metrics>,
}

pub enum AgentEvent {
    ResearcherDone(ResearcherOutput),
    ArchitectDone(ArchitectOutput),
    CoderDone(CoderOutput),
    ReviewerDone(ReviewerOutput),
    TesterDone(TesterOutput),
    UserAction(UserAction),
    Timeout(String),
}

pub enum UserAction {
    Accept,
    Reject,
    RejectWithFeedback(String),
}
```

### 4.2 RAG Pipeline (Rust)

```rust
pub struct RagPipeline {
    watcher: notify::Watcher,               // Folder watch
    chunker: OptimizedChunker,              // Tree-sitter chunks
    llm_pool: LlamaPool,                    // Embedding pool
    db: ChromaDB,                           // Local persistence
}

impl RagPipeline {
    pub async fn ingest_folder(&self, path: &Path) {
        // 1. Watch folder for changes
        // 2. Extract file tree (ignore .git, node_modules, etc.)
        // 3. Chunk by semantic boundaries (functions, classes)
        // 4. Embed via llama.cpp (parallel pool)
        // 5. Persist to ChromaDB
    }

    pub async fn query(&self, prompt: &str, limit: usize) -> Vec<RagDocument> {
        // 1. Embed query via llama.cpp
        // 2. Search ChromaDB (cosine similarity)
        // 3. Ancestry boost (git log recency)
        // 4. Return top-k docs + relevance scores
    }
}

pub struct OptimizedChunker {
    tree_sitter: Parser,
    language: Language,
}

impl OptimizedChunker {
    pub fn chunk(&self, code: &str, lang: &str) -> Vec<String> {
        // Parse AST, split by:
        // - Top-level functions/classes
        // - Nested blocks (max 100 lines per chunk)
        // - Keep context (imports, types)
        // - Overlap windows (20% context)
    }
}
```

### 4.3 Docker Sandbox API (Rust)

```rust
pub struct Sandbox {
    docker_client: docker::Client,
}

impl Sandbox {
    pub async fn new_image(&self, dockerfile: &str) -> Result<String> {
        // Build Docker image with:
        // - Base: python:3.11-slim or rust:1.75-slim
        // - Copy project + dependencies
        // - Seccomp profile (drop CAP_SYS_ADMIN, CAP_NET_ADMIN)
        // - Resource limits (2GB RAM, 4 CPU)
        // - Read-only FS (except /tmp, /output)
    }

    pub async fn run_test(&self, image: &str, cmd: &str) -> SandboxResult {
        // 1. Spawn container from image
        // 2. Run pytest / cargo test / npm test
        // 3. Capture stdout, stderr, exit code
        // 4. Timeout: 30s
        // 5. Kill on timeout
        // 6. Return result + container logs
    }

    pub async fn shadow_sim(&self, diff: &str) -> ShadowResult {
        // 1. Clone repo to temp dir
        // 2. Apply diff
        // 3. Run linter + type check (Pyright, rust-analyzer)
        // 4. Run unit tests
        // 5. Return: errors, warnings, test results, duration
    }
}

pub struct ShadowResult {
    pub lsp_errors: Vec<String>,
    pub lint_warnings: Vec<String>,
    pub tests_passed: usize,
    pub tests_failed: usize,
    pub duration_ms: u64,
}
```

### 4.4 Metrics & Audit (Rust)

```rust
pub struct Metrics {
    pub prompts_total: AtomicU64,
    pub hallucinations: AtomicU64,
    pub accepted: AtomicU64,
    pub rejected: AtomicU64,
    pub time_saved_minutes: AtomicU64,
    pub avg_latency_ms: AtomicU64,
}

pub struct AuditLog {
    pub event: String,                 // "swarm_started", "user_accepted", etc.
    pub timestamp: u64,
    pub user: String,
    pub agent: String,
    pub input: serde_json::Value,
    pub output: serde_json::Value,
    pub duration_ms: u64,
    pub trace_id: String,
}

impl AuditLog {
    pub fn write_to_file(&self, path: &Path) {
        // Append JSON line to audit.jsonl
        // Rotate when >100MB
        // Compress old logs (gzip)
    }
}
```

---

## 5. llama.cpp Integration

### 5.1 Sidecar Process

- **Binary:** `llama-server` bundled in `$APPDIR/bin/llama-cpp/`
- **Model:** Bundled `model.gguf` (e.g., Mistral 7B Q4, 4GB)
- **Launch:** On app start, auto-spawn with `--port 8080`
- **Protocol:** HTTP REST (compatible with OpenAI API)
- **Pooling:** 4 parallel inference slots (configurable)

### 5.2 Rust FFI Wrapper

```rust
pub struct LlamaPool {
    base_url: String,                   // http://127.0.0.1:8080
    pool_size: usize,
    active_slots: Arc<Mutex<Vec<InferenceSlot>>>,
}

impl LlamaPool {
    pub async fn complete(&self, prompt: &str) -> String {
        // POST /v1/completions
        // Stream tokens, return full response
    }

    pub async fn embed(&self, text: &str) -> Vec<f32> {
        // POST /v1/embeddings
        // Return 768-dim embedding
    }

    pub async fn health_check(&self) -> bool {
        // GET /health
    }
}
```

---

## 6. Success Criteria & Acceptance Tests

### 1 (UI + RAG + Basic Swarm)

- [ ] Qt app launches (native menu bar, no console)
- [ ] Editor opens file, syntax highlighting works
- [ ] Terminal tabs spawn (Git Bash, PowerShell, CMD)
- [ ] Folder watch ingests code, ChromaDB persists
- [ ] RAG query returns top-5 docs with scores
- [ ] Swarm prompt triggers Researcher → Coder → output (no sandbox yet)
- [ ] Metrics logged to audit.jsonl
- [ ] <2s latency (research + generation)

### 2 (Sandbox + Full Swarm)

- [ ] Docker sandbox builds shadow image
- [ ] Diff applied, tests run in sandbox
- [ ] Review agent validates LSP errors
- [ ] HITL dialog blocks, user can accept/reject
- [ ] Rejected diffs logged, Janitor re-embeds
- [ ] 3+ rejects → escalate (toast notification)
- [ ] Metrics: hallucination rate <5%, acceptance >70%

### 3 (Voice + Ancestry + Polishing)

- [ ] Voice dictate (Whisper.cpp) → prompt
- [ ] Git blame integrated into editor
- [ ] Ancestry RAG boosts recent edits
- [ ] Janitor background task active
- [ ] LSP (Pyright, tsserver) in sandbox
- [ ] Build/bundle complete, installers sign

---

## 7. Risk Mitigation Table

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|-----------|
| RAG empty → hallucinations | High | High | Show "no context" warning, HITL mandatory for new files |
| Sandbox escape (Docker) | Critical | Low | Seccomp + read-only FS + resource limits; regular security audit |
| Qt compile bloat | Medium | Medium | Static link, UPX compress, slim build (strip symbols) |
| llama.cpp OOM | High | Medium | Monitor vram, auto-fallback to CPU, swap warnings |
| Terminal compat (Windows) | Medium | Medium | QTermWidget + bundled Git Bash / PowerShell |
| Network isolation | Medium | Low | Sandbox container no network (--net=none by default) |
| Model drift | High | Medium | Grammar constraints, output validation, test suite |

---

## 8. Roadmap

### Step 1
- Qt 6 app (menu bar, editor, terminals)
- Folder watcher, chunker, ChromaDB setup
- Basic Swarm (Researcher, Coder agents)
- Metrics logging

### Step 2
- Docker sandbox API
- Reviewer, Tester, Janitor agents
- HITL checkpoint
- LSP integration (basic)

### Step 3
- Voice (Whisper.cpp)
- Git ancestry boost
- LSP (Pyright, tsserver in sandbox)
- Build/bundle, signing

### Step 4
- Security audit
- Performance tuning
- Installer (NSIS, DMG, deb)
- Beta launch

---

## 9. Deployment & Bundling

**Deliverables:**
- `DroxIDE-x.x.x-Setup.exe` (Windows, signed)
- `DroxIDE-x.x.x.dmg` (macOS, notarized)
- `droxide_x.x.x_amd64.deb` (Linux)

**Bundle Contents:**
- Qt 6 libraries (dynamic)
- Rust runtime
- llama-cpp binary + model.gguf
- ChromaDB + dependencies
- Git Bash (Windows only)
- License + docs

**Auto-Update:** Check for updates on startup, download + install (user prompted).

---

## 10. Security & Compliance

- **Audit Logs:** All user actions + agent decisions logged to `~/.droxide/audit.jsonl`
- **Sandbox:** Docker container (seccomp, read-only FS, no network)
- **Data:** No cloud, no telemetry (local .db only)
- **Code Signing:** Self-signed cert (or CA) on binaries
- **GDPR:** No PII collected; user controls RAG data (can delete anytime)

---


