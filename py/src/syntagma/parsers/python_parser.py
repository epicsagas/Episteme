"""
Python code smell parser using AST analysis
Refactored from detect_smells.py to use LanguageParser interface
"""

import ast
from typing import List, Set

from .base import CodeMetrics, LanguageParser, SmellDetection, SmellType

_PRIMITIVE_TYPES = frozenset({"int", "str", "float", "bool", "bytes"})


def _count_chain_depth(node: ast.expr) -> int:
    """
    Return the longest attribute-call chain depth rooted at *node*.

    Pattern counted: a.b().c().d()  — each `.attr()` hop adds 1.
    We walk: Call → func (Attribute) → value (Call) → func (Attribute) → ...
    """
    depth = 0
    current = node
    while isinstance(current, ast.Call):
        func = current.func
        if not isinstance(func, ast.Attribute):
            break
        depth += 1
        current = func.value
    return depth


class MetricsCalculator(ast.NodeVisitor):
    """Calculate code metrics from Python AST"""

    def __init__(self):
        self.loc = 0
        self.cc = 1  # Start at 1 for base complexity
        self.max_nesting = 0
        self.current_nesting = 0
        self.param_count = 0
        self.primitive_params = 0
        self.local_vars: Set[str] = set()
        self.return_count = 0
        self.external_calls = 0
        self.branch_count = 0
        self.method_call_chains = 0  # max chain depth seen in this function

    def visit_AsyncFunctionDef(self, node):
        self.visit_FunctionDef(node)

    def visit_FunctionDef(self, node):
        """Visit function definition"""
        self.param_count = len(node.args.args)
        self.primitive_params = self._count_primitive_params(node.args.args)

        # Count lines (excluding docstring)
        start_line = node.lineno
        end_line = node.end_lineno if hasattr(node, "end_lineno") else start_line
        self.loc = end_line - start_line + 1

        # Check for docstring
        if (
            node.body
            and isinstance(node.body[0], ast.Expr)
            and isinstance(node.body[0].value, ast.Constant)
            and isinstance(node.body[0].value.value, str)
        ):
            # Subtract docstring lines
            docstring_node = node.body[0]
            docstring_lines = (
                docstring_node.end_lineno - docstring_node.lineno + 1
                if hasattr(docstring_node, "end_lineno")
                else 1
            )
            self.loc -= docstring_lines

        self.generic_visit(node)

    @staticmethod
    def _count_primitive_params(args: list) -> int:
        """Count parameters that are primitive or have no type annotation."""
        count = 0
        for arg in args:
            ann = arg.annotation
            if ann is None:
                count += 1
            elif isinstance(ann, ast.Name) and ann.id in _PRIMITIVE_TYPES:
                count += 1
        return count

    def visit_If(self, node):
        """Count if statements (CC +1, nesting +1, branch +1)"""
        self.cc += 1
        self.branch_count += 1
        self.current_nesting += 1
        self.max_nesting = max(self.max_nesting, self.current_nesting)
        self.generic_visit(node)
        self.current_nesting -= 1

    def visit_IfExp(self, node):
        """Count ternary expressions as a branch"""
        self.branch_count += 1
        self.generic_visit(node)

    def visit_Match(self, node):
        """Count match statements (Python 3.10+) — each case is a branch"""
        self.branch_count += len(node.cases)
        self.generic_visit(node)

    def visit_While(self, node):
        """Count while loops (CC +1, nesting +1)"""
        self.cc += 1
        self.current_nesting += 1
        self.max_nesting = max(self.max_nesting, self.current_nesting)
        self.generic_visit(node)
        self.current_nesting -= 1

    def visit_For(self, node):
        """Count for loops (CC +1, nesting +1)"""
        self.cc += 1
        self.current_nesting += 1
        self.max_nesting = max(self.max_nesting, self.current_nesting)
        self.generic_visit(node)
        self.current_nesting -= 1

    def visit_ExceptHandler(self, node):
        """Count except handlers (CC +1)"""
        self.cc += 1
        self.generic_visit(node)

    def visit_With(self, node):
        """Count with statements (CC +1)"""
        self.cc += 1
        self.generic_visit(node)

    def visit_BoolOp(self, node):
        """Count boolean operators (and/or) (CC +1 per operator)"""
        if isinstance(node.op, (ast.And, ast.Or)):
            self.cc += len(node.values) - 1
        self.generic_visit(node)

    def visit_Assign(self, node):
        """Track local variable assignments"""
        for target in node.targets:
            if isinstance(target, ast.Name):
                self.local_vars.add(target.id)
        self.generic_visit(node)

    def visit_Return(self, node):
        """Count return statements"""
        self.return_count += 1
        self.generic_visit(node)

    def visit_Call(self, node):
        """Count external method calls and track method call chain depth"""
        if isinstance(node.func, ast.Attribute):
            self.external_calls += 1
        depth = _count_chain_depth(node)
        if depth > self.method_call_chains:
            self.method_call_chains = depth
        self.generic_visit(node)


