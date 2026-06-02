---
name: episteme
description: "SW engineering knowledge graph — patterns, laws, refactorings, smells. Trigger: code quality concern, design decision, architecture review, or any engineering question."
---

# Episteme Knowledge Graph

Hybrid search + graph traversal over a curated database of design patterns, engineering laws, code smells, and refactoring techniques. All queries use the Episteme HTTP API via curl.

## Prerequisites

- API server must be running: check with `curl -sf http://localhost:58302/health`, start with `epis api start`
- Default base URL: `http://localhost:58302`
- Auth (optional): `X-API-Key: $EPISTEME_API_KEY` header — dev mode if no key set

## When to Use

Activate automatically when the user expresses any of the following, regardless of technical phrasing:

**Code problems — translate complaint into a knowledge graph query:**

| User says | API call (run via Bash) |
|-----------|---------|
| "this class does too much" / file > 300 lines | `curl -s 'http://localhost:58302/search?q=god+class+large+class+single+responsibility&limit=5'` |
| "this function is too long" | `curl -s 'http://localhost:58302/search?q=long+method+extract+method&limit=5'` |
| "code is too complex" / hard to follow | `curl -s 'http://localhost:58302/search?q=complexity+smell+cognitive+overload&limit=5'` |
| "calling DB directly in business logic" | `curl -s 'http://localhost:58302/search?q=coupling+persistence+repository+data+access+layer&limit=5'` |
| "hard to test" / can't write unit tests | `curl -s 'http://localhost:58302/search?q=testability+dependency+injection+mockability&limit=5'` |
| "copy-pasted this" / duplicated logic | `curl -s 'http://localhost:58302/search?q=duplicated+code+clone+smell&limit=5'` |
| "changing X breaks Y" / change ripple | `curl -s 'http://localhost:58302/search?q=brittle+coupling+change+propagation+rigidity&limit=5'` |
| "adding a new type means touching everywhere" / growing switch | `curl -s 'http://localhost:58302/search?q=open+closed+principle+strategy+polymorphism&limit=5'` |
| "is this thread-safe?" / concurrency concerns | `curl -s 'http://localhost:58302/search?q=thread+safety+race+condition+shared+mutable+state&limit=5'` |
| "API is slow" / performance issues | `curl -s 'http://localhost:58302/search?q=N%2B1+query+lazy+loading+caching+performance&limit=5'` |
| User shares code for review | `curl -s -X POST http://localhost:58302/analyze -H 'Content-Type: application/json' -d '{"code":"...","language":"python"}'` then `curl -s -X POST http://localhost:58302/refactor -H 'Content-Type: application/json' -d '{"code":"...","language":"python"}'` |
| User wants to refactor or improve code | `curl -s -X POST http://localhost:58302/refactor -H 'Content-Type: application/json' -d '{"code":"...","language":"python"}'` |

**Architecture discussions:**

| User says | API call (run via Bash) |
|-----------|---------|
| "microservices vs monolith" / how to split | `curl -s 'http://localhost:58302/search?q=monolith+microservice+decomposition+bounded+context&limit=5'` |
| "is this architecture okay?" / architecture review | `curl -s 'http://localhost:58302/search?q=layered+architecture+coupling+cohesion+separation+concerns&limit=5'` |
| "where should this go?" / code placement | `curl -s 'http://localhost:58302/search?q=responsibility+assignment+package+structure&limit=5'` |
| Team/org structure affects code | `curl -s 'http://localhost:58302/search?q=Conway+law+organizational+structure+architecture&limit=5'` |

**Follow-up exploration:**

| User says | API call (run via Bash) |
|-----------|--------|
| Entity ID mentioned (DP-xxx, LAW-xxx, RF-xxx, SMELL-xxx) | `curl -s 'http://localhost:58302/graph/DP-005?detail=full'` |
| "how does X relate to Y" | `curl -s -X POST http://localhost:58302/graph/path -H 'Content-Type: application/json' -d '{"from_id":"DP-005","to_id":"SMELL-01","max_depth":5}'` or `curl -s 'http://localhost:58302/graph/DP-005/neighbors'` |
| "tell me more" about a result | `curl -s 'http://localhost:58302/graph/ID?detail=full'` for full details, `curl -s 'http://localhost:58302/graph/ID/neighbors'` for connections |

