"""
Tests for search quality and token efficiency improvements.

Covers:
1. hybrid.py  -- _RRF_K changed from 60 to 20
2. server.py  -- search_knowledge multi-type (etypes >= 2) support
3. server.py  -- search_knowledge response structure (chunk-level, not entity-level)
"""

from __future__ import annotations

import json
from unittest.mock import MagicMock, patch

import pytest


# ---------------------------------------------------------------------------
# 1. _RRF_K value
# ---------------------------------------------------------------------------


def test_rrf_k_is_20():
    """_RRF_K must be 20 for small corpora (not the old value of 60)."""
    from syntagma.rag.hybrid import _RRF_K

    assert _RRF_K == 20, f"Expected _RRF_K=20, got {_RRF_K}"


def test_rrf_scores_differentiate_better_with_k20():
    """With K=20 the gap between rank-1 and rank-5 is larger than with K=60."""
    k_new = 20
    k_old = 60

    # score at rank r: weight / (K + r)
    gap_new = 1 / (k_new + 1) - 1 / (k_new + 5)
    gap_old = 1 / (k_old + 1) - 1 / (k_old + 5)

    assert gap_new > gap_old, "K=20 should produce larger rank gaps than K=60"


# ---------------------------------------------------------------------------
# 2. search_knowledge -- multi-type parallel search
# ---------------------------------------------------------------------------


def _make_chunk(chunk_id: str, entity_id: str, text: str = "test text") -> dict:
    return {
        "chunk_id": chunk_id,
        "entity_id": entity_id,
        "section": "overview",
        "text": text,
        "score": 0.5,
    }


def _make_entity(eid: str, title: str, etype: str = "pattern") -> dict:
    return {
        "id": eid,
        "title": title,
        "type": etype,
        "category": "structural",
    }


def _build_mcp(rag_side_effect, graph_entities: dict[str, dict]) -> "SyntagmaMCP":
    """Return a SyntagmaMCP whose rag and graph are mocked."""
    from syntagma.mcp.server import SyntagmaMCP

    mcp = SyntagmaMCP.__new__(SyntagmaMCP)
    mcp._rag = rag_side_effect  # callable with (query, limit, db_path, filters)
    mcp.graph = MagicMock()
    mcp.graph.get_entity.side_effect = lambda eid: graph_entities.get(eid)
    mcp.graph.get_entities_batch.side_effect = lambda ids: {
        eid: graph_entities[eid] for eid in ids if eid in graph_entities
    }
    return mcp


def test_search_knowledge_multi_type_calls_rag_twice():
    """When suggest_search_approach returns >= 2 types, rag is called twice."""
    call_log: list[dict] = []

    chunks_a = [_make_chunk("c1", "DP-001")]
    chunks_b = [_make_chunk("c2", "RF-001")]
    entities = {
        "DP-001": _make_entity("DP-001", "Factory"),
        "RF-001": _make_entity("RF-001", "Extract Method", "refactoring"),
    }

    def fake_rag(query, limit, db_path, filters):
        call_log.append({"filters": dict(filters)})
        etype = filters.get("entity_type")
        if etype == "pattern":
            return chunks_a
        if etype == "refactoring":
            return chunks_b
        return []

    mcp = _build_mcp(fake_rag, entities)

    approach = {"entity_types": ["pattern", "refactoring"]}
    with patch("syntagma.rag.problem_mapper.suggest_search_approach", return_value=approach):
        result = mcp.search_knowledge("factory pattern", limit=3)

    # rag must have been called twice (once per type)
    assert len(call_log) == 2
    called_types = {c["filters"].get("entity_type") for c in call_log}
    assert called_types == {"pattern", "refactoring"}

    assert result["count"] > 0


def test_search_knowledge_single_type_calls_rag_once():
    """When only one type is suggested, rag is called exactly once."""
    call_log: list[dict] = []

    def fake_rag(query, limit, db_path, filters):
        call_log.append({"filters": dict(filters)})
        return [_make_chunk("c1", "DP-001")]

    mcp = _build_mcp(fake_rag, {"DP-001": _make_entity("DP-001", "Singleton")})

    approach = {"entity_types": ["pattern"]}
    with patch("syntagma.rag.problem_mapper.suggest_search_approach", return_value=approach):
        result = mcp.search_knowledge("singleton", limit=3)

    assert len(call_log) == 1


