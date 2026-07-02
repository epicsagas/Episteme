#!/usr/bin/env python3
"""
Lightweight-model boost eval: does querying Episteme's knowledge graph
improve a LIGHTWEIGHT model's software-engineering answers versus the model alone?

Design
------
For each question in lightweight_eval_bank.json we run two conditions on the
same lightweight model (default: qwen2.5-coder:latest, a 7B-class local model):

  (a) ALONE   : the question is sent to the model with no extra context.
  (b) BOOSTED : the top-k Episteme /search hits for the question are prepended
                as a "Relevant knowledge" block, then the same question.

Both calls use temperature=0 (greedy) so results are reproducible. Answers are
scored against a PRE-REGISTERED rubric (see the bank file): a concept is "hit"
if any of its aliases appears in the lowercased, markdown-stripped answer.
Per-question score = concepts_hit / concepts_required. There is no LLM judge
and no model in the scoring loop -- scoring is deterministic and objective.

Prerequisites
-------------
* Episteme HTTP API running, e.g.:  epis api  (default http://127.0.0.1:58302)
* ollama running with a lightweight model, e.g.:
    ollama pull qwen2.5-coder:latest
    (check: curl -s localhost:11434/api/tags)

Run
---
    python3 benchmarks/boost_lightweight.py
    python3 benchmarks/boost_lightweight.py --model qwen2.5-coder:3b --k 5 --repeats 1
    python3 benchmarks/boost_lightweight.py --out benchmarks/results/boost_run.json

Output: a per-question table, aggregate scores (alone vs boosted, delta), and a
written JSON results file. Pass --json-only to suppress the table.
"""

from __future__ import annotations

import argparse
import json
import re
import statistics
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path

DEFAULT_BANK = Path(__file__).resolve().parent / "lightweight_eval_bank.json"
DEFAULT_OUT = Path(__file__).resolve().parent / "results" / "boost_run.json"


# ----------------------------- HTTP helpers --------------------------------- #


def http_json(url: str, payload: dict | None = None, timeout: int = 120) -> dict:
    """GET (payload=None) or POST JSON. Returns parsed JSON. Raises on error."""
    if payload is None:
        req = urllib.request.Request(url)
    else:
        body = json.dumps(payload).encode("utf-8")
        req = urllib.request.Request(
            url, data=body, headers={"Content-Type": "application/json"}
        )
    with urllib.request.urlopen(req, timeout=timeout) as resp:
        return json.loads(resp.read().decode("utf-8"))


# ----------------------------- Episteme /search ----------------------------- #


def episteme_search(episteme_url: str, query: str, k: int) -> list[dict]:
    """Return up to k search hits from Episteme. Empty list on failure."""
    q = urllib.parse.urlencode({"q": query, "limit": k})
    url = f"{episteme_url.rstrip('/')}/search?{q}"
    try:
        data = http_json(url, timeout=15)
    except (urllib.error.URLError, json.JSONDecodeError, TimeoutError) as exc:
        print(f"  [warn] episteme search failed for {query!r}: {exc}", file=sys.stderr)
        return []
    return data.get("results", [])[:k]


def format_context(hits: list[dict]) -> str:
    """Render search hits as a compact context block for the model."""
    if not hits:
        return ""
    lines = ["[Relevant knowledge retrieved from the Episteme SE knowledge graph]"]
    for i, h in enumerate(hits, 1):
        eid = h.get("entity_id", "?")
        title = h.get("title", "")
        typ = h.get("type", "")
        text = (h.get("text") or "").replace("\n", " ").strip()
        text = re.sub(r"\s+", " ", text)
        if len(text) > 280:
            text = text[:277] + "..."
        lines.append(f"{i}. [{eid}] {title} ({typ}): {text}")
    lines.append("Use the above context to inform your answer.\n")
    return "\n".join(lines) + "\n"


# ----------------------------- ollama chat ---------------------------------- #


def ollama_chat(ollama_url: str, model: str, user_msg: str, temperature: float) -> str:
    """Send a single-turn user message, return the model's text reply."""
    payload = {
        "model": model,
        "messages": [{"role": "user", "content": user_msg}],
        "stream": False,
        "options": {"temperature": temperature},
    }
    data = http_json(f"{ollama_url.rstrip('/')}/api/chat", payload=payload, timeout=300)
    return data["message"]["content"].strip()


