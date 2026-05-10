# Alcove Ecosystem — Architecture & Capability Analysis

> A detailed comparison of Episteme's Tacit Knowledge layer (TK-*) and the Alcove documentation ecosystem, covering storage models, search capabilities, lifecycle management, and use-case guidance.

---

## 1. Architecture Overview

### Episteme Tacit Knowledge (TK-*)

| Aspect | Detail |
|--------|--------|
| **Storage** | SQLite single file (`~/.episteme/user_knowledge.db`) |
| **Schema** | 5 tables: `user_entities`, `user_relations`, `user_embeddings`, `user_entities_fts` (FTS5 virtual), `insight_seq` |
| **Unit** | One insight = one `UserEntity` row (TK-xxx ID) |
| **Graph** | Merged with canonical graph via `CompositeGraph` at runtime — enables cross-layer path traversal (TK-001 → DP-005 → SMELL-01) |
| **Concurrency** | `Mutex<Connection>` + WAL mode for MCP + CLI simultaneous access |

### Alcove Documentation System

| Aspect | Detail |
|--------|--------|
| **Storage** | Markdown files on filesystem + Tantivy BM25 index + sqlite-vec embeddings |
| **Structure** | 3-tier classification: Core (7), Supplementary (19), Public (15) files per project |
| **Unit** | One structured Markdown file (PRD, ARCHITECTURE, DECISIONS, etc.) |
| **Graph** | wikilink + file-path based loose connections |
| **Concurrency** | File-based lock (`.index_lock`) per docs root, per-vault index isolation |
| **Vaults** | 3 symlinks to Obsidian PARA folders: areas (8 docs), resources (71), zettelkasten (17) |

---

## 2. Storage Model Comparison

### Episteme TK-* Schema

```sql
-- Core table
CREATE TABLE user_entities (
    id TEXT PRIMARY KEY,           -- TK-001, TK-002, ...
    title TEXT,                    -- Auto: first line, max 80 chars
    content TEXT,                  -- Free-text (no max length)
    author TEXT DEFAULT 'user',
    confidence REAL DEFAULT 0.5,   -- +0.05 per confirmed link, cap 1.0
    evidence_count INTEGER DEFAULT 0,
    last_validated TEXT DEFAULT '',
    tags TEXT DEFAULT '[]',        -- JSON array
    relations TEXT DEFAULT '{}',   -- JSON HashMap<relation_type, Vec<entity_id>>
    link_provenance TEXT DEFAULT '{}',
    created_at TEXT,
    updated_at TEXT
);

-- Normalized relations (derives_from, applies_to, supersedes)
CREATE TABLE user_relations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    from_id TEXT,
    relation_type TEXT,
    to_id TEXT,
    UNIQUE(from_id, relation_type, to_id)
);

-- FTS5 full-text search
CREATE VIRTUAL TABLE user_entities_fts USING fts5(title, content, tags, content=user_entities);
```

### Alcove File Structure

```
~/.alcove/
  config.toml                    # Global config (docs_root, core/team/public file lists, embedding model)
  docs -> symlink                # → Obsidian/SecondBrain/99-Archives/projects
  vaults/
    areas -> symlink             # → Obsidian/02-Areas (8 docs)
    resources -> symlink         # → Obsidian/03-Resources (71 docs)
    zettelkasten -> symlink      # → Obsidian/10-Zettelkasten (17 docs)
  models/                        # Cached ONNX embedding models
  logs/

<docs_root>/<project>/
  .alcove/
    index/                       # Tantivy BM25 index files
    index_meta.json              # File fingerprints (mtime + size)
    vectors.db                   # sqlite-vec embeddings
  PRD.md                         # Product requirements
  ARCHITECTURE.md                # System design
  PROGRESS.md                    # Milestones & status
  DECISIONS.md                   # Architecture Decision Records
  CONVENTIONS.md                 # Coding standards
  SECRETS_MAP.md                 # Environment variables & secrets
  DEBT.md                        # Technical debt register
```

---

## 3. Knowledge Character

| Dimension | Episteme TK-* | Alcove |
|-----------|---------------|--------|
| **Type** | Momentary insights, lessons learned, team decisions | Structured project documentation (requirements, architecture, decisions) |
| **Mutability** | Mutable (SQLite CRUD) | Mutable (file edits + index rebuild) |
| **Source** | User-contributed free text | User-written + agent-generated from templates |
| **Authority** | Personal/team observation | Team mandate / organizational policy |
| **Granularity** | Atomic (one insight per entry) | Sectioned (multiple ADRs per DECISIONS.md) |
| **Linking** | Auto-detected to canonical entities (keyword scoring) | Manual wikilinks + markdown links |
| **Versioning** | None (SQLite only) | Git-based (file = source of truth) |

