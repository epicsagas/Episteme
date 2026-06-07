# Evaluation System

Episteme ships a comprehensive evaluation framework that measures search quality, smell detection accuracy, and graph traversal correctness. It detects regressions automatically on every PR.

## Quick Start

```bash
# Build the binary
cargo build

# Run all evaluation suites
python3 benchmarks/eval_runner.py full

# Run a single suite
python3 benchmarks/eval_runner.py search-positive
python3 benchmarks/eval_runner.py search-negative
python3 benchmarks/eval_runner.py smell-negative
python3 benchmarks/eval_runner.py analyze-positive
python3 benchmarks/eval_runner.py traversal
```

> Requires Python ≥ 3.12. The binary must be built first (`cargo build` produces `target/debug/episteme`).

---

## Architecture

```
benchmarks/
  eval_runner.py                  # 1040-line evaluation runner (6 suites)
  search_eval_set.json            # 58 positive search queries (3 tiers × 4 domains)
  search_negative_eval_set.json   # 70 negative search queries (homonyms, intent collisions)
  analyze_eval_set.json           # 50 positive smell detection cases
  traversal_eval_set.json         # 40 graph traversal cases (neighbors + paths)
  smell_negative_corpus/          # 22 clean code files across 6 languages
  test_eval_runner.py             # 281-line unit test suite
  search_benchmark.py             # Legacy search-only benchmark (still maintained)
  results/                        # Historical benchmark/eval results
  dashboard/                      # Svelte 5 visualization SPA
```

---

## Evaluation Suites

### 1. Search Positive (`search-positive`)

Measures whether `epis explore` finds the correct entities.

| Metric | Description |
|--------|-------------|
| hit@1 | Relevant entity in top 1 result |
| hit@3 | Relevant entity in top 3 results |
| hit@5 | Relevant entity in top 5 results |
| MRR@5 | Mean Reciprocal Rank at 5 |
| NDCG@5 | Normalized Discounted Cumulative Gain at 5 |

**Test set**: `search_eval_set.json` — 58 queries across 3 difficulty tiers:

| Tier | Queries | Category | Example |
|------|---------|----------|---------|
| Easy | 40 | `exact_name` | "strategy pattern" → DP-020 |
| Medium | 13 | `conceptual` | "swap algorithms at runtime" → DP-020 |
| Hard | 12 | `cross_domain` | "avoid rigid design that breaks on every change" → LAW-042, SMELL-09, RF-039, DP-020 |

### 2. Search Negative (`search-negative`)

Measures false positive rate — queries that should **not** return certain entities.

| Metric | Description |
|--------|-------------|
| FP@1 | Must-not-contain entity at rank 1 |
| FP@3 | Must-not-contain entity in top 3 |
| FP@5 | Must-not-contain entity in top 5 |
| Specificity | True negative rate (clean queries with no FP) |

**Test set**: `search_negative_eval_set.json` — 70 queries covering:

- **Homonyms**: "factory safety protocols" should not return Factory pattern (DP-003)
- **Partial matches**: "singleton" in non-software contexts
- **Intent collisions**: queries that share keywords but differ in intent
- **Cross-domain**: queries where domain-specific terms have different meanings

### 3. Smell Negative (`smell-negative`)

Runs `epis analyze` on **clean code** — files with no intentional smells. Any detection is a false positive.

| Metric | Description |
|--------|-------------|
| FP Rate | Fraction of clean files flagged |
| Specificity | 1 − FP Rate |
| Per-detector FP | Which smell detectors fire on clean code |
| Per-language FP | FP rate broken down by language |

**Test corpus**: `smell_negative_corpus/` — 22 files across 6 languages:

| Language | Files |
|----------|-------|
| Rust | 5 (builder, delegation, enum dispatch, init, data transfer) |
| Python | 4 (dataclass, fluent API, context manager, API docs) |
| TypeScript | 4 (factory, event emitter, value object, interface impl) |
| Go | 4 (config struct, error switch, functional options, interface delegation) |
| Ruby | 3 (case statement, delegation, struct) |
| Java | 2 (enum switch, record) |

### 4. Analyze Positive (`analyze-positive`)

Measures whether `epis analyze` correctly detects known smells in deliberately smelly code.

| Metric | Description |
|--------|-------------|
| Recall | Fraction of expected smells detected |
| Per-smell recall | Recall broken down by individual smell type |

**Test set**: `analyze_eval_set.json` — 50 cases with inline code snippets and expected smell IDs.

### 5. Traversal (`traversal`)

Measures graph traversal accuracy via `epis graph neighbors` and `epis graph path`.

| Metric | Description |
|--------|-------------|
| Neighbor recall | Expected neighbors found (≥50% coverage) |
| Path recall | Expected paths found within max depth |

**Test set**: `traversal_eval_set.json` — 40 cases (20 neighbor + 20 path, including negative path tests).

### 6. Full (`full`)

Runs all 5 suites above and computes a composite score.