def safe_ollama(
    ollama_url: str, model: str, user_msg: str, temperature: float
) -> tuple[str, str | None]:
    """ollama_chat that never raises -- returns (text, error_str_or_None)."""
    try:
        return ollama_chat(ollama_url, model, user_msg, temperature), None
    except Exception as exc:  # network / timeout / JSON / OOM-kill of ollama, etc.
        return "", f"{type(exc).__name__}: {exc}"


# ----------------------------- rubric scorer -------------------------------- #

_MD_RE = re.compile(r"[*_`#>|]")


def _normalize(s: str) -> str:
    """Lowercase, strip common markdown punctuation, collapse whitespace."""
    s = _MD_RE.sub(" ", s.lower())
    s = re.sub(r"\s+", " ", s).strip()
    return s


def score_answer(answer: str, rubric: list[list[str]]) -> tuple[float, int, int]:
    """
    rubric: list of concept-groups; a concept is hit if ANY alias in the group
    appears as a substring of the normalized answer. Returns (score, hits, n).
    """
    norm = _normalize(answer)
    hits = 0
    for group in rubric:
        aliases = [_normalize(a) for a in group]
        if any(a and a in norm for a in aliases):
            hits += 1
    n = len(rubric)
    return (hits / n if n else 0.0, hits, n)


# ----------------------------- main eval ------------------------------------ #


def run(
    bank: dict,
    episteme_url: str,
    ollama_url: str,
    model: str,
    k: int,
    repeats: int,
    temperature: float,
    out_path: Path | None = None,
) -> dict:
    questions = bank["questions"]
    rows = []
    for q in questions:
        hits = episteme_search(episteme_url, q["query"], k)
        ctx = format_context(hits)
        alone_scores: list[float] = []
        boosted_scores: list[float] = []
        alone_ans: str = ""
        boosted_ans: str = ""
        alone_err: str | None = None
        boosted_err: str | None = None
        for _ in range(repeats):
            alone_ans, alone_err = safe_ollama(
                ollama_url, model, q["question"], temperature
            )
            s, _, _ = score_answer(alone_ans, q["rubric"])
            alone_scores.append(s)
            prompt = (ctx + q["question"]) if ctx else q["question"]
            boosted_ans, boosted_err = safe_ollama(
                ollama_url, model, prompt, temperature
            )
            s2, _, _ = score_answer(boosted_ans, q["rubric"])
            boosted_scores.append(s2)
        alone = statistics.mean(alone_scores)
        boosted = statistics.mean(boosted_scores)
        rows.append(
            {
                "id": q["id"],
                "category": q["category"],
                "difficulty": q["difficulty"],
                "query": q["query"],
                "top_hit": (hits[0]["entity_id"] + ":" + hits[0]["title"])
                if hits
                else "NONE",
                "alone_score": round(alone, 3),
                "boosted_score": round(boosted, 3),
                "delta": round(boosted - alone, 3),
                "alone_ans": alone_ans,
                "boosted_ans": boosted_ans,
                "alone_err": alone_err,
                "boosted_err": boosted_err,
                "n_context_hits": len(hits),
            }
        )
        print(
            f"  {q['id']} [{q['category']:7}] {q['difficulty']:7} "
            f"alone={alone:.2f} boosted={boosted:.2f} delta={boosted - alone:+.2f} "
            f"top={rows[-1]['top_hit']}",
            file=sys.stderr,
        )
        # incremental checkpoint so a crash/OOM-kill never loses all results
        if out_path is not None:
            result = {"model": model, "k": k, "repeats": repeats, "rows": rows}
            result["summary"] = summarize(rows)
            out_path.parent.mkdir(parents=True, exist_ok=True)
            out_path.write_text(json.dumps(result, indent=2, ensure_ascii=False))
    return {"model": model, "k": k, "repeats": repeats, "rows": rows}


def summarize(rows: list[dict]) -> dict:
    def mean(key):
        return statistics.mean(r[key] for r in rows)

    def by_cat(cat):
        rs = [r for r in rows if r["category"] == cat]
        if not rs:
            return {}
        return {
            "n": len(rs),
            "alone": round(statistics.mean(r["alone_score"] for r in rs), 3),
            "boosted": round(statistics.mean(r["boosted_score"] for r in rs), 3),
            "delta": round(
                statistics.mean(r["boosted_score"] for r in rs)
                - statistics.mean(r["alone_score"] for r in rs),
                3,
            ),
        }

    return {
        "overall": {
            "n": len(rows),
            "alone": round(mean("alone_score"), 3),
            "boosted": round(mean("boosted_score"), 3),
            "delta": round(mean("boosted_score") - mean("alone_score"), 3),
        },
        "by_category": {c: by_cat(c) for c in sorted({r["category"] for r in rows})},
        "n_boosted_up": sum(1 for r in rows if r["delta"] > 0.001),
        "n_boosted_down": sum(1 for r in rows if r["delta"] < -0.001),
        "n_unchanged": sum(1 for r in rows if abs(r["delta"]) <= 0.001),
    }