### Insight Lifecycle (Episteme TK-*)

```
add_insight(text, tags?, project?, linked_entities?)
  │
  ├── Generate TK-xxx ID (atomic sequence)
  ├── detect_canonical_links() — keyword matching → top 5 canonical entities
  │     score >= 0.5 → Auto link (derives_from)
  │     score < 0.5 → Suggested link
  ├── FTS5 duplicate detection → DuplicateCandidate[]
  ├── Persist to SQLite + in-memory cache
  └── Return: { id, auto_links, suggested_links, duplicates, confidence }

confirm_links(id, accepted[], rejected[])
  │
  ├── Add derives_from/applies_to relations
  ├── Upgrade link_provenance source to "manual"
  ├── Bump confidence (+0.05 per link, cap 1.0)
  └── Persist updates

search_insights(query, limit?)
  │
  └── FTS5 MATCH query → ranked results
```

### Document Lifecycle (Alcove)

```
init_project(project_name, project_path?)
  │
  ├── Create 7 core docs from templates (PRD, ARCHITECTURE, ...)
  ├── Optionally create public docs (README, CHANGELOG, ...)
  └── Rebuild search index

validate_docs()
  │
  ├── Check required file existence
  ├── Check template placeholders (TODO, FIXME)
  ├── Check required section headings
  ├── Check minimum list item counts
  └── Return: pass/warn/fail per file

lint_project()
  │
  ├── Detect broken [[wikilinks]] and markdown links
  ├── Find orphan files (not linked from any doc)
  ├── Find stale markers (WIP, TODO, FIXME, DRAFT, DEPRECATED)
  └── Find stale year references (2+ years old)

audit_project()
  │
  ├── Scan private doc-repo for missing required docs
  ├── Scan public project repo for exposed internal docs
  ├── Classify files into tiers
  └── Return: suggested_actions[]
```

---

## 4. Search Capabilities

| Capability | Episteme TK-* | Alcove |
|------------|---------------|--------|
| **Engine** | FTS5 (keyword match) | Tantivy BM25 + sqlite-vec cosine similarity |
| **Fusion** | None | RRF (Reciprocal Rank Fusion, k=60) |
| **CJK** | No special support | NgramTokenizer (min=2, max=3) |
| **Chunking** | N/A (one row = one insight) | 200–500 char chunks |
| **Incremental** | N/A (single table) | mtime + size fingerprint comparison |
| **Vector search** | Schema exists (`user_embeddings`) but **not wired** | Fully operational (MultilingualE5Small, 384d) |
| **Scope** | Single database | Per-project or global (cross-project) |
| **Fallback** | None | grep substring match when no index |

---

## 5. Feature Completeness

| Feature | Episteme TK-* | Alcove |
|---------|---------------|--------|
| Create | `add_insight` | `init_project`, file editing |
| Read | `search_insights` (search only, no get by ID) | `get_doc_file`, `search_project_docs` |
| Update | Not exposed via MCP | Direct file edit + `rebuild_index` |
| Delete | Not exposed via MCP | File delete + `rebuild_index` |
| Validation | None | `validate_docs`, `lint_project` |
| Audit | None | `audit_project` (public/private separation) |
| Backup | None | `backup_vault` (git commit snapshot) |
| Import | None | `promote_document` (Obsidian → doc-repo) |
| Policy | None | `policy.toml` with enforce levels |
| Templates | None | 7 core + 19 supplementary + 15 public |

---

## 6. Alcove Vault System

Three vaults, symlinked to Obsidian PARA structure:

| Vault | Target | Docs | Purpose |
|-------|--------|------|---------|
| `areas` | `02-Areas` | 8 | Domain areas: MCP agents, DevOps, Rust, LLM/RAG, Open Source |
| `resources` | `03-Resources` | 71 | Reference: AWS, Laws of Software Engineering, Technical docs |
| `zettelkasten` | `10-Zettelkasten` | 17 | Atomic notes: AI architecture, BM25, knowledge graphs, Rust patterns |

