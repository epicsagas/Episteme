# Architectural Diagnosis: Decomposing the God Module

> Demonstrates using Syntagma to diagnose structural rot in a rapidly growing CLI project. Traces the path from identifying a **God Module** to proposing a **Domain-Module** (Modular Monolith) architecture based on SRP and Lehman’s Laws.

## Scenario

A developer realized they had been dumping all new features into `src/hooks/`, regardless of whether they were actual hooks. They want to know the appropriate software architecture to regain maintainability.

## Tools Used

| Tool | Purpose |
|------|---------|
| `search_knowledge` | Find architecture-related patterns and laws |
| `get_entity` | Retrieve details on "God Module" and "SRP" |
| `get_neighbors` | Understand relationships between smells and refactorings |
| `read_file` | Analyze actual module roles and code volume |

## Step-by-step Walkthrough

⏺ **Searching for architecture-related concepts with Syntagma.**

  *Called syntagma 3 times (ctrl+o to expand)*

⏺ **Analyzing core files to identify actual module roles.**

  *Read 3 files, called syntagma 2 times (ctrl+o to expand)*

⏺ **Analysis complete. Diagnosing based on Syntagma knowledge graph.**

  **Diagnosis: God Module (SMELL-21)**

  The `src/hooks/` directory has become a "catch-all" for the entire project. While it contains 16 modules totaling ~21,400 lines of code, only 5 of these are actually functional Claude Code hooks.

### 2. Map the Current Responsibility Bloat

Analyzing the contents of `src/hooks/` reveals several distinct domains that violate the **Single Responsibility Principle** (LAW-042).

| Actual Hooks | Non-Hook Domains (Should be separate) |
| :--- | :--- |
| **guard** — Command safety checks | **mem/** — Memory subsystem (SQLite, MCP, Graph) |
| **observe** — Tool call monitoring | **team/** — Team management & storage |
| **polish** — Auto-formatting/Linting | **reflect** — Evolution engine (3,472 lines) |
| **resume** — Session restoration | **install** — Cross-tool installer logic |
| **snapshot** — State persistence | **orchestrate/** — Pipeline & state management |
| | **serve** — Dashboard web server |
| | **telemetry** — Usage tracking |
| | **config** — Configuration schemas |
| | **common** — Generic utilities (1,630 lines) |

### 3. Apply Architectural Laws

The diagnosis is grounded in three core principles from the Syntagma knowledge graph:

1. **SRP (LAW-042-S):** The current module has too many reasons to change (memory, hooks, installation, etc.).
2. **Lehman’s Laws (LAW-034):** Boundaries must be established early to prevent complexity from propagating across the system.
3. **Conway’s Law (LAW-001):** The code structure should mirror the cognitive path of the developer. Finding memory features in a `hooks/` folder is counter-intuitive.

---

## Proposed Architecture: Domain-Module

The recommended structure is a **Modular Monolith**. Since Syntagma is a single Rust binary, we should decompose it by business domains rather than technical layers.

### Target File Structure

```text
src/
├── main.rs                  # CLI dispatcher (unaffected)
├── lib.rs                   # Re-exports
│
├── hooks/                   # Ring 0: Pure Hook Logic
│   ├── mod.rs
│   ├── guard.rs
│   ├── observe.rs
│   ├── polish.rs
│   ├── resume.rs
│   └── snapshot.rs
│
├── evolve/                  # Ring 3: Self-Evolution Engine
│   ├── mod.rs
│   ├── reflect.rs           # Analysis & skill evolution
│   └── scoring.rs           # Extracted from common.rs
│
├── memory/                  # Unified Memory Subsystem
│   ├── mod.rs
│   ├── cli.rs
│   ├── graph.rs
│   ├── mcp.rs
│   ├── server.rs
│   └── store/
│
├── team/                    # Team Management
├── orchestrate/             # Pipeline State
├── install/                 # Installer Wizard
├── config.rs                # Shared config types
├── serve.rs                 # Web dashboard
└── telemetry.rs             # Telemetry
```

---

## Findings

### The "Extract Class" (RF-010) Strategy for `common.rs`

The 1,630-line `common.rs` is a secondary God Object. Decomposing it is critical:
- **HookInput** should move to the `hooks/` module.
- **Scoring dimensions** belong in `evolve/scoring.rs`.
- **Path utilities** (like `harness_dir()`) should reside in a top-level `paths.rs` or `config.rs`.

---

## Key Takeaways

1. **Module boundaries are cognitive paths.** Don't hide core features (like Memory or Team management) inside specialized technical folders (like Hooks).
2. **Address God Modules early.** 21K lines is the "tipping point" where complexity propagation starts slowing down all development.
3. **Refactor by Domain, not Layer.** In Rust CLI tools, domain-based top-level modules provide the clearest separation of concerns and the most intuitive navigation for contributors.
