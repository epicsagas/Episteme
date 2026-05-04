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

Syntagma includes 4 specialized agents for different workflows:

### 1. `syntagma-advisor`

**When to use**: Making engineering decisions (pattern selection, architecture trade-offs)

**Example Prompt**:
```
"Should I use Singleton for my ConfigManager class?"
```

**Agent Response** (uses `get_entity` + `get_neighbors`):
```
Singleton Pattern (DP-006) Analysis:

Pros:
- Global access to config
- Single instance guarantee

Cons:
- Violates Dependency Injection principle (DP-018)
- Difficult to test (hard to mock)
- Hidden dependencies (breaks Law of Demeter - LAW-023)

Alternative: Dependency Injection (DP-018)
- Inject ConfigManager via constructor
- Better testability (easy to mock)
- Explicit dependencies

Recommendation: Use DI instead of Singleton for ConfigManager.
Reference: Martin Fowler's "Inversion of Control" pattern.
```

---

### 2. `syntagma-researcher`

**When to use**: Exploring relationships in the knowledge graph

**Example Prompt**:
```
"What laws does the Observer Pattern enforce?"
```

**Agent Response** (uses `get_neighbors` + `find_path`):
```
Observer Pattern (DP-012) enforces:

1. Open/Closed Principle (LAW-002)
   - Open for extension (new observers)
   - Closed for modification (subject unchanged)

2. Dependency Inversion Principle (LAW-005)
   - Subject depends on Observer interface, not concrete classes

3. Single Responsibility Principle (LAW-001)
   - Subject handles state, observers handle reactions

Connection path: DP-012 --enforces--> LAW-002, LAW-005, LAW-001
```

---

### 3. `code-reviewer`

**When to use**: Reviewing code for smells and SOLID violations

**Example Prompt**:
```
"Review this authentication module"
[pastes code]
```

**Agent Response** (uses `analyze_code` + `suggest_refactorings`):
```
Code Review: AuthenticationModule

Smells Detected (2):
1. God Object (SMELL-03) - High confidence
   - 15 methods, 8 fields
   - Handles auth + session + logging + validation
   
2. Long Method (SMELL-01) - Medium confidence
   - validateUser() is 42 lines

SOLID Violations:
- SRP violated: Multiple responsibilities
- OCP violated: Must modify class to add auth methods

Recommended Refactorings:
1. Extract Class (RF-018) - Create SessionManager
2. Extract Method (RF-001) - Break up validateUser()
3. Extract Interface (RF-042) - AuthProvider interface

Priority: Start with Extract Class to separate session logic.
```

---

### 4. `architecture-analyst`

**When to use**: Evaluating system architecture against scalability/laws

**Example Prompt**:
```
"Is this microservices architecture following Conway's Law?"
[describes architecture]
```

**Agent Response** (uses `get_entity` for LAW-042 + domain knowledge):
```
Conway's Law (LAW-042) states: "Organizations design systems 
that mirror their communication structure."

Your Architecture:
- 5 microservices: Auth, Payment, Inventory, Shipping, Analytics
- 3 teams: Platform, Commerce, Data

Analysis:
✅ Auth service = Platform team (aligned)
✅ Payment + Inventory = Commerce team (aligned)
❌ Shipping spans Commerce + Platform (misaligned)
❌ Analytics owned by Data team but depends on all services (tight coupling)

Recommendations:
1. Move Shipping fully to Commerce team
2. Introduce Event Bus for Analytics (decouple via events)
3. Apply Bounded Context pattern (DDD) for service boundaries

Result: Better alignment with Conway's Law reduces coordination overhead.
```

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

## Next Steps

1. **Try agents**: Ask syntagma-advisor "Should I use Singleton?"
2. **Analyze code**: Paste a function and ask code-reviewer to check smells
3. **Explore graph**: Use syntagma-researcher to find pattern relationships
4. **Custom workflows**: Combine tools (analyze → suggest → search)

For more examples, see:
- [Alcove Integration](../docs/alcove-integration.md) — Team knowledge + Syntagma
- [Monitoring Setup](../monitoring/README.md) — Track pattern usage
- [API Reference](../docs/api.md) — REST endpoints