def print_table(result: dict, summary: dict) -> None:
    print("\n=== Per-question results ===")
    hdr = f"{'ID':<4} {'CAT':<8} {'DIFF':<8} {'ALONE':>6} {'BOOST':>6} {'DELTA':>7}  TOP_HIT"
    print(hdr)
    print("-" * len(hdr))
    for r in result["rows"]:
        print(
            f"{r['id']:<4} {r['category']:<8} {r['difficulty']:<8} "
            f"{r['alone_score']:>6.2f} {r['boosted_score']:>6.2f} "
            f"{r['delta']:>+7.2f}  {r['top_hit']}"
        )
    print("\n=== Aggregate (rubric concept-coverage, 0..1) ===")
    o = summary["overall"]
    print(
        f"OVERALL  n={o['n']:>2}  alone={o['alone']:.3f}  boosted={o['boosted']:.3f}  delta={o['delta']:+.3f}"
    )
    for cat, c in summary["by_category"].items():
        print(
            f"  {cat:<8} n={c['n']:>2}  alone={c['alone']:.3f}  boosted={c['boosted']:.3f}  delta={c['delta']:+.3f}"
        )
    print(
        f"\nper-question: +{summary['n_boosted_up']} improved, "
        f"{summary['n_unchanged']} unchanged, {summary['n_boosted_down']} regressed"
    )
    print(f"model={result['model']}  k={result['k']}  repeats={result['repeats']}")


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[1])
    ap.add_argument("--bank", type=Path, default=DEFAULT_BANK)
    ap.add_argument("--episteme-url", default="http://127.0.0.1:58302")
    ap.add_argument("--ollama-url", default="http://127.0.0.1:11434")
    ap.add_argument("--model", default="qwen2.5-coder:latest")
    ap.add_argument("--k", type=int, default=5, help="Episteme hits to prepend")
    ap.add_argument(
        "--repeats",
        type=int,
        default=1,
        help="Repeats per condition (temp=0 -> identical)",
    )
    ap.add_argument("--temperature", type=float, default=0.0)
    ap.add_argument("--out", type=Path, default=DEFAULT_OUT)
    ap.add_argument("--json-only", action="store_true")
    args = ap.parse_args(argv)

    bank = json.loads(args.bank.read_text())
    # sanity checks
    try:
        h = http_json(f"{args.episteme_url.rstrip('/')}/health", timeout=5)
        print(
            f"[ok] Episteme: {h.get('status')} (kg={h.get('components', {}).get('knowledge_graph')})",
            file=sys.stderr,
        )
    except Exception as exc:
        print(
            f"[fatal] Episteme unreachable at {args.episteme_url}: {exc}",
            file=sys.stderr,
        )
        print(
            "       Start it with:  epis api  (or)  epis api serve --port 58302",
            file=sys.stderr,
        )
        return 2
    try:
        tags = http_json(f"{args.ollama_url.rstrip('/')}/api/tags", timeout=5)
        names = [m["name"] for m in tags.get("models", [])]
        if args.model not in names:
            print(
                f"[fatal] model {args.model!r} not in ollama tags: {names}",
                file=sys.stderr,
            )
            return 2
        print(
            f"[ok] ollama: {len(names)} models; using {args.model!r}", file=sys.stderr
        )
    except Exception as exc:
        print(
            f"[fatal] ollama unreachable at {args.ollama_url}: {exc}", file=sys.stderr
        )
        return 2

    t0 = time.time()
    print(
        f"[run] {len(bank['questions'])} questions, k={args.k}, repeats={args.repeats}",
        file=sys.stderr,
    )
    result = run(
        bank,
        args.episteme_url,
        args.ollama_url,
        args.model,
        args.k,
        args.repeats,
        args.temperature,
        args.out,
    )
    result["elapsed_secs"] = round(time.time() - t0, 1)
    summary = summarize(result["rows"])
    result["summary"] = summary

    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(result, indent=2, ensure_ascii=False))
    print(f"[done] wrote {args.out}  ({result['elapsed_secs']}s)", file=sys.stderr)

    if not args.json_only:
        print_table(result, summary)
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
