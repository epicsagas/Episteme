# Contributing to Syntagma

Thank you for your interest in contributing to Syntagma! This document provides guidelines for contributing to the project.

---

## Quick Start for Contributors

1. **Fork and clone**
   ```bash
   git clone https://github.com/YOUR_USERNAME/Syntagma.git
   cd Syntagma
   ```

2. **Build the project**
   ```bash
   cargo build
   ```

3. **Run tests**
   ```bash
   cargo test
   ```

4. **Create a feature branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

5. **Make your changes, then verify**
   ```bash
   cargo test
   cargo clippy -- -D warnings
   cargo fmt --check
   ```

6. **Commit and push**
   ```bash
   git add .
   git commit -m "feat(scope): add your feature description"
   git push origin feature/your-feature-name
   ```

7. **Open a Pull Request** on GitHub.

---

## Development Setup

**Prerequisites:** Rust 1.95+ via [rustup](https://rustup.rs).

```bash
rustup update stable
rustup show   # confirm active toolchain >= 1.95
```

**Build and run:**
```bash
cargo build                           # debug build
cargo build --release                 # optimized build
cargo run -- analyze src/main.rs      # analyze a file
cargo run -- explore "strategy"       # search knowledge graph
```

**Seed data:**
```bash
cargo run -- install claude    # seeds DB + wires MCP config
```

For the full architecture, tech stack, and API reference, see [DEVELOPMENT.md](DEVELOPMENT.md).

---

## Code Quality

Before submitting a PR, ensure all checks pass:

```bash
cargo test                    # all 171+ unit tests
cargo clippy -- -D warnings   # zero warnings
cargo fmt --check             # formatted code
```

CI runs these checks on every push and PR. PRs with failing checks will not be merged.

---

## Contribution Areas

### 1. Add a New Language Detector

**Location:** `src/adapters/regex_parsers.rs`

Syntagma uses regex-based multi-language detection. To add a new language:

1. Add a new `Language` variant in `src/domain/types.rs`
2. Add detection patterns in `src/adapters/regex_parsers.rs`
3. Add test cases with sample code snippets
4. Update the language count in README.md

### 2. Add a New Code Smell Detector

**Location:** `src/domain/detectors/`

1. Add smell metadata to the knowledge base (`raw/` directory)
2. Add detection logic as a new detector module
3. Wire it into the `AnalysisEngine`
4. Add tests with positive and negative cases

### 3. Improve Documentation

- `README.md` — User-facing overview
- `DEVELOPMENT.md` — Architecture and developer guide
- `docs/` — Guides and API reference
- `CHANGELOG.md` — Release history (Keep a Changelog format)

### 4. Enhance the Knowledge Base

**Location:** `raw/` (markdown source) → `dist/` (built data)

- New design patterns, refactorings, or engineering laws
- Additional code examples in multiple languages
- Semantic relationships between entities

After editing raw data, rebuild:
```bash
cargo run -- build
```

---

## Commit Message Guidelines

We follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <subject>
```

**Types:** `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `chore`

**Examples:**
```
feat(detectors): add feature envy detection for Rust
fix(mcp): correct entity neighbor traversal order
docs(readme): add demo GIF to hero section
```

---

## Reporting Issues

### Bug Reports

Include:
1. **Description** — What went wrong
2. **Steps to Reproduce** — Minimal commands
3. **Expected vs Actual Behavior**
4. **Environment** — OS, Syntagma version (`syntagma --version`), Rust version (`rustc --version`)

### Feature Requests

Include:
1. **Use Case** — Why is this needed?
2. **Proposed Solution** — How should it work?
3. **Alternatives Considered**

---

## Security

**Do not open public issues for security vulnerabilities.**

Report security issues privately via [GitHub Security Advisories](https://github.com/epicsagas/Syntagma/security/advisories/new).

See [SECURITY.md](SECURITY.md) for supported versions and response times.

---

## License

By contributing to Syntagma, you agree that your contributions will be licensed under the [Apache License 2.0](LICENSE).

---

## Thank You

Your contributions make Syntagma better for everyone. We appreciate your time and effort!

- [Development Guide](DEVELOPMENT.md) — Architecture, tech stack, API reference
- [README](README.md) — Project overview and installation
- [Changelog](CHANGELOG.md) — Release history
