#!/usr/bin/env python3
"""
Refactoring Inference Engine
Maps code smells to refactoring suggestions using knowledge graph
"""

import json
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Dict, List, Optional

from syntagma import config as _config
from syntagma.cli.analyze import CodeSmellDetector, SmellDetection
from syntagma.graph.api import KnowledgeGraph


@dataclass
class RefactoringSuggestion:
    """A refactoring suggestion with priority score"""

    refactoring_id: str
    title: str
    priority_score: float  # 0.0 - 1.0
    effort: str  # small/medium/large
    principles_enforced: List[str]
    description: str
    metadata: Dict


class RefactoringRanker:
    """Ranks refactoring suggestions based on multiple criteria"""

    # Effort weights (inverse - smaller effort = higher score)
    EFFORT_WEIGHTS = {"small": 1.0, "medium": 0.6, "large": 0.3}

    def __init__(self, graph: KnowledgeGraph):
        self.graph = graph

    def rank_refactorings(
        self, smell_detection: SmellDetection, refactoring_ids: List[str]
    ) -> List[RefactoringSuggestion]:
        """
        Rank refactorings based on multiple criteria

        Scoring formula:
        - Severity weight: 40% (smell confidence)
        - Effort inverse: 30% (prefer smaller effort)
        - Principle alignment: 20% (how many violated principles it fixes)
        - Usage frequency: 10% (popularity heuristic)
        """
        suggestions = []

        for rf_id in refactoring_ids:
            rf_entity = self.graph.get_entity(rf_id)
            if not rf_entity:
                continue

            # Calculate composite score
            severity_score = smell_detection.confidence  # 0.0 - 1.0
            effort_score = self._calculate_effort_score(rf_entity)
            principle_score = self._calculate_principle_alignment(smell_detection, rf_entity)
            usage_score = self._calculate_usage_frequency(rf_id)

            priority_score = (
                0.4 * severity_score
                + 0.3 * effort_score
                + 0.2 * principle_score
                + 0.1 * usage_score
            )

            # Get effort level
            effort = self._extract_effort(rf_entity)

            # Get enforced principles
            enforced = rf_entity.get("relations", {}).get("enforces", [])

            # Generate description
            description = self._generate_description(smell_detection, rf_entity, enforced)

            suggestions.append(
                RefactoringSuggestion(
                    refactoring_id=rf_id,
                    title=rf_entity.get("title", "Unknown"),
                    priority_score=priority_score,
                    effort=effort,
                    principles_enforced=enforced,
                    description=description,
                    metadata={
                        "severity_score": severity_score,
                        "effort_score": effort_score,
                        "principle_score": principle_score,
                        "usage_score": usage_score,
                    },
                )
            )

        # Sort by priority descending
        suggestions.sort(key=lambda x: x.priority_score, reverse=True)

        return suggestions

    def _calculate_effort_score(self, rf_entity: Dict) -> float:
        """Estimate effort score (inverse of effort)"""
        effort = self._extract_effort(rf_entity)
        return self.EFFORT_WEIGHTS.get(effort, 0.5)

    def _extract_effort(self, rf_entity: Dict) -> str:
        """Extract effort estimate from entity metadata"""
        # Check context for effort hints
        context = rf_entity.get("context", {})
        when_to_use = " ".join(context.get("when_to_use", []))
        benefits = " ".join(context.get("benefits", []))

        # Heuristics
        if "simple" in when_to_use.lower() or "quick" in benefits.lower():
            return "small"
        elif "complex" in when_to_use.lower() or "significant" in benefits.lower():
            return "large"
        else:
            return "medium"

    def _calculate_principle_alignment(
        self, smell_detection: SmellDetection, rf_entity: Dict
    ) -> float:
        """
        Calculate principle alignment score

        How many principles violated by the smell are enforced by the refactoring?
        """
        # Get smell definition
        smell_entity = self.graph.get_entity(smell_detection.smell_id)
        if not smell_entity:
            return 0.5

        violated_laws = set(smell_entity.get("relations", {}).get("violates", []))
        enforced_laws = set(rf_entity.get("relations", {}).get("enforces", []))

        if not violated_laws:
            return 0.5  # No violation info, neutral score

        # Calculate overlap
        overlap = len(violated_laws & enforced_laws)
        max_possible = len(violated_laws)

        return overlap / max_possible if max_possible > 0 else 0.0

    def _calculate_usage_frequency(self, rf_id: str) -> float:
        """
        Estimate usage frequency (popularity heuristic)

        More related entities = more popular
        """
        rf_entity = self.graph.get_entity(rf_id)
        if not rf_entity:
            return 0.0

        relations = rf_entity.get("relations", {})
        total_relations = sum(len(targets) for targets in relations.values())

        # Normalize to 0.0-1.0 (assume max 20 relations)
        return min(total_relations / 20.0, 1.0)

    def _generate_description(
        self, smell_detection: SmellDetection, rf_entity: Dict, enforced_laws: List[str]
    ) -> str:
        """Generate human-readable description"""
        rf_title = rf_entity.get("title", "Unknown")

        # Get principle names
        law_names = []
        for law_id in enforced_laws[:3]:  # Limit to 3
            law_entity = self.graph.get_entity(law_id)
            if law_entity:
                law_names.append(law_entity.get("title", law_id))

        principles_text = ", ".join(law_names) if law_names else "code quality"

        # Extract context benefits
        context = rf_entity.get("context", {})
        benefits = context.get("benefits", [])
        benefit_text = benefits[0] if benefits else "improve code structure"

        return (
            f"Apply {rf_title} to {benefit_text}. "
            f"This addresses {smell_detection.smell_name} and improves {principles_text}."
        )


