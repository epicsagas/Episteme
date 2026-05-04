"""
Tests for new metrics (primitive_params, branch_count, method_call_chains)
and new smell detectors (SMELL-03, SMELL-06, SMELL-20) in PythonParser.
"""

import pytest
from syntagma.parsers.python_parser import PythonParser, MetricsCalculator
import ast


# ---------------------------------------------------------------------------
# 1. MetricsCalculator — primitive_params
# ---------------------------------------------------------------------------


class TestPrimitiveParams:
    def _calc(self, code: str) -> MetricsCalculator:
        tree = ast.parse(code)
        func = next(n for n in ast.walk(tree) if isinstance(n, ast.FunctionDef))
        calc = MetricsCalculator()
        calc.visit(func)
        return calc

    def test_all_primitives_no_annotation(self):
        code = "def f(a, b, c): pass"
        calc = self._calc(code)
        # a, b, c — no annotation → counted as primitive
        assert calc.primitive_params == 3

    def test_primitive_type_annotations(self):
        code = "def f(x: int, y: str, z: float, w: bool): pass"
        calc = self._calc(code)
        assert calc.primitive_params == 4

    def test_non_primitive_annotation_excluded(self):
        code = "def f(x: int, obj: MyClass): pass"
        calc = self._calc(code)
        assert calc.primitive_params == 1

    def test_mixed_params(self):
        code = "def f(a: int, b: str, c: MyObj, d): pass"
        calc = self._calc(code)
        # a(int), b(str), d(no annotation) → 3; c(MyObj) → 0
        assert calc.primitive_params == 3

    def test_bytes_is_primitive(self):
        code = "def f(data: bytes): pass"
        calc = self._calc(code)
        assert calc.primitive_params == 1

    def test_no_params(self):
        code = "def f(): pass"
        calc = self._calc(code)
        assert calc.primitive_params == 0


# ---------------------------------------------------------------------------
# 2. MetricsCalculator — branch_count
# ---------------------------------------------------------------------------


class TestBranchCount:
    def _calc(self, code: str) -> MetricsCalculator:
        tree = ast.parse(code)
        func = next(n for n in ast.walk(tree) if isinstance(n, ast.FunctionDef))
        calc = MetricsCalculator()
        calc.visit(func)
        return calc

    def test_if_elif_chain(self):
        code = """
def f(x):
    if x == 1: return 'a'
    elif x == 2: return 'b'
    elif x == 3: return 'c'
    elif x == 4: return 'd'
    elif x == 5: return 'e'
    elif x == 6: return 'f'
    return 'z'
"""
        calc = self._calc(code)
        # 6 if/elif nodes → branch_count == 6
        assert calc.branch_count >= 6

    def test_no_branches(self):
        code = "def f(x): return x + 1"
        calc = self._calc(code)
        assert calc.branch_count == 0

    def test_ternary_expression(self):
        code = "def f(x): return 'a' if x else 'b'"
        calc = self._calc(code)
        # IfExp counts as a branch
        assert calc.branch_count >= 1


# ---------------------------------------------------------------------------
# 3. MetricsCalculator — method_call_chains
# ---------------------------------------------------------------------------


class TestMethodCallChains:
    def _calc(self, code: str) -> MetricsCalculator:
        tree = ast.parse(code)
        func = next(n for n in ast.walk(tree) if isinstance(n, ast.FunctionDef))
        calc = MetricsCalculator()
        calc.visit(func)
        return calc

    def test_long_chain(self):
        # a.b().c().d().e() — chain of 4 attributes after method calls
        code = "def f(a): return a.b().c().d().e()"
        calc = self._calc(code)
        assert calc.method_call_chains >= 3

    def test_no_chain(self):
        code = "def f(x): return x.name"
        calc = self._calc(code)
        assert calc.method_call_chains <= 1

    def test_moderate_chain(self):
        # a.b().c()
        code = "def f(a): return a.b().c()"
        calc = self._calc(code)
        assert calc.method_call_chains >= 2


# ---------------------------------------------------------------------------
# 4. PythonParser.parse_code — SMELL-06 (Switch Statements)
# ---------------------------------------------------------------------------


