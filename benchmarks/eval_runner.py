#!/usr/bin/env python3
"""
Comprehensive batch evaluation runner for Episteme CLI (epis).

Subcommands:
  full              - Run all evaluation suites
  search-positive  - Run positive search quality evaluation (existing benchmark)
  search-negative  - Run negative search false-positive evaluation
  smell-negative   - Run smell detection negative corpus evaluation
  analyze-positive - Run positive code smell detection evaluation
  traversal        - Run graph traversal evaluation

Metrics:
  Search:   precision@K, FP@K, specificity, recall (hit@1/3/5, MRR, NDCG)
  Smell:    FP rate per detector, per language, overall
  Traversal: path found rate, neighbor coverage
  Composite: 0.4 * recall + 0.3 * precision + 0.3 * specificity

Regression detection:
  - Fails (exit 1) if composite drops >= 0.02 vs previous run
  - Fails if any single metric drops >= 0.05
  - Fails if any must_not_contain entity appears at rank 1
"""

from __future__ import annotations

import argparse
import json
import math
import os
import re
import statistics
import subprocess
import sys
import tempfile
import time
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

ENTITY_ID_RE = re.compile(r"\[([A-Z]+-[A-Z0-9-]+)\]")
ENTITY_ID_RE_RELAXED = re.compile(r"([A-Z]{2,6}-[0-9]{2,3}(?:-[A-Z])?)")

BIN_DEFAULT = "target/debug/episteme"


def repo_root() -> Path:
    return Path(__file__).resolve().parents[1]


def bin_path(arg_bin: str | None) -> Path:
    p = (repo_root() / (arg_bin or BIN_DEFAULT)).resolve()
    if not p.exists():
        raise SystemExit(f"binary not found: {p} (build with `cargo build`)")
    return p


def run_cli(args: list[str], timeout: int = 30) -> subprocess.CompletedProcess[str]:
    return subprocess.run(args, capture_output=True, text=True, check=False, timeout=timeout)


def parse_entity_ids(output: str) -> list[str]:
    ids: list[str] = []
    for line in output.splitlines():
        for m in ENTITY_ID_RE.finditer(line):
            ids.append(m.group(1))
    return ids


def parse_json_output(output: str) -> dict | list | None:
    """Try to parse JSON from CLI output (may have non-JSON lines)."""
    for line in reversed(output.splitlines()):
        line = line.strip()
        if line.startswith("{") or line.startswith("["):
            try:
                return json.loads(line)
            except json.JSONDecodeError:
                continue
    # Try the whole output
    try:
        return json.loads(output)
    except json.JSONDecodeError:
        return None


def dedup(ids: list[str]) -> list[str]:
    seen: set[str] = set()
    result: list[str] = []
    for i in ids:
        if i not in seen:
            seen.add(i)
            result.append(i)
    return result


# ---------------------------------------------------------------------------
# Search Positive (extends existing benchmark)
# ---------------------------------------------------------------------------

def eval_search_positive(epis: Path, top_k: int, repeats: int) -> dict:
    """Run positive search quality eval using existing search_eval_set.json."""
    eval_path = repo_root() / "benchmarks" / "search_eval_set.json"
    if not eval_path.exists():
        return {"status": "skipped", "reason": "search_eval_set.json not found"}

    data = json.loads(eval_path.read_text())
    queries = data.get("queries", [])
    if not queries:
        return {"status": "skipped", "reason": "no queries in eval set"}

    # Warmup
    for q in queries[:3]:
        run_cli([str(epis), "explore", q["query"], "--limit", str(top_k)])

    hit1 = hit3 = hit5 = 0
    mrr_sum = ndcg_sum = 0.0
    per_query: list[dict] = []

    for q in queries:
        text = q["query"]
        relevant = set(q["relevant_ids"])
        ids_last: list[str] = []

        for _ in range(repeats):
            proc = run_cli([str(epis), "explore", text, "--limit", str(top_k)])
            ids_last = dedup(parse_entity_ids(proc.stdout))

        q_hit1 = 1 if any(i in relevant for i in ids_last[:1]) else 0
        q_hit3 = 1 if any(i in relevant for i in ids_last[:3]) else 0
        q_hit5 = 1 if any(i in relevant for i in ids_last[:5]) else 0
        q_rr = _reciprocal_rank(ids_last, relevant, top_k)
        q_ndcg = _ndcg_at_k(ids_last, relevant, top_k)

        hit1 += q_hit1
        hit3 += q_hit3
        hit5 += q_hit5
        mrr_sum += q_rr
        ndcg_sum += q_ndcg

        per_query.append({
            "query": text,
            "top_ids": ids_last[:top_k],
            "hit@1": q_hit1,
            "hit@3": q_hit3,
            "hit@5": q_hit5,
            "rr@5": round(q_rr, 6),
            "ndcg@5": round(q_ndcg, 6),
        })

    n = len(queries)
    metrics = {
        "hit@1": round(hit1 / n, 6),
        "hit@3": round(hit3 / n, 6),
        "hit@5": round(hit5 / n, 6),
        "mrr@5": round(mrr_sum / n, 6),
        "ndcg@5": round(ndcg_sum / n, 6),
    }

    return {"status": "ok", "queries": n, "metrics": metrics, "per_query": per_query}


