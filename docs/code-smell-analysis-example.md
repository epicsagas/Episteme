# Code Smell Analysis Example: Claudy

> Real-world analysis of a Rust hexagonal-architecture CLI using Syntagma MCP tools. Demonstrates how to systematically detect, categorize, and plan refactoring for code smells in a production codebase.

## Target Project

- **Repository**: epicsagas/claudy
- **Language**: Rust 2024 edition
- **Architecture**: Hexagonal (ports & adapters)
- **Total lines**: 21,483 across ~60 source files
- **Analysis date**: 2026-05-05

## Methodology

1. **Metric scan** — `wc -l` per file + public function count per file to identify size outliers
2. **Knowledge graph query** — `search_knowledge` for relevant smell definitions (SMELL-01 through SMELL-23)
3. **Targeted deep review** — Agent-based analysis of the 6 largest/most complex files
4. **Cross-file pattern detection** — Duplication audit across adapter trio (Telegram/Slack/Discord)

## Findings Summary

| Severity | Smell Type | Files Affected | Instances |
|----------|-----------|----------------|-----------|
| Critical | God Object (SMELL-21) | `sqlite_store.rs`, `channel_cmd.rs`, `commands.rs` | 3 |
| Critical | Long Method (SMELL-01) | `sqlite_store.rs`, `event_dispatch.rs`, `channel_cmd.rs` | 6 |
| High | Duplicate Code (SMELL-13) | `sqlite_store.rs`, `commands.rs`, webhook handlers | 5 patterns |
| Medium | Data Clumps (SMELL-05) | `commands.rs` | 11+ call sites |
| Medium | Feature Envy (SMELL-18) | `channel_cmd.rs`, `commands.rs` | 3 |
| Medium | Divergent Change | `registry.rs` (AppRegistry) | 1 |
| Low | Primitive Obsession (SMELL-03) | `sqlite_store.rs` | Throughout |

---

## Detailed Analysis

### 1. God Object: `sqlite_store.rs` (1,402 lines)

**Smell**: SMELL-21 (God Object) + SMELL-04 (Large Class)

A single `SqliteAnalyticsStore` struct implements two trait interfaces and carries six distinct responsibilities:

| Responsibility | Lines | Methods |
|----------------|-------|---------|
| Schema management | 1000-1120 | `initialize_schema`, `SCHEMA` constant |
| CRUD for projects | 71-137 | `upsert_project`, `get_project_by_encoded_dir`, `list_projects` |
| CRUD for sessions/turns | 139-273 | `upsert_session`, `update_session_completion`, `get_sessions`, etc. |
| CRUD for token usage / tool calls | 275-329 | `insert_token_usage`, `insert_tool_call`, etc. |
| Aggregate analytics queries | 425-842 | `aggregate_token_trends`, `aggregate_tool_distribution`, etc. |
| Pricing data access | 844-997 | `upsert_model_pricing`, `recalculate_costs`, etc. |

**Principle violations**: SRP (one struct changes for 6+ independent reasons), DRY (conditional query pattern repeated 11 times).

### 2. Long Method: `aggregate_cost_metrics` (208 lines)

**Smell**: SMELL-01 (Long Method)

```
sqlite_store.rs:634-842  (208 lines)
```

Contains four sub-queries each with duplicated `match project_id` arms, deep nesting (4 levels), and mixed concerns (query building, parameter binding, row mapping, arithmetic).

**Other long methods**:

| Method | File | Lines |
|--------|------|-------|
| `handle_text_message` | `event_dispatch.rs` | ~300 |
| `run_add` | `channel_cmd.rs` | 140 |
| `prompt_channel_overrides` | `channel_cmd.rs` | 107 |
| `aggregate_dashboard_stats` | `sqlite_store.rs` | 99 |
| `handle_history` | `commands.rs` | 75 |

### 3. Duplicate Code: Conditional Query Building (11 sites)

**Smell**: SMELL-13 (Duplicate Code)

The same `match project_id { Some(pid) => sql_a, None => sql_b }` pattern appears 11 times in `sqlite_store.rs`, each producing nearly identical SQL differing only by an `AND project_id = ?N` clause.

**Locations**: lines 186-227, 427-448, 469-502, 541-558, 561-584, 587-609, 642-659, 662-672, 675-700, 774-785.

### 4. Duplicate Code: Authorize-and-Spawn (6 sites)

**Smell**: SMELL-13 (Duplicate Code)

Every webhook handler and Discord gateway handler contains the same 15-line block: extract user_id, check authorization, log on failure, clone state, spawn async task.

