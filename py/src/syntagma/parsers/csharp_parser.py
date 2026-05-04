"""
C# code smell parser using regex-based analysis
"""

import re
from typing import List

from .base import (
    CodeMetrics,
    LanguageParser,
    SmellDetection,
)


class CsharpParser(LanguageParser):
    """C# parser using regex patterns"""

    # Regex patterns for C# methods and classes
    METHOD_PATTERN = re.compile(
        r"(?:public|private|protected|internal)?\s*(?:static)?\s*(?:async)?\s*(?:override)?\s*"
        r"(?:[\w<>\[\],\s]+\s+)?(\w+)\s*\([^)]*\)\s*(?:=>\s*[^;]+;|\{)",
        re.MULTILINE,
    )

    CLASS_PATTERN = re.compile(
        r"(?:public|private|protected|internal)?\s*(?:partial)?\s*(?:abstract)?\s*(?:sealed)?\s*"
        r"class\s+(\w+)(?:\s*:\s*[\w<>,\s]+)?\s*\{",
        re.MULTILINE,
    )

    PROPERTY_PATTERN = re.compile(
        r"(?:public|private|protected|internal)?\s*(?:static)?\s*(?:virtual)?\s*(?:override)?\s*"
        r"[\w<>]+\s+(\w+)\s*\{\s*(?:get|set)",
        re.MULTILINE,
    )

    def get_supported_extensions(self) -> List[str]:
        """Return supported file extensions"""
        return [".cs"]

    def parse_file(self, file_path: str) -> List[SmellDetection]:
        """Parse a C# file and detect code smells"""
        try:
            with open(file_path, "r", encoding="utf-8") as f:
                source_code = f.read()
            return self.parse_code(source_code, file_path)
        except FileNotFoundError:
            return []
        except Exception:
            return []

    def parse_code(self, code: str, file_name: str = "temp.cs") -> List[SmellDetection]:
        """Parse C# code string and detect code smells"""
        detections = []

        # Remove comments to avoid false positives
        code_no_comments = self._remove_comments(code)

        # Parse methods
        detections.extend(self._parse_methods(code_no_comments, file_name))

        # Parse classes
        detections.extend(self._parse_classes(code_no_comments, file_name))

        return detections

    def _remove_comments(self, code: str) -> str:
        """Remove single-line and multi-line comments"""
        # Remove multi-line comments
        code = re.sub(r"/\*.*?\*/", "", code, flags=re.DOTALL)
        # Remove single-line comments
        code = re.sub(r"//.*?$", "", code, flags=re.MULTILINE)
        return code

    def _parse_methods(self, code: str, file_name: str) -> List[SmellDetection]:
        """Parse and analyze methods"""
        detections = []

        # Simplified method pattern for C#
        method_pattern = re.compile(
            r"(?:public|private|protected|internal)?\s*(?:static)?\s*(?:async)?\s*(?:override)?\s*"
            r"(?:[\w<>\[\],\s]+\s+)?(\w+)\s*\(([^)]*)\)\s*\{",
            re.MULTILINE,
        )

        for match in method_pattern.finditer(code):
            method_name = match.group(1)
            params_str = match.group(2)
            start_pos = match.start()
            line_no = code[:start_pos].count("\n") + 1

            # Find method body
            body_start = code.find("{", start_pos)
            if body_start == -1:
                continue

            body_end = self._find_matching_brace(code, body_start)
            if body_end == -1:
                continue

            method_body = code[body_start : body_end + 1]

            # Calculate metrics
            metrics = self._calculate_method_metrics(params_str, method_body)
            location = f"{file_name}:{line_no}"

            # Detect smells
            smell = self.detect_long_method(metrics, location, method_name)
            if smell:
                detections.append(smell)

            smell = self.detect_long_parameter_list(metrics, location, method_name)
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

            # Find class body
            body_start = code.find("{", start_pos)
            if body_start == -1:
                continue

            body_end = self._find_matching_brace(code, body_start)
            if body_end == -1:
                continue

            class_body = code[body_start : body_end + 1]

            # Count methods and properties
            method_count = self._count_class_methods(class_body)
            property_count = self._count_class_properties(class_body)
            # Calculate metrics
            metrics = self._calculate_class_metrics(class_body, method_count, property_count)
            location = f"{file_name}:{line_no}"

            # Detect smells
            smell = self.detect_large_class(metrics, location, class_name)
            if smell:
                detections.append(smell)

        return detections

    def _calculate_method_metrics(self, params_str: str, body: str) -> CodeMetrics:
        """Calculate metrics for a method"""
        # Count lines (excluding braces and empty lines)
        loc = len(
            [line for line in body.split("\n") if line.strip() and line.strip() not in ["{", "}"]]
        )

        # Count parameters
        if params_str.strip():
            param_count = len([p for p in params_str.split(",") if p.strip()])
        else:
            param_count = 0

        # Calculate cyclomatic complexity
        cc = self._calculate_cc(body)

        # Calculate nesting depth
        nesting = self._calculate_nesting(body)

        # Count local variables
        local_vars = len(re.findall(r"\b(?:int|string|bool|double|float|var|decimal)\s+\w+", body))

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

    def _calculate_class_metrics(
        self, class_body: str, method_count: int, property_count: int
    ) -> CodeMetrics:
        """Calculate metrics for a class"""
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
            field_count=property_count,  # Properties count as fields in C#
        )

    def _count_class_methods(self, class_body: str) -> int:
        """Count methods in class"""
        method_pattern = re.compile(
            r"(?:public|private|protected|internal)?\s*(?:static)?\s*(?:async)?\s*(?:override)?\s*"
            r"[\w<>]+\s+\w+\s*\([^)]*\)\s*\{",
            re.MULTILINE,
        )
        return len(method_pattern.findall(class_body))

    def _count_class_properties(self, class_body: str) -> int:
        """Count properties in class"""
        property_count = 0
        for line in class_body.split("\n"):
            line = line.strip()
            # Properties have get/set accessors
            if re.match(
                r"(?:public|private|protected|internal)?\s*(?:static)?\s*(?:virtual)?\s*(?:override)?\s*[\w<>]+\s+\w+\s*\{\s*(?:get|set)",
                line,
            ):
                property_count += 1
        return property_count

    def _calculate_cc(self, code: str) -> int:
        """Calculate cyclomatic complexity"""
        cc = 1  # Base complexity

        # Count control flow keywords
        cc += len(re.findall(r"\bif\b", code))
        cc += len(re.findall(r"\belse\s+if\b", code))
        cc += len(re.findall(r"\bfor\b", code))
        cc += len(re.findall(r"\bforeach\b", code))
        cc += len(re.findall(r"\bwhile\b", code))
        cc += len(re.findall(r"\bswitch\b", code))
        cc += len(re.findall(r"\bcase\b", code))
        cc += len(re.findall(r"\btry\b", code))
        cc += len(re.findall(r"\bcatch\b", code))
        cc += len(re.findall(r"\&\&", code))  # Logical AND
        cc += len(re.findall(r"\|\|", code))  # Logical OR
        # LINQ complexity
        cc += len(re.findall(r"\bfrom\b", code))
        cc += len(re.findall(r"\bwhere\b", code))
        cc += len(re.findall(r"\bselect\b", code))

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
        in_string = False
        in_char = False
        escape = False

        for i in range(start_pos, len(code)):
            char = code[i]

            # Handle string and char literals
            if char == '"' and not escape and not in_char:
                in_string = not in_string
            elif char == "'" and not escape and not in_string:
                in_char = not in_char
            elif char == "\\" and not escape:
                escape = True
                continue

            if not in_string and not in_char:
                if char == "{":
                    brace_count += 1
                elif char == "}":
                    brace_count -= 1
                    if brace_count == 0:
                        return i

            escape = False

        return -1