**Always:**
- Translate informal language into technical queries. User says "it's a tangled mess", you search "coupling tangled dependency".
- Present the named concept and explain it in the user's own language.
- Cite entity IDs (DP-005, LAW-003) in responses.

## HTTP API Reference

All commands use `curl` via the Bash tool. Add `-H "X-API-Key: $EPISTEME_API_KEY"` if auth is configured.

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/search?q=QUERY&limit=N&type=TYPE` | GET | Hybrid keyword + semantic search |
| `/search` | POST | Search with JSON body: `{"query":"...","limit":5,"entity_type":"pattern"}` |
| `/graph/{ID}?detail=full` | GET | Full details for a specific entity |
| `/graph/{ID}/neighbors?type=solves` | GET | Explore related entities |
| `/graph/neighbors` | POST | Neighbors with JSON body: `{"entity_id":"DP-005","relation_type":"solves"}` |
| `/graph/path` | POST | Trace connection: `{"from_id":"DP-005","to_id":"SMELL-01","max_depth":5}` |
| `/analyze` | POST | Detect code smells: `{"code":"...","language":"python"}` |
| `/refactor` | POST | Ranked refactoring suggestions: `{"code":"...","language":"python","top_k":3}` |
| `/insights` | POST | Add user insight: `{"text":"...","tags":["t1"],"linked_entities":["DP-005"]}` |
| `/health` | GET | Health check |
| `/stats` | GET | Graph statistics |

### curl examples

```bash
# Search
curl -s 'http://localhost:58302/search?q=dependency+injection&limit=5'

# POST search with entity type filter
curl -s -X POST http://localhost:58302/search \
  -H 'Content-Type: application/json' \
  -d '{"query":"dependency injection","limit":5,"entity_type":"pattern"}'

# Get entity details
curl -s 'http://localhost:58302/graph/DP-005?detail=full'

# Get neighbors
curl -s 'http://localhost:58302/graph/DP-005/neighbors?type=enforces'

# POST neighbors
curl -s -X POST http://localhost:58302/graph/neighbors \
  -H 'Content-Type: application/json' \
  -d '{"entity_id":"DP-005","relation_type":"enforces"}'

# Find path between entities
curl -s -X POST http://localhost:58302/graph/path \
  -H 'Content-Type: application/json' \
  -d '{"from_id":"DP-005","to_id":"SMELL-01","max_depth":5}'

# Analyze code
curl -s -X POST http://localhost:58302/analyze \
  -H 'Content-Type: application/json' \
  -d '{"code":"class GodClass { ... }","language":"python"}'

# Get refactoring suggestions
curl -s -X POST http://localhost:58302/refactor \
  -H 'Content-Type: application/json' \
  -d '{"code":"class GodClass { ... }","language":"python","top_k":3}'

# Add insight
curl -s -X POST http://localhost:58302/insights \
  -H 'Content-Type: application/json' \
  -d '{"text":"Team decided to use Repository pattern","tags":["decision"],"linked_entities":["DP-005"]}'
```

### Code analysis workflow

For code analysis, pass the code string directly in the JSON body — no temp file needed:

```bash
# Analyze smells
curl -s -X POST http://localhost:58302/analyze \
  -H 'Content-Type: application/json' \
  -d '{"code":"class UserManager:\n    def create(self): pass\n    def delete(self): pass\n    def email(self): pass","language":"python"}'

# Get ranked refactorings
curl -s -X POST http://localhost:58302/refactor \
  -H 'Content-Type: application/json' \
  -d '{"code":"class UserManager:\n    def create(self): pass\n    def delete(self): pass\n    def email(self): pass","language":"python","top_k":3}'