class TestSwitchStatementsDetection:
    def setup_method(self):
        self.parser = PythonParser()

    def test_detects_long_if_elif_chain(self):
        code = """
def route(x):
    if x == 1: return 'a'
    elif x == 2: return 'b'
    elif x == 3: return 'c'
    elif x == 4: return 'd'
    elif x == 5: return 'e'
    elif x == 6: return 'f'
    return 'z'
"""
        results = self.parser.parse_code(code, "test.py")
        smell_ids = [r.smell_id for r in results]
        assert "SMELL-06" in smell_ids

    def test_no_false_positive_few_branches(self):
        code = """
def f(x):
    if x == 1: return 'a'
    elif x == 2: return 'b'
    return 'c'
"""
        results = self.parser.parse_code(code, "test.py")
        smell_ids = [r.smell_id for r in results]
        assert "SMELL-06" not in smell_ids

    def test_confidence_very_high_for_many_branches(self):
        code = """
def big_route(x):
    if x == 1: return 'a'
    elif x == 2: return 'b'
    elif x == 3: return 'c'
    elif x == 4: return 'd'
    elif x == 5: return 'e'
    elif x == 6: return 'f'
    elif x == 7: return 'g'
    elif x == 8: return 'h'
    elif x == 9: return 'i'
    elif x == 10: return 'j'
    elif x == 11: return 'k'
    return 'z'
"""
        results = self.parser.parse_code(code, "test.py")
        smell = next((r for r in results if r.smell_id == "SMELL-06"), None)
        assert smell is not None
        assert smell.confidence >= 0.9


# ---------------------------------------------------------------------------
# 5. PythonParser.parse_code — SMELL-03 (Primitive Obsession)
# ---------------------------------------------------------------------------


class TestPrimitiveObsessionDetection:
    def setup_method(self):
        self.parser = PythonParser()

    def test_detects_many_primitive_params(self):
        code = """
def create_user(name, age, email, phone, address, country):
    pass
"""
        results = self.parser.parse_code(code, "test.py")
        smell_ids = [r.smell_id for r in results]
        assert "SMELL-03" in smell_ids

    def test_no_false_positive_typed_params(self):
        code = """
def create_user(user: User, address: Address):
    pass
"""
        results = self.parser.parse_code(code, "test.py")
        smell_ids = [r.smell_id for r in results]
        assert "SMELL-03" not in smell_ids

    def test_detects_typed_primitive_params(self):
        code = """
def process(a: int, b: str, c: float, d: bool, e: int):
    pass
"""
        results = self.parser.parse_code(code, "test.py")
        smell_ids = [r.smell_id for r in results]
        assert "SMELL-03" in smell_ids


# ---------------------------------------------------------------------------
# 6. PythonParser.parse_code — SMELL-20 (Message Chains)
# ---------------------------------------------------------------------------


class TestMessageChainsDetection:
    def setup_method(self):
        self.parser = PythonParser()

    def test_detects_long_method_chain(self):
        code = """
def f(a):
    return a.get_user().get_address().get_city().get_name()
"""
        results = self.parser.parse_code(code, "test.py")
        smell_ids = [r.smell_id for r in results]
        assert "SMELL-20" in smell_ids

    def test_no_false_positive_simple_access(self):
        code = """
def f(a):
    return a.name
"""
        results = self.parser.parse_code(code, "test.py")
        smell_ids = [r.smell_id for r in results]
        assert "SMELL-20" not in smell_ids


# ---------------------------------------------------------------------------
# 7. Smell ID correctness — base.py collision fixes
# ---------------------------------------------------------------------------


class TestSmellIdCorrectness:
    """Verify that each detector emits the correct canonical SMELL-XX id."""

    def setup_method(self):
        self.parser = PythonParser()

    def test_duplicate_code_uses_smell_13(self):
        """detect_duplicate_code must emit SMELL-13, not SMELL-03."""
        from syntagma.parsers.base import CodeMetrics, LanguageParser

        class _P(LanguageParser):
            def parse_file(self, f):
                return []

            def parse_code(self, c, fn="t"):
                return []

            def get_supported_extensions(self):
                return []

        p = _P()
        metrics = CodeMetrics(
            loc=10,
            cyclomatic_complexity=1,
            nesting_depth=0,
            parameter_count=0,
            local_variables=0,
            return_statements=0,
            ast_hash="abc123",
        )
        all_hashes = {"abc123": ["file1.py:10", "file2.py:20"]}
        result = p.detect_duplicate_code(metrics, "file1.py:10", "fn", all_hashes)
        assert result is not None
        assert result.smell_id == "SMELL-13", (
            f"detect_duplicate_code must use SMELL-13, got {result.smell_id}"
        )

    def test_primitive_obsession_still_smell_03(self):
        """detect_primitive_obsession must still emit SMELL-03."""
        from syntagma.parsers.base import CodeMetrics, LanguageParser

        class _P(LanguageParser):
            def parse_file(self, f):
                return []

            def parse_code(self, c, fn="t"):
                return []

            def get_supported_extensions(self):
                return []

        p = _P()
        metrics = CodeMetrics(
            loc=5,
            cyclomatic_complexity=1,
            nesting_depth=0,
            parameter_count=5,
            local_variables=0,
            return_statements=0,
            primitive_params=5,
        )
        result = p.detect_primitive_obsession(metrics, "f.py:1", "fn")
        assert result is not None
        assert result.smell_id == "SMELL-03", (
            f"detect_primitive_obsession must use SMELL-03, got {result.smell_id}"
        )


