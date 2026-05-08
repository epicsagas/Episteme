# Episteme Benchmark Dashboard

A Svelte 5 single-page application for visualizing search quality and latency benchmark results from the `epis explore` CLI command.

## Features

- **Trend Chart** — Line chart tracking hit@1, MRR@5, and NDCG@5 across all benchmark runs over time
- **Latency Summary** — Quality metrics card (hit@1/3/5, MRR, NDCG) plus a bar chart showing latency distribution (min / p50 / mean / p95 / p99 / max)
- **Per-Query Table** — Sortable table with color-coded rows: red for hit@1 miss, yellow for hit@1 hit but imperfect rank, white for perfect retrieval
- **Query Modal** — Click any query row to inspect relevant IDs vs. actual top-k IDs side by side, with color-coded match chips

## Stack

| Layer | Technology |
|-------|-----------|
| Framework | [Svelte 5](https://svelte.dev) |
| Bundler | [Vite 6](https://vite.dev) |
| Charts | [Chart.js 4](https://www.chartjs.org) |
| Data source | `../results/*.json` (loaded via `import.meta.glob`) |

## Getting Started

### Prerequisites

- Node.js 18+
- npm 9+

### Development

```bash
cd benchmarks/dashboard
npm install
npm run dev
```

Open [http://localhost:5173](http://localhost:5173) in your browser.

### Production Build

```bash
npm run build      # outputs to dist/
npm run preview    # preview the production build locally
```

## Adding Benchmark Runs

The dashboard automatically picks up any JSON file matching `benchmarks/results/search_benchmark_*.json`. To add a new run:

```bash
# From the project root
python benchmarks/search_benchmark.py --bin target/release/episteme
```

Then rebuild the dashboard (`npm run build`) or restart the dev server — the new run will appear in the run selector dropdown and trend chart automatically.

## Data Format

Each result file is produced by `benchmarks/search_benchmark.py` and follows this shape:

```json
{
  "summary": {
    "timestamp_utc": "2026-05-04T11:43:51.199073+00:00",
    "queries": 40,
    "repeats": 5,
    "top_k": 5,
    "latency_ms": { "mean": 143.1, "p50": 141.7, "p95": 148.3, "p99": 150.0, "min": 139.1, "max": 150.5 },
    "quality": { "hit@1": 1.0, "hit@3": 1.0, "hit@5": 1.0, "mrr@5": 1.0, "ndcg@5": 0.909 }
  },
  "per_query": [
    {
      "query": "strategy pattern",
      "relevant_ids": ["DP-020"],
      "top_ids": ["DP-020", "DP-021"],
      "hit@1": 1, "hit@3": 1, "hit@5": 1,
      "rr@5": 1.0, "ndcg@5": 1.0,
      "latency_mean_ms": 141.1
    }
  ]
}
```

## Project Structure

```
benchmarks/dashboard/
├── src/
│   ├── main.js              # Svelte 5 mount entry point
│   ├── App.svelte           # Root component — loads JSON files, run selector
│   └── lib/
│       ├── TrendChart.svelte    # hit@1 / MRR / NDCG trend over time
│       ├── LatencySummary.svelte # quality card + latency bar chart
│       ├── QueryTable.svelte    # per-query result table with color coding
│       └── QueryModal.svelte   # drill-down modal for a single query
├── index.html               # SPA entry point
├── vite.config.js           # Vite + Svelte plugin config
└── package.json
```
