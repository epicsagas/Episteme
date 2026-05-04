#!/usr/bin/env python3
"""
Code Smell Detector — multi-language CLI entry point.

Delegates analysis to language-specific parsers in syntagma.parsers.
The legacy Python-only MetricsCalculator / CodeSmellDetector classes are
kept for backwards-compatibility (mcp/server.py and api/main.py still
import them directly).
"""

import ast
import json
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Dict, List, Optional, Set

from syntagma import config as _config

# ---------------------------------------------------------------------------
# Extension → language name mapping (mirrors parsers/__init__.py get_parser)
# ---------------------------------------------------------------------------
# Directories to skip during recursive file collection, keyed by language.
# "all" entries apply regardless of language.
_IGNORE_DIRS: Dict[str, Set[str]] = {
    "all": {
        ".git",
        ".hg",
        ".svn",
        "worktrees",
        "node_modules",
        ".pnpm-store",
        "__pycache__",
        ".mypy_cache",
        ".pytest_cache",
        ".ruff_cache",
        ".venv",
        "venv",
        "env",
        ".env",
        "dist",
        "build",
        "out",
        ".next",
        ".nuxt",
        ".svelte-kit",
        ".cache",
        ".parcel-cache",
    },
    "rust": {"target"},
    "go": {"vendor"},
    "java": {"build", ".gradle", ".m2"},
    "kotlin": {"build", ".gradle"},
    "cpp": {"CMakeFiles", "cmake-build-debug", "cmake-build-release"},
    "javascript": {"coverage", ".turbo"},
    "typescript": {"coverage", ".turbo"},
    "python": {"site-packages", "dist-info", "egg-info"},
}


def _is_ignored(path: Path, lang: Optional[str]) -> bool:
    """Return True if path contains any ignore-dir segment for the given language."""
    blocked = set(_IGNORE_DIRS["all"])
    if lang and lang in _IGNORE_DIRS:
        blocked |= _IGNORE_DIRS[lang]
    return any(part in blocked for part in path.parts)


_EXT_TO_LANG: Dict[str, str] = {
    ".py": "python",
    ".java": "java",
    ".ts": "typescript",
    ".tsx": "typescript",
    ".js": "javascript",
    ".jsx": "javascript",
    ".go": "go",
    ".rs": "rust",
    ".cpp": "cpp",
    ".cc": "cpp",
    ".cxx": "cpp",
    ".h": "cpp",
    ".hpp": "cpp",
    ".hxx": "cpp",
    ".cs": "csharp",
    ".php": "php",
    ".rb": "ruby",
    ".kt": "kotlin",
    ".kts": "kotlin",
}

SUPPORTED_EXTENSIONS = set(_EXT_TO_LANG.keys())


# ---------------------------------------------------------------------------
# Legacy dataclasses (kept for callers that import them directly)
# ---------------------------------------------------------------------------


@dataclass
class CodeMetrics:
    """Code metrics for a function/class"""

    loc: int
    cyclomatic_complexity: int
    nesting_depth: int
    parameter_count: int
    local_variables: int
    return_statements: int


@dataclass
class SmellDetection:
    """Detected code smell with confidence and location"""

    smell_id: str
    smell_name: str
    confidence: float
    location: str
    function_name: str
    metrics: CodeMetrics
    reasons: List[str]


# ---------------------------------------------------------------------------
# Legacy MetricsCalculator (Python AST — kept for CodeSmellDetector)
# ---------------------------------------------------------------------------


class MetricsCalculator(ast.NodeVisitor):
    """Calculate code metrics from AST"""

    def __init__(self):
        self.loc = 0
        self.cc = 1
        self.max_nesting = 0
        self.current_nesting = 0
        self.param_count = 0
        self.local_vars: Set[str] = set()
        self.return_count = 0

    def visit_FunctionDef(self, node):
        self.param_count = len(node.args.args)
        start_line = node.lineno
        end_line = node.end_lineno if hasattr(node, "end_lineno") else start_line
        self.loc = end_line - start_line + 1
        if (
            node.body
            and isinstance(node.body[0], ast.Expr)
            and isinstance(node.body[0].value, ast.Constant)
            and isinstance(node.body[0].value.value, str)
        ):
            docstring_node = node.body[0]
            docstring_lines = (
                docstring_node.end_lineno - docstring_node.lineno + 1
                if hasattr(docstring_node, "end_lineno")
                else 1
            )
            self.loc -= docstring_lines
        self.generic_visit(node)

    def visit_If(self, node):
        self.cc += 1
        self.current_nesting += 1
        self.max_nesting = max(self.max_nesting, self.current_nesting)
        self.generic_visit(node)
        self.current_nesting -= 1

    def visit_While(self, node):
        self.cc += 1
        self.current_nesting += 1
        self.max_nesting = max(self.max_nesting, self.current_nesting)
        self.generic_visit(node)
        self.current_nesting -= 1

    def visit_For(self, node):
        self.cc += 1
        self.current_nesting += 1
        self.max_nesting = max(self.max_nesting, self.current_nesting)
        self.generic_visit(node)
        self.current_nesting -= 1

    def visit_ExceptHandler(self, node):
        self.cc += 1
        self.generic_visit(node)

    def visit_With(self, node):
        self.cc += 1
        self.generic_visit(node)

    def visit_BoolOp(self, node):
        if isinstance(node.op, (ast.And, ast.Or)):
            self.cc += len(node.values) - 1
        self.generic_visit(node)

    def visit_Assign(self, node):
        for target in node.targets:
            if isinstance(target, ast.Name):
                self.local_vars.add(target.id)
        self.generic_visit(node)

    def visit_Return(self, node):
        self.return_count += 1
        self.generic_visit(node)


