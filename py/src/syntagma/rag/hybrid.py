from __future__ import annotations

import threading
from pathlib import Path
from typing import Any

from syntagma import config
from syntagma.rag.keyword import keyword_search

_rag_instance = None
_rag_lock = threading.Lock()

_RRF_K = 20


def _get_rag() -> Any:
    global _rag_instance
    if _rag_instance is None:
        with _rag_lock:
            if _rag_instance is None:  # double-checked locking
                from syntagma.rag.build_v2 import SyntagmaRAG

                _rag_instance = SyntagmaRAG()
    return _rag_instance


def hybrid_search(
    query: str,
    limit: int = 5,
    db_path: Path | None = None,
    filters: dict | None = None,
    keyword_weight: float = 0.4,
    semantic_weight: float = 0.6,
) -> list[dict]:
    db_path = db_path or config.DB_PATH
    filters = filters or {}

    keyword_results: list[dict] = []
    semantic_results: list[dict] = []

    try:
        keyword_results = keyword_search(
            query,
            limit=limit * 2,
            db_path=db_path,
            entity_type=filters.get("entity_type"),
        )
    except Exception:
        keyword_results = []

    try:
        rag = _get_rag()
        semantic_results = rag.search(query, top_k=limit * 2, filters=filters)
    except Exception:
        semantic_results = []

    if not keyword_results and not semantic_results:
        return []

    if not keyword_results:
        return [{**r, "score": r.get("similarity", 0.0)} for r in semantic_results[:limit]]

    if not semantic_results:
        return [{**r, "score": abs(r.get("relevance_score", 0.0))} for r in keyword_results[:limit]]

    chunk_scores: dict[str, dict] = {}

    for rank, result in enumerate(keyword_results, start=1):
        cid = result["chunk_id"]
        chunk_scores[cid] = {
            **result,
            "keyword_rank": rank,
            "semantic_rank": None,
            "score": keyword_weight / (_RRF_K + rank),
        }

    for rank, result in enumerate(semantic_results, start=1):
        cid = result["chunk_id"]
        if cid in chunk_scores:
            chunk_scores[cid]["semantic_rank"] = rank
            chunk_scores[cid]["score"] += semantic_weight / (_RRF_K + rank)
        else:
            chunk_scores[cid] = {
                **result,
                "keyword_rank": None,
                "semantic_rank": rank,
                "score": semantic_weight / (_RRF_K + rank),
            }

    for entry in chunk_scores.values():
        entry.pop("relevance_score", None)
        entry.pop("similarity", None)

    ranked = sorted(chunk_scores.values(), key=lambda x: x["score"], reverse=True)
    return ranked[:limit]