**Locations**:
- `webhook_handlers.rs`: lines 59-76, 148-165, 195-210, 282-298
- `discord/gateway.rs`: lines 274-286, 305-316

### 5. Duplicate Code: ChannelIdentity Construction (9 sites)

**Smell**: SMELL-13 (Duplicate Code)

Every normalize function builds `ChannelIdentity` with the same field-order pattern, differing only in the `Platform` enum variant and ownership style.

**Locations**: `telegram/normalize.rs` (80-88, 142-150), `slack/normalize.rs` (104-110, 137-143), `discord/normalize.rs` (15-21, 71-77), plus 3 more in event construction.

### 6. Divergent Change: `AppRegistry`

**Smell**: SMELL-21 adjacent (Divergent Change)

`AppRegistry` in `registry.rs` (491 lines, 18 public functions) changes for four independent reasons:
- Persistence format changes
- Provider resolution strategy changes
- OpenRouter normalization rule changes
- Custom provider management changes

The `BridgeSettings` struct in the same file is already well-extracted and serves as the model for further decomposition.

---

## Refactoring Plan (Priority Order)

### Priority 1: Extract shared helpers (low risk, high DRY impact)

| Refactoring | Smell Addressed | Estimated Reduction |
|-------------|----------------|-------------------|
| `apply_project_filter(sql, project_id)` helper | Duplicate Code (SMELL-13) | ~80 lines |
| `authorize_and_spawn(state, platform, event)` | Duplicate Code (SMELL-13) | ~90 lines |
| `ChannelIdentity::new_for_platform()` constructor | Duplicate Code (SMELL-13) | ~40 lines |
| `send_text()` / `send_with_buttons()` in commands | Duplicate Code (SMELL-13) | ~60 lines |
| `ChannelState::with_write()` helper | Duplicate Code (SMELL-13) | ~30 lines |

### Priority 2: Extract long methods (medium risk)

| Refactoring | Target | Into |
|-------------|--------|------|
| Decompose `aggregate_cost_metrics` (208 lines) | `sqlite_store.rs:634-842` | 5 focused query methods |
| Decompose `handle_text_message` (~300 lines) | `event_dispatch.rs:12-311` | 4-5 handler phases |
| Decompose `run_add` (140 lines) | `channel_cmd.rs:335-475` | 4 prompt+persist functions |

### Priority 3: Split God Objects (higher risk, SRP alignment)

| Refactoring | Target | Into |
|-------------|--------|------|
| Split `SqliteAnalyticsStore` | `sqlite_store.rs` | `ProjectRepo`, `SessionRepo`, `AnalyticsQueryRepo`, `PricingRepo` |
| Split `channel_cmd` module | `channel_cmd.rs` | `dispatch.rs`, `interactive.rs`, `lifecycle.rs`, `status.rs` |
| Split `commands` module | `commands.rs` | `handlers.rs`, `callbacks.rs`, `formatting.rs`, `session_io.rs` |
| Extract `OpenRouterConfig` from `AppRegistry` | `registry.rs` | Separate struct for alias management |

---

## Tool Usage

```bash
# Step 1: File size metrics
find src -name "*.rs" -exec wc -l {} + | sort -rn | head -25

# Step 2: Function count metrics
find src -name "*.rs" | xargs grep -l "pub fn " | while read f; do \
  echo "$(grep -c 'pub fn ' $f) $f"; done | sort -rn | head -15

# Step 3: Knowledge graph queries (Syntagma MCP)
search_knowledge("god class large class single responsibility", entity_type="smell")
search_knowledge("long method function too many parameters", entity_type="smell")

# Step 4: Targeted code review (Agent subagents)
# - sqlite_store.rs deep analysis
# - channel_cmd.rs + commands.rs analysis
# - Cross-adapter duplication audit
# - registry.rs + state.rs God Object check
```

## Key Takeaways

1. **Hexagonal architecture helps but doesn't prevent adapter bloat** — the ports are clean, but adapters accumulate responsibilities when multiple entity types share one storage backend.
2. **Duplication clusters around platform variants** — Telegram/Slack/Discord adapters repeat patterns that differ only in enum variants. A shared helper layer eliminates this.
3. **Long methods are the root cause** — most God Objects started as reasonable files that grew through additive changes. Extracting methods first makes God Object decomposition obvious.
4. **Query builders are the highest-leverage DRY fix** — a single `apply_project_filter` helper eliminates 11 duplication sites in the largest file.
