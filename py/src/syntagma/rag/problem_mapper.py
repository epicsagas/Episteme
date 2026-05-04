"""Route natural language problem descriptions to search strategies."""

from __future__ import annotations

from typing import Dict, List, Tuple

PROBLEM_KEYWORDS: Dict[str, List[str]] = {
    "teams": [
        "team", "organization", "communication", "coordination",
        "collaboration", "silos", "structure", "hierarchy", "roles",
        "responsibilities", "staffing", "hiring", "onboarding",
    ],
    "planning": [
        "timeline", "schedule", "estimate", "planning", "deadline",
        "milestone", "sprint", "backlog", "prioritization", "velocity",
        "burndown", "risk", "resource",
    ],
    "architecture": [
        "architecture", "design", "system", "microservice", "module",
        "component", "interface", "api", "abstraction", "coupling",
        "cohesion", "pattern", "framework", "infrastructure", "monolith",
    ],
    "quality": [
        "test", "testing", "quality", "bug", "defect", "error",
        "coverage", "review", "qa", "stability", "reliability",
        "debugging", "technical debt", "code smell", "refactor",
    ],
    "scalability": [
        "scale", "performance", "optimization", "throughput", "latency",
        "load", "capacity", "bottleneck", "cache", "database",
        "distributed", "concurrent", "parallel",
    ],
    "design": [
        "design", "principle", "pattern", "abstraction", "separation",
        "consistency", "simplicity", "readability", "clean code",
        "solid", "dry", "kiss",
    ],
    "decisions": [
        "decision", "choice", "alternative", "tradeoff", "trade-off",
        "versus", "vs", "evaluation", "comparison", "pros", "cons",
        "criteria", "methodology",
    ],
}

ENTITY_TYPE_KEYWORDS: Dict[str, List[str]] = {
    "pattern": [
        "pattern", "design pattern", "goth", "singleton", "factory",
        "observer", "strategy", "adapter", "decorator", "proxy", "bridge",
    ],
    "refactoring": [
        "refactor", "refactoring", "extract", "inline", "move", "rename",
        "replace", "simplify", "decompose", "restructure",
    ],
    "law": [
        "law", "principle", "rule", "theorem", "effect", "theory",
        "conway", "brooks", "solid", "dry", "kiss", "yagni",
    ],
    "smell": [
        "smell", "code smell", "antipattern", "anti-pattern", "bad code",
        "messy", "spaghetti", "technical debt",
    ],
}


def _score(problem: str, keyword_map: Dict[str, List[str]]) -> List[Tuple[str, float]]:
    """Score each key in keyword_map by keyword match density against the problem text."""
    text = problem.lower()
    results: List[Tuple[str, float]] = []
    for key, keywords in keyword_map.items():
        matches = sum(1 for kw in keywords if kw in text)
        if matches:
            score = min(1.0, matches / max(len(keywords), 5))
            results.append((key, score))
    results.sort(key=lambda t: t[1], reverse=True)
    return results


def map_problem_to_categories(problem: str, limit: int = 3) -> List[Tuple[str, float]]:
    """Return top matching categories scored by keyword overlap."""
    return _score(problem, PROBLEM_KEYWORDS)[:limit]


def map_problem_to_entity_types(problem: str) -> List[Tuple[str, float]]:
    """Return all entity types with non-zero keyword scores, sorted descending."""
    return _score(problem, ENTITY_TYPE_KEYWORDS)


def suggest_search_approach(problem: str) -> Dict:
    """Select category, hybrid, or semantic search strategy based on keyword confidence."""
    cat_scores = map_problem_to_categories(problem, limit=2)
    etype_scores = map_problem_to_entity_types(problem)
    etypes = [t for t, _ in etype_scores]

    if cat_scores and cat_scores[0][1] > 0.7:
        return {
            "strategy": "category",
            "category": cat_scores[0][0],
            "entity_types": etypes,
            "confidence": cat_scores[0][1],
        }

    if cat_scores and cat_scores[0][1] > 0.3:
        return {
            "strategy": "hybrid",
            "categories": [c for c, _ in cat_scores],
            "entity_types": etypes,
            "confidence": cat_scores[0][1],
        }

    return {
        "strategy": "semantic",
        "categories": [],
        "entity_types": [],
        "confidence": 0.0,
    }
