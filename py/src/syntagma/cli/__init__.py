"""
Syntagma CLI Tools

Command-line interface tools for code analysis, exploration, and refactoring inference.
"""

from syntagma.cli.analyze import CodeSmellDetector, SmellDetection
from syntagma.cli.infer import RefactoringInferenceEngine, RefactoringSuggestion

__all__ = [
    "CodeSmellDetector",
    "SmellDetection",
    "RefactoringInferenceEngine",
    "RefactoringSuggestion",
]
