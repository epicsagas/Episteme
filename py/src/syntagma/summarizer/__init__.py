"""
Syntagma Summarizer

Token-efficient text summarization for code documentation and knowledge extraction.
"""

from syntagma.summarizer.token_efficient import (
    DetailLevel,
    optimize_response,
    select_detail_level,
    summarize_entity,
)

__all__ = [
    "DetailLevel",
    "summarize_entity",
    "optimize_response",
    "select_detail_level",
]
