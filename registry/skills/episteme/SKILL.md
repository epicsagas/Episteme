---
name: episteme
description: "SW engineering knowledge graph — patterns, laws, refactorings, smells. Trigger: code quality concern, design decision, architecture review, or any engineering question."
---

# Episteme Knowledge Graph

Hybrid search + graph traversal over a curated database of design patterns, engineering laws, code smells, and refactoring techniques.

## When to Use

Activate automatically when the user expresses any of the following, regardless of technical phrasing:

**Code problems — translate complaint into a knowledge graph query:**

| User says | Query |
|-----------|-------|
| "this class does too much" / file > 300 lines | `search_knowledge("god class large class single responsibility")` |
| "this function is too long" | `search_knowledge("long method extract method")` |
| "code is too complex" / hard to follow | `search_knowledge("complexity smell cognitive overload")` |
| "calling DB directly in business logic" | `search_knowledge("coupling persistence repository data access layer")` |
| "hard to test" / can't write unit tests | `search_knowledge("testability dependency injection mockability")` |
| "copy-pasted this" / duplicated logic | `search_knowledge("duplicated code clone smell")` |
| "changing X breaks Y" / change ripple | `search_knowledge("brittle coupling change propagation rigidity")` |
| "adding a new type means touching everywhere" / growing switch | `search_knowledge("open closed principle strategy polymorphism")` |
| "is this thread-safe?" / concurrency concerns | `search_knowledge("thread safety race condition shared mutable state")` |
| "API is slow" / performance issues | `search_knowledge("N+1 query lazy loading caching performance")` |
| User shares code for review | `analyze_code(code)` then `suggest_refactorings(code)` |
| User wants to refactor or improve code | `suggest_refactorings(code)` |

**Architecture discussions:**

| User says | Query |
|-----------|-------|
| "microservices vs monolith" / how to split | `search_knowledge("monolith microservice decomposition bounded context")` |
| "is this architecture okay?" / architecture review | `search_knowledge("layered architecture coupling cohesion separation concerns")` |
| "where should this go?" / code placement | `search_knowledge("responsibility assignment package structure")` |
| Team/org structure affects code | `search_knowledge("Conway law organizational structure architecture")` |

**Follow-up exploration:**

| User says | Action |
|-----------|--------|
| Entity ID mentioned (DP-xxx, LAW-xxx, RF-xxx, SMELL-xxx) | `get_entity(id)` |
| "how does X relate to Y" | `find_path` or `get_neighbors` |
| "tell me more" about a result | `get_entity` for full details, `get_neighbors` for connections |

**Always:**
- Translate informal language into technical queries. User says "it's a tangled mess", you search "coupling tangled dependency".
- Present the named concept and explain it in the user's own language.
- Cite entity IDs (DP-005, LAW-003) in responses.

## Tools

| Tool | Purpose | Key params |
|------|---------|------------|
| `search_knowledge` | Hybrid keyword + semantic search | `query` (required), `limit` (1-20, default 5), `entity_type` (pattern/refactoring/law/smell) |
| `get_entity` | Full details for a specific entity | `entity_id` (required, e.g. DP-005), `detail_level` (minimal/summary/detailed/full) |
| `get_neighbors` | Explore related entities | `entity_id` (required), `relation_type` (solves/solved_by/enforces/violates/related_to) |
| `find_path` | Trace connection between two concepts | `from_id`, `to_id` (both required), `max_depth` (1-10, default 5) |
| `analyze_code` | Detect code smells in source code | `code` (required), `language` (default "python") |
| `suggest_refactorings` | Ranked refactoring suggestions for code | `code` (required), `top_k` (1-10, default 3) |

## Workflow Chains

Multi-step flows that chain tools and agents into end-to-end workflows. After each chain completes, present interactive follow-up options to the user.

### Chain 1: Code Review Pipeline

```
analyze_code(code) → suggest_refactorings(code) → get_neighbors(smell, "solved_by")
  → find_path(smell_A, smell_B)  [for each pair of detected smells]
  → Report with causation graph
  → Present options:
     1. 🔧 Apply refactoring [RF-xxx] → spawn refactoring-expert agent
     2. 🔍 Deep dive into [SMELL-xxx] root cause → spawn episteme-advisor agent
     3. 🏗️ Architecture assessment → spawn architecture-analyst agent
     4. 📖 Explain [entity ID] in detail → get_entity(detail_level="full")
```

### Chain 2: Architecture Review Pipeline

```
search_knowledge(query) → get_entity(top_results) → get_neighbors(entity, "enforces")
  → get_neighbors(entity, "violates") → find_path(entity_A, entity_B)
  → Report with compliance scores and risk assessment
  → Present options:
     1. 🔧 Get refactoring plan for violations → spawn code-reviewer agent
     2. 💡 Advisory on resolving tensions → spawn episteme-advisor agent
     3. 📊 Research alternatives → spawn episteme-researcher agent
     4. 🏗️ Deep dive into specific law/pattern → get_entity(detail_level="full")
```

### Chain 3: Problem Diagnosis Pipeline

