"""Token-efficient summarizer for Syntagma knowledge graph entities."""

from enum import Enum
from typing import Dict, List, Tuple

from syntagma.config import MAX_TOKENS_PER_RESPONSE


class DetailLevel(Enum):
    MINIMAL = 1  # ~50 tokens: id, title, type only
    SUMMARY = 2  # ~150 tokens: + category + context summary (1 line each)
    DETAILED = 3  # ~300 tokens: + full context + relations summary
    FULL = 4  # Everything: all context + full relations


def estimate_tokens(text: str) -> int:
    """Rough token count: 1 token per 4 characters."""
    return max(1, len(text) // 4)


def _build_minimal(entity: Dict) -> Dict:
    meta = entity.get("metadata", {})
    return {
        "id": entity.get("id") or entity.get("entity_id") or meta.get("entity_id", ""),
        "title": entity.get("title", ""),
        "type": entity.get("type") or entity.get("entity_type") or meta.get("type", ""),
        "category": entity.get("category") or meta.get("category", ""),
    }


def _build_summary(entity: Dict) -> Dict:
    result = _build_minimal(entity)
    ctx = entity.get("context", {})
    parts: list[str] = []
    for key in ("benefits", "when_to_use", "drawbacks"):
        items = ctx.get(key, [])
        if items:
            parts.append(f"{key}: {items[0]}")
    result["summary"] = "; ".join(parts)
    return result


def _truncate_context(ctx: Dict, limit: int = 2) -> Dict:
    return {k: v[:limit] for k, v in ctx.items()}


def summarize_entity(entity: Dict, detail_level: DetailLevel = DetailLevel.SUMMARY) -> Dict:
    """Return a representation of *entity* trimmed to *detail_level*."""
    match detail_level:
        case DetailLevel.MINIMAL:
            return _build_minimal(entity)
        case DetailLevel.SUMMARY:
            return _build_summary(entity)
        case DetailLevel.DETAILED:
            result = _build_summary(entity)
            result["context"] = _truncate_context(entity.get("context", {}))
            relations = entity.get("relations", {})
            result["relation_counts"] = {k: len(v) for k, v in relations.items()}
            return result
        case DetailLevel.FULL:
            return dict(entity)


OVERHEAD_PER_ENTITY = 20


def _estimate_entity_tokens(entity: Dict) -> int:
    import json

    return estimate_tokens(json.dumps(entity)) + OVERHEAD_PER_ENTITY


def optimize_response(
    entities: List[Dict],
    max_tokens: int = MAX_TOKENS_PER_RESPONSE,
) -> Tuple[List[Dict], int]:
    """Fit *entities* into a token budget, adjusting detail level per entity."""
    result: List[Dict] = []
    tokens_used = 0

    for entity in entities:
        remaining = max_tokens - tokens_used

        if remaining < 50:
            break
        elif remaining >= 300:
            level = DetailLevel.FULL
        elif remaining >= 200:
            level = DetailLevel.DETAILED
        elif remaining >= 100:
            level = DetailLevel.SUMMARY
        else:
            level = DetailLevel.MINIMAL

        summarized = summarize_entity(entity, level)
        cost = _estimate_entity_tokens(summarized)

        result.append(summarized)
        tokens_used += cost

    return result, tokens_used


_DEPTH_WORDS = frozenset(
    {
        "explain",
        "detail",
        "how",
        "why",
        "example",
        "implement",
    }
)


def select_detail_level(
    query: str | None = None,
    token_budget: int = MAX_TOKENS_PER_RESPONSE,
) -> DetailLevel:
    """Auto-select detail level based on query keywords and budget."""
    if token_budget < 100:
        return DetailLevel.MINIMAL
    if token_budget >= 300:
        return DetailLevel.FULL
    if query:
        lowered = query.lower()
        if any(word in lowered for word in _DEPTH_WORDS):
            return DetailLevel.DETAILED
    if token_budget >= 150:
        return DetailLevel.SUMMARY
    return DetailLevel.MINIMAL
