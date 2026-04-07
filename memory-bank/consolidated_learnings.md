# Cline Consolidated Learnings

This file contains curated, distilled, actionable knowledge. This is the authoritative knowledge base for future tasks.

---

## Project Architecture

### DroxIDE Project Structure
```
Project Root
├── src/                 C++ QT6 frontend GUI code
├── src-rust/            Rust backend engine
├── --typescript/        Svelte frontend components
├── docker/              Container definitions
├── .clinerules/         Cline operation rules
└── memory-bank/         Continuous improvement knowledge base
```

### Build Systems
✅ **CMake** for C++ QT6 compilation
✅ **Cargo** for Rust backend
✅ **Vite** for TypeScript/Svelte frontend

---

## CI/CD Pipeline Configuration

### Mixed Language Pipeline Pattern
For projects with C++/Rust/TypeScript:
1.  **Matrix Build:** Use separate jobs per language domain
2.  **Cache Management:** Enable caching for `target/`, `node_modules/`, CMake build directory
3.  **Artifact Passing:** Use CI workspace artifacts between stages
4.  **Parallel Execution:** Run language test suites in parallel

---

## Active .clinerules Status

✅ **C#-guide.md** - Valid
✅ **cline-continuous-improvement-protocol.md** - Valid
✅ **self-improving-cline.md** - Valid
✅ **setup-ci-cd-pipeline.md** - Valid
✅ **workflow-rules.md** - Valid
✅ **RULE_EXECUTION_ORDER.md** - Valid, highest priority

---

## CMake + Rust Integration

### Corrosion FindRust Module
- Full Rust toolchain detection with rustup support
- Automatic target triple parsing and native library detection
- Proper MSVC/GNU ABI handling for Windows targets
- Cross-compile support with architecture auto-detection
- CMake imported targets: `Rust::Rustc` and `Rust::Cargo`

### Build System Integration
✅ **CMake 3.12+** for C++ QT6 frontend
✅ **Corrosion 0.5+** for Rust / C++ interop
✅ **Vite 5** for Svelte frontend bundling
✅ **Parallel build pipeline** with proper dependency ordering

## Protocol Execution Order
1.  **Always execute first:** Read all `.clinerules` in working directory
2.  **Pre-task scan:** List files and analyze project structure
3.  **Continuous Improvement Protocol:** Runs before `attempt_completion`
4.  **Self-Reflection Check:** Runs before `attempt_completion` if feedback was received