class RefactoringInferenceEngine:
    """Main engine for code smell → refactoring inference"""

    def __init__(self, base_dir: str | None = None):
        self.base_dir = Path(base_dir) if base_dir else _config.SYNTAGMA_HOME
        self.detector = CodeSmellDetector(base_dir)
        self.graph = KnowledgeGraph(base_dir)
        self.ranker = RefactoringRanker(self.graph)

    def analyze_file(
        self, file_path: str, top_k: int = 3, language_hint: Optional[str] = None
    ) -> List[Dict]:
        """
        Analyze a source file or directory and suggest refactorings.

        Args:
            file_path: Path to a source file or directory
            top_k: Number of suggestions per smell
            language_hint: Optional language override (e.g. "go", "java")

        Returns:
            List of smell detections with ranked refactoring suggestions
        """
        from syntagma.cli.analyze import analyze_path

        # Step 1: Detect smells (multi-language)
        detections = analyze_path(Path(file_path), language_hint=language_hint)

        results = []

        for detection in detections:
            # Step 2: Find refactorings via graph traversal
            refactoring_ids = self._find_refactorings_for_smell(detection.smell_id)

            # Step 3: Rank refactorings
            suggestions = self.ranker.rank_refactorings(detection, refactoring_ids)

            results.append(
                {
                    "smell": asdict(detection),
                    "suggestions": [asdict(s) for s in suggestions[:top_k]],
                }
            )

        return results

    def _find_refactorings_for_smell(self, smell_id: str) -> List[str]:
        """Find refactorings that solve a code smell"""
        smell_entity = self.graph.get_entity(smell_id)
        if not smell_entity:
            return []

        relations = smell_entity.get("relations", {})
        return list(relations.get("solved_by", []))


def main(argv=None):
    """CLI for refactoring inference"""
    import argparse

    parser = argparse.ArgumentParser(description="Refactoring Inference Engine")
    parser.add_argument("file", help="Source file or directory to analyze")
    parser.add_argument("--top-k", type=int, default=3, help="Top K suggestions per smell")
    parser.add_argument("--json", action="store_true", help="Output JSON format")
    parser.add_argument("--language", help="Language hint (e.g. java, typescript, go)")

    args = parser.parse_args(argv)

    from syntagma.cli.analyze import analyze_path

    target = Path(args.file)
    engine = RefactoringInferenceEngine()
    detections = analyze_path(target, language_hint=args.language)

    results = []
    for detection in detections:
        from dataclasses import asdict

        refactoring_ids = engine._find_refactorings_for_smell(detection.smell_id)
        suggestions = engine.ranker.rank_refactorings(detection, refactoring_ids)
        results.append(
            {
                "smell": asdict(detection),
                "suggestions": [asdict(s) for s in suggestions[: args.top_k]],
            }
        )

    if args.json:
        print(json.dumps(results, indent=2))
    else:
        # Human-readable output
        if not results:
            print(f"✅ No code smells detected in {args.file}")
            return

        print(f"\n🔍 Refactoring Analysis for {args.file}\n")
        print("=" * 80)

        for i, result in enumerate(results, 1):
            smell = result["smell"]
            suggestions = result["suggestions"]

            print(f"\n{i}. ⚠️  {smell['smell_name']} (confidence: {smell['confidence']:.2f})")
            print(f"   Location: {smell['location']} ({smell['function_name']})")
            print(
                f"   Metrics: LOC={smell['metrics']['loc']}, "
                f"CC={smell['metrics']['cyclomatic_complexity']}, "
                f"Depth={smell['metrics']['nesting_depth']}, "
                f"Params={smell['metrics']['parameter_count']}"
            )
            print("\n   Reasons:")
            for reason in smell["reasons"]:
                print(f"     - {reason}")

            if not suggestions:
                print("\n   ℹ️  No refactoring suggestions available")
                continue

            print("\n   💡 Suggested Refactorings:")

            for j, suggestion in enumerate(suggestions, 1):
                print(f"\n   {j}. {suggestion['title']} ({suggestion['refactoring_id']})")
                print(
                    f"      Priority: {suggestion['priority_score']:.2f} "
                    f"(Effort: {suggestion['effort']})"
                )

                if suggestion["principles_enforced"]:
                    principles = ", ".join(suggestion["principles_enforced"])
                    print(f"      Enforces: {principles}")

                print(f"      → {suggestion['description']}")

            print()

        print("=" * 80)


if __name__ == "__main__":
    main()