```
search_knowledge(symptom_queries) → get_entity(top_results)
  → get_neighbors(entity, "solved_by") → get_neighbors(entity, "violates")
  → Report: Root Cause → Symptoms → Fixes → Trade-offs
  → Present options:
     1. 🔧 Apply recommended fix → spawn refactoring-expert agent
     2. 💡 Get advisory on approach → spawn episteme-advisor agent
     3. 🔬 Verify fix works → suggest_refactorings on proposed solution
     4. 📖 Explore related patterns → get_neighbors with "related_to"
```

### Chain 4: Learning & Exploration Pipeline

```
search_knowledge(topic) → get_entity(results) → get_neighbors(entity, "related_to")
  → find_path(entity_A, entity_B)  [for contrasting concepts]
  → Report: Core Concept → Related → Contrasting → When to Use
  → Present options:
     1. 🔍 See practical code examples → search_knowledge with entity name + "example"
     2. 🏗️ Apply this pattern to my code → spawn code-reviewer agent
     3. 📊 Compare alternatives → spawn episteme-researcher agent
```

## Cross-Tool Chaining Rules

These rules ensure every tool call leads naturally to the next:

| After calling... | Always follow up with... |
|-------------------|--------------------------|
| `analyze_code` | `suggest_refactorings` on detected smells |
| `suggest_refactorings` | `get_neighbors(smell_id, "solved_by")` to check for alternative fixes |
| `search_knowledge` | `get_entity` on top 1-2 results for full context |
| `get_entity` (smell) | `get_neighbors(id, "violates")` to find impacted principles |
| `get_entity` (pattern) | `get_neighbors(id, "enforces")` to find enforced laws |
| `get_entity` (refactoring) | `get_neighbors(id, "solved_by")` inverse to find what it solves |
| Multiple smells detected | `find_path(smell_A, smell_B)` to map causation |
| `get_neighbors` returns >3 entities | Summarize and offer `get_entity` on user's choice |

## Agent Handoff Protocol

When presenting next-step options after any workflow chain, use this format:

```
## Next Steps
1. **[Action verb]** — [Description] → [agent name or tool]
2. **[Action verb]** — [Description] → [agent name or tool]
3. **[Action verb]** — [Description] → [agent name or tool]
```

Available agents for handoff:
- `episteme-advisor` — For trade-off analysis, decision guidance, and root-cause deep dives
- `episteme-researcher` — For broad research across entity types and finding alternatives
- `code-reviewer` — For code smell detection, refactoring ranking, and causation analysis
- `architecture-analyst` — For system-level evaluation against laws and structural risks
- `refactoring-expert` (external) — For implementing specific refactoring steps on code

Handoff rules:
- After **detection**: offer implementation (refactoring-expert) or deeper analysis (episteme-advisor)
- After **analysis**: offer alternative approaches (episteme-researcher) or architecture impact (architecture-analyst)
- After **advisory**: offer verification (code-reviewer) or implementation (refactoring-expert)
- Never hand off without context — always pass the entity IDs and key findings to the next agent

## Agent Instructions

### Code review / analysis workflow

1. Call `analyze_code` with the shared code snippet.
2. If smells found, call `suggest_refactorings` for ranked fixes.
3. For each smell, call `get_entity` to map it to principles and laws.
4. For each pair of smells, call `find_path` to discover causation chains.
5. Present: smell name (entity ID), causation graph, refactoring options, and the underlying principle.
6. Offer follow-up actions per the Agent Handoff Protocol.

### Architecture discussion workflow

1. Call `search_knowledge` with the core architectural question.
2. For top results, call `get_neighbors` to find complementary and conflicting concepts.
3. Call `find_path` between opposing concepts to surface trade-off chains.
4. Present grounded trade-off analysis citing entity IDs.
5. Offer follow-up actions per the Agent Handoff Protocol.

### Problem diagnosis workflow

1. Translate the user's informal complaint into 2-3 technical search queries.
2. Call `search_knowledge` with each, merge results by relevance score.
3. Deep-dive top 2-3 entities with `get_entity(detail_level="detailed")`.
4. Explore relationships with `get_neighbors` to find root causes.
5. Map causation with `find_path` between related smells.
6. Synthesize: Problem Summary, Root Cause (entity IDs), Actionable Recommendations, Trade-offs.
7. Offer follow-up actions per the Agent Handoff Protocol.

### Information density control

Match the user's requested depth without overloading:

| User says | `detail_level` | Approx. tokens per entity |
|-----------|----------------|--------------------------|
| "what is it?" / "quick definition" | minimal | ~50 |
| (default / no specification) | summary | ~100 |
| "how to apply?" / "explain in detail" | detailed | ~200 |
| "everything about" / "complete guide" | full | ~300 |

For multi-entity responses, cap total at ~500 tokens. Allocate 40% to top result, remainder split equally.

## Entity ID Conventions

- Design Patterns: `DP-001` ... `DP-NNN`
- Refactorings: `RF-001` ... `RF-NNN`
- Engineering Laws: `LAW-001` ... `LAW-NNN`
- Code Smells: `SMELL-001` ... `SMELL-NNN`