def _reciprocal_rank(ids: list[str], relevant: set[str], k: int) -> float:
    for rank, doc_id in enumerate(ids[:k], start=1):
        if doc_id in relevant:
            return 1.0 / rank
    return 0.0


def _ndcg_at_k(ids: list[str], relevant: set[str], k: int) -> float:
    dcg = 0.0
    for i, doc_id in enumerate(ids[:k], start=1):
        if doc_id in relevant:
            dcg += 1.0 / math.log2(i + 1)
    ideal_hits = min(k, len(relevant))
    if ideal_hits == 0:
        return 0.0
    idcg = sum(1.0 / math.log2(i + 1) for i in range(1, ideal_hits + 1))
    return dcg / idcg if idcg > 0 else 0.0


# ---------------------------------------------------------------------------
# Search Negative (new)
# ---------------------------------------------------------------------------

def eval_search_negative(epis: Path, top_k: int) -> dict:
    """Run negative search FP evaluation."""
    eval_path = repo_root() / "benchmarks" / "search_negative_eval_set.json"
    if not eval_path.exists():
        return {"status": "skipped", "reason": "search_negative_eval_set.json not found"}

    data = json.loads(eval_path.read_text())
    queries = data.get("queries", [])
    if not queries:
        return {"status": "skipped", "reason": "no queries"}

    fp_at_1 = 0
    fp_at_3 = 0
    fp_at_5 = 0
    tn = 0
    per_query: list[dict] = []

    for q in queries:
        text = q["query"]
        must_not = set(q.get("must_not_contain", []))
        category = q.get("category", "unknown")

        proc = run_cli([str(epis), "explore", text, "--limit", str(top_k)])
        ids = dedup(parse_entity_ids(proc.stdout))

        # Check for false positives
        fp1 = 1 if any(i in must_not for i in ids[:1]) else 0
        fp3 = 1 if any(i in must_not for i in ids[:3]) else 0
        fp5 = 1 if any(i in must_not for i in ids[:5]) else 0

        fp_at_1 += fp1
        fp_at_3 += fp3
        fp_at_5 += fp5
        if fp5 == 0:
            tn += 1

        violations = [i for i in ids[:top_k] if i in must_not]

        per_query.append({
            "query": text,
            "category": category,
            "must_not_contain": sorted(must_not),
            "top_ids": ids[:top_k],
            "fp@1": fp1,
            "fp@3": fp3,
            "fp@5": fp5,
            "violations": violations,
        })

    n = len(queries)
    specificity = round(tn / n, 6) if n > 0 else 0.0
    metrics = {
        "fp@1": round(fp_at_1 / n, 6) if n > 0 else 0.0,
        "fp@3": round(fp_at_3 / n, 6) if n > 0 else 0.0,
        "fp@5": round(fp_at_5 / n, 6) if n > 0 else 0.0,
        "specificity": specificity,
        "true_negatives": tn,
        "total": n,
    }

    return {"status": "ok", "metrics": metrics, "per_query": per_query}


# ---------------------------------------------------------------------------
# Smell Negative Corpus
# ---------------------------------------------------------------------------

