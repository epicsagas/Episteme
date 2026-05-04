"""
Go code smell parser using regex-based analysis
"""

import re
from typing import List

from .base import (
    CodeMetrics,
    LanguageParser,
    SmellDetection,
)


class GoParser(LanguageParser):
    """Go parser using regex patterns"""

    # Regex patterns
    FUNCTION_PATTERN = re.compile(
        r'func\s+(?:\([^)]*\)\s*)?(\w+)\s*\([^)]*\)',
        re.MULTILINE
    )
    STRUCT_PATTERN = re.compile(
        r'type\s+(\w+)\s+struct\s*\{',
        re.MULTILINE
    )
    METHOD_PATTERN = re.compile(
        r'func\s+\([^)]*\)\s*(\w+)\s*\([^)]*\)',
        re.MULTILINE
    )
    FIELD_PATTERN = re.compile(
        r'^\s*(\w+)\s+[\w\[\]\*\.]+',
        re.MULTILINE
    )

    def get_supported_extensions(self) -> List[str]:
        """Return supported file extensions"""
        return ['.go']

    def parse_file(self, file_path: str) -> List[SmellDetection]:
        """Parse a Go file and detect code smells"""
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                source_code = f.read()
            return self.parse_code(source_code, file_path)
        except FileNotFoundError:
            return []
        except Exception:
            return []

    def parse_code(self, code: str, file_name: str = "temp.go") -> List[SmellDetection]:
        """Parse Go code string and detect code smells"""
        detections = []

        # Remove comments to avoid false positives
        code_no_comments = self._remove_comments(code)

        # Parse functions
        detections.extend(self._parse_functions(code_no_comments, file_name))

        # Parse structs (similar to classes)
        detections.extend(self._parse_structs(code_no_comments, file_name))

        return detections

    def _remove_comments(self, code: str) -> str:
        """Remove single-line and multi-line comments"""
        # Remove multi-line comments
        code = re.sub(r'/\*.*?\*/', '', code, flags=re.DOTALL)
        # Remove single-line comments
        code = re.sub(r'//.*?$', '', code, flags=re.MULTILINE)
        return code

    def _parse_functions(self, code: str, file_name: str) -> List[SmellDetection]:
        """Parse and analyze functions"""
        detections = []

        for match in self.FUNCTION_PATTERN.finditer(code):
            func_name = match.group(1)
            start_pos = match.start()
            line_no = code[:start_pos].count('\n') + 1

            # Extract function signature for parameter counting
            signature = match.group(0)

            # Find function body
            body_start = code.find('{', start_pos)
            if body_start == -1:
                continue

            body_end = self._find_matching_brace(code, body_start)
            if body_end == -1:
                continue

            func_body = code[body_start:body_end + 1]

            # Calculate metrics
            metrics = self._calculate_function_metrics(signature, func_body)
            location = f"{file_name}:{line_no}"

            # Detect smells
            smell = self.detect_long_method(metrics, location, func_name)
            if smell:
                detections.append(smell)

            smell = self.detect_long_parameter_list(metrics, location, func_name)
            if smell:
                detections.append(smell)

        return detections

    def _parse_structs(self, code: str, file_name: str) -> List[SmellDetection]:
        """Parse and analyze structs (Go's equivalent of classes)"""
        detections = []

        for match in self.STRUCT_PATTERN.finditer(code):
            struct_name = match.group(1)
            start_pos = match.start()
            line_no = code[:start_pos].count('\n') + 1

            # Find struct body
            body_start = code.find('{', start_pos)
            if body_start == -1:
                continue

            body_end = self._find_matching_brace(code, body_start)
            if body_end == -1:
                continue

            struct_body = code[body_start:body_end + 1]

            # Find methods associated with this struct
            method_count = self._count_struct_methods(code, struct_name)

            # Calculate metrics
            metrics = self._calculate_struct_metrics(struct_body, method_count)
            location = f"{file_name}:{line_no}"

            # Detect smells
            smell = self.detect_large_class(metrics, location, struct_name)
            if smell:
                detections.append(smell)



        return detections

    def _calculate_function_metrics(self, signature: str, body: str) -> CodeMetrics:
        """Calculate metrics for a function"""
        # Count lines (excluding braces and empty lines)
        loc = len([line for line in body.split('\n') if line.strip() and line.strip() not in ['{', '}']])

        # Count parameters from signature
        # Extract parameter list from func signature
        param_start = signature.find('(')
        param_end = signature.rfind(')')
        if param_start != -1 and param_end != -1:
            params_str = signature[param_start + 1:param_end].strip()
            if params_str:
                # Split by comma, but be careful with types containing commas
                param_count = len([p for p in params_str.split(',') if p.strip()])
            else:
                param_count = 0
        else:
            param_count = 0

        # Calculate cyclomatic complexity
        cc = self._calculate_cc(body)

        # Calculate nesting depth
        nesting = self._calculate_nesting(body)

        # Count local variables (var keyword)
        local_vars = len(re.findall(r'\bvar\s+\w+', body))
        # Also count := declarations
        local_vars += len(re.findall(r'\w+\s*:=', body))

        # Count return statements
        returns = len(re.findall(r'\breturn\b', body))

        return CodeMetrics(
            loc=loc,
            cyclomatic_complexity=cc,
            nesting_depth=nesting,
            parameter_count=param_count,
            local_variables=local_vars,
            return_statements=returns
        )

    def _calculate_struct_metrics(self, struct_body: str, method_count: int) -> CodeMetrics:
        """Calculate metrics for a struct"""
        # Count fields
        field_count = 0
        for line in struct_body.split('\n'):
            line = line.strip()
            if line and line not in ['{', '}'] and not line.startswith('//'):
                # Simple heuristic: lines that look like field declarations
                if re.match(r'\w+\s+[\w\[\]\*\.]+', line):
                    field_count += 1

        # Count lines
        loc = len([line for line in struct_body.split('\n') if line.strip() and line.strip() not in ['{', '}']])

        return CodeMetrics(
            loc=loc,
            cyclomatic_complexity=0,
            nesting_depth=0,
            parameter_count=0,
            local_variables=0,
            return_statements=0,
            method_count=method_count,
            field_count=field_count
        )

    def _count_struct_methods(self, code: str, struct_name: str) -> int:
        """Count methods associated with a struct"""
        # Pattern: func (receiver Type) MethodName
        pattern = re.compile(
            rf'func\s+\([^)]*\s+\*?{struct_name}\)\s+\w+',
            re.MULTILINE
        )
        return len(pattern.findall(code))

    def _calculate_cc(self, code: str) -> int:
        """Calculate cyclomatic complexity"""
        cc = 1  # Base complexity

        # Count control flow keywords
        cc += len(re.findall(r'\bif\b', code))
        cc += len(re.findall(r'\bfor\b', code))
        cc += len(re.findall(r'\bswitch\b', code))
        cc += len(re.findall(r'\bcase\b', code))
        cc += len(re.findall(r'\bselect\b', code))
        cc += len(re.findall(r'\&\&', code))  # Logical AND
        cc += len(re.findall(r'\|\|', code))  # Logical OR

        return cc

    def _calculate_nesting(self, code: str) -> int:
        """Calculate maximum nesting depth"""
        max_depth = 0
        current_depth = 0

        for char in code:
            if char == '{':
                current_depth += 1
                max_depth = max(max_depth, current_depth)
            elif char == '}':
                current_depth -= 1

        return max_depth

    def _find_matching_brace(self, code: str, start_pos: int) -> int:
        """Find the matching closing brace for an opening brace"""
        brace_count = 0
        in_string = False
        escape = False

        for i in range(start_pos, len(code)):
            char = code[i]

            # Handle string literals
            if char == '"' and not escape:
                in_string = not in_string
            elif char == '\\' and not escape:
                escape = True
                continue

            if not in_string:
                if char == '{':
                    brace_count += 1
                elif char == '}':
                    brace_count -= 1
                    if brace_count == 0:
                        return i

            escape = False

        return -1
