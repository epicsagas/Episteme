#!/usr/bin/env python3
"""
Search benchmark for Rust CLI `syntagma explore`.

Outputs latency and retrieval quality metrics:
- latency: mean/p50/p95/p99/min/max
- quality: hit@1/hit@3/hit@5/mrr@5/ndcg@5
"""

from __future__ import annotations

import argparse
import json
import math
import re
import statistics
import subprocess
import time
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


ID_PATTERN = re.compile(r"\[([A-Z]+-[A-Z0-9-]+)\]")


def percentile(values: list[float], p: int) -> float:
    ordered = sorted(values)
    idx = min(len(ordered) - 1, int(len(ordered) * p / 100))
    return ordered[idx]


def parse_ids(output: str) -> list[str]:
    ids: list[str] = []
    for line in output.splitlines():
        m = ID_PATTERN.search(line)
        if m:
            ids.append(m.group(1))
    return ids


def reciprocal_rank(top_ids: list[str], relevant: set[str], k: int) -> float:
    for rank, doc_id in enumerate(top_ids[:k], start=1):
        if doc_id in relevant:
            return 1.0 / rank
    return 0.0


def ndcg_at_k(top_ids: list[str], relevant: set[str], k: int) -> float:
    # binary relevance
    dcg = 0.0
    for i, doc_id in enumerate(top_ids[:k], start=1):
        rel = 1.0 if doc_id in relevant else 0.0
        if rel > 0:
            dcg += rel / math.log2(i + 1)

    ideal_hits = min(k, len(relevant))
    if ideal_hits == 0:
        return 0.0
    idcg = sum(1.0 / math.log2(i + 1) for i in range(1, ideal_hits + 1))
    return dcg / idcg if idcg > 0 else 0.0


def dedup_ids(ids: list[str]) -> list[str]:
    """Remove duplicate entity IDs while preserving rank order (first occurrence wins)."""
    seen: set[str] = set()
    result: list[str] = []
    for doc_id in ids:
        if doc_id not in seen:
            seen.add(doc_id)
            result.append(doc_id)
    return result


def run_query(
    bin_path: Path, query: str, top_k: int
) -> tuple[float, int, list[str], str]:
    start = time.perf_counter()
    proc = subprocess.run(
        [str(bin_path), "explore", query, "--limit", str(top_k)],
        capture_output=True,
        text=True,
        check=False,
    )
    elapsed_ms = (time.perf_counter() - start) * 1000
    # Deduplicate entity IDs so the same entity cannot inflate NDCG/MRR.
    ids = dedup_ids(parse_ids(proc.stdout))
    return elapsed_ms, proc.returncode, ids, proc.stdout


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Benchmark search quality/latency for syntagma"
    )
    parser.add_argument(
        "--eval-set",
        default="benchmarks/search_eval_set.json",
        help="Path to query/relevance dataset JSON",
    )
    parser.add_argument(
        "--bin",
        default="target/debug/syntagma",
        help="Path to syntagma binary",
    )
    parser.add_argument(
        "--repeats",
        type=int,
        default=5,
        help="Per-query latency repeat count",
    )
    parser.add_argument(
        "--top-k",
        type=int,
        default=5,
        help="Top-k used for both retrieval and quality metrics",
    )
    parser.add_argument(
        "--output",
        default="",
        help="Optional output json path (default: benchmarks/results/search_benchmark_<timestamp>.json)",
    )
    args = parser.parse_args()

    repo_root = Path(__file__).resolve().parents[1]
    eval_set_path = (repo_root / args.eval_set).resolve()
    bin_path = (repo_root / args.bin).resolve()

    if not eval_set_path.exists():
        raise SystemExit(f"eval set not found: {eval_set_path}")

    if not bin_path.exists():
        raise SystemExit(
            f"binary not found: {bin_path} (build first with `cargo build`)"
        )

    data = json.loads(eval_set_path.read_text(encoding="utf-8"))
    queries: list[dict[str, Any]] = data.get("queries", [])
    if not queries:
        raise SystemExit("eval set has no queries")

    # warmup
    for q in queries[: min(3, len(queries))]:
        run_query(bin_path, q["query"], args.top_k)

    all_latencies: list[float] = []
    per_query: list[dict[str, Any]] = []
    hit1 = hit3 = hit5 = 0
    mrr_sum = 0.0
    ndcg_sum = 0.0

    for q in queries:
        text = q["query"]
        relevant = set(q["relevant_ids"])

        samples: list[float] = []
        ids_last: list[str] = []
        rc_last = 0
        stdout_last = ""
        for _ in range(max(1, args.repeats)):
            elapsed_ms, rc, ids, out = run_query(bin_path, text, args.top_k)
            samples.append(elapsed_ms)
            ids_last = ids
            rc_last = rc
            stdout_last = out

        all_latencies.extend(samples)
        q_hit1 = 1 if any(i in relevant for i in ids_last[:1]) else 0
        q_hit3 = 1 if any(i in relevant for i in ids_last[:3]) else 0
        q_hit5 = 1 if any(i in relevant for i in ids_last[:5]) else 0
        q_rr = reciprocal_rank(ids_last, relevant, 5)
        q_ndcg = ndcg_at_k(ids_last, relevant, 5)

        hit1 += q_hit1
        hit3 += q_hit3
        hit5 += q_hit5
        mrr_sum += q_rr
        ndcg_sum += q_ndcg

        per_query.append(
            {
                "query": text,
                "relevant_ids": sorted(relevant),
                "return_code": rc_last,
                "latency_mean_ms": round(statistics.mean(samples), 3),
                "latency_p95_ms": round(percentile(samples, 95), 3),
                "top_ids": ids_last[: args.top_k],
                "hit@1": q_hit1,
                "hit@3": q_hit3,
                "hit@5": q_hit5,
                "rr@5": round(q_rr, 6),
                "ndcg@5": round(q_ndcg, 6),
                "raw_stdout": stdout_last,
            }
        )

    n = len(queries)
    summary = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "queries": n,
        "repeats": args.repeats,
        "top_k": args.top_k,
        "latency_ms": {
            "mean": round(statistics.mean(all_latencies), 3),
            "p50": round(statistics.median(all_latencies), 3),
            "p95": round(percentile(all_latencies, 95), 3),
            "p99": round(percentile(all_latencies, 99), 3),
            "min": round(min(all_latencies), 3),
            "max": round(max(all_latencies), 3),
        },
        "quality": {
            "hit@1": round(hit1 / n, 6),
            "hit@3": round(hit3 / n, 6),
            "hit@5": round(hit5 / n, 6),
            "mrr@5": round(mrr_sum / n, 6),
            "ndcg@5": round(ndcg_sum / n, 6),
        },
    }

    result = {
        "summary": summary,
        "per_query": per_query,
    }

    if args.output:
        output_path = (repo_root / args.output).resolve()
    else:
        out_dir = repo_root / "benchmarks" / "results"
        out_dir.mkdir(parents=True, exist_ok=True)
        ts = datetime.now().strftime("%Y%m%d_%H%M%S")
        output_path = out_dir / f"search_benchmark_{ts}.json"

    output_path.write_text(
        json.dumps(result, ensure_ascii=False, indent=2), encoding="utf-8"
    )

    print(json.dumps(summary, ensure_ascii=False, indent=2))
    print(f"\nSaved: {output_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
