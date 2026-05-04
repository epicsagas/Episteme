"""
TypeScript/JavaScript code smell parser using regex-based analysis
"""

import re
from typing import List

from .base import (
    CodeMetrics,
    LanguageParser,
    SmellDetection,
)


class TypeScriptParser(LanguageParser):
    """TypeScript/JavaScript parser using regex patterns"""

    # Regex patterns
    FUNCTION_PATTERN = re.compile(
        r"(?:export\s+)?(?:async\s+)?(?:function\s+(\w+)|(?:const|let|var)\s+(\w+)\s*=\s*(?:async\s+)?(?:\([^)]*\)|(\w+))\s*=>|(\w+)\s*\([^)]*\)\s*\{)",
        re.MULTILINE,
    )
    CLASS_PATTERN = re.compile(r"(?:export\s+)?(?:abstract\s+)?class\s+(\w+)", re.MULTILINE)
    METHOD_PATTERN = re.compile(
        r"(?:public|private|protected|static|async)?\s*(\w+)\s*\([^)]*\)\s*(?::\s*\w+\s*)?\{",
        re.MULTILINE,
    )
    PARAMETER_PATTERN = re.compile(r"\([^)]*\)")
    FIELD_PATTERN = re.compile(
        r"(?:public|private|protected|readonly)?\s*(\w+)\s*(?::\s*[\w<>[\]|]+)?(?:\s*=|;)",
        re.MULTILINE,
    )

    def get_supported_extensions(self) -> List[str]:
        """Return supported file extensions"""
        return [".ts", ".tsx", ".js", ".jsx"]

    def parse_file(self, file_path: str) -> List[SmellDetection]:
        """Parse a TypeScript/JavaScript file and detect code smells"""
        try:
            with open(file_path, "r", encoding="utf-8") as f:
                source_code = f.read()
            return self.parse_code(source_code, file_path)
        except FileNotFoundError:
            return []
        except Exception:
            return []

    def parse_code(self, code: str, file_name: str = "temp.ts") -> List[SmellDetection]:
        """Parse TypeScript/JavaScript code string and detect code smells"""
        detections = []

        # Parse functions
        detections.extend(self._parse_functions(code, file_name))

        # Parse classes
        detections.extend(self._parse_classes(code, file_name))

        return detections

    def _parse_functions(self, code: str, file_name: str) -> List[SmellDetection]:
        """Parse and analyze functions"""
        detections = []

        # Find all function declarations
        for match in self.FUNCTION_PATTERN.finditer(code):
            func_name = (
                match.group(1) or match.group(2) or match.group(3) or match.group(4) or "anonymous"
            )
            start_pos = match.start()
            line_no = code[:start_pos].count("\n") + 1

            # Extract function body
            body_start = code.find("{", start_pos)
            if body_start == -1:
                continue

            body_end = self._find_matching_brace(code, body_start)
            if body_end == -1:
                continue

            func_body = code[body_start : body_end + 1]

            # Calculate metrics
            metrics = self._calculate_function_metrics(match.group(0), func_body)
            location = f"{file_name}:{line_no}"

            # Detect smells
            smell = self.detect_long_method(metrics, location, func_name)
            if smell:
                detections.append(smell)

            smell = self.detect_long_parameter_list(metrics, location, func_name)
            if smell:
                detections.append(smell)

        return detections

    def _parse_classes(self, code: str, file_name: str) -> List[SmellDetection]:
        """Parse and analyze classes"""
        detections = []

        for match in self.CLASS_PATTERN.finditer(code):
            class_name = match.group(1)
            start_pos = match.start()
            line_no = code[:start_pos].count("\n") + 1

            # Extract class body
            body_start = code.find("{", start_pos)
            if body_start == -1:
                continue

            body_end = self._find_matching_brace(code, body_start)
            if body_end == -1:
                continue

            class_body = code[body_start : body_end + 1]

            # Calculate metrics
            metrics = self._calculate_class_metrics(class_body)
            location = f"{file_name}:{line_no}"

            # Detect smells
            smell = self.detect_large_class(metrics, location, class_name)
            if smell:
                detections.append(smell)

        return detections

    def _calculate_function_metrics(self, signature: str, body: str) -> CodeMetrics:
        """Calculate metrics for a function"""
        # Count lines (excluding braces)
        loc = len(
            [line for line in body.split("\n") if line.strip() and line.strip() not in ["{", "}"]]
        )

        # Count parameters
        param_match = self.PARAMETER_PATTERN.search(signature)
        param_count = 0
        if param_match:
            params_str = param_match.group(0)[1:-1]  # Remove parentheses
            if params_str.strip():
                param_count = len([p for p in params_str.split(",") if p.strip()])

        # Calculate cyclomatic complexity
        cc = self._calculate_cc(body)

        # Calculate nesting depth
        nesting = self._calculate_nesting(body)

        # Count local variables
        local_vars = len(re.findall(r"\b(?:const|let|var)\s+\w+", body))

        # Count return statements
        returns = len(re.findall(r"\breturn\b", body))

        return CodeMetrics(
            loc=loc,
            cyclomatic_complexity=cc,
            nesting_depth=nesting,
            parameter_count=param_count,
            local_variables=local_vars,
            return_statements=returns,
        )

    def _calculate_class_metrics(self, class_body: str) -> CodeMetrics:
        """Calculate metrics for a class"""
        # Count methods
        methods = self.METHOD_PATTERN.findall(class_body)
        method_count = len(methods)

        # Count fields (properties)
        fields = self.FIELD_PATTERN.findall(class_body)
        field_count = len(fields)

        # Count lines
        loc = len(
            [
                line
                for line in class_body.split("\n")
                if line.strip() and line.strip() not in ["{", "}"]
            ]
        )

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

    def _calculate_cc(self, code: str) -> int:
        """Calculate cyclomatic complexity"""
        cc = 1  # Base complexity

        # Count control flow keywords
        cc += len(re.findall(r"\bif\b", code))
        cc += len(re.findall(r"\belse\s+if\b", code))
        cc += len(re.findall(r"\bfor\b", code))
        cc += len(re.findall(r"\bwhile\b", code))
        cc += len(re.findall(r"\bcase\b", code))
        cc += len(re.findall(r"\bcatch\b", code))
        cc += len(re.findall(r"\b\?\s*.*?\s*:", code))  # Ternary
        cc += len(re.findall(r"\&\&", code))  # Logical AND
        cc += len(re.findall(r"\|\|", code))  # Logical OR

        return cc

    def _calculate_nesting(self, code: str) -> int:
        """Calculate maximum nesting depth"""
        max_depth = 0
        current_depth = 0

        for char in code:
            if char == "{":
                current_depth += 1
                max_depth = max(max_depth, current_depth)
            elif char == "}":
                current_depth -= 1

        return max_depth

    def _find_matching_brace(self, code: str, start_pos: int) -> int:
        """Find the matching closing brace for an opening brace"""
        brace_count = 0
        for i in range(start_pos, len(code)):
            if code[i] == "{":
                brace_count += 1
            elif code[i] == "}":
                brace_count -= 1
                if brace_count == 0:
                    return i
        return -1