def eval_smell_negative(epis: Path, min_confidence: float) -> dict:
    """Run smell detection on clean code corpus. Should detect zero smells."""
    corpus_dir = repo_root() / "benchmarks" / "smell_negative_corpus"
    if not corpus_dir.exists():
        return {"status": "skipped", "reason": "smell_negative_corpus/ not found"}

    results: list[dict] = []
    total = 0
    fp_count = 0
    per_detector_fp: dict[str, int] = {}
    per_language_fp: dict[str, int] = {}
    per_language_total: dict[str, int] = {}

    lang_map = {
        ".rs": "rust", ".py": "python", ".ts": "typescript",
        ".tsx": "typescript", ".go": "go", ".rb": "ruby", ".java": "java",
    }

    for entry in sorted(corpus_dir.iterdir()):
        if entry.is_dir():
            continue
        ext = entry.suffix
        lang = lang_map.get(ext)
        if lang is None:
            continue

        total += 1
        per_language_total[lang] = per_language_total.get(lang, 0) + 1

        proc = run_cli(
            [str(epis), "analyze", str(entry), "--json", "--min-confidence", str(min_confidence)],
            timeout=60,
        )

        detected: list[str] = []
        parsed = parse_json_output(proc.stdout)
        if parsed and isinstance(parsed, dict):
            smells = parsed.get("smells", [])
            for s in smells:
                sid = s.get("smell_id", "UNKNOWN")
                detected.append(sid)
                per_detector_fp[sid] = per_detector_fp.get(sid, 0) + 1

        is_fp = len(detected) > 0
        if is_fp:
            fp_count += 1
            per_language_fp[lang] = per_language_fp.get(lang, 0) + 1

        results.append({
            "file": entry.name,
            "language": lang,
            "false_positive": is_fp,
            "detected_smells": detected,
        })

    fp_rate = round(fp_count / total, 6) if total > 0 else 0.0
    per_lang_rate = {}
    for lang in per_language_total:
        fp = per_language_fp.get(lang, 0)
        tot = per_language_total[lang]
        per_lang_rate[lang] = round(fp / tot, 6) if tot > 0 else 0.0

    metrics = {
        "fp_rate": fp_rate,
        "fp_count": fp_count,
        "total": total,
        "specificity": round(1.0 - fp_rate, 6),
        "per_detector": {k: v for k, v in sorted(per_detector_fp.items())},
        "per_language": per_lang_rate,
    }

    return {"status": "ok", "metrics": metrics, "per_file": results}


# ---------------------------------------------------------------------------
# Analyze Positive
# ---------------------------------------------------------------------------

def eval_analyze_positive(epis: Path, min_confidence: float) -> dict:
    """Run positive code smell detection evaluation."""
    eval_path = repo_root() / "benchmarks" / "analyze_eval_set.json"
    if not eval_path.exists():
        return {"status": "skipped", "reason": "analyze_eval_set.json not found"}

    data = json.loads(eval_path.read_text())
    cases = data.get("cases", [])
    if not cases:
        return {"status": "skipped", "reason": "no cases"}

    results: list[dict] = []
    total = 0
    hits = 0
    per_smell_hits: dict[str, dict[str, int]] = {}

    for case in cases:
        case_id = case["id"]
        lang = case["language"]
        expected = set(case.get("expected_smells", []))
        code = case.get("code", "")

        if not code:
            continue

        # Write to temp file
        ext_map = {
            "rust": ".rs", "python": ".py", "typescript": ".ts",
            "go": ".go", "ruby": ".rb", "java": ".java",
        }
        ext = ext_map.get(lang, ".txt")

        with tempfile.NamedTemporaryFile(mode="w", suffix=ext, delete=False) as f:
            f.write(code)
            tmp_path = f.name

        try:
            proc = run_cli(
                [str(epis), "analyze", tmp_path, "--language", lang,
                 "--json", "--min-confidence", str(min_confidence)],
                timeout=30,
            )

            detected_ids: set[str] = set()
            parsed = parse_json_output(proc.stdout)
            if parsed and isinstance(parsed, dict):
                for s in parsed.get("smells", []):
                    sid = s.get("smell_id", "")
                    if sid:
                        detected_ids.add(sid)

            # Check recall: for each expected smell, was it detected?
            true_positives = expected & detected_ids
            false_negatives = expected - detected_ids
            false_positives = detected_ids - expected

            case_hit = len(true_positives) == len(expected) and len(false_negatives) == 0
            if case_hit:
                hits += 1
            total += 1

            # Per-smell tracking
            for sid in expected:
                if sid not in per_smell_hits:
                    per_smell_hits[sid] = {"detected": 0, "total": 0}
                per_smell_hits[sid]["total"] += 1
                if sid in detected_ids:
                    per_smell_hits[sid]["detected"] += 1

            results.append({
                "id": case_id,
                "language": lang,
                "expected": sorted(expected),
                "detected": sorted(detected_ids),
                "true_positives": sorted(true_positives),
                "false_negatives": sorted(false_negatives),
                "false_positives": sorted(false_positives),
                "hit": case_hit,
            })
        finally:
            os.unlink(tmp_path)

    recall = round(hits / total, 6) if total > 0 else 0.0
    per_smell_recall = {
        k: round(v["detected"] / v["total"], 6) if v["total"] > 0 else 0.0
        for k, v in sorted(per_smell_hits.items())
    }

    metrics = {
        "recall": recall,
        "hits": hits,
        "total": total,
        "per_smell_recall": per_smell_recall,
    }

    return {"status": "ok", "metrics": metrics, "per_case": results}


