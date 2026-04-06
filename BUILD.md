# DroxIDE – Build & Deployment Guide

## Prerequisites

- **Qt 6.5+** (libraries + dev tools)
- **CMake 3.20+**
- **Rust 1.75+** (with cargo)
- **Docker** (for sandbox tests)
- **llama.cpp** sidecar binary pre-built
- **Python 3.10+** (for llama.cpp server)

### Ubuntu/Debian

```bash
sudo apt-get update
sudo apt-get install -y \
    build-essential cmake ninja-build \
    qt6-base-dev qt6-tools-dev \
    rustc cargo \
    docker.io git
```

### macOS

```bash
brew install cmake ninja qt@6 rust docker
```

### Windows (MSVC + Visual Studio)

- Visual Studio 2022 Community (MSVC toolchain)
- CMake 3.20+
- Rust (via `rustup-init.exe`)
- Docker Desktop

---

## Build Steps

### 1. Clone & Init

```bash
git clone <repo> DroxIDE
cd DroxIDE
git submodule update --init --recursive
```

### 2. Build Rust Core

```bash
cd DroxIDE
cargo build --release
```

Output: `target/release/libdroxide_rust.so` (Linux) / `.dylib` (macOS) / `.dll` (Windows)

### 3. Prepare llama.cpp

Download pre-built `llama-server` binary:

```bash
mkdir -p bin/llama-cpp
# Download or build llama-cpp
# On Ubuntu:
curl -L https://github.com/ggerganov/llama.cpp/releases/download/b3255/llama-server-linux-x64 \
  -o bin/llama-cpp/llama-server
chmod +x bin/llama-cpp/llama-server
```

Download model:

```bash
curl -L https://huggingface.co/TheBloke/Mistral-7B-Instruct-v0.2-GGUF/resolve/main/mistral-7b-instruct-v0.2.Q4_K_M.gguf \
  -o bin/llama-cpp/model.gguf
```

### 4. Configure CMake

```bash
cmake -B build -G Ninja \
  -DCMAKE_BUILD_TYPE=Release \
  -DCMAKE_PREFIX_PATH=/usr/lib/cmake/Qt6 \
  -DCORROSION_RUST_TOOLCHAIN=stable
```

**macOS:**
```bash
cmake -B build -G Ninja \
  -DCMAKE_BUILD_TYPE=Release \
  -DCMAKE_PREFIX_PATH=$(brew --prefix qt)/lib/cmake
```

### 5. Build Qt App

```bash
cmake --build build --config Release -j$(nproc)
```

Output: `build/DroxIDE` (Linux) / `build/DroxIDE.app` (macOS) / `build/Release/DroxIDE.exe` (Windows)

### 6. Run

```bash
# Linux
./build/DroxIDE

# macOS
open build/DroxIDE.app

# Windows
.\build\Release\DroxIDE.exe
```

---

## Cross-Platform Bundling

### Linux (.deb)

```bash
cpack -G DEB -C Release
# Outputs: `droxide_1.0.0_amd64.deb`
```

Install:
```bash
sudo dpkg -i droxide_1.0.0_amd64.deb
droxide
```

### macOS (.dmg)

```bash
cpack -G DragNDrop -C Release
# Outputs: `DroxIDE-1.0.0.dmg`
```

### Windows (.exe Installer)

```bash
cpack -G NSIS -C Release
# Outputs: `DroxIDE-1.0.0-Setup.exe`
```

---

## Docker-Based Sandbox Setup

### Build Sandbox Image

The app auto-builds sandbox Docker images on first use. Pre-warm them:

```bash
# For Python projects
docker pull python:3.11-slim
docker build -f docker/Dockerfile.py3.11 -t droxsandbox:py3.11 .

# For Rust projects
docker pull rust:1.75-slim
docker build -f docker/Dockerfile.rust1.75 -t droxsandbox:rust1.75 .

# For Node.js projects
docker pull node:20-slim
docker build -f docker/Dockerfile.node20 -t droxsandbox:node20 .
```

