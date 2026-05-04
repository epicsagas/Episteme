#!/usr/bin/env python3
"""
Syntagma Agent Hooks Bridge — Standardized interface for AI agent lifecycles.
Provides grounding, guarding (smell detection), and auditing.
"""

import argparse
import json
import subprocess
import sys
from dataclasses import asdict
from pathlib import Path
from typing import List

from syntagma.cli.analyze import SmellDetection, analyze_path
from syntagma.rag import SyntagmaRAG

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _run_git(args: List[str]) -> str:
    try:
        return subprocess.check_output(["git"] + args, stderr=subprocess.STDOUT).decode("utf-8")
    except (subprocess.CalledProcessError, FileNotFoundError):
        return ""


def _get_staged_files() -> List[Path]:
    output = _run_git(["diff", "--cached", "--name-only"])
    return [Path(f.strip()) for f in output.splitlines() if f.strip()]


# ---------------------------------------------------------------------------
# Command: ground
# ---------------------------------------------------------------------------


def handle_ground(args):
    """Grounding: Search for relevant patterns/laws based on user prompt."""
    rag = SyntagmaRAG()
    query = args.prompt
    if not query and not sys.stdin.isatty():
        query = sys.stdin.read().strip()

    if not query:
        print("Error: No prompt provided for grounding.", file=sys.stderr)
        return 1

    results = rag.search(query, top_k=args.limit)

    if args.json:
        print(json.dumps(results, indent=2))
        return 0

    print("<!-- SYNTAGMA-GROUNDING-START -->")
    print("## Syntagma Engineering Grounding")
    print("Based on your task, the following engineering principles and patterns apply:\n")

    for res in results:
        eid = res.get("id", "N/A")
        name = res.get("name", "N/A")
        etype = res.get("type", "N/A").upper()
        definition = res.get("definition", "")
        print(f"### {name} ([{eid}]) — {etype}")
        print(f"- **Definition**: {definition}")
        print(f"- **Relevance Score**: {res.get('score', 0):.2f}\n")

    print("Use these entities to guide your design and implementation decisions.")
    print("<!-- SYNTAGMA-GROUNDING-END -->")
    return 0


# ---------------------------------------------------------------------------
# Command: sniff
# ---------------------------------------------------------------------------


def handle_sniff(args):
    """Guarding: Detect code smells in staged or specific files."""
    files = []
    if args.staged:
        files = _get_staged_files()
    elif args.files:
        files = [Path(f) for f in args.files]

    if not files:
        if args.staged:
            # No files staged, nothing to sniff
            return 0
        print("Error: No files specified for sniffing.", file=sys.stderr)
        return 1

    all_detections: List[SmellDetection] = []
    for f in files:
        if f.exists() and f.is_file():
            detections = analyze_path(f, min_confidence=args.min_confidence)
            all_detections.extend(detections)

    if args.json:
        print(json.dumps([asdict(d) for d in all_detections], indent=2))
        return 1 if all_detections else 0

    if not all_detections:
        # Silently pass if no smells found in hook mode to avoid noise
        if not args.verbose:
            return 0
        print("✅ No code smells detected in changed files.")
        return 0

    print("<!-- SYNTAGMA-SMELL-WARNING-START -->")
    print("## ⚠️ Syntagma Code Smell Warning")
    print(
        "The following code smells were detected in your recent changes. Please address them or justify why they are acceptable:\n"
    )

    for d in all_detections:
        print(f"### {d.smell_name} ([{d.smell_id}])")
        print(f"- **Location**: `{d.location}` ({d.function_name})")
        print(f"- **Confidence**: {d.confidence:.2f}")
        print("- **Reasons**:")
        for r in d.reasons:
            print(f"  - {r}")
        print()

    print("Tip: Use `suggest_refactorings` tool for actionable fixes.")
    print("<!-- SYNTAGMA-SMELL-WARNING-END -->")
    return 1


# ---------------------------------------------------------------------------
# Command: audit
# ---------------------------------------------------------------------------


def _read_hook_input() -> dict:
    """Read Claude Code hook stdin JSON (non-blocking — returns {} if no pipe)."""
    if sys.stdin.isatty():
        return {}
    try:
        return json.loads(sys.stdin.read())
    except (json.JSONDecodeError, OSError):
        return {}


def handle_audit(args):
    """Auditing: Final check of the session's engineering quality."""
    # Prefer explicit --file arg, then stdin tool_input.file_path, then cwd
    if args.file:
        target = Path(args.file)
    else:
        hook_input = _read_hook_input()
        tool_input = hook_input.get("tool_input", {})
        file_path = tool_input.get("file_path") or tool_input.get("path")
        target = Path(file_path) if file_path else Path.cwd()

    if not target.exists():
        return 0
    detections = analyze_path(target, min_confidence=0.7)

    if args.json:
        print(json.dumps([asdict(d) for d in detections], indent=2))
        return 0

    print("## Syntagma Engineering Audit Report")
    if not detections:
        print("✅ No critical code smells detected. Structural integrity is maintained.")
    else:
        print(
            f"⚠️  Found {len(detections)} critical code smell(s) requiring attention before shipping:\n"
        )
        for d in detections:
            print(f"- **{d.smell_name}** at `{d.location}`")

    print("\nAudit status: " + ("PASS" if not detections else "WARN"))
    return 0


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------


def main(argv=None):
    parser = argparse.ArgumentParser(description="Syntagma Hooks Bridge")
    subparsers = parser.add_subparsers(dest="command", required=True)

    # ground
    p_ground = subparsers.add_parser("ground", help="Grounding: find related entities for a prompt")
    p_ground.add_argument("prompt", nargs="?", help="User prompt to analyze")
    p_ground.add_argument("--limit", type=int, default=3)
    p_ground.add_argument("--json", action="store_true")

    # sniff
    p_sniff = subparsers.add_parser("sniff", help="Guarding: analyze changed files for smells")
    p_sniff.add_argument("files", nargs="*", help="Files to analyze")
    p_sniff.add_argument("--staged", action="store_true", help="Analyze git staged files")
    p_sniff.add_argument("--min-confidence", type=float, default=0.6)
    p_sniff.add_argument("--json", action="store_true")
    p_sniff.add_argument("--verbose", action="store_true")

    # audit
    p_audit = subparsers.add_parser("audit", help="Auditing: final quality check")
    p_audit.add_argument("--file", default=None, help="Specific file to audit (default: cwd)")
    p_audit.add_argument("--json", action="store_true")

    args = parser.parse_args(argv)

    if args.command == "ground":
        return handle_ground(args)
    elif args.command == "sniff":
        return handle_sniff(args)
    elif args.command == "audit":
        return handle_audit(args)

    return 0


if __name__ == "__main__":
    sys.exit(main())