# ---------------------------------------------------------------------------
# Traversal
# ---------------------------------------------------------------------------

def eval_traversal(epis: Path) -> dict:
    """Run graph traversal evaluation."""
    eval_path = repo_root() / "benchmarks" / "traversal_eval_set.json"
    if not eval_path.exists():
        return {"status": "skipped", "reason": "traversal_eval_set.json not found"}

    data = json.loads(eval_path.read_text())
    cases = data.get("cases", [])
    if not cases:
        return {"status": "skipped", "reason": "no cases"}

    results: list[dict] = []
    neighbor_hits = 0
    neighbor_total = 0
    path_hits = 0
    path_total = 0

    for case in cases:
        case_type = case.get("type", "")
        case_id = case.get("id", "")

        if case_type == "neighbors":
            entity_id = case.get("entity_id", "")
            expected = set(case.get("expected_neighbors", []))
            min_count = case.get("min_expected_count", 0)

            proc = run_cli([str(epis), "graph", "neighbors", entity_id])
            found_ids = set(dedup(parse_entity_ids(proc.stdout)))

            coverage = len(expected & found_ids) / len(expected) if expected else 0.0
            count_ok = len(found_ids) >= min_count
            hit = coverage >= 0.5 and count_ok

            if hit:
                neighbor_hits += 1
            neighbor_total += 1

            results.append({
                "id": case_id,
                "type": "neighbors",
                "entity_id": entity_id,
                "expected": sorted(expected),
                "found": sorted(found_ids),
                "coverage": round(coverage, 6),
                "count_ok": count_ok,
                "hit": hit,
            })

        elif case_type == "path":
            from_id = case.get("from", "")
            to_id = case.get("to", "")
            max_depth = case.get("max_depth", 5)
            should_find = case.get("should_find_path", True)
            expected_len = case.get("expected_path_length")

            proc = run_cli([
                str(epis), "graph", "path", from_id, to_id,
                "--max-depth", str(max_depth),
            ])

            output = proc.stdout
            path_found = "→" in output or "->" in output or "Path found" in output.lower()
            path_ids = dedup(parse_entity_ids(output))

            if should_find:
                hit = path_found
                if expected_len is not None and path_ids:
                    actual_len = len(path_ids)
                    hit = hit and actual_len <= expected_len + 1
            else:
                hit = not path_found

            if hit:
                path_hits += 1
            path_total += 1

            results.append({
                "id": case_id,
                "type": "path",
                "from": from_id,
                "to": to_id,
                "should_find_path": should_find,
                "path_found": path_found,
                "path_ids": path_ids,
                "hit": hit,
            })

    neighbor_recall = round(neighbor_hits / neighbor_total, 6) if neighbor_total > 0 else 0.0
    path_recall = round(path_hits / path_total, 6) if path_total > 0 else 0.0

    metrics = {
        "neighbors": {"recall": neighbor_recall, "hits": neighbor_hits, "total": neighbor_total},
        "paths": {"recall": path_recall, "hits": path_hits, "total": path_total},
    }

    return {"status": "ok", "metrics": metrics, "per_case": results}


# ---------------------------------------------------------------------------
# Composite Score & Regression
# ---------------------------------------------------------------------------

def compute_composite(
    search_positive: dict,
    search_negative: dict,
    smell_negative: dict,
) -> dict:
    """Compute composite quality score."""
    recall = search_positive.get("metrics", {}).get("hit@5", 0.0) if search_positive.get("status") == "ok" else 0.0
    precision = 1.0 - search_negative.get("metrics", {}).get("fp@5", 0.0) if search_negative.get("status") == "ok" else 0.0
    specificity = smell_negative.get("metrics", {}).get("specificity", 0.0) if smell_negative.get("status") == "ok" else 0.0

    composite = 0.4 * recall + 0.3 * precision + 0.3 * specificity

    return {
        "recall": round(recall, 6),
        "precision": round(precision, 6),
        "specificity": round(specificity, 6),
        "composite": round(composite, 6),
        "formula": "0.4 * recall + 0.3 * precision + 0.3 * specificity",
    }