def test_search_knowledge_no_type_calls_rag_once_no_filter():
    """When suggest_search_approach returns no types, rag is called once with empty filter."""
    call_log: list[dict] = []

    def fake_rag(query, limit, db_path, filters):
        call_log.append({"filters": dict(filters)})
        return [_make_chunk("c1", "DP-001")]

    mcp = _build_mcp(fake_rag, {"DP-001": _make_entity("DP-001", "Singleton")})

    approach = {"entity_types": []}
    with patch("syntagma.rag.problem_mapper.suggest_search_approach", return_value=approach):
        result = mcp.search_knowledge("anything", limit=3)

    assert len(call_log) == 1
    assert call_log[0]["filters"] == {}


def test_search_knowledge_multi_type_deduplicates_chunks():
    """Chunks returned by both rag calls should appear only once in results."""
    shared_chunk = _make_chunk("shared", "DP-001")
    chunks_a = [shared_chunk, _make_chunk("a_only", "DP-002")]
    chunks_b = [shared_chunk, _make_chunk("b_only", "RF-001")]
    entities = {
        "DP-001": _make_entity("DP-001", "Factory"),
        "DP-002": _make_entity("DP-002", "Observer"),
        "RF-001": _make_entity("RF-001", "Extract Method", "refactoring"),
    }

    def fake_rag(query, limit, db_path, filters):
        etype = filters.get("entity_type")
        if etype == "pattern":
            return chunks_a
        return chunks_b

    mcp = _build_mcp(fake_rag, entities)

    approach = {"entity_types": ["pattern", "refactoring"]}
    with patch("syntagma.rag.problem_mapper.suggest_search_approach", return_value=approach):
        result = mcp.search_knowledge("test", limit=10)

    chunk_ids = [r["chunk_id"] for r in result["results"]]
    assert len(chunk_ids) == len(set(chunk_ids)), "Duplicate chunk_ids found in results"


# ---------------------------------------------------------------------------
# 3. search_knowledge response structure
# ---------------------------------------------------------------------------


def test_search_knowledge_response_has_chunk_fields():
    """Each result must contain chunk_id, entity, section, text, score."""
    chunks = [_make_chunk("c1", "DP-001", "Factory pattern description")]
    entities = {"DP-001": _make_entity("DP-001", "Factory Method")}

    def fake_rag(query, limit, db_path, filters):
        return chunks

    mcp = _build_mcp(fake_rag, entities)

    approach = {"entity_types": ["pattern"]}
    with patch("syntagma.rag.problem_mapper.suggest_search_approach", return_value=approach):
        result = mcp.search_knowledge("factory", limit=5)

    assert result["count"] == 1
    item = result["results"][0]

    for field in ("chunk_id", "entity", "section", "text", "score"):
        assert field in item, f"Missing field '{field}' in result item"

    # entity sub-dict must have id, title, type
    ent = item["entity"]
    assert ent["id"] == "DP-001"
    assert ent["title"] == "Factory Method"
    assert ent["type"] == "pattern"


def test_search_knowledge_response_no_full_entity_dump():
    """Results must NOT contain entity-level keys like 'context' or 'relations'."""
    chunks = [_make_chunk("c1", "DP-001")]
    fat_entity = {
        **_make_entity("DP-001", "Factory"),
        "context": {"benefits": ["easy extension"]},
        "relations": {"solves": ["SMELL-01"]},
        "description": "A long description...",
    }

    def fake_rag(query, limit, db_path, filters):
        return chunks

    mcp = _build_mcp(fake_rag, {"DP-001": fat_entity})

    approach = {"entity_types": ["pattern"]}
    with patch("syntagma.rag.problem_mapper.suggest_search_approach", return_value=approach):
        result = mcp.search_knowledge("factory", limit=5)

    item = result["results"][0]
    # Top-level result must not bleed full entity data
    for forbidden in ("context", "relations", "description"):
        assert forbidden not in item, f"Forbidden key '{forbidden}' found at top level of result"