class ClassMetricsCalculator(ast.NodeVisitor):
    """Calculate metrics for Python classes"""

    def __init__(self):
        self.loc = 0
        self.method_count = 0
        self.field_count = 0

    def visit_ClassDef(self, node):
        """Visit class definition"""
        start_line = node.lineno
        end_line = node.end_lineno if hasattr(node, "end_lineno") else start_line
        self.loc = end_line - start_line + 1

        # Count methods and fields
        for item in node.body:
            if isinstance(item, (ast.FunctionDef, ast.AsyncFunctionDef)):
                self.method_count += 1
            elif isinstance(item, ast.Assign):
                self.field_count += len(item.targets)
            elif isinstance(item, ast.AnnAssign):
                self.field_count += 1


class PythonParser(LanguageParser):
    """Python code smell parser using AST analysis"""

    def get_supported_extensions(self) -> List[str]:
        """Return supported file extensions"""
        return [".py"]

    def parse_file(self, file_path: str) -> List[SmellDetection]:
        """Parse a Python file and detect code smells"""
        try:
            with open(file_path, "r", encoding="utf-8") as f:
                source_code = f.read()
            return self.parse_code(source_code, file_path)
        except FileNotFoundError:
            return []
        except Exception:
            return []

    def parse_code(self, code: str, file_name: str = "temp.py") -> List[SmellDetection]:
        """Parse Python code string and detect code smells"""
        try:
            tree = ast.parse(code)
        except SyntaxError:
            return []

        detections = []

        # Analyze each function (sync and async)
        for node in ast.walk(tree):
            if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
                metrics = self._calculate_function_metrics(node)
                location = f"{file_name}:{node.lineno}"

                # Detect smells
                smell = self.detect_long_method(metrics, location, node.name)
                if smell:
                    detections.append(smell)

                smell = self.detect_long_parameter_list(metrics, location, node.name)
                if smell:
                    detections.append(smell)

                smell = self._detect_feature_envy(metrics, location, node.name)
                if smell:
                    detections.append(smell)

                smell = self.detect_primitive_obsession(metrics, location, node.name)
                if smell:
                    detections.append(smell)

                smell = self.detect_switch_statements(metrics, location, node.name)
                if smell:
                    detections.append(smell)

                smell = self.detect_message_chains(metrics, location, node.name)
                if smell:
                    detections.append(smell)

            elif isinstance(node, ast.ClassDef):
                metrics = self._calculate_class_metrics(node)
                location = f"{file_name}:{node.lineno}"

                # Detect class-level smells
                smell = self.detect_large_class(metrics, location, node.name)
                if smell:
                    detections.append(smell)

        return detections

    def _calculate_function_metrics(
        self, func_node: ast.FunctionDef | ast.AsyncFunctionDef
    ) -> CodeMetrics:
        """Calculate metrics for a function"""
        calculator = MetricsCalculator()
        calculator.visit(func_node)

        return CodeMetrics(
            loc=calculator.loc,
            cyclomatic_complexity=calculator.cc,
            nesting_depth=calculator.max_nesting,
            parameter_count=calculator.param_count,
            local_variables=len(calculator.local_vars),
            return_statements=calculator.return_count,
            external_calls=calculator.external_calls,
            primitive_params=calculator.primitive_params,
            branch_count=calculator.branch_count,
            method_call_chains=calculator.method_call_chains,
        )

    def _calculate_class_metrics(self, class_node: ast.ClassDef) -> CodeMetrics:
        """Calculate metrics for a class"""
        calculator = ClassMetricsCalculator()
        calculator.visit(class_node)

        return CodeMetrics(
            loc=calculator.loc,
            cyclomatic_complexity=0,
            nesting_depth=0,
            parameter_count=0,
            local_variables=0,
            return_statements=0,
            method_count=calculator.method_count,
            field_count=calculator.field_count,
        )

    def _detect_feature_envy(
        self, metrics: CodeMetrics, location: str, func_name: str
    ) -> SmellDetection | None:
        """
        Detect Feature Envy smell

        Heuristics:
        - High external calls suggest interaction with other objects
        - Multiple return statements suggest complex logic
        - High CC with moderate LOC suggests branching
        """
        reasons = []
        confidence = 0.0

        if metrics.external_calls > 5:
            reasons.append(f"External calls={metrics.external_calls} exceeds 5")
            confidence += 0.4

        if metrics.return_statements > 5:
            reasons.append(f"Return statements={metrics.return_statements} exceeds 5")
            confidence += 0.3

        if metrics.cyclomatic_complexity > 8 and metrics.loc < 40:
            reasons.append("High CC with moderate LOC suggests complex branching")
            confidence += 0.2

        if confidence >= 0.5:
            return SmellDetection(
                smell_id="SMELL-18",
                smell_name=SmellType.FEATURE_ENVY.value,
                confidence=min(confidence, 1.0),
                location=location,
                function_name=func_name,
                metrics=metrics,
                reasons=reasons,
            )

        return None
