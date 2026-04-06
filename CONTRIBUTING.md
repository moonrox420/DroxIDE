# Contributing to DroxIDE

Thank you for your interest in contributing to DroxIDE! This document provides guidelines and instructions for contributing to the project.

## 🚀 Quick Start

### Prerequisites

- **Rust 1.75+** (via [rustup](https://rustup.rs/))
- **Qt 6.8+** (via [vcpkg](https://vcpkg.io/) or system packages)
- **CMake 3.30+**
- **Docker Desktop** (for sandbox testing)
- **Git**

### Setup

```bash
# Fork and clone the repository
git clone https://github.com/YOUR_USERNAME/DroxIDE.git
cd DroxIDE

# Install Rust dependencies
cargo fetch

# Build the Rust core
cargo build

# Build the full application (see BUILD.md for details)
cmake -B build -G Ninja -DCMAKE_BUILD_TYPE=Debug
cmake --build build
```

## 📋 Development Workflow

### 1. Create a Branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/issue-description
```

**Branch naming conventions:**
- `feature/` - New features
- `fix/` - Bug fixes
- `docs/` - Documentation changes
- `refactor/` - Code refactoring
- `test/` - Test additions/fixes
- `chore/` - Maintenance tasks

### 2. Make Your Changes

Follow these guidelines:

- **Rust code**: Run `cargo fmt` and `cargo clippy` before committing
- **C++ code**: Follow Qt coding style, run clang-format
- **Tests**: Add tests for new functionality
- **Documentation**: Update relevant docs if behavior changes

### 3. Run Tests

```bash
# Rust tests
cargo test

# C++ tests (if applicable)
cd build && ctest --output-on-failure
```

### 4. Commit Your Changes

We follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add Docker sandbox integration
fix: resolve race condition in orchestrator
docs: update BUILD.md with Windows instructions
test: add unit tests for metrics module
refactor: simplify agent message handling
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, no logic change)
- `refactor`: Code refactoring
- `test`: Test additions/fixes
- `chore`: Maintenance tasks

### 5. Submit a Pull Request

1. Push to your fork: `git push origin feature/your-feature-name`
2. Open a PR against the `main` branch
3. Fill out the PR template with:
   - Description of changes
   - Related issues
   - Test coverage
   - Screenshots (for UI changes)

## 🏗️ Architecture Overview

### Project Structure

```
DroxIDE/
├── src/              # Qt 6 C++ UI layer
│   ├── main.cpp
│   ├── mainwindow.cpp
│   ├── editor/       # Code editor with syntax highlighting
│   ├── terminal/     # Terminal integration
│   ├── explorer/     # File explorer
│   ├── panels/       # Agent trace, RAG heatmap
│   ├── dialogs/      # User dialogs
│   ├── git/          # Git operations
│   └── lsp/          # LSP client
│
├── src-rust/         # Rust core
│   ├── lib.rs        # FFI bridge (cxx-qt)
│   ├── orchestrator.rs  # FSM state machine
│   ├── agent.rs      # 7 agent types
│   ├── rag.rs        # RAG pipeline
│   ├── sandbox.rs    # Docker sandbox
│   ├── llama.rs      # LLM integration
│   ├── metrics.rs    # Metrics counters
│   ├── audit.rs      # Audit logging
│   └── git.rs        # Git operations
│
└── docker/           # Sandbox Dockerfiles
```

### Key Components

1. **Qt 6 UI** (`src/`): Native desktop application
2. **Rust Core** (`src-rust/`): Business logic, agents, RAG, sandbox
3. **FFI Bridge** (`lib.rs`): cxx-qt communication between Qt and Rust
4. **External Services**: llama.cpp (LLM), Docker (sandbox), ChromaDB (vector DB)

## 🧪 Testing Guidelines

### Rust Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature() {
        // Arrange
        let input = ...;
        
        // Act
        let result = ...;
        
        // Assert
        assert_eq!(result, expected);
    }
}
```

Run tests: `cargo test`

### Integration Tests

Place integration tests in `src-rust/tests/`:

```rust
#[test]
fn test_full_workflow() {
    // Test complete swarm workflow
}
```

## 📝 Code Style

### Rust

```bash
# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings
```

**Style guide:** Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)

### C++ (Qt)

```bash
# Format code (if clang-format is available)
clang-format -i src/*.cpp src/*.h
```

**Style guide:** Follow [Qt Coding Style](https://wiki.qt.io/Qt_Coding_Style)

## 🔍 Code Review Process

All PRs are reviewed by maintainers. Review checklist:

- [ ] Code follows project style
- [ ] Tests pass (`cargo test`, `ctest`)
- [ ] No clippy warnings
- [ ] Documentation updated
- [ ] No security vulnerabilities
- [ ] Performance impact assessed (if applicable)

## 🐛 Reporting Bugs

Open an issue with:

1. **Description**: Clear description of the bug
2. **Steps to Reproduce**: Numbered list
3. **Expected Behavior**: What should happen
4. **Actual Behavior**: What actually happens
5. **Environment**: OS, Qt version, Rust version
6. **Logs**: `~/.droxide/debug.log` if applicable

## 💡 Feature Requests

Open an issue with:

1. **Description**: What you want
2. **Use Case**: Why you need it
3. **Proposed Solution**: How you think it should work (optional)

## 📚 Documentation

- **README.md**: Project overview
- **BUILD.md**: Build instructions
- **QUICKSTART.md**: Getting started guide
- **CHANGELOG.md**: Version history
- **PDR_v2_PRODUCTION.md**: Full architecture document

## 🤝 Community

- Be respectful and constructive
- Help others learn
- Credit contributors in CHANGELOG

## 📜 License

By contributing, you agree that your contributions will be licensed under the MIT License.

---

**Thank you for contributing to DroxIDE!** 🚀