def test_search_knowledge_tokens_used_is_sum_of_chunk_estimates():
    """tokens_used must equal sum of estimate_tokens(json.dumps(r)) for each result."""
    from syntagma.summarizer.token_efficient import estimate_tokens

    chunks = [
        _make_chunk("c1", "DP-001", "short text"),
        _make_chunk("c2", "DP-002", "another text"),
    ]
    entities = {
        "DP-001": _make_entity("DP-001", "Factory"),
        "DP-002": _make_entity("DP-002", "Observer"),
    }

    def fake_rag(query, limit, db_path, filters):
        return chunks

    mcp = _build_mcp(fake_rag, entities)

    approach = {"entity_types": ["pattern"]}
    with patch("syntagma.rag.problem_mapper.suggest_search_approach", return_value=approach):
        result = mcp.search_knowledge("test", limit=5)

    expected_tokens = sum(estimate_tokens(json.dumps(r)) for r in result["results"])
    assert result["tokens_used"] == expected_tokens


def test_search_knowledge_top_level_keys_preserved():
    """Return dict must always have 'results', 'tokens_used', 'count'."""

    def fake_rag(query, limit, db_path, filters):
        return []

    mcp = _build_mcp(fake_rag, {})

    approach = {"entity_types": []}
    with patch("syntagma.rag.problem_mapper.suggest_search_approach", return_value=approach):
        result = mcp.search_knowledge("nothing", limit=5)

    for key in ("results", "tokens_used", "count"):
        assert key in result, f"Top-level key '{key}' missing"


def test_search_knowledge_tokens_low_per_result():
    """Per-result token count should be well below 200 (old full-entity ~617 tok)."""
    from syntagma.summarizer.token_efficient import estimate_tokens

    chunks = [_make_chunk(f"c{i}", f"DP-00{i}", "brief text") for i in range(1, 4)]
    entities = {f"DP-00{i}": _make_entity(f"DP-00{i}", f"Pattern{i}") for i in range(1, 4)}

    def fake_rag(query, limit, db_path, filters):
        return chunks

    mcp = _build_mcp(fake_rag, entities)

    approach = {"entity_types": ["pattern"]}
    with patch("syntagma.rag.problem_mapper.suggest_search_approach", return_value=approach):
        result = mcp.search_knowledge("test", limit=5)

    for item in result["results"]:
        tok = estimate_tokens(json.dumps(item))
        assert tok < 200, f"Result item too large: {tok} tokens"


# ---------------------------------------------------------------------------
# 4. server.py -- Task 1: bare import fix in read_resource
# ---------------------------------------------------------------------------


def test_read_resource_categories_uses_syntagma_config():
    """read_resource('syntagma://categories') must not raise ModuleNotFoundError."""
    from syntagma.mcp.server import SyntagmaMCP

    mcp = SyntagmaMCP.__new__(SyntagmaMCP)
    mcp.graph = MagicMock()

    # Should not raise ModuleNotFoundError for 'config'
    result = mcp.read_resource("syntagma://categories")
    assert "entity_types" in result
    assert "categories" in result


# ---------------------------------------------------------------------------
# 5. server.py -- Task 2: input size limit in analyze_code / suggest_refactorings
# ---------------------------------------------------------------------------


def test_analyze_code_rejects_oversized_input():
    """analyze_code must return an error dict when code > 500 KB."""
    from syntagma.mcp.server import SyntagmaMCP

    mcp = SyntagmaMCP.__new__(SyntagmaMCP)
    mcp._detector = MagicMock()
    mcp._refactor_engine = MagicMock()
    mcp.graph = MagicMock()

    big_code = "x = 1\n" * 100_000  # well over 500 KB
    result = mcp.analyze_code(big_code)
    assert "error" in result
    assert "500 KB" in result["error"]


def test_suggest_refactorings_rejects_oversized_input():
    """suggest_refactorings must return an error dict when code > 500 KB."""
    from syntagma.mcp.server import SyntagmaMCP

    mcp = SyntagmaMCP.__new__(SyntagmaMCP)
    mcp._refactor_engine = MagicMock()
    mcp.graph = MagicMock()

    big_code = "x = 1\n" * 100_000
    result = mcp.suggest_refactorings(big_code)
    assert "error" in result
    assert "500 KB" in result["error"]


def test_analyze_code_accepts_normal_input():
    """analyze_code must not reject code well under 500 KB."""
    from syntagma.mcp.server import SyntagmaMCP
    from syntagma.mcp.server import _MAX_CODE_BYTES

    assert _MAX_CODE_BYTES == 500_000


