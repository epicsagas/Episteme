"""
Syntagma RAG (Retrieval-Augmented Generation)

Hybrid search system combining semantic embeddings with keyword-based retrieval
for code smell detection and refactoring recommendations.
"""

from syntagma.rag.build_v2 import SyntagmaRAG
from syntagma.rag.hybrid import hybrid_search
from syntagma.rag.keyword import build_fts_index, keyword_search
from syntagma.rag.problem_mapper import suggest_search_approach

__all__ = [
    "hybrid_search",
    "keyword_search",
    "build_fts_index",
    "suggest_search_approach",
    "SyntagmaRAG",
]
