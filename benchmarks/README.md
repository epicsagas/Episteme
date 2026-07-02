# Episteme benchmarks

Search-quality, smell-detection, and traversal benchmarks for the Episteme
knowledge graph. The composite score and regression gates are described in
`AGENTS.md` (run `python3 benchmarks/eval_runner.py full`).

## How to run the lightweight-model boost eval

**Question under test:** does querying Episteme's knowledge graph measurably
improve a LIGHTWEIGHT model's (e.g. 7B-class local) software-engineering
answers versus the same model alone?

This eval tests the defensible core of Episteme's value proposition — not
"does the graph know things frontier LLMs don't" (it doesn't), but "does it
boost models that don't already know this".

### Prerequisites

1. **Episteme HTTP API running** with the knowledge graph loaded:
   ```bash
   epis api                      # foreground, default port 58302
   # or: epis api serve --port 58302
   curl -s localhost:58302/health   # expect status: ok, knowledge_graph: loaded
   ```
2. **ollama running** with a lightweight model:
   ```bash
   ollama pull qwen2.5-coder:latest     # 7.6B Q4_K_M, ~4.7 GB
   curl -s localhost:11434/api/tags     # confirm it's listed
   ```
3. Python >= 3.9 (stdlib only — no pip install needed).

### Run

```bash
# default: qwen2.5-coder:latest, top-5 Episteme hits, temperature=0
python3 benchmarks/boost_lightweight.py

# other lightweight models
python3 benchmarks/boost_lightweight.py --model qwen2.5-coder:3b
python3 benchmarks/boost_lightweight.py --model llama3.2:3b
python3 benchmarks/boost_lightweight.py --model gemma3:latest

# write results to a specific path
python3 benchmarks/boost_lightweight.py --out benchmarks/results/boost_run.json
```

### What it measures

For each of the 18 scenario questions in `lightweight_eval_bank.json`
(6 code smells, 6 design patterns, 6 laws/principles — a mix of easy and
obscure items), it runs the same model under two conditions:

- **ALONE** — the question only.
- **BOOSTED** — the top-k `Episteme /search` hits are prepended as a
  "Relevant knowledge" block, then the same question.

Both calls use `temperature=0` (greedy, reproducible). Scoring is
deterministic and **no model is in the scoring loop**: each question has a
pre-registered rubric (a list of concept-groups with aliases); a concept is
"hit" if any alias appears in the lowercased, markdown-stripped answer.
Per-question score = concepts_hit / concepts_required (range 0..1).

### Output

A per-question table (alone vs boosted vs delta), aggregate scores overall
and per category, and a written JSON file (`benchmarks/results/boost_run.json`
by default) containing every model answer for audit. A positive `delta` on
the harder/obscure questions is the expected signal that the graph helps the
model where its parametric memory is weak.