# ---------------------------------------------------------------------------
# 8. Async function support in python_parser.py
# ---------------------------------------------------------------------------


class TestAsyncFunctionSupport:
    """Async functions must be treated the same as sync functions."""

    def setup_method(self):
        self.parser = PythonParser()

    def test_async_function_metrics_tracked(self):
        """MetricsCalculator should handle AsyncFunctionDef via alias."""
        import ast as _ast
        from syntagma.parsers.python_parser import MetricsCalculator

        code = "async def f(a, b, c): pass"
        tree = _ast.parse(code)
        func = next(n for n in _ast.walk(tree) if isinstance(n, _ast.AsyncFunctionDef))
        calc = MetricsCalculator()
        calc.visit(func)
        assert calc.param_count == 3

    def test_async_method_counted_in_class(self):
        """ClassMetricsCalculator must count async methods."""
        import ast as _ast
        from syntagma.parsers.python_parser import ClassMetricsCalculator

        code = """
class Svc:
    async def do_a(self): pass
    async def do_b(self): pass
    def do_c(self): pass
"""
        tree = _ast.parse(code)
        cls = next(n for n in _ast.walk(tree) if isinstance(n, _ast.ClassDef))
        calc = ClassMetricsCalculator()
        calc.visit(cls)
        assert calc.method_count == 3

    def test_async_function_smell_detected(self):
        """parse_code must detect smells in async functions."""
        # A long async function with many branches should trigger SMELL-06
        branches = "\n    ".join(f"elif x == {i}: return {i}" for i in range(2, 13))
        code = f"""
async def dispatch(x):
    if x == 1: return 1
    {branches}
    return 0
"""
        results = self.parser.parse_code(code, "test.py")
        smell_ids = [r.smell_id for r in results]
        assert "SMELL-06" in smell_ids, "async function with many branches must trigger SMELL-06"


# ---------------------------------------------------------------------------
# 9. God Object uses SMELL-21
# ---------------------------------------------------------------------------


class TestGodObjectSmellId:
    """detect_god_object must emit SMELL-21."""

    def test_god_object_uses_smell_21(self):
        from syntagma.parsers.base import CodeMetrics, LanguageParser

        class _P(LanguageParser):
            def parse_file(self, f):
                return []

            def parse_code(self, c, fn="t"):
                return []

            def get_supported_extensions(self):
                return []

        p = _P()
        metrics = CodeMetrics(
            loc=600,
            cyclomatic_complexity=60,
            nesting_depth=0,
            parameter_count=0,
            local_variables=0,
            return_statements=0,
            method_count=35,
            field_count=25,
        )
        result = p.detect_god_object(metrics, "big.py:1", "GodClass")
        assert result is not None
        assert result.smell_id == "SMELL-21", (
            f"detect_god_object must use SMELL-21, got {result.smell_id}"
        )


# ---------------------------------------------------------------------------
# 10. detect_data_class — redundant inner guard removed
# ---------------------------------------------------------------------------


class TestDataClassGuard:
    """detect_data_class must not return None when method_count > 0 with valid ratio."""

    def setup_method(self):
        from syntagma.parsers.base import LanguageParser

        class _P(LanguageParser):
            def parse_file(self, f):
                return []

            def parse_code(self, c, fn="t"):
                return []

            def get_supported_extensions(self):
                return []

        self.p = _P()

    def test_detects_data_class_with_nonzero_methods(self):
        from syntagma.parsers.base import CodeMetrics

        metrics = CodeMetrics(
            loc=30,
            cyclomatic_complexity=1,
            nesting_depth=0,
            parameter_count=0,
            local_variables=0,
            return_statements=0,
            method_count=3,
            field_count=10,
        )
        result = self.p.detect_data_class(metrics, "m.py:1", "MyData")
        assert result is not None
        assert result.smell_id == "SMELL-07"

    def test_skips_zero_method_count(self):
        from syntagma.parsers.base import CodeMetrics

        metrics = CodeMetrics(
            loc=10,
            cyclomatic_complexity=1,
            nesting_depth=0,
            parameter_count=0,
            local_variables=0,
            return_statements=0,
            method_count=0,
            field_count=10,
        )
        result = self.p.detect_data_class(metrics, "m.py:1", "Empty")
        assert result is None
