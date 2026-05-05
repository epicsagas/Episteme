# MCP Integration Guide

> Integrate Syntagma's knowledge graph into Claude Code, Cursor, and other MCP-compatible AI tools

## Rust MCP HTTP Mode (Current)
Use standalone HTTP transport directly:

```bash
# Start MCP over HTTP
syntagma mcp --http --host 127.0.0.1 --port 43175
```

Authentication behavior:
- If `SYNTAGMA_API_KEYS` is configured, requests must include:
```http
Authorization: Bearer <api-key>
```
- If no keys are configured, authentication is skipped (development mode).
- `GET /health` is always public for health checks.

Note:
- `syntagma service` manages this same MCP HTTP mode in background (`start|stop|status|enable|disable`).
- Older `--proxy` examples are deprecated; use `mcp --http`/`service` directly.

## What is MCP?

[Model Context Protocol (MCP)](https://modelcontextprotocol.io) is an open standard that allows AI assistants to access external tools and data sources. Syntagma provides 6 MCP tools that give AI agents direct access to software engineering knowledge.

---

## Quick Start (Claude Code)

### 1. Install Syntagma

```bash
# Install from source
git clone https://github.com/epicsagas/Syntagma.git
cd Syntagma && cargo build --release

# Install agents and MCP server into Claude Code
# (seeds data and builds knowledge DB automatically)
./target/release/syntagma install claude
```

### 2. Verify Installation

Check `~/.claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "syntagma": {
      "command": "syntagma",
      "args": ["mcp"]
    }
  }
}
```

### 3. Start Using

Restart Claude Code. You now have access to 6 Syntagma tools:

```
User: "What's the best way to fix a God Object smell?"

Claude (using search_knowledge tool):
  → Searches for "God Object" refactorings
  → Returns: RF-018 (Extract Class), RF-023 (Move Method)
  
Claude: "The God Object anti-pattern (SMELL-03) violates Single 
Responsibility Principle (LAW-001). Best refactorings:

1. Extract Class (RF-018) - Move related methods/fields to new class
2. Move Method (RF-023) - Relocate methods to appropriate classes

Both enforce SOLID principles and improve testability."
```

---

## MCP Tools Reference

### 1. `search_knowledge`

**Purpose**: Semantic search across all entities (patterns, laws, refactorings, smells)

**Parameters**:
```typescript
{
  query: string          // Natural language query
  top_k?: number         // Results to return (default: 5)
  filter_type?: string   // "pattern", "law", "refactoring", "smell"
}
```

**Returns**:
```typescript
{
  results: [{
    entity_id: string     // e.g., "DP-023"
    title: string
    entity_type: string
    similarity: number    // 0.0-1.0
    summary: string
  }]
}
```

**Example Conversation**:
```
User: "How do I make my code more testable?"

Claude calls: search_knowledge({
  query: "improve testability",
  top_k: 3
})

Returns:
- LAW-001: Single Responsibility Principle
- DP-018: Dependency Injection
- RF-042: Extract Interface

Claude: "Three key approaches to improve testability:
1. Apply SRP (LAW-001) - One class, one reason to change
2. Use Dependency Injection (DP-023) - Inject dependencies
3. Extract Interface (RF-042) - Mock external dependencies"
```

---

### 2. `get_entity`

**Purpose**: Get full details for a specific entity by ID

**Parameters**:
```typescript
{
  entity_id: string   // e.g., "DP-023", "RF-001", "SMELL-01"
}
```

**Returns**:
```typescript
{
  entity_id: string
  title: string
  type: string
  description: string
  implementation: string    // Code examples
  when_to_use: string
  benefits: string[]
  trade_offs: string[]
  related_entities: {
    relation_type: string
    target_id: string
    description: string
  }[]
}
```

**Example Conversation**:
```
User: "Explain Strategy Pattern in detail"

Claude calls: get_entity({ entity_id: "DP-023" })

Returns:
- Full description of Strategy Pattern
- When to use (multiple algorithms, runtime switching)
- Implementation (Context, Strategy interface, ConcreteStrategies)
- Benefits: Open/Closed Principle, runtime flexibility
- Trade-offs: Increased class count
- Related: LAW-002 (Open/Closed), RF-034 (Replace Conditional with Polymorphism)

Claude: "Strategy Pattern (DP-023) encapsulates algorithms in separate classes...
[provides detailed explanation with code example]

Related concepts:
- Enforces Open/Closed Principle (LAW-002)
- Often applied via Replace Conditional with Polymorphism (RF-034)"
```

---

### 3. `get_neighbors`

**Purpose**: Explore entities connected to a given entity

**Parameters**:
```typescript
{
  entity_id: string
  relation_type?: string   // "solves", "enforces", "violates", "relates_to"
  max_depth?: number       // Default: 1
}
```

**Returns**:
```typescript
{
  neighbors: [{
    entity_id: string
    title: string
    relation_type: string
    distance: number
  }]
}
```

**Example Conversation**:
```
User: "What refactorings solve Long Method smell?"

Claude calls: get_neighbors({
  entity_id: "SMELL-01",
  relation_type: "solved_by"
})

Returns:
- RF-001: Extract Method
- RF-034: Decompose Conditional
- RF-056: Replace Temp with Query

Claude: "Long Method (SMELL-01) can be resolved with:
1. Extract Method (RF-001) - Break into smaller methods
2. Decompose Conditional (RF-034) - Simplify complex conditions
3. Replace Temp with Query (RF-056) - Eliminate temporary variables"
```

---

### 4. `find_path`

**Purpose**: Find connection between two entities (e.g., how a law relates to a pattern)

**Parameters**:
```typescript
{
  from_id: string
  to_id: string
  max_depth?: number   // Default: 5
}
```

**Returns**:
```typescript
{
  path: [{
    entity_id: string
    title: string
    relation_to_next: string
  }],
  path_found: boolean
  hops: number
}
```

**Example Conversation**:
```
User: "How does Single Responsibility Principle relate to Extract Class refactoring?"

Claude calls: find_path({
  from_id: "LAW-001",
  to_id: "RF-018"
})

Returns path:
LAW-001 (SRP) --enforces--> DP-007 (Facade) --achieved_via--> RF-018 (Extract Class)

Claude: "SRP (LAW-001) connects to Extract Class (RF-018) in 2 hops:
1. SRP enforces Facade Pattern (DP-007)
2. Facade is often achieved via Extract Class (RF-018)

Insight: When a class has multiple responsibilities, extracting 
classes (RF-018) helps satisfy SRP (LAW-001)."
```

---

### 5. `analyze_code`

**Purpose**: Detect code smells via AST analysis

**Parameters**:
```typescript
{
  code: string
  language: string   // "python", "java", "typescript", etc.
  min_confidence?: number   // Default: 0.5
}
```

**Returns**:
```typescript
{
  smells_detected: number,
  detections: [{
    smell_id: string
    smell_name: string
    confidence: number
    location: string
    metrics: {
      loc: number
      cyclomatic_complexity: number
      parameter_count: number
    }
  }]
}
```

**Example Conversation**:
```
User: "Review this payment validation code"
[pastes code]

Claude calls: analyze_code({
  code: "...",
  language: "python"
})

Returns:
- SMELL-01: Long Method (confidence 0.87, LOC=45)
- SMELL-08: Long Parameter List (confidence 0.92, params=9)

Claude: "I found 2 code smells:

1. Long Method (SMELL-01, confidence 87%)
   - 45 lines of code (threshold: 20)
   - High cyclomatic complexity (12)
   - Recommended: Extract Method (RF-001)

2. Long Parameter List (SMELL-08, confidence 92%)
   - 9 parameters (threshold: 4)
   - Recommended: Introduce Parameter Object (RF-029)"
```

---

### 6. `suggest_refactorings`

**Purpose**: Get ranked refactoring suggestions for detected smells

**Parameters**:
```typescript
{
  code: string
  language: string
  top_k?: number   // Number of suggestions (default: 3)
  min_confidence?: number
}
```

**Returns**:
```typescript
{
  smells_analyzed: number,
  results: [{
    smell: {
      smell_id: string
      smell_name: string
      confidence: number
    },
    suggestions: [{
      refactoring_id: string
      title: string
      priority_score: number
      severity: string
      effort: string
      principles: string[]
    }]
  }]
}
```

**Example Conversation**:
```
User: "What should I refactor in this code?"
[pastes large class]

Claude calls: suggest_refactorings({
  code: "...",
  language: "java",
  top_k: 3
})

Returns ranked suggestions for God Object smell:
1. RF-018: Extract Class (priority 0.89, High severity, Medium effort)
2. RF-023: Move Method (priority 0.76, Medium severity, Low effort)
3. RF-042: Extract Interface (priority 0.65, Low severity, Low effort)

Claude: "This code shows God Object characteristics (SMELL-03).

Top refactoring priorities:

1. Extract Class (RF-018) - Priority 0.89
   - Severity: High
   - Effort: Medium
   - Move payment-related methods to PaymentProcessor class
   - Enforces: SRP (LAW-001), SoC (LAW-012)

2. Move Method (RF-023) - Priority 0.76
   - Severity: Medium
   - Effort: Low
   - Relocate validation methods to Validator class
   - Quick win for improving cohesion

Start with #2 (Move Method) for immediate improvement, 
then tackle #1 (Extract Class) for long-term design."
```

---

## AI Agents

Syntagma includes 4 specialized agents that work together as a connected system. Each agent uses knowledge graph tools for analysis and presents interactive follow-up options that can hand off to other agents.

### Agent Network

```
code-reviewer ←→ syntagma-advisor
      ↕                ↕
architecture-analyst ←→ syntagma-researcher
      ↕
refactoring-expert (external)
```

Each agent ends its report with **Next Steps** — interactive options that connect to other agents or tools. This creates end-to-end workflows from detection to remediation.

---

### 1. `code-reviewer`

**When to use**: Reviewing code for smells, SOLID violations, and refactoring priorities

**What it does differently**: Goes beyond detection — traces causation chains between smells (which smells are root causes vs. downstream symptoms), validates findings against language ecosystem conventions (Rust, Go, etc.), and calibrates severity contextually.

**Tools used**: `analyze_code` → `suggest_refactorings` → `get_entity` → `get_neighbors` → `find_path`

**Example Prompt**:
```
"Review this authentication module"
[pastes code]
```

**Agent Response**:
```
# Syntagma Code Review

## Summary
AuthenticationModule has a root-cause God Object smell causing 3
downstream issues. Fixing the root cause will cascade improvements.

## Smell Detections
| Smell | Location | Severity | Confidence | Related Principle |
|-------|----------|----------|------------|-------------------|
| God Object (SMELL-03) | auth.rs:1-420 | Critical | confirmed | LAW-001 (SRP) |
| Long Method (SMELL-01) | auth.rs:88-142 | High | confirmed | LAW-001 (SRP) |
| Divergent Change (SMELL-09) | auth.rs:200-380 | Medium | confirmed | LAW-012 (SoC) |
| Feature Envy (SMELL-15) | auth.rs:290-333 | Medium | confirmed | LAW-003 (LSP) |

## Causation Analysis
Root causes:
- SMELL-03 → causes → SMELL-01, SMELL-09
  (fixing God Object will cascade improvements)

Independent:
- SMELL-15 — standalone, no downstream impact

## Ranked Refactorings
1. **Extract Class** (RF-018) — Priority: Critical
   - Addresses: SMELL-03 (root cause)
   - Cascade effect: also resolves SMELL-01, SMELL-09
   - What to do: Extract SessionManager, EmailVerifier classes

## Next Steps
1. Refactor — Apply RF-018 (Extract Class) → use refactoring-expert agent
2. Deep dive — Explore root cause chain for SMELL-03 → use syntagma-advisor
3. Architecture check — Assess structural impact → use architecture-analyst
4. Learn more — Get full explanation of SMELL-03 → get_entity(detail_level="full")
```

**Validation**: Before confirming a smell, the agent cross-checks against language idioms (e.g., large structs are normal in Rust), domain context (test files, generated code), and real impact (cosmetic vs. development-blocking).

---

### 2. `syntagma-advisor`

**When to use**: Making engineering decisions (pattern selection, architecture trade-offs, root-cause deep dives)

**Tools used**: `search_knowledge` → `get_entity` → `get_neighbors` → `find_path`

**Example Prompt**:
```
"Should I use Singleton for my ConfigManager class?"
```

**Agent Response**:
```
# Advisory: Singleton vs DI for ConfigManager

## Applicable Entities
### 1. Singleton Pattern (DP-006) - Primary
- **What it says**: Ensure a class has only one instance
- **What it means for you**: Global config access but hidden coupling
- **Recommendation**: Avoid — violates testability

### 2. Dependency Injection (DP-018) - Alternative
- **What it says**: Inject dependencies via constructor
- **What it means for you**: Explicit dependencies, easy mocking
- **Recommendation**: Prefer for ConfigManager

## Trade-offs
- Singleton (DP-006) offers convenience but violates DI (DP-018)
- DI adds boilerplate but enables LAW-005 (DIP) compliance
- Resolution: Use DI with a config provider interface

## Action Plan
1. Immediate: Inject ConfigManager via constructor
2. Short-term: Extract ConfigProvider interface
3. Long-term: Consider environment-specific config strategies

## Next Steps
1. Implement — Apply DI pattern → use code-reviewer to validate
2. Explore alternatives — Compare approaches → use syntagma-researcher
3. Architecture impact — Assess downstream effects → use architecture-analyst
```

---

### 3. `syntagma-researcher`

**When to use**: Exploring relationships in the knowledge graph, finding alternatives

**Tools used**: `search_knowledge` → `get_entity` → `get_neighbors` → `find_path`

**Example Prompt**:
```
"What laws does the Observer Pattern enforce?"
```

**Agent Response**:
```
# Research Report: Observer Pattern Enforced Laws

## Primary Entities
1. **Observer Pattern** (DP-012) - Behavioral
   - Definition: Define one-to-many dependency between objects
   - When change to one object alters others automatically

## Connection Map
DP-012 --enforces--> LAW-002 (Open/Closed)
DP-012 --enforces--> LAW-005 (Dependency Inversion)
DP-012 --enforces--> LAW-001 (Single Responsibility)

## Contradictions & Trade-offs
- Observer (DP-012) enables loose coupling but can cause unexpected
  update chains — contrasted with Mediator (DP-014) which centralizes

## Next Steps
1. Get advice — Turn findings into guidance → use syntagma-advisor
2. Check code — Detect Observer in your codebase → use code-reviewer
3. Assess architecture — Evaluate structural fit → use architecture-analyst
```

---

### 4. `architecture-analyst`

**When to use**: Evaluating system architecture against laws, patterns, and structural risks

**Tools used**: `search_knowledge` → `get_entity` → `get_neighbors` → `find_path`

**Example Prompt**:
```
"Is this microservices architecture following Conway's Law?"
[describes architecture]
```

**Agent Response**:
```
# Architectural Analysis: Microservices

## Law & Principle Compliance
| Principle | Status | Evidence | Impact |
|-----------|--------|----------|--------|
| Conway's Law (LAW-042) | violated | Shipping spans 2 teams | Coordination overhead |
| SRP (LAW-001) | at risk | Analytics depends on all | Tight coupling |
| Bounded Context (LAW-031) | violated | No clear domain boundaries | Shared data confusion |

## Key Tensions
- Conway's Law (LAW-042) requires team↔service alignment
  but Shipping service spans Commerce + Platform teams
- Traced via: LAW-042 → related_to → LAW-001 → enforced_by → DP-026 (Strangler Fig)

## Architectural Recommendations
1. **Critical**: Move Shipping to Commerce team — LAW-042 predicts coordination failure
2. **High**: Introduce Event Bus for Analytics — decouple via async events
3. **Medium**: Define Bounded Contexts — align service boundaries with domain

## Compliance Scores
- Overall: 5/10 | Structure: 4/10 | Scalability: 6/10 | Maintainability: 5/10

## Next Steps
1. Get advice — Resolve key tensions → use syntagma-advisor
2. Check code — Detect structural smells → use code-reviewer
3. Research alternatives — Find better patterns → use syntagma-researcher
```

---

## Workflow Chains

Agents and tools connect into end-to-end pipelines. Each chain produces a report followed by interactive follow-up options.

### Chain 1: Code Review Pipeline
```
analyze_code → suggest_refactorings → get_neighbors("solved_by")
  → find_path(smell_A, smell_B) → Report with causation graph
  → User chooses: Apply fix / Deep dive / Architecture check / Learn more
```

### Chain 2: Architecture Review Pipeline
```
search_knowledge → get_entity → get_neighbors("enforces")
  → get_neighbors("violates") → find_path → Compliance report
  → User chooses: Refactoring plan / Advisory / Research alternatives
```

### Chain 3: Problem Diagnosis Pipeline
```
search_knowledge(symptoms) → get_entity → get_neighbors("solved_by")
  → Root cause report → User chooses: Apply fix / Advisory / Verify
```

### Chain 4: Learning Pipeline
```
search_knowledge(topic) → get_entity → get_neighbors("related_to")
  → Concept map → User chooses: Code examples / Apply to code / Compare
```

### Cross-Tool Chaining Rules

Every tool call naturally leads to the next:

| After calling... | Always follow up with... |
|-------------------|--------------------------|
| `analyze_code` | `suggest_refactorings` on detected smells |
| `suggest_refactorings` | `get_neighbors(smell_id, "solved_by")` for alternatives |
| `search_knowledge` | `get_entity` on top 1-2 results |
| `get_entity` (smell) | `get_neighbors(id, "violates")` for impacted principles |
| `get_entity` (pattern) | `get_neighbors(id, "enforces")` for enforced laws |
| Multiple smells detected | `find_path(smell_A, smell_B)` for causation mapping |

---

## Installation for Other Tools

### Cursor

```bash
syntagma install cursor
```

Adds MCP config to `~/.cursor/mcp.json`:
```json
{
  "mcpServers": {
    "syntagma": {
      "command": "syntagma",
      "args": ["mcp"]
    }
  }
}
```

### Codex (OpenAI)

```bash
syntagma install codex
```

Generates `AGENTS.md` in project root with agent definitions.

### Custom MCP Integration

If your tool supports MCP, manually configure:

```json
{
  "mcpServers": {
    "syntagma": {
      "command": "/path/to/syntagma",
      "args": ["mcp"],
      "env": {
        "SYNTAGMA_DATA_DIR": "~/.syntagma/data",
        "SYNTAGMA_DB_PATH": "~/.syntagma/db/syntagma.db"
      }
    }
  }
}
```

---

## Running as Background Service

For better performance, run Syntagma MCP as a persistent HTTP proxy:

```bash
# Start background service
syntagma service start

# Check status
syntagma service status
# Output: Running on http://localhost:43175 (PID 12345)

# Enable auto-start on boot (macOS)
syntagma service enable

# Stop service
syntagma service stop
```

Update MCP config to use HTTP proxy:

```json
{
  "mcpServers": {
    "syntagma": {
      "command": "syntagma",
      "args": ["mcp", "--proxy", "http://localhost:43175"]
    }
  }
}
```

Logs: `~/.syntagma/logs/mcp.out.log`

---

## Troubleshooting

### Tools not showing up in Claude

1. Check config file exists: `cat ~/.claude/claude_desktop_config.json`
2. Verify syntagma is in PATH: `which syntagma`
3. Test MCP directly: `syntagma mcp`
4. Check logs: `tail -f ~/.syntagma/logs/mcp.err.log`

### "Database not found" error

```bash
# Rebuild knowledge database
syntagma build --rebuild
```

### Slow search responses

```bash
# Use GPU acceleration
syntagma build --gpu

# Or run as background service (faster warmup)
syntagma service start
```

### Agent not using tools

Make sure agent has tool-calling capability. In Claude Code:
```
User: "Use Syntagma to find patterns for retry logic"
      ^^^^ explicitly mention tool usage
```

---

## Advanced: Custom Knowledge Integration

Combine Syntagma (generic knowledge) with Alcove (team knowledge):

```json
{
  "mcpServers": {
    "syntagma": {
      "command": "syntagma",
      "args": ["mcp"]
    },
    "alcove": {
      "command": "npx",
      "args": ["-y", "@joshuarileydev/alcove-mcp"]
    }
  }
}
```

See [Alcove Integration Guide](../docs/alcove-integration.md) for dual-source patterns.

---

## API Alternative

If your AI tool doesn't support MCP, use the REST API:

```bash
# Start API server
docker-compose up -d

# Use from any tool
curl http://localhost:8000/search?q=strategy+pattern
```

See [API Documentation](../docs/api.md) for endpoints.

---

## Automatic Triggering (Claude Code)

When you describe a problem in natural language, Claude Code automatically detects the intent and calls the appropriate Syntagma tool — **you don't need to mention Syntagma explicitly**. Below are the exact trigger patterns and examples.

### How It Works

```
Your natural language input
    ↓ Claude detects keywords/patterns
    ↓ Syntagma tool is called automatically
    ↓ Knowledge graph returns verified data
    ↓ (Design Patterns · Code Smells · Refactoring Techniques · Engineering Laws)
    ↓ Claude's response is grounded in evidence
```

> **Note:** This is prompt-based auto-triggering, not a hard hook. To guarantee a call, use the `/syntagma` skill directly.

### Code Structure Problems

| What you say (examples) | What Syntagma detects | Automatic tool call |
|-------------------------|-----------------------|---------------------|
| "This class does too much", "This file is over 300 lines" | God Class, Large Class, Single Responsibility | `search_knowledge("god class large class single responsibility")` |
| "This function is too long", "Too many lines in this method" | Long Method | `search_knowledge("long method extract method")` |
| "The code is too complex", "Hard to follow" | Complexity, Cognitive Overload | `search_knowledge("complexity smell cognitive overload")` |
| "I copy-pasted this everywhere", "There's duplicated logic" | Duplicated Code, Clone | `search_knowledge("duplicated code clone smell")` |

### Coupling & Dependency Problems

| What you say (examples) | What Syntagma detects | Automatic tool call |
|-------------------------|-----------------------|---------------------|
| "Business logic calls DB directly" | Coupling, Persistence, Repository | `search_knowledge("coupling persistence repository data access layer")` |
| "Changing X breaks Y", "Changes ripple everywhere" | Brittle Coupling, Change Propagation | `search_knowledge("brittle coupling change propagation rigidity")` |
| "Adding a new type means touching everywhere", "switch-case keeps growing" | Open/Closed, Strategy, Polymorphism | `search_knowledge("open closed principle strategy polymorphism")` |

### Testing & Quality Problems

| What you say (examples) | What Syntagma detects | Automatic tool call |
|-------------------------|-----------------------|---------------------|
| "This is hard to test", "Can't write unit tests for this" | Testability, Dependency Injection | `search_knowledge("testability dependency injection mockability")` |

### Performance & Concurrency Problems

| What you say (examples) | What Syntagma detects | Automatic tool call |
|-------------------------|-----------------------|---------------------|
| "The API is slow", "Response time is too high" | N+1 Query, Lazy Loading, Caching | `search_knowledge("N+1 query lazy loading caching performance")` |
| "Is this thread-safe?", "Any concurrency issues?" | Thread Safety, Race Condition | `search_knowledge("thread safety race condition shared mutable state")` |

### Code Review & Analysis

| What you say (examples) | Automatic tool call |
|-------------------------|---------------------|
| Share a code snippet and ask for review | `analyze_code(code)` → `suggest_refactorings(code)` |
| "Improve this code", "Refactor this for me" | `suggest_refactorings(code)` |

### Architecture Discussions

| What you say (examples) | Automatic tool call |
|-------------------------|---------------------|
| "Microservices vs monolith", "How should we split this?" | `search_knowledge("monolith microservice decomposition bounded context")` |
| "Is this architecture okay?", "Review this design" | `search_knowledge("layered architecture coupling responsibility")` |

### Real-World Conversation Examples

**Example 1 — Detecting a God Object:**

```
You: "UserService is getting out of hand. It handles authentication,
      session management, email verification, and password resets."

Claude (auto-calls search_knowledge):
  → Detects: God Object / Single Responsibility violation
  → Returns: SMELL-03 (God Object), LAW-001 (SRP), RF-018 (Extract Class)

Claude: "UserService shows God Object characteristics (SMELL-03).
  It violates Single Responsibility Principle (LAW-001).

  Recommended refactorings:
  1. Extract Class (RF-018) — Create SessionManager for session logic
  2. Move Method (RF-023) — Move email verification to EmailService
  3. Extract Interface (RF-042) — Define AuthService interface

  Start with #2 for quick wins, then #1 for structural improvement."
```

**Example 2 — Performance Investigation:**

```
You: "The product listing API is really slow. It loads fine individually
      but the list page takes 5+ seconds."

Claude (auto-calls search_knowledge):
  → Detects: N+1 Query problem
  → Returns: Performance patterns, Lazy Loading, Caching strategies

Claude: "This sounds like an N+1 query problem — each product triggers
  individual DB calls instead of a batch query.

  Solutions from the knowledge graph:
  1. Batch loading / Eager loading — Fetch related data in one query
  2. Data Mapper (DP-019) — Decouple query logic from domain objects
  3. Apply Lazy Loading (RF-050) — Load relations on demand only

  Would you like me to check the code for N+1 patterns?"
```

**Example 3 — Code Review with Snippet:**

```
You: [pastes Python code]
     "Can you review this? Something feels off."

Claude (auto-calls analyze_code → suggest_refactorings):
  → analyze_code detects: Long Method (confidence 0.87), Long Parameter List (0.92)
  → suggest_refactorings returns ranked suggestions

Claude: "Found 2 code smells:

  1. Long Method (SMELL-01) — 45 lines, cyclomatic complexity 12
     → Extract Method (RF-001), priority 0.89

  2. Long Parameter List (SMELL-08) — 9 parameters
     → Introduce Parameter Object (RF-029), priority 0.92

  Start with RF-029 (Parameter Object) — it's the highest priority
  and makes the subsequent Extract Method easier."
```

---

## Next Steps

1. **Try agents**: Ask syntagma-advisor "Should I use Singleton?"
2. **Analyze code**: Paste a function and ask code-reviewer to check smells
3. **Explore graph**: Use syntagma-researcher to find pattern relationships
4. **Custom workflows**: Combine tools (analyze → suggest → search)

For more examples, see:
- [Alcove Integration](../docs/alcove-integration.md) — Team knowledge + Syntagma
- [Monitoring Setup](../monitoring/README.md) — Track pattern usage
- [API Reference](../docs/api.md) — REST endpoints
