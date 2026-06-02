---
name: episteme
description: "SW engineering knowledge graph — patterns, laws, refactorings, smells. Trigger: code quality concern, design decision, architecture review, or any engineering question."
---

# Episteme Knowledge Graph

Hybrid search + graph traversal over a curated database of design patterns, engineering laws, code smells, and refactoring techniques. All commands use the `epis` CLI.

## When to Use

Activate automatically when the user expresses any of the following, regardless of technical phrasing:

**Code problems — translate complaint into a knowledge graph query:**

| User says | Command |
|-----------|---------|
| "this class does too much" / file > 300 lines | `epis search "god class large class single responsibility" --json` |
| "this function is too long" | `epis search "long method extract method" --json` |
| "code is too complex" / hard to follow | `epis search "complexity smell cognitive overload" --json` |
| "calling DB directly in business logic" | `epis search "coupling persistence repository data access layer" --json` |
| "hard to test" / can't write unit tests | `epis search "testability dependency injection mockability" --json` |
| "copy-pasted this" / duplicated logic | `epis search "duplicated code clone smell" --json` |
| "changing X breaks Y" / change ripple | `epis search "brittle coupling change propagation rigidity" --json` |
| "adding a new type means touching everywhere" / growing switch | `epis search "open closed principle strategy polymorphism" --json` |
| "is this thread-safe?" / concurrency concerns | `epis search "thread safety race condition shared mutable state" --json` |
| "API is slow" / performance issues | `epis search "N+1 query lazy loading caching performance" --json` |
| User shares code for review | Write to temp file, then `epis analyze FILE --language LANG --json` then `epis infer FILE --language LANG --json` |
| User wants to refactor or improve code | Write to temp file, then `epis infer FILE --language LANG --json` |

**Architecture discussions:**

| User says | Command |
|-----------|---------|
| "microservices vs monolith" / how to split | `epis search "monolith microservice decomposition bounded context" --json` |
| "is this architecture okay?" / architecture review | `epis search "layered architecture coupling cohesion separation concerns" --json` |
| "where should this go?" / code placement | `epis search "responsibility assignment package structure" --json` |
| Team/org structure affects code | `epis search "Conway law organizational structure architecture" --json` |

**Follow-up exploration:**

| User says | Action |
|-----------|--------|
| Entity ID mentioned (DP-xxx, LAW-xxx, RF-xxx, SMELL-xxx) | `epis graph entity ID` |
| "how does X relate to Y" | `epis graph path FROM TO --json` or `epis graph neighbors ID --json` |
| "tell me more" about a result | `epis graph entity ID` for full details, `epis graph neighbors ID --json` for connections |

**Always:**
- Translate informal language into technical queries. User says "it's a tangled mess", you search "coupling tangled dependency".
- Present the named concept and explain it in the user's own language.
- Cite entity IDs (DP-005, LAW-003) in responses.

## CLI Commands

| Command | Purpose | Key flags |
|---------|---------|-----------|
| `epis search "QUERY"` | Hybrid keyword + semantic search | `--limit N` (default 5), `--entity-type TYPE` (pattern/refactoring/law/smell), `--json` |
| `epis graph entity ID` | Full details for a specific entity (always JSON output) | ID is required (e.g. DP-005) |
| `epis graph neighbors ID` | Explore related entities | `--relation-type TYPE` (solves/solved_by/enforces/violates/related_to), `--json` |
| `epis graph path FROM TO` | Trace connection between two concepts | `--max-depth N` (default 5), `--json` |
| `epis analyze FILE` | Detect code smells in a source file | `--language LANG`, `--min-confidence 0.0`, `--json` |
| `epis infer FILE` | Ranked refactoring suggestions for a source file | `--language LANG`, `--top-k N` (default 3), `--json` |
| `epis insight add "TITLE" "CONTENT"` | Add a user insight | `--tags "t1,t2"`, `--link "DP-005,SMELL-01"`, `--json` |
| `epis insight search "QUERY"` | Search user insights by keyword | `--limit N` (default 10), `--json` |

### Code analysis workflow

For `epis analyze` / `epis infer`, code must be written to a temp file first:

```bash
cat > /tmp/code_snippet.py << 'EOF'
{code}
EOF
epis analyze /tmp/code_snippet.py --language python --json
epis infer /tmp/code_snippet.py --language python --json
```

## Workflow Chains

Multi-step flows that chain commands into end-to-end workflows. After each chain completes, present interactive follow-up options to the user.

### Chain 1: Code Review Pipeline

```
epis analyze FILE --language LANG --json
  -> epis infer FILE --language LANG --json
  -> epis graph neighbors SMELL_ID --relation-type solved_by --json
  -> epis graph path SMELL_A SMELL_B --json  [for each pair of detected smells]
  -> Report with causation graph
  -> Present options:
     1. Apply refactoring [RF-xxx] -> spawn refactoring-expert agent
     2. Deep dive into [SMELL-xxx] root cause -> spawn episteme-advisor agent
     3. Architecture assessment -> spawn architecture-analyst agent
     4. Explain [entity ID] in detail -> epis graph entity ID
```