---

## Composite Score

```
composite = 0.3 × recall + 0.3 × precision + 0.2 × specificity + 0.2 × smell_recall
```

| Component | Source | Weight | What it measures |
|-----------|--------|--------|------------------|
| recall | search-positive hit@5 | 0.3 | Does search find the right answers? |
| precision | 1 − search-negative FP@5 | 0.3 | Does search avoid wrong answers? |
| specificity | smell-negative specificity | 0.2 | Does smell detection avoid false alarms on clean code? |
| smell_recall | analyze-positive recall | 0.2 | Does smell detection find known smells? |

The four-component formula ensures the composite reflects both search quality (60%) and smell detection quality (40%), with both positive accuracy (recall) and negative accuracy (precision/specificity) represented.

---

## Regression Detection

The runner automatically compares against the previous result and **fails (exit 1)** if:

| Condition | Threshold |
|-----------|-----------|
| Composite score dropped | ≥ 0.02 |
| Any individual metric dropped | ≥ 0.05 |
| Search negative rank-1 FP detected | Any occurrence |

This runs in CI (`.github/workflows/eval.yml`) on every PR that touches `src/`, `meta/`, or `benchmarks/`.

---

## CLI Reference

```bash
# Run all suites
python3 benchmarks/eval_runner.py full

# Run individual suites
python3 benchmarks/eval_runner.py search-positive
python3 benchmarks/eval_runner.py search-negative
python3 benchmarks/eval_runner.py smell-negative
python3 benchmarks/eval_runner.py analyze-positive
python3 benchmarks/eval_runner.py traversal

# Options
python3 benchmarks/eval_runner.py full \
  --bin target/debug/episteme \    # binary path (default: target/debug/episteme)
  --top-k 5 \                      # top-K for search (default: 5)
  --repeats 1 \                    # latency samples per query (default: 1)
  --min-confidence 0.5 \           # smell detection threshold (default: 0.5)
  --compare benchmarks/results/latest.json \  # compare against previous run
  --output benchmarks/results/eval_custom.json  # custom output path
```

Results are saved to `benchmarks/results/eval_<timestamp>.json` with a `latest.json` symlink for easy comparison.

---

## CI Integration

`.github/workflows/eval.yml` triggers on:

- Pull requests affecting `src/**`, `meta/**`, `benchmarks/**`
- Push to `main` branch

The CI runs `eval_runner.py full` and fails the PR if regression is detected.

---

## Dashboard

The `benchmarks/dashboard/` directory contains a Svelte 5 SPA for visualizing evaluation results over time:

```bash
cd benchmarks/dashboard
npm install
npm run dev
```

Features:
- Latency trend charts (mean/p95 across runs)
- Tier breakdown (easy/medium/hard metrics)
- Per-query drill-down tables

---

## Adding New Test Cases

### Search queries

Add to `search_eval_set.json` (positive) or `search_negative_eval_set.json` (negative):

```json
{
  "query": "your query here",
  "relevant_ids": ["DP-001"],
  "tier": "easy",
  "category": "exact_name",
  "domain": "design_patterns"
}
```

For negative cases:

```json
{
  "query": "factory safety protocols",
  "must_not_contain": ["DP-003", "DP-001"],
  "category": "homonym",
  "domain": "design_patterns"
}
```

### Smell detection cases

Add inline code cases to `analyze_eval_set.json`:

```json
{
  "id": "CASE-051",
  "language": "rust",
  "expected_smells": ["SMELL-01"],
  "code": "fn long_function() { ... }"
}
```

### Clean corpus files

Add clean code files to `smell_negative_corpus/` following the naming convention `clean_<description>.<ext>`. Files must demonstrate legitimate patterns that should **not** trigger smell detectors.

### Graph traversal cases

Add to `traversal_eval_set.json`:

```json
{
  "id": "NEIGHBOR-021",
  "type": "neighbors",
  "entity_id": "SMELL-01",
  "expected_neighbors": ["RF-001", "RF-002"],
  "min_expected_count": 3
}
```

---

## Baseline Results

Initial baseline from PR #41 (commit `9471089`):

```
Composite Score: 0.5461
  Recall:      0.7692  (search positive)
  Precision:   0.3857  (search negative)
  Specificity: 0.4091  (smell negative)

Search Positive:  hit@1=0.74  hit@3=0.75  hit@5=0.77  MRR=0.75  NDCG=0.65
Search Negative:  FP@1=0.49   FP@3=0.59   FP@5=0.61   Specificity=0.39
Smell Negative:   FP Rate=0.59  (13/22 files flagged)
Traversal:        Neighbors=0.95  Paths=1.00
```

Key findings that drove FP reduction work:
- **SMELL-11 (Lazy Class)** fires on 19/22 clean files — threshold too aggressive
- **43/70 negative search queries** trigger at least one FP — homonym handling needed
- **SMELL-14, 18, 20** show 0% recall — detector thresholds need tuning