---

## Configuration Files

### ~/.droxide/config.toml

```toml
[ui]
theme = "dark"
font_size = 11
auto_save = true

[rag]
pool_size = 4
top_k = 5
model_path = "~/.droxide/models/model.gguf"

[sandbox]
enable = true
timeout_seconds = 30
memory_limit_mb = 2048
cpu_limit = 4
network_enabled = false

[git]
auto_commit = false
push_on_accept = false

[metrics]
log_path = "~/.droxide/audit.jsonl"
```

### Audit Logs

Location: `~/.droxide/audit.jsonl`

Format (JSONL, one event per line):

```json
{"event":"swarm_started","timestamp":1701234567890,"user":"local","agent":"Orchestrator","trace_id":"uuid","input":{"prompt":"..."},"output":{},"duration_ms":0}
{"event":"researcher_done","timestamp":1701234567950,"user":"local","agent":"Researcher","trace_id":"uuid","input":{},"output":{"docs":3,"context_size":2048},"duration_ms":60}
```

---

## Performance Tuning

### llama.cpp Sidecar

Set GPU acceleration (CUDA / Metal):

```bash
# Linux (CUDA)
export CUDA_VISIBLE_DEVICES=0
./bin/llama-cpp/llama-server -m bin/llama-cpp/model.gguf -ngl 35 --port 8080

# macOS (Metal)
./bin/llama-cpp/llama-server -m bin/llama-cpp/model.gguf -ngl 35 --port 8080

# Windows (CUDA)
set CUDA_VISIBLE_DEVICES=0
.\bin\llama-cpp\llama-server.exe -m .\bin\llama-cpp\model.gguf -ngl 35 --port 8080
```

### RAG Embedding Pool

Increase pool size in `config.toml` if hardware supports:

```toml
[rag]
pool_size = 8  # Up to num_cpus
```

### Docker Limits

Adjust per-project in project settings or `config.toml`:

```toml
[sandbox]
memory_limit_mb = 4096  # For large test suites
cpu_limit = 8
```

---

## Testing

### Unit Tests

```bash
cd build
ctest --output-on-failure
```

### Integration Tests

```bash
# Start app in headless mode
./build/DroxIDE --test --project /path/to/test/project

# Check metrics
cat ~/.droxide/audit.jsonl | jq '.event' | sort | uniq -c
```

### End-to-End Test

```bash
# 1. Open test project
# 2. Run swarm with prompt: "Add async/await to main function"
# 3. Accept diff
# 4. Check audit logs
./build/DroxIDE --open-folder /path/to/test/rust/project
```

---

## Troubleshooting

### Build Errors

**Qt not found:**
```bash
export Qt6_DIR=/path/to/Qt6
cmake -B build ...
```

**Rust errors:**
```bash
rustup update stable
cargo clean
cargo build --release
```

### Runtime Issues

**llama.cpp won't start:**
```bash
# Test manually
./bin/llama-cpp/llama-server -m bin/llama-cpp/model.gguf --port 8080
curl http://127.0.0.1:8080/health
```

**Docker not running:**
```bash
systemctl start docker  # Linux
open -a Docker         # macOS
```

**RAG index corrupted:**
```bash
rm -rf ~/.droxide/chromadb
# App will reingest on next startup
```

---

## Release Checklist

- [ ] All unit tests pass
- [ ] E2E test on all platforms
- [ ] Security audit (sandbox, audit logs)
- [ ] Code signing (macOS notarization, Windows cert)
- [ ] Version bump in `CMakeLists.txt` + `Cargo.toml`
- [ ] Tag git release: `git tag -a v1.0.0 -m "Release 1.0.0"`
- [ ] Build + test installers
- [ ] Upload to release pages
- [ ] Update docs

---

**Next:** Deploy binaries to GitHub Releases, brew tap, deb repos, etc.