# ---------------------------------------------------------------------------
# Legacy CodeSmellDetector (Python-only, kept for MCP/API backwards-compat)
# ---------------------------------------------------------------------------


class CodeSmellDetector:
    """Detects code smells in Python code (legacy — use MultiLanguageDetector for all languages)"""

    def __init__(self, base_dir: str | None = None):
        self.base_dir = Path(base_dir) if base_dir else _config.SYNTAGMA_HOME
        self.meta_dir = _config.DATA_DIR

        with open(self.meta_dir / "code_smells.json", "r") as f:
            data = json.load(f)
            self.smells = data.get("smells", {})

    def analyze_file(self, file_path: str) -> List[SmellDetection]:
        path = Path(file_path)
        if not path.exists():
            raise FileNotFoundError(f"File not found: {file_path}")

        with open(file_path, "r", encoding="utf-8") as f:
            source_code = f.read()

        try:
            tree = ast.parse(source_code)
        except SyntaxError as e:
            print(f"Syntax error in {file_path}: {e}")
            return []

        detections = []
        for node in ast.walk(tree):
            if isinstance(node, ast.FunctionDef):
                metrics = self._calculate_metrics(node)
                smells = self._detect_smells_in_function(path.name, node.name, node.lineno, metrics)
                detections.extend(smells)
        return detections

    def _calculate_metrics(self, func_node: ast.FunctionDef) -> CodeMetrics:
        calculator = MetricsCalculator()
        calculator.visit(func_node)
        return CodeMetrics(
            loc=calculator.loc,
            cyclomatic_complexity=calculator.cc,
            nesting_depth=calculator.max_nesting,
            parameter_count=calculator.param_count,
            local_variables=len(calculator.local_vars),
            return_statements=calculator.return_count,
        )

    def _detect_smells_in_function(
        self, filename: str, func_name: str, lineno: int, metrics: CodeMetrics
    ) -> List[SmellDetection]:
        detections = []
        long_method = self._detect_long_method(filename, func_name, lineno, metrics)
        if long_method:
            detections.append(long_method)
        long_params = self._detect_long_parameter_list(filename, func_name, lineno, metrics)
        if long_params:
            detections.append(long_params)
        feature_envy = self._detect_feature_envy(filename, func_name, lineno, metrics)
        if feature_envy:
            detections.append(feature_envy)
        return detections

    def _detect_long_method(
        self, filename: str, func_name: str, lineno: int, metrics: CodeMetrics
    ) -> Optional[SmellDetection]:
        reasons = []
        score = 0.0
        if metrics.loc > 30:
            reasons.append(f"LOC={metrics.loc} exceeds 30")
            score += 0.4
        if metrics.cyclomatic_complexity > 10:
            reasons.append(f"CC={metrics.cyclomatic_complexity} exceeds 10")
            score += 0.3
        if metrics.nesting_depth > 4:
            reasons.append(f"Nesting depth={metrics.nesting_depth} exceeds 4")
            score += 0.2
        if metrics.local_variables > 10:
            reasons.append(f"Too many local variables ({metrics.local_variables})")
            score += 0.1
        if score >= 0.5:
            return SmellDetection(
                smell_id="SMELL-01",
                smell_name="Long Method",
                confidence=min(score, 1.0),
                location=f"{filename}:{lineno}",
                function_name=func_name,
                metrics=metrics,
                reasons=reasons,
            )
        return None

    def _detect_long_parameter_list(
        self, filename: str, func_name: str, lineno: int, metrics: CodeMetrics
    ) -> Optional[SmellDetection]:
        reasons = []
        score = 0.0
        if metrics.parameter_count > 5:
            reasons.append(f"{metrics.parameter_count} parameters (> 5)")
            score = 0.9
        elif metrics.parameter_count > 3:
            reasons.append(f"{metrics.parameter_count} parameters (> 3)")
            score = 0.6
            if metrics.loc > 20:
                score += 0.2
                reasons.append("Combined with long method")
        if score >= 0.6:
            return SmellDetection(
                smell_id="SMELL-03",
                smell_name="Long Parameter List",
                confidence=min(score, 1.0),
                location=f"{filename}:{lineno}",
                function_name=func_name,
                metrics=metrics,
                reasons=reasons,
            )
        return None

    def _detect_feature_envy(
        self, filename: str, func_name: str, lineno: int, metrics: CodeMetrics
    ) -> Optional[SmellDetection]:
        reasons = []
        score = 0.0
        if metrics.return_statements > 5:
            reasons.append(f"{metrics.return_statements} return statements")
            score += 0.5
        if metrics.cyclomatic_complexity > 8 and metrics.loc < 40:
            reasons.append("High CC with moderate LOC suggests complex branching")
            score += 0.3
        if score >= 0.5:
            return SmellDetection(
                smell_id="SMELL-18",
                smell_name="Feature Envy",
                confidence=min(score, 1.0),
                location=f"{filename}:{lineno}",
                function_name=func_name,
                metrics=metrics,
                reasons=reasons,
            )
        return None


