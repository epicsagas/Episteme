"""
Java code smell parser using javalang library
"""

from typing import List

try:
    import javalang
except ImportError:
    javalang = None

from .base import (
    CodeMetrics,
    LanguageParser,
    SmellDetection,
)


class JavaParser(LanguageParser):
    """Java code smell parser using javalang"""

    def __init__(self):
        if javalang is None:
            raise ImportError(
                "javalang library is required for Java parsing. "
                "Install it with: pip install javalang"
            )

    def get_supported_extensions(self) -> List[str]:
        """Return supported file extensions"""
        return [".java"]

    def parse_file(self, file_path: str) -> List[SmellDetection]:
        """Parse a Java file and detect code smells"""
        try:
            with open(file_path, "r", encoding="utf-8") as f:
                source_code = f.read()
            return self.parse_code(source_code, file_path)
        except FileNotFoundError:
            return []
        except Exception:
            return []

    def parse_code(self, code: str, file_name: str = "temp.java") -> List[SmellDetection]:
        """Parse Java code string and detect code smells"""
        try:
            tree = javalang.parse.parse(code)
        except Exception:
            # Syntax error or parse failure
            return []

        detections = []

        # Analyze methods
        for _, node in tree.filter(javalang.tree.MethodDeclaration):
            metrics = self._calculate_method_metrics(node, code)
            location = f"{file_name}:{node.position.line if node.position else 0}"

            # Detect method-level smells
            smell = self.detect_long_method(metrics, location, node.name)
            if smell:
                detections.append(smell)

            smell = self.detect_long_parameter_list(metrics, location, node.name)
            if smell:
                detections.append(smell)

        # Analyze classes
        for _, node in tree.filter(javalang.tree.ClassDeclaration):
            metrics = self._calculate_class_metrics(node, code)
            location = f"{file_name}:{node.position.line if node.position else 0}"

            # Detect class-level smells
            smell = self.detect_large_class(metrics, location, node.name)
            if smell:
                detections.append(smell)

        return detections

    def _calculate_method_metrics(self, method_node, source_code: str) -> CodeMetrics:
        """Calculate metrics for a Java method"""
        # Get method body if available
        if not method_node.body:
            return CodeMetrics(
                loc=0,
                cyclomatic_complexity=1,
                nesting_depth=0,
                parameter_count=len(method_node.parameters) if method_node.parameters else 0,
                local_variables=0,
                return_statements=0,
            )

        # Count parameters
        param_count = len(method_node.parameters) if method_node.parameters else 0

        # Count lines (approximate from position)
        loc = self._count_lines_in_node(method_node, source_code)

        # Calculate cyclomatic complexity
        cc = self._calculate_cc(method_node)

        # Calculate nesting depth
        nesting = self._calculate_nesting(method_node)

        # Count local variables
        local_vars = self._count_local_variables(method_node)

        # Count return statements
        returns = self._count_returns(method_node)

        return CodeMetrics(
            loc=loc,
            cyclomatic_complexity=cc,
            nesting_depth=nesting,
            parameter_count=param_count,
            local_variables=local_vars,
            return_statements=returns,
        )

    def _calculate_class_metrics(self, class_node, source_code: str) -> CodeMetrics:
        """Calculate metrics for a Java class"""
        # Count methods
        methods = list(class_node.filter(javalang.tree.MethodDeclaration))
        method_count = len(methods)

        # Count fields (filter returns tuples of (path, node))
        fields = [node for path, node in class_node.filter(javalang.tree.FieldDeclaration)]
        field_count = sum(len(f.declarators) for f in fields)

        # Count lines
        loc = self._count_lines_in_node(class_node, source_code)

        return CodeMetrics(
            loc=loc,
            cyclomatic_complexity=0,
            nesting_depth=0,
            parameter_count=0,
            local_variables=0,
            return_statements=0,
            method_count=method_count,
            field_count=field_count,
        )

    def _count_lines_in_node(self, node, source_code: str) -> int:
        """Count lines in a node using position information"""
        if not hasattr(node, "position") or not node.position:
            return 0

        # Try to find the end of the node by counting braces
        lines = source_code.split("\n")
        start_line = node.position.line - 1
        if start_line >= len(lines):
            return 1

        # Simple heuristic: find matching closing brace
        brace_count = 0
        for i in range(start_line, len(lines)):
            line = lines[i]
            brace_count += line.count("{") - line.count("}")
            if brace_count == 0 and "{" in lines[start_line]:
                return int(i - start_line + 1)

        return 10  # Default fallback

    def _calculate_cc(self, node) -> int:
        """Calculate cyclomatic complexity for Java node"""
        cc = 1  # Base complexity

        # Count control flow statements
        for _, _child in node.filter(javalang.tree.IfStatement):
            cc += 1

        for _, _child in node.filter(javalang.tree.WhileStatement):
            cc += 1

        for _, _child in node.filter(javalang.tree.ForStatement):
            cc += 1

        for _, _child in node.filter(javalang.tree.DoStatement):
            cc += 1

        for _, _child in node.filter(javalang.tree.SwitchStatement):
            cc += 1

        for _, child in node.filter(javalang.tree.TryStatement):
            # Each catch block adds complexity
            if hasattr(child, "catches"):
                cc += len(child.catches)

        for _, _child in node.filter(javalang.tree.TernaryExpression):
            cc += 1

        # Logical operators
        for _, child in node.filter(javalang.tree.BinaryOperation):
            if child.operator in ("&&", "||"):
                cc += 1

        return int(cc)

    def _calculate_nesting(self, node, current_depth: int = 0, max_depth: int = 0) -> int:
        """Calculate maximum nesting depth"""
        if not hasattr(node, "children") and not hasattr(node, "filter"):
            return int(max_depth)

        # Track nesting depth for control structures
        nesting_types = (
            javalang.tree.IfStatement,
            javalang.tree.WhileStatement,
            javalang.tree.ForStatement,
            javalang.tree.DoStatement,
            javalang.tree.TryStatement,
        )

        try:
            for _, child in node.filter(nesting_types):
                new_depth = current_depth + 1
                max_depth = max(max_depth, new_depth)
                # Recursively check nested structures
                child_max = self._calculate_nesting(child, new_depth, max_depth)
                max_depth = max(max_depth, child_max)
        except Exception:
            pass

        return int(max_depth)

    def _count_local_variables(self, node) -> int:
        """Count local variable declarations"""
        try:
            local_vars = list(node.filter(javalang.tree.LocalVariableDeclaration))
            return sum(len(v.declarators) for v in local_vars)
        except Exception:
            return 0

    def _count_returns(self, node) -> int:
        """Count return statements"""
        try:
            returns = list(node.filter(javalang.tree.ReturnStatement))
            return int(len(returns))
        except Exception:
            return 0