```

## Workflow Chains

Multi-step flows that chain API calls into end-to-end workflows. After each chain completes, present interactive follow-up options to the user.

### Chain 1: Code Review Pipeline

```
curl -s -X POST http://localhost:58302/analyze -d '{"code":"...","language":"LANG"}'
  -> curl -s -X POST http://localhost:58302/refactor -d '{"code":"...","language":"LANG"}'
  -> curl -s 'http://localhost:58302/graph/SMELL_ID/neighbors?type=solved_by'
  -> curl -s -X POST http://localhost:58302/graph/path -d '{"from_id":"SMELL_A","to_id":"SMELL_B"}'  [for each pair of detected smells]
  -> Report with causation graph
  -> Present options:
     1. Apply refactoring [RF-xxx] -> spawn refactoring-expert agent
     2. Deep dive into [SMELL-xxx] root cause -> spawn episteme-advisor agent
     3. Architecture assessment -> spawn architecture-analyst agent
     4. Explain [entity ID] in detail -> curl -s 'http://localhost:58302/graph/ID?detail=full'
```

### Chain 2: Architecture Review Pipeline

```
curl -s 'http://localhost:58302/search?q=QUERY&limit=5'
  -> curl -s 'http://localhost:58302/graph/TOP_RESULT?detail=full'
  -> curl -s 'http://localhost:58302/graph/ENTITY/neighbors?type=enforces'
  -> curl -s 'http://localhost:58302/graph/ENTITY/neighbors?type=violates'
  -> curl -s -X POST http://localhost:58302/graph/path -d '{"from_id":"ENTITY_A","to_id":"ENTITY_B"}'
  -> Report with compliance scores and risk assessment
  -> Present options:
     1. Get refactoring plan for violations -> spawn code-reviewer agent
     2. Advisory on resolving tensions -> spawn episteme-advisor agent
     3. Research alternatives -> spawn episteme-researcher agent
     4. Deep dive into specific law/pattern -> curl -s 'http://localhost:58302/graph/ID?detail=full'
```

### Chain 3: Problem Diagnosis Pipeline

```
curl -s 'http://localhost:58302/search?q=SYMPTOM_QUERY&limit=5'
  -> curl -s 'http://localhost:58302/graph/TOP_RESULT?detail=full'
  -> curl -s 'http://localhost:58302/graph/ENTITY/neighbors?type=solved_by'
  -> curl -s 'http://localhost:58302/graph/ENTITY/neighbors?type=violates'
  -> Report: Root Cause -> Symptoms -> Fixes -> Trade-offs
  -> Present options:
     1. Apply recommended fix -> spawn refactoring-expert agent
     2. Get advisory on approach -> spawn episteme-advisor agent
     3. Verify fix works -> curl -s -X POST http://localhost:58302/refactor -d '{"code":"...","language":"python"}' on proposed solution
     4. Explore related patterns -> curl -s 'http://localhost:58302/graph/ID/neighbors?type=related_to'
```

### Chain 4: Learning & Exploration Pipeline

```
curl -s 'http://localhost:58302/search?q=TOPIC&limit=5'
  -> curl -s 'http://localhost:58302/graph/ID?detail=full' [for each result]
  -> curl -s 'http://localhost:58302/graph/ID/neighbors?type=related_to'
  -> curl -s -X POST http://localhost:58302/graph/path -d '{"from_id":"ENTITY_A","to_id":"ENTITY_B"}'  [for contrasting concepts]
  -> Report: Core Concept -> Related -> Contrasting -> When to Use
  -> Present options:
     1. See practical code examples -> curl -s 'http://localhost:58302/search?q=ENTITY+example&limit=5'
     2. Apply this pattern to my code -> spawn code-reviewer agent
     3. Compare alternatives -> spawn episteme-researcher agent