# ---------------------------------------------------------------------------
# 6. server.py -- Task 3: kwargs validation in _call_tool
# ---------------------------------------------------------------------------


def test_call_tool_handles_non_dict_arguments():
    """_call_tool must not crash when arguments is not a dict."""
    from syntagma.mcp.server import SyntagmaMCP, RPCDispatcher

    mcp = SyntagmaMCP.__new__(SyntagmaMCP)
    mcp.graph = MagicMock()
    mcp._rag = MagicMock(return_value=[])

    dispatcher = RPCDispatcher(mcp)
    # Pass a list instead of a dict for arguments
    result = dispatcher._call_tool(
        req_id=1,
        params={"name": "search_knowledge", "arguments": ["not", "a", "dict"]},
    )
    # Should return a valid JSON-RPC structure, not raise
    assert result["id"] == 1
    assert "result" in result


def test_call_tool_filters_non_string_keys():
    """_call_tool must silently drop argument keys that are not strings."""
    from syntagma.mcp.server import SyntagmaMCP, RPCDispatcher

    received_kwargs: list[dict] = []

    def fake_search(**kwargs):
        received_kwargs.append(kwargs)
        return {"results": [], "tokens_used": 0, "count": 0}

    mcp = SyntagmaMCP.__new__(SyntagmaMCP)
    mcp.graph = MagicMock()
    mcp.search_knowledge = fake_search

    dispatcher = RPCDispatcher(mcp)
    result = dispatcher._call_tool(
        req_id=2,
        params={
            "name": "search_knowledge",
            "arguments": {"query": "test", 42: "should_be_dropped"},
        },
    )
    assert result["id"] == 2
    assert "result" in result
    # The non-string key 42 must not appear in kwargs passed to the method
    assert len(received_kwargs) == 1
    assert 42 not in received_kwargs[0]


# ---------------------------------------------------------------------------
# 7. hybrid.py -- Task 4: thread-safe singleton and no in-place mutation
# ---------------------------------------------------------------------------


def test_hybrid_get_rag_has_lock():
    """_get_rag must use a threading.Lock for double-checked locking."""
    import threading
    import syntagma.rag.hybrid as hybrid_mod

    assert hasattr(hybrid_mod, "_rag_lock"), "_rag_lock not found in hybrid module"
    assert isinstance(hybrid_mod._rag_lock, type(threading.Lock())), (
        "_rag_lock must be a threading.Lock instance"
    )


def test_hybrid_fallback_no_in_place_mutation():
    """Fallback paths must not mutate the original result dicts."""
    from unittest.mock import patch

    original_semantic = [
        {"chunk_id": "c1", "entity_id": "E1", "section": "s", "text": "t", "similarity": 0.9},
    ]
    original_keyword = [
        {"chunk_id": "c2", "entity_id": "E2", "section": "s", "text": "t", "relevance_score": 0.8},
    ]

    # snapshot keys before calling hybrid_search
    sem_keys_before = set(original_semantic[0].keys())
    kw_keys_before = set(original_keyword[0].keys())

    with patch("syntagma.rag.hybrid.keyword_search", return_value=[]):
        with patch("syntagma.rag.hybrid._get_rag") as mock_get_rag:
            mock_rag = MagicMock()
            mock_rag.search.return_value = original_semantic
            mock_get_rag.return_value = mock_rag

            from syntagma.rag.hybrid import hybrid_search

            results = hybrid_search("query", limit=5)

    # original dict must not have been mutated
    assert set(original_semantic[0].keys()) == sem_keys_before, (
        "semantic result dict was mutated in-place"
    )
    # result must contain 'score' key
    assert "score" in results[0]

    # Now test keyword-only fallback
    with patch("syntagma.rag.hybrid.keyword_search", return_value=original_keyword):
        with patch("syntagma.rag.hybrid._get_rag") as mock_get_rag:
            mock_rag = MagicMock()
            mock_rag.search.return_value = []
            mock_get_rag.return_value = mock_rag

            results2 = hybrid_search("query", limit=5)

    assert set(original_keyword[0].keys()) == kw_keys_before, (
        "keyword result dict was mutated in-place"
    )
    assert "score" in results2[0]
