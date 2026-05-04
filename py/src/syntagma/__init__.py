"""
Syntagma - Multi-Language Knowledge Graph System for Code Smell Detection and Refactoring

A comprehensive system for analyzing code across multiple programming languages,
detecting code smells, and providing refactoring suggestions based on design patterns
and software engineering laws.
"""

__version__ = "0.0.5"
__author__ = "EpicSagas Research Team"

# Expose commonly used components at package level
from syntagma.config import (
    BASE_DIR,
    DB_PATH,
    ENTITY_TYPES,
    SIMILARITY_THRESHOLD,
)

__all__ = [
    "__version__",
    "__author__",
    "BASE_DIR",
    "DB_PATH",
    "ENTITY_TYPES",
    "SIMILARITY_THRESHOLD",
]