# ---------------------------------------------------------------------------
# Multi-language detector — used by the CLI
# ---------------------------------------------------------------------------


def _lang_from_path(file_path: Path, hint: Optional[str] = None) -> Optional[str]:
    """Return language name for a file, preferring explicit hint."""
    if hint:
        return hint.lower()
    return _EXT_TO_LANG.get(file_path.suffix.lower())


def _collect_files(target: Path, language_hint: Optional[str]) -> List[Path]:
    """Return the list of files to analyze under target."""
    if not target.is_dir():
        return [target]

    lang = language_hint.lower() if language_hint else None

    def _keep(p: Path) -> bool:
        return not _is_ignored(p, lang)

    if lang:
        exts = {ext for ext, l in _EXT_TO_LANG.items() if l == lang}
        lang_files: List[Path] = []
        for ext in sorted(exts):
            lang_files.extend(f for f in target.rglob(f"*{ext}") if _keep(f))
        return sorted(set(lang_files))

    files: List[Path] = []
    for ext in sorted(SUPPORTED_EXTENSIONS):
        files.extend(f for f in target.rglob(f"*{ext}") if _keep(f))
    return sorted(set(files))


def _to_legacy_smell(raw) -> SmellDetection:
    """Convert parsers.base.SmellDetection to the legacy dataclass used by the CLI."""
    if isinstance(raw, SmellDetection):
        return raw

    raw_m = raw.metrics
    metrics = CodeMetrics(
        loc=raw_m.loc,
        cyclomatic_complexity=raw_m.cyclomatic_complexity,
        nesting_depth=raw_m.nesting_depth,
        parameter_count=raw_m.parameter_count,
        local_variables=raw_m.local_variables,
        return_statements=raw_m.return_statements,
    )
    return SmellDetection(
        smell_id=raw.smell_id,
        smell_name=raw.smell_name,
        confidence=raw.confidence,
        location=raw.location,
        function_name=raw.function_name,
        metrics=metrics,
        reasons=raw.reasons,
    )


def analyze_path(
    target: Path,
    language_hint: Optional[str] = None,
    min_confidence: float = 0.5,
) -> List[SmellDetection]:
    """Analyze a file or directory using the appropriate language parser(s)."""
    from syntagma.parsers import get_parser

    files = _collect_files(target, language_hint)
    results: List[SmellDetection] = []

    for f in files:
        lang = _lang_from_path(f, language_hint)
        if not lang:
            continue
        try:
            parser = get_parser(lang)
            raw_detections = parser.parse_file(str(f))
            for raw in raw_detections:
                smell = _to_legacy_smell(raw)
                if smell.confidence >= min_confidence:
                    results.append(smell)
        except Exception as e:
            print(f"Warning: could not analyze {f}: {e}")

    return results


# ---------------------------------------------------------------------------
# CLI entry point
# ---------------------------------------------------------------------------


def main(argv=None):
    """CLI for code smell detection"""
    import argparse

    parser = argparse.ArgumentParser(description="Code Smell Detector")
    parser.add_argument("file", help="Source file or directory to analyze")
    parser.add_argument("--json", action="store_true", help="Output JSON format")
    parser.add_argument(
        "--min-confidence", type=float, default=0.5, help="Minimum confidence threshold (0.0-1.0)"
    )
    parser.add_argument("--language", help="Language hint (e.g. java, typescript, go)")

    args = parser.parse_args(argv)

    target = Path(args.file)
    detections = analyze_path(
        target, language_hint=args.language, min_confidence=args.min_confidence
    )

    if args.json:
        print(json.dumps([asdict(d) for d in detections], indent=2))
    else:
        if not detections:
            print(f"✅ No code smells detected in {args.file}")
        else:
            print(f"\n⚠️  Found {len(detections)} code smell(s) in {args.file}:\n")
            for detection in detections:
                print(f"🔴 {detection.smell_name} (confidence: {detection.confidence:.2f})")
                print(f"   Location: {detection.location} ({detection.function_name})")
                print(
                    f"   Metrics: LOC={detection.metrics.loc}, CC={detection.metrics.cyclomatic_complexity}, "
                    f"Depth={detection.metrics.nesting_depth}, Params={detection.metrics.parameter_count}"
                )
                print("   Reasons:")
                for reason in detection.reasons:
                    print(f"     - {reason}")
                print()


if __name__ == "__main__":
    main()