def check_regression(current: dict, prev_path: Path) -> dict:
    """Check for regression against previous run."""
    if not prev_path.exists():
        return {"status": "no_previous", "previous_file": str(prev_path)}

    prev = json.loads(prev_path.read_text())
    prev_composite = prev.get("composite_score", {}).get("composite", 0.0)
    curr_composite = current.get("composite", 0.0)

    delta = round(curr_composite - prev_composite, 6)
    regressions: list[str] = []

    # Composite regression check
    if delta < -0.02:
        regressions.append(f"composite dropped by {abs(delta):.4f} (threshold: 0.02)")

    # Per-metric checks
    for metric in ["recall", "precision", "specificity"]:
        prev_val = prev.get("composite_score", {}).get(metric, 0.0)
        curr_val = current.get(metric, 0.0)
        m_delta = round(curr_val - prev_val, 6)
        if m_delta < -0.05:
            regressions.append(f"{metric} dropped by {abs(m_delta):.4f} (threshold: 0.05)")

    # Rank-1 FP check
    search_neg = prev.get("suites", {}).get("search_negative", {})
    if search_neg.get("status") == "ok":
        for q in search_neg.get("per_query", []):
            if q.get("fp@1", 0) == 1:
                regressions.append(f"must_not_contain entity at rank 1: '{q['query']}'")
                break  # one is enough to flag

    status = "FAIL" if regressions else "PASS"
    return {
        "status": status,
        "prev_composite": prev_composite,
        "delta": delta,
        "regressions": regressions,
    }


# ---------------------------------------------------------------------------
# Report
# ---------------------------------------------------------------------------