Each vault has independent:
- BM25 index (Tantivy)
- Vector database (sqlite-vec)
- File fingerprint tracking (`index_meta.json`)
- Cache isolation (separate `OnceLock<Mutex<HashMap>>`)

---

## 7. Alcove Configuration System

### Global: `~/.alcove/config.toml`

```toml
docs_root = "/path/to/Obsidian/SecondBrain/99-Archives/projects"

[core]
files = ["PRD.md", "ARCHITECTURE.md", "PROGRESS.md", "DECISIONS.md",
         "CONVENTIONS.md", "SECRETS_MAP.md", "DEBT.md"]

[team]
files = ["ENV_SETUP.md", "ONBOARDING.md", "DATA_MODEL.md", "SCHEMA.md",
         "DEPLOYMENT.md", "RUNBOOK.md", "PLAYBOOK.md", "MONITORING.md", ...]  # 19 files

[public]
files = ["README.md", "CHANGELOG.md", "CONTRIBUTING.md", "SECURITY.md", ...]  # 15 files

[embedding]
model = "MultilingualE5Small"
auto_download = true
enabled = true
```

### Per-project: `alcove.toml`

Overrides global defaults for: `diagram_format`, `core_files`, `team_files`, `public_files`.

### Policy: `policy.toml`

Defines:
- `enforce` level: `strict` | `warn` | `off`
- Required documents with section headings and minimum item counts
- Naming conventions (`UPPER_SNAKE`, `lower_snake`, `kebab`, `free`)
- Priority: project > team > built-in defaults

---

## 8. Use-Case Decision Matrix

| Situation | Recommended Tool | Rationale |
|-----------|-----------------|-----------|
| "Record a lesson learned from production incident" | **Episteme TK-*** | Auto-links to relevant smells/laws for future cross-referencing |
| "Start documentation for a new project" | **Alcove** `init_project` | 7 core templates auto-generated |
| "Check if any docs are outdated" | **Alcove** `lint_project` | Auto-detects WIP/TODO/DEPRECATED/stale dates |
| "Find what the team decided about auth middleware" | **Alcove** `search_project_docs` | Searches structured DECISIONS.md with BM25 + vector |
| "Detect code smells in a module" | **Episteme** `analyze_code` | Pattern/regex-based smell detection |
| "Ensure PRD has all required sections" | **Alcove** `validate_docs` | Policy-based section and item count validation |
| "Link an insight to Strategy pattern" | **Episteme** `confirm_links` | Creates `derives_from` edge to canonical entity |
| "Import Obsidian notes for agent access" | **Alcove** `promote_document` | Imports into doc-repo with auto project detection |
| "Find relationship between SRP and Extract Class" | **Episteme** `find_path` | Multi-hop graph traversal across entity types |
| "Back up project documentation state" | **Alcove** `backup_vault` | Git commit snapshot with timestamp |
| "Audit for exposed internal docs in public repo" | **Alcove** `audit_project` | Scans both private and public locations |
| "Get ranked refactoring suggestions for code" | **Episteme** `suggest_refactorings` | Composite scoring: severity × effort × principle alignment |

---

## 9. Complementary Roles

```
Episteme TK-*                     Alcove
"What universal principle          "What did our team
 applies here?"                     decide about this?"

 Momentary insight ←────────────→ Structured decision record
 Keyword auto-linking               Template-based scaffolding
 Cross-layer graph traversal         Cross-project document search
 Code analysis → smell detection     Doc analysis → staleness detection
```

**When both are active**: Episteme provides the universal "why" (laws, patterns), Alcove provides the project-specific "what we decided" (ADRs, conventions). Agents should cite both sources, with Alcove taking precedence when team rules conflict with generic guidance.

---

## 10. Scale & Performance

| Metric | Episteme TK-* | Alcove |
|--------|---------------|--------|
| **Designed capacity** | Hundreds of insights | ~10,000 files |
| **Search latency** | FTS5 instant (in-memory) | BM25 < 500ms for overview |
| **Token efficiency** | Single insight per result | Top-5 chunks ~1.5k tokens (vs ~8k for grep) |
| **Index rebuild** | Not needed (FTS5 triggers) | Incremental: only changed files |
| **Model size** | N/A (not wired) | 15MB (ArcticEmbedXS) to 2.3GB (BGE-M3) |

---

*See also: [Alcove Integration Guide](alcove-integration.md) for usage patterns and workflow examples.*