### Chain 2: Architecture Review Pipeline

```
epis search "QUERY" --json
  -> epis graph entity TOP_RESULT
  -> epis graph neighbors ENTITY --relation-type enforces --json
  -> epis graph neighbors ENTITY --relation-type violates --json
  -> epis graph path ENTITY_A ENTITY_B --json
  -> Report with compliance scores and risk assessment
  -> Present options:
     1. Get refactoring plan for violations -> spawn code-reviewer agent
     2. Advisory on resolving tensions -> spawn episteme-advisor agent
     3. Research alternatives -> spawn episteme-researcher agent
     4. Deep dive into specific law/pattern -> epis graph entity ID
```

### Chain 3: Problem Diagnosis Pipeline

```
epis search "SYMPTOM_QUERY" --json
  -> epis graph entity TOP_RESULT
  -> epis graph neighbors ENTITY --relation-type solved_by --json
  -> epis graph neighbors ENTITY --relation-type violates --json
  -> Report: Root Cause -> Symptoms -> Fixes -> Trade-offs
  -> Present options:
     1. Apply recommended fix -> spawn refactoring-expert agent
     2. Get advisory on approach -> spawn episteme-advisor agent
     3. Verify fix works -> epis infer FILE --json on proposed solution
     4. Explore related patterns -> epis graph neighbors ID --relation-type related_to --json
```

### Chain 4: Learning & Exploration Pipeline

```
epis search "TOPIC" --json
  -> epis graph entity RESULTS
  -> epis graph neighbors ENTITY --relation-type related_to --json
  -> epis graph path ENTITY_A ENTITY_B --json  [for contrasting concepts]
  -> Report: Core Concept -> Related -> Contrasting -> When to Use
  -> Present options:
     1. See practical code examples -> epis search "ENTITY example" --json
     2. Apply this pattern to my code -> spawn code-reviewer agent
     3. Compare alternatives -> spawn episteme-researcher agent
```

## Cross-Tool Chaining Rules

These rules ensure every command leads naturally to the next:

| After running... | Always follow up with... |
|-------------------|--------------------------|
| `epis analyze` | `epis infer` on the same file for ranked refactorings |
| `epis infer` | `epis graph neighbors SMELL_ID --relation-type solved_by --json` to check for alternative fixes |
| `epis search` | `epis graph entity` on top 1-2 results for full context |
| `epis graph entity` (smell) | `epis graph neighbors ID --relation-type violates --json` to find impacted principles |
| `epis graph entity` (pattern) | `epis graph neighbors ID --relation-type enforces --json` to find enforced laws |
| `epis graph entity` (refactoring) | `epis graph neighbors ID --relation-type solved_by --json` inverse to find what it solves |
| Multiple smells detected | `epis graph path SMELL_A SMELL_B --json` to map causation |
| `epis graph neighbors` returns >3 entities | Summarize and offer `epis graph entity` on user's choice |

## Agent Handoff Protocol

When presenting next-step options after any workflow chain, use this format:

```
## Next Steps
1. **[Action verb]** -- [Description] -> [agent name or command]
2. **[Action verb]** -- [Description] -> [agent name or command]
3. **[Action verb]** -- [Description] -> [agent name or command]
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

1. Write code to a temp file, then run `epis analyze /tmp/code_snippet.LANG --language LANG --json`.
2. If smells found, run `epis infer /tmp/code_snippet.LANG --language LANG --json` for ranked fixes.
3. For each smell, run `epis graph entity SMELL_ID` to map it to principles and laws.
4. For each pair of smells, run `epis graph path SMELL_A SMELL_B --json` to discover causation chains.
5. Present: smell name (entity ID), causation graph, refactoring options, and the underlying principle.
6. Offer follow-up actions per the Agent Handoff Protocol.

### Architecture discussion workflow

1. Run `epis search "QUERY" --json` with the core architectural question.
2. For top results, run `epis graph neighbors ID --json` to find complementary and conflicting concepts.
3. Run `epis graph path ENTITY_A ENTITY_B --json` between opposing concepts to surface trade-off chains.
4. Present grounded trade-off analysis citing entity IDs.
5. Offer follow-up actions per the Agent Handoff Protocol.

### Problem diagnosis workflow

1. Translate the user's informal complaint into 2-3 technical search queries.
2. Run `epis search "QUERY" --json` with each, merge results by relevance score.
3. Deep-dive top 2-3 entities with `epis graph entity ID`.
4. Explore relationships with `epis graph neighbors ID --json` to find root causes.
5. Map causation with `epis graph path SMELL_A SMELL_B --json` between related smells.
6. Synthesize: Problem Summary, Root Cause (entity IDs), Actionable Recommendations, Trade-offs.
7. Offer follow-up actions per the Agent Handoff Protocol.

## Entity ID Conventions

- Design Patterns: `DP-001` ... `DP-NNN`
- Refactorings: `RF-001` ... `RF-NNN`
- Engineering Laws: `LAW-001` ... `LAW-NNN`
- Code Smells: `SMELL-001` ... `SMELL-NNN`
