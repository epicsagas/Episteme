"""
Ruby code smell parser using regex-based analysis
"""

import re
from typing import List

from .base import (
    CodeMetrics,
    LanguageParser,
    SmellDetection,
)


class RubyParser(LanguageParser):
    """Ruby parser using regex patterns"""

    # Regex patterns for Ruby functions and classes
    METHOD_PATTERN = re.compile(
        r'def\s+(?:self\.)?(\w+)\s*(?:\([^)]*\))?',
        re.MULTILINE
    )

    CLASS_PATTERN = re.compile(
        r'class\s+(\w+)(?:\s*<\s*\w+)?',
        re.MULTILINE
    )

    def get_supported_extensions(self) -> List[str]:
        """Return supported file extensions"""
        return ['.rb']

    def parse_file(self, file_path: str) -> List[SmellDetection]:
        """Parse a Ruby file and detect code smells"""
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                source_code = f.read()
            return self.parse_code(source_code, file_path)
        except FileNotFoundError:
            return []
        except Exception:
            return []

    def parse_code(self, code: str, file_name: str = "temp.rb") -> List[SmellDetection]:
        """Parse Ruby code string and detect code smells"""
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
        # Remove multi-line comments (=begin...=end)
        code = re.sub(r'=begin.*?=end', '', code, flags=re.DOTALL | re.MULTILINE)
        # Remove single-line comments
        code = re.sub(r'#.*?$', '', code, flags=re.MULTILINE)
        return code

    def _parse_methods(self, code: str, file_name: str) -> List[SmellDetection]:
        """Parse and analyze methods"""
        detections = []

        for match in self.METHOD_PATTERN.finditer(code):
            method_name = match.group(1)
            start_pos = match.start()
            line_no = code[:start_pos].count('\n') + 1

            # Find method body (Ruby uses 'end' keyword instead of braces)
            body_start = code.find('\n', start_pos)
            if body_start == -1:
                continue

            # Find the corresponding 'end' keyword
            body_end = self._find_method_end(code, body_start)
            if body_end == -1:
                continue

            method_body = code[body_start:body_end]

            # Extract signature for parameter counting
            signature = match.group(0)

            # Calculate metrics
            metrics = self._calculate_method_metrics(signature, method_body)
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
            line_no = code[:start_pos].count('\n') + 1

            # Find class body
            body_start = code.find('\n', start_pos)
            if body_start == -1:
                continue

            # Find the corresponding 'end' keyword
            body_end = self._find_method_end(code, body_start)
            if body_end == -1:
                continue

            class_body = code[body_start:body_end]

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

    def _calculate_method_metrics(self, signature: str, body: str) -> CodeMetrics:
        """Calculate metrics for a method"""
        # Count lines (excluding empty lines)
        loc = len([line for line in body.split('\n') if line.strip()])

        # Count parameters from signature
        param_start = signature.find('(')
        param_end = signature.rfind(')')
        if param_start != -1 and param_end != -1:
            params_str = signature[param_start + 1:param_end].strip()
            if params_str:
                # Remove *args, **kwargs, &block syntax
                params_str = re.sub(r'[*&]+', '', params_str)
                param_count = len([p for p in params_str.split(',') if p.strip()])
            else:
                param_count = 0
        else:
            param_count = 0

        # Calculate cyclomatic complexity
        cc = self._calculate_cc(body)

        # Calculate nesting depth (Ruby often uses blocks)
        nesting = self._calculate_nesting(body)

        # Count local variables (var = ...)
        local_vars = len(re.findall(r'^\s*\w+\s*=', body, re.MULTILINE))

        # Count return statements (explicit returns)
        returns = len(re.findall(r'\breturn\b', body))

        return CodeMetrics(
            loc=loc,
            cyclomatic_complexity=cc,
            nesting_depth=nesting,
            parameter_count=param_count,
            local_variables=local_vars,
            return_statements=returns
        )

    def _calculate_class_metrics(self, class_body: str, method_count: int, property_count: int) -> CodeMetrics:
        """Calculate metrics for a class"""
        # Count lines
        loc = len([line for line in class_body.split('\n') if line.strip()])

        return CodeMetrics(
            loc=loc,
            cyclomatic_complexity=0,
            nesting_depth=0,
            parameter_count=0,
            local_variables=0,
            return_statements=0,
            method_count=method_count,
            field_count=property_count
        )

    def _count_class_methods(self, class_body: str) -> int:
        """Count methods in class"""
        method_pattern = re.compile(r'^\s*def\s+(?:self\.)?(\w+)', re.MULTILINE)
        return len(method_pattern.findall(class_body))

    def _count_class_properties(self, class_body: str) -> int:
        """Count properties/attributes in class"""
        property_count = 0
        # Count @instance_variable assignments
        for line in class_body.split('\n'):
            if re.search(r'@\w+\s*=', line):
                property_count += 1
        return property_count

    def _calculate_cc(self, code: str) -> int:
        """Calculate cyclomatic complexity"""
        cc = 1  # Base complexity

        # Count control flow keywords
        cc += len(re.findall(r'\bif\b', code))
        cc += len(re.findall(r'\bunless\b', code))  # Ruby's unless is like if not
        cc += len(re.findall(r'\belsif\b', code))
        cc += len(re.findall(r'\bfor\b', code))
        cc += len(re.findall(r'\beach\b', code))  # Ruby's each iterator
        cc += len(re.findall(r'\bwhile\b', code))
        cc += len(re.findall(r'\buntil\b', code))
        cc += len(re.findall(r'\bcase\b', code))
        cc += len(re.findall(r'\bwhen\b', code))
        cc += len(re.findall(r'\brescue\b', code))  # Exception handling
        cc += len(re.findall(r'\&\&', code))  # Logical AND
        cc += len(re.findall(r'\|\|', code))  # Logical OR
        cc += len(re.findall(r'\?.*?:', code))  # Ternary operator

        return cc

    def _calculate_nesting(self, code: str) -> int:
        """Calculate maximum nesting depth (Ruby uses do...end and {...})"""
        max_depth = 0
        current_depth = 0

        # Track both do...end blocks and braces
        i = 0
        while i < len(code):
            # Look for keywords that increase nesting
            if re.match(r'\b(def|class|if|unless|for|each|while|until|case|begin|do)\b', code[i:]):
                current_depth += 1
                max_depth = max(max_depth, current_depth)
            # Look for keywords that decrease nesting
            elif re.match(r'\b(end)\b', code[i:]):
                current_depth -= 1
            # Also track braces
            elif code[i] == '{':
                current_depth += 1
                max_depth = max(max_depth, current_depth)
            elif code[i] == '}':
                current_depth -= 1

            i += 1

        return max_depth

    def _find_method_end(self, code: str, start_pos: int) -> int:
        """Find the 'end' keyword that closes a method/class definition"""
        end_pattern = re.compile(r'\bend\b', re.MULTILINE)

        # Track nesting level
        nesting_level = 1
        current_pos = start_pos

        # Count 'def', 'class', 'if', 'while', etc. to track nesting
        structure_keywords = ['def', 'class', 'if', 'unless', 'for', 'each', 'while', 'until', 'case', 'begin', 'do']

        while current_pos < len(code):
            # Look for next 'end' or structure keyword
            remaining_code = code[current_pos:]

            # Find next 'end'
            end_match = end_pattern.search(remaining_code)
            end_pos = end_match.start() + current_pos if end_match else -1

            # Find next structure keyword
            next_struct_pos = len(code)
            for keyword in structure_keywords:
                pattern = re.compile(rf'\b{keyword}\b', re.MULTILINE)
                match = pattern.search(remaining_code)
                if match:
                    next_struct_pos = min(next_struct_pos, match.start() + current_pos)

            if end_pos == -1:
                return -1

            if end_pos < next_struct_pos:
                # We found an 'end' before the next structure keyword
                nesting_level -= 1
                if nesting_level == 0:
                    return end_pos
                current_pos = end_pos + 3  # Move past 'end'
            else:
                # We found a structure keyword
                nesting_level += 1
                current_pos = next_struct_pos + 1

        return -1
