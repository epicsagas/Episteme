---
name: syntagma
description: >
  Software engineering knowledge graph — patterns, laws, refactorings, smells.
  Activates on any code quality concern, design decision, architecture review,
  or engineering question — even when the user describes problems informally.
  Use the Syntagma MCP tools to ground every engineering answer in proven concepts.
---

# Syntagma Knowledge Graph

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

## Agent Instructions

### Code review / analysis workflow

1. Call `analyze_code` with the shared code snippet.
2. If smells found, call `suggest_refactorings` for ranked fixes.
3. For each smell, call `get_entity` to map it to principles and laws.
4. Present: smell name (entity ID), confidence, refactoring options, and the underlying principle.

### Architecture discussion workflow

1. Call `search_knowledge` with the core architectural question.
2. For top results, call `get_neighbors` to find complementary and conflicting concepts.
3. Present grounded trade-off analysis citing entity IDs.

### Problem diagnosis workflow

1. Translate the user's informal complaint into 2-3 technical search queries.
2. Call `search_knowledge` with each, merge results by relevance score.
3. Deep-dive top 2-3 entities with `get_entity(detail_level="detailed")`.
4. Explore relationships with `get_neighbors` to find root causes.
5. Synthesize: Problem Summary, Root Cause (entity IDs), Actionable Recommendations, Trade-offs.

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