```

## Cross-Tool Chaining Rules

These rules ensure every API call leads naturally to the next:

| After running... | Always follow up with... |
|-------------------|--------------------------|
| `POST /analyze` | `POST /refactor` on the same code for ranked refactorings |
| `POST /refactor` | `GET /graph/{SMELL_ID}/neighbors?type=solved_by` to check for alternative fixes |
| `GET /search` | `GET /graph/{ID}?detail=full` on top 1-2 results for full context |
| `GET /graph/{ID}?detail=full` (smell) | `GET /graph/{ID}/neighbors?type=violates` to find impacted principles |
| `GET /graph/{ID}?detail=full` (pattern) | `GET /graph/{ID}/neighbors?type=enforces` to find enforced laws |
| `GET /graph/{ID}?detail=full` (refactoring) | `GET /graph/{ID}/neighbors?type=solved_by` inverse to find what it solves |
| Multiple smells detected | `POST /graph/path` between smell pairs to map causation |
| `GET /graph/{ID}/neighbors` returns >3 entities | Summarize and offer `GET /graph/{ID}?detail=full` on user's choice |

## Agent Handoff Protocol

When presenting next-step options after any workflow chain, use this format:

```
## Next Steps
1. **[Action verb]** -- [Description] -> [agent name or API call]
2. **[Action verb]** -- [Description] -> [agent name or API call]
3. **[Action verb]** -- [Description] -> [agent name or API call]
```

Available agents for handoff:
- `episteme-advisor` -- For trade-off analysis, decision guidance, and root-cause deep dives
- `episteme-researcher` -- For broad research across entity types and finding alternatives
- `code-reviewer` -- For code smell detection, refactoring ranking, and causation analysis
- `architecture-analyst` -- For system-level evaluation against laws and structural risks
- `refactoring-expert` (external) -- For implementing specific refactoring steps on code

Handoff rules:
- After **detection**: offer implementation (refactoring-expert) or deeper analysis (episteme-advisor)
- After **analysis**: offer alternative approaches (episteme-researcher) or architecture impact (architecture-analyst)
- After **advisory**: offer verification (code-reviewer) or implementation (refactoring-expert)
- Never hand off without context -- always pass the entity IDs and key findings to the next agent

## Agent Instructions

### Code review / analysis workflow

1. Run `curl -s -X POST http://localhost:58302/analyze -H 'Content-Type: application/json' -d '{"code":"...","language":"LANG"}'` to detect smells.
2. If smells found, run `curl -s -X POST http://localhost:58302/refactor -H 'Content-Type: application/json' -d '{"code":"...","language":"LANG"}'` for ranked fixes.
3. For each smell, run `curl -s 'http://localhost:58302/graph/SMELL_ID?detail=full'` to map it to principles and laws.
4. For each pair of smells, run `curl -s -X POST http://localhost:58302/graph/path -H 'Content-Type: application/json' -d '{"from_id":"SMELL_A","to_id":"SMELL_B"}'` to discover causation chains.
5. Present: smell name (entity ID), causation graph, refactoring options, and the underlying principle.
6. Offer follow-up actions per the Agent Handoff Protocol.

### Architecture discussion workflow

1. Run `curl -s 'http://localhost:58302/search?q=QUERY&limit=5'` with the core architectural question.
2. For top results, run `curl -s 'http://localhost:58302/graph/ID/neighbors'` to find complementary and conflicting concepts.
3. Run `curl -s -X POST http://localhost:58302/graph/path -H 'Content-Type: application/json' -d '{"from_id":"ENTITY_A","to_id":"ENTITY_B"}'` between opposing concepts to surface trade-off chains.
4. Present grounded trade-off analysis citing entity IDs.
5. Offer follow-up actions per the Agent Handoff Protocol.

### Problem diagnosis workflow

1. Translate the user's informal complaint into 2-3 technical search queries.
2. Run `curl -s 'http://localhost:58302/search?q=QUERY&limit=5'` with each, merge results by relevance score.
3. Deep-dive top 2-3 entities with `curl -s 'http://localhost:58302/graph/ID?detail=full'`.
4. Explore relationships with `curl -s 'http://localhost:58302/graph/ID/neighbors'` to find root causes.
5. Map causation with `curl -s -X POST http://localhost:58302/graph/path -H 'Content-Type: application/json' -d '{"from_id":"SMELL_A","to_id":"SMELL_B"}'` between related smells.
6. Synthesize: Problem Summary, Root Cause (entity IDs), Actionable Recommendations, Trade-offs.
7. Offer follow-up actions per the Agent Handoff Protocol.

## Entity ID Conventions

- Design Patterns: `DP-001` ... `DP-NNN`
- Refactorings: `RF-001` ... `RF-NNN`
- Engineering Laws: `LAW-001` ... `LAW-NNN`
- Code Smells: `SMELL-001` ... `SMELL-NNN`
