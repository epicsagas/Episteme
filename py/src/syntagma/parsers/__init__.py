"""
Multi-Language Parser Framework
Supports Python, Java, TypeScript, Go, Rust, C++, C#, PHP, Ruby, Kotlin
"""

from .base import CodeMetrics, LanguageParser, SmellDetection
from .cpp_parser import CppParser
from .csharp_parser import CsharpParser
from .go_parser import GoParser
from .java_parser import JavaParser
from .kotlin_parser import KotlinParser
from .php_parser import PhpParser
from .python_parser import PythonParser
from .ruby_parser import RubyParser
from .rust_parser import RustParser
from .typescript_parser import TypeScriptParser

__all__ = [
    "LanguageParser",
    "CodeMetrics",
    "SmellDetection",
    "PythonParser",
    "JavaParser",
    "TypeScriptParser",
    "GoParser",
    "RustParser",
    "CppParser",
    "CsharpParser",
    "PhpParser",
    "RubyParser",
    "KotlinParser",
]


def get_parser(language: str) -> LanguageParser:
    """Factory function to get appropriate parser"""
    parsers = {
        "python": PythonParser,
        "java": JavaParser,
        "typescript": TypeScriptParser,
        "javascript": TypeScriptParser,  # TypeScript parser handles JS too
        "go": GoParser,
        "rust": RustParser,
        "c++": CppParser,
        "cpp": CppParser,
        "c#": CsharpParser,
        "csharp": CsharpParser,
        "php": PhpParser,
        "ruby": RubyParser,
        "kotlin": KotlinParser,
    }

    parser_class = parsers.get(language.lower())
    if not parser_class:
        raise ValueError(f"Unsupported language: {language}")

    return parser_class()  # type: ignore[abstract]