def print_report(
    composite: dict,
    suites: dict,
    regression: dict | None,
) -> None:
    """Print console summary report."""
    print("\n" + "=" * 70)
    print("  EPISTEME EVALUATION REPORT")
    print("=" * 70)

    print(f"\n  Composite Score: {composite['composite']:.4f}")
    print(f"    Recall:      {composite['recall']:.4f}")
    print(f"    Precision:   {composite['precision']:.4f}")
    print(f"    Specificity: {composite['specificity']:.4f}")

    # Search positive
    sp = suites.get("search_positive", {})
    if sp.get("status") == "ok":
        m = sp["metrics"]
        print(f"\n  Search Positive (recall):")
        print(f"    hit@1:  {m['hit@1']:.4f}  hit@3: {m['hit@3']:.4f}  hit@5: {m['hit@5']:.4f}")
        print(f"    MRR@5:  {m['mrr@5']:.4f}  NDCG@5: {m['ndcg@5']:.4f}")

    # Search negative
    sn = suites.get("search_negative", {})
    if sn.get("status") == "ok":
        m = sn["metrics"]
        print(f"\n  Search Negative (precision):")
        print(f"    FP@1:   {m['fp@1']:.4f}  FP@3: {m['fp@3']:.4f}  FP@5: {m['fp@5']:.4f}")
        print(f"    Specificity: {m['specificity']:.4f}  ({m['true_negatives']}/{m['total']} clean)")

    # Smell negative
    smn = suites.get("smell_negative", {})
    if smn.get("status") == "ok":
        m = smn["metrics"]
        print(f"\n  Smell Negative Corpus:")
        print(f"    FP Rate:     {m['fp_rate']:.4f}  ({m['fp_count']}/{m['total']} files)")
        print(f"    Specificity: {m['specificity']:.4f}")
        if m["per_detector"]:
            print(f"    Per Detector FP:")
            for sid, count in m["per_detector"].items():
                print(f"      {sid}: {count}")
        if m["per_language"]:
            print(f"    Per Language FP Rate:")
            for lang, rate in m["per_language"].items():
                print(f"      {lang}: {rate:.4f}")

    # Analyze positive
    ap = suites.get("analyze_positive", {})
    if ap.get("status") == "ok":
        m = ap["metrics"]
        print(f"\n  Analyze Positive:")
        print(f"    Recall: {m['recall']:.4f}  ({m['hits']}/{m['total']} cases)")
        if m["per_smell_recall"]:
            print(f"    Per Smell Recall:")
            for sid, r in m["per_smell_recall"].items():
                print(f"      {sid}: {r:.4f}")

    # Traversal
    tr = suites.get("traversal", {})
    if tr.get("status") == "ok":
        m = tr["metrics"]
        print(f"\n  Traversal:")
        print(f"    Neighbors: {m['neighbors']['recall']:.4f} ({m['neighbors']['hits']}/{m['neighbors']['total']})")
        print(f"    Paths:     {m['paths']['recall']:.4f} ({m['paths']['hits']}/{m['paths']['total']})")

    # Regression
    if regression:
        print(f"\n  Regression: {regression['status']}")
        if regression.get("delta") is not None:
            print(f"    Delta: {regression['delta']:+.4f} vs previous")
        for r in regression.get("regressions", []):
            print(f"    ⚠ {r}")

    print("\n" + "=" * 70)


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main() -> int:
    parser = argparse.ArgumentParser(
        description="Episteme batch evaluation runner"
    )
    parser.add_argument(
        "suite",
        choices=["full", "search-positive", "search-negative",
                 "smell-negative", "analyze-positive", "traversal"],
        default="full",
        nargs="?",
        help="Evaluation suite to run (default: full)",
    )
    parser.add_argument("--bin", default=None, help="Path to epis binary")
    parser.add_argument("--top-k", type=int, default=5, help="Top-K for search")
    parser.add_argument("--repeats", type=int, default=1, help="Repeats per query")
    parser.add_argument("--min-confidence", type=float, default=0.5, help="Min confidence for smell detection")
    parser.add_argument("--compare", default="", help="Previous eval result to compare against")
    parser.add_argument("--output", default="", help="Output JSON path")
    args = parser.parse_args()

    epis = bin_path(args.bin)
    print(f"Using binary: {epis}")

    suites: dict[str, dict] = {}

    # Run selected suite(s)
    if args.suite in ("full", "search-positive"):
        print("\n▶ Running search-positive evaluation...")
        suites["search_positive"] = eval_search_positive(epis, args.top_k, args.repeats)

    if args.suite in ("full", "search-negative"):
        print("▶ Running search-negative evaluation...")
        suites["search_negative"] = eval_search_negative(epis, args.top_k)

    if args.suite in ("full", "smell-negative"):
        print("▶ Running smell-negative evaluation...")
        suites["smell_negative"] = eval_smell_negative(epis, args.min_confidence)

    if args.suite in ("full", "analyze-positive"):
        print("▶ Running analyze-positive evaluation...")
        suites["analyze_positive"] = eval_analyze_positive(epis, args.min_confidence)

    if args.suite in ("full", "traversal"):
        print("▶ Running traversal evaluation...")
        suites["traversal"] = eval_traversal(epis)

    # Compute composite
    composite = compute_composite(
        suites.get("search_positive", {}),
        suites.get("search_negative", {}),
        suites.get("smell_negative", {}),
    )

    # Regression check
    prev_path = Path(args.compare) if args.compare else repo_root() / "benchmarks" / "results" / "latest.json"
    regression = check_regression(composite, prev_path) if args.suite == "full" else None

    # Build report
    report = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "git_commit": _git_commit(),
        "suite": args.suite,
        "composite_score": composite,
        "suites": suites,
    }
    if regression:
        report["regression"] = regression

    # Save results
    out_dir = repo_root() / "benchmarks" / "results"
    out_dir.mkdir(parents=True, exist_ok=True)

    if args.output:
        out_path = Path(args.output)
    else:
        ts = datetime.now().strftime("%Y%m%d_%H%M%S")
        out_path = out_dir / f"eval_{ts}.json"

    out_path.write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding="utf-8")

    # Update latest symlink
    latest = out_dir / "latest.json"
    try:
        latest.unlink(missing_ok=True)
        os.symlink(out_path.name, latest)
    except OSError:
        pass

    # Print report
    print_report(composite, suites, regression)
    print(f"\nSaved: {out_path}")

    # Exit code
    if regression and regression.get("status") == "FAIL":
        print("\n❌ REGRESSION DETECTED — evaluation failed")
        return 1

    # Check for rank-1 FP in search negative
    sn = suites.get("search_negative", {})
    if sn.get("status") == "ok":
        for q in sn.get("per_query", []):
            if q.get("fp@1") == 1:
                print(f"\n❌ CRITICAL: must_not_contain entity at rank 1 for query: '{q['query']}'")
                return 1

    print("\n✅ Evaluation passed")
    return 0


def _git_commit() -> str:
    try:
        proc = subprocess.run(
            ["git", "rev-parse", "--short", "HEAD"],
            capture_output=True, text=True, check=False,
            cwd=str(repo_root()),
        )
        return proc.stdout.strip() or "unknown"
    except Exception:
        return "unknown"


if __name__ == "__main__":
    raise SystemExit(main())
