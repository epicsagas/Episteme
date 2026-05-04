#!/usr/bin/env python3
"""
Knowledge Graph API for Syntagma
Provides graph traversal, path finding, and relationship inference
"""

import json
from collections import deque
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, List, Optional, Set, Tuple

from syntagma import config as _config


@dataclass
class GraphNode:
    """Represents an entity in the knowledge graph"""

    id: str
    title: str
    type: str
    category: str
    relations: Dict[str, List[str]]
    context: Dict[str, List[str]]


@dataclass
class GraphEdge:
    """Represents a relationship between entities"""

    from_id: str
    to_id: str
    relation_type: str


class KnowledgeGraph:
    """Knowledge graph traversal and analysis engine"""

    def __init__(self, base_dir: str | None = None):
        self.base_dir = Path(base_dir) if base_dir else _config.SYNTAGMA_HOME
        self.meta_dir = _config.DATA_DIR

        # Load relations
        with open(self.meta_dir / "relations.json", "r") as f:
            data = json.load(f)
            self.entities = {
                k: v for k, v in data.items() if k.startswith(("DP-", "RF-", "LAW-", "SMELL-"))
            }

        # Build reverse index for fast lookups
        self._build_reverse_index()

    def _build_reverse_index(self):
        """Build reverse relationship index"""
        self.reverse_relations = {}

        for entity_id, entity in self.entities.items():
            relations = entity.get("relations", {})

            for rel_type, targets in relations.items():
                for target_id in targets:
                    if target_id not in self.reverse_relations:
                        self.reverse_relations[target_id] = {}

                    if rel_type not in self.reverse_relations[target_id]:
                        self.reverse_relations[target_id][rel_type] = []

                    self.reverse_relations[target_id][rel_type].append(entity_id)

    # ===== Basic Queries =====

    def get_entity(self, entity_id: str) -> Optional[Dict]:
        """Get entity by ID"""
        return self.entities.get(entity_id)

    def get_entities_batch(self, entity_ids: List[str]) -> Dict[str, Dict]:
        """Return a {id: entity} mapping for all requested IDs in one call."""
        return {eid: self.entities[eid] for eid in entity_ids if eid in self.entities}

    def get_neighbors(self, entity_id: str, relation_type: Optional[str] = None) -> List[str]:
        """
        Get neighbors of an entity

        Args:
            entity_id: Entity ID
            relation_type: Filter by relation type (optional)

        Returns:
            List of neighbor entity IDs
        """
        entity = self.entities.get(entity_id)
        if not entity:
            return []

        relations = entity.get("relations", {})

        if relation_type:
            return list(relations.get(relation_type, []))

        # Return all neighbors
        neighbors = []
        for targets in relations.values():
            neighbors.extend(targets)

        return list(set(neighbors))

    def get_all_edges(self, entity_id: str) -> List[GraphEdge]:
        """Get all outgoing edges from an entity"""
        entity = self.entities.get(entity_id)
        if not entity:
            return []

        edges = []
        relations = entity.get("relations", {})

        for rel_type, targets in relations.items():
            for target_id in targets:
                edges.append(GraphEdge(from_id=entity_id, to_id=target_id, relation_type=rel_type))

        return edges

    def get_neighborhood(self, entity_id: str) -> Optional[Dict]:
        """
        Get complete neighborhood of an entity

        Returns:
            {
                'entity': {...},
                'outgoing': {'relation_type': [entity_ids]},
                'incoming': {'relation_type': [entity_ids]}
            }
        """
        entity = self.entities.get(entity_id)
        if not entity:
            return None

        outgoing = entity.get("relations", {})
        incoming = self.reverse_relations.get(entity_id, {})

        return {"entity": entity, "outgoing": outgoing, "incoming": incoming}

    # ===== Multi-hop Queries =====

    def traverse_chain(self, start_id: str, path: List[str]) -> List[List[str]]:
        """
        Traverse a chain of relationships

        Args:
            start_id: Starting entity ID
            path: List of relation types to follow

        Returns:
            List of paths (each path is a list of entity IDs)

        Example:
            traverse_chain("SMELL-01", ["solved_by", "enforces"])
            → [["SMELL-01", "RF-001", "LAW-042-S"], ...]
        """
        if not path:
            return [[start_id]]

        current_paths = [[start_id]]

        for rel_type in path:
            next_paths = []

            for current_path in current_paths:
                current_id = current_path[-1]
                neighbors = self.get_neighbors(current_id, rel_type)

                for neighbor_id in neighbors:
                    next_paths.append(current_path + [neighbor_id])

            current_paths = next_paths

        return current_paths

    def find_shortest_path(
        self, from_id: str, to_id: str, max_depth: int = 5
    ) -> Optional[List[str]]:
        """
        Find shortest path between two entities (BFS)

        Args:
            from_id: Start entity ID
            to_id: Target entity ID
            max_depth: Maximum search depth

        Returns:
            List of entity IDs forming the path, or None if no path found
        """
        if from_id == to_id:
            return [from_id]

        if from_id not in self.entities or to_id not in self.entities:
            return None

        queue = deque([(from_id, [from_id])])
        visited = {from_id}

        while queue:
            current_id, path = queue.popleft()

            if len(path) > max_depth:
                continue

            neighbors = self.get_neighbors(current_id)

            for neighbor_id in neighbors:
                if neighbor_id == to_id:
                    return path + [neighbor_id]

                if neighbor_id not in visited:
                    visited.add(neighbor_id)
                    queue.append((neighbor_id, path + [neighbor_id]))

        return None

    def extract_subgraph(self, center_id: str, radius: int = 2) -> Tuple[Set[str], List[GraphEdge]]:
        """
        Extract subgraph within radius N hops from center

        Args:
            center_id: Center entity ID
            radius: Maximum distance from center

        Returns:
            (nodes, edges) tuple
        """
        if center_id not in self.entities:
            return (set(), [])

        nodes = {center_id}
        edges = []
        current_layer = {center_id}

        for _ in range(radius):
            next_layer = set()

            for node_id in current_layer:
                node_edges = self.get_all_edges(node_id)

                for edge in node_edges:
                    if edge.to_id in self.entities:
                        nodes.add(edge.to_id)
                        next_layer.add(edge.to_id)
                        edges.append(edge)

            current_layer = next_layer

        return (nodes, edges)

    # ===== Advanced Analysis =====

    def find_contradictions(self) -> List[Dict]:
        """
        Find entities that both enforce and violate the same principle

        Returns:
            List of contradictions with details
        """
        contradictions = []

        for entity_id, entity in self.entities.items():
            relations = entity.get("relations", {})
            enforces = set(relations.get("enforces", []))
            violates = set(relations.get("violates", []))

            conflicts = enforces & violates

            if conflicts:
                contradictions.append(
                    {
                        "entity_id": entity_id,
                        "title": entity.get("title", entity.get("name", "Unknown")),
                        "conflicts": list(conflicts),
                    }
                )

        return contradictions

    def infer_transitive_enforcements(self) -> List[Tuple[str, str, str]]:
        """
        Infer transitive enforcement relationships

        Logic: If RF solves SMELL, and SMELL violates LAW, then RF enforces LAW

        Returns:
            List of (rf_id, smell_id, law_id) tuples
        """
        inferred = []

        for rf_id, rf_entity in self.entities.items():
            if not rf_id.startswith("RF-"):
                continue

            relations = rf_entity.get("relations", {})
            solved_smells = relations.get("solves", [])

            for smell_id in solved_smells:
                smell_entity = self.entities.get(smell_id)
                if not smell_entity:
                    continue

                smell_relations = smell_entity.get("relations", {})
                violated_laws = smell_relations.get("violates", [])

                for law_id in violated_laws:
                    # Check if already explicitly enforced
                    if law_id not in relations.get("enforces", []):
                        inferred.append((rf_id, smell_id, law_id))

        return inferred

    def find_similar_entities(
        self, entity_id: str, threshold: float = 0.7
    ) -> List[Tuple[str, float]]:
        """
        Find similar entities based on shared relationships

        Args:
            entity_id: Entity to compare
            threshold: Minimum Jaccard similarity (0.0-1.0)

        Returns:
            List of (entity_id, similarity_score) tuples
        """
        entity = self.entities.get(entity_id)
        if not entity:
            return []

        # Get all outgoing edges as a set
        entity_edges = set()
        for rel_type, targets in entity.get("relations", {}).items():
            for target in targets:
                entity_edges.add(f"{rel_type}:{target}")

        similar = []

        for other_id, other_entity in self.entities.items():
            if other_id == entity_id:
                continue

            # Get other entity's edges
            other_edges = set()
            for rel_type, targets in other_entity.get("relations", {}).items():
                for target in targets:
                    other_edges.add(f"{rel_type}:{target}")

            # Calculate Jaccard similarity
            if not entity_edges and not other_edges:
                continue

            intersection = len(entity_edges & other_edges)
            union = len(entity_edges | other_edges)

            if union > 0:
                similarity = intersection / union

                if similarity >= threshold:
                    similar.append((other_id, similarity))

        # Sort by similarity descending
        similar.sort(key=lambda x: x[1], reverse=True)

        return similar

    # ===== Statistics =====

    def stats(self) -> Dict:
        """Get graph statistics"""
        total_entities = len(self.entities)

        by_type: Dict[str, int] = {}
        total_edges = 0

        for _entity_id, entity in self.entities.items():
            entity_type = entity.get("type", "unknown")
            by_type[entity_type] = by_type.get(entity_type, 0) + 1

            relations = entity.get("relations", {})
            for targets in relations.values():
                total_edges += len(targets)

        # Count entities with relations
        with_relations = sum(
            1 for e in self.entities.values() if any(e.get("relations", {}).values())
        )

        return {
            "total_entities": total_entities,
            "total_edges": total_edges,
            "by_type": by_type,
            "entities_with_relations": with_relations,
            "avg_edges_per_entity": total_edges / total_entities if total_entities > 0 else 0,
        }


def main():
    """Demo graph API"""
    import argparse

    parser = argparse.ArgumentParser(description="Syntagma Knowledge Graph API")
    parser.add_argument("--neighbors", type=str, help="Get neighbors of entity")
    parser.add_argument("--relation", type=str, help="Filter by relation type")
    parser.add_argument("--path", nargs=2, help="Find path between two entities")
    parser.add_argument("--subgraph", type=str, help="Extract subgraph around entity")
    parser.add_argument("--radius", type=int, default=2, help="Subgraph radius")
    parser.add_argument("--contradictions", action="store_true", help="Find contradictions")
    parser.add_argument("--infer", action="store_true", help="Infer transitive relations")
    parser.add_argument("--similar", type=str, help="Find similar entities")
    parser.add_argument("--stats", action="store_true", help="Show statistics")

    args = parser.parse_args()

    graph = KnowledgeGraph()

    if args.neighbors:
        neighbors = graph.get_neighbors(args.neighbors, args.relation)
        print(f"\nNeighbors of {args.neighbors}:")
        for n in neighbors:
            entity = graph.get_entity(n)
            title = entity.get("title", entity.get("name", "Unknown")) if entity else "Unknown"
            print(f"  {n}: {title}")

    if args.path:
        from_id, to_id = args.path
        path = graph.find_shortest_path(from_id, to_id)
        if path:
            print(f"\nShortest path from {from_id} to {to_id}:")
            print(" → ".join(path))
        else:
            print(f"\nNo path found between {from_id} and {to_id}")

    if args.subgraph:
        nodes, edges = graph.extract_subgraph(args.subgraph, args.radius)
        print(f"\nSubgraph around {args.subgraph} (radius {args.radius}):")
        print(f"  Nodes: {len(nodes)}")
        print(f"  Edges: {len(edges)}")
        for node_id in sorted(nodes):
            entity = graph.get_entity(node_id)
            title = entity.get("title", entity.get("name", "Unknown")) if entity else "Unknown"
            print(f"    {node_id}: {title}")

    if args.contradictions:
        contradictions = graph.find_contradictions()
        print(f"\nFound {len(contradictions)} contradictions:")
        for c in contradictions:
            print(f"  {c['entity_id']} ({c['title']}):")
            print(f"    Conflicts: {', '.join(c['conflicts'])}")

    if args.infer:
        inferred = graph.infer_transitive_enforcements()
        print(f"\nInferred {len(inferred)} transitive enforcements:")
        for rf_id, smell_id, law_id in inferred[:10]:
            print(f"  {rf_id} → {smell_id} → {law_id}")

    if args.similar:
        similar = graph.find_similar_entities(args.similar)
        print(f"\nSimilar entities to {args.similar}:")
        for entity_id, score in similar[:10]:
            entity = graph.get_entity(entity_id)
            title = entity.get("title", entity.get("name", "Unknown")) if entity else "Unknown"
            print(f"  {entity_id} ({title}): {score:.3f}")

    if args.stats:
        stats = graph.stats()
        print("\n📊 Graph Statistics:")
        print(f"   Total entities: {stats['total_entities']}")
        print(f"   Total edges: {stats['total_edges']}")
        print(f"   Entities with relations: {stats['entities_with_relations']}")
        print(f"   Avg edges per entity: {stats['avg_edges_per_entity']:.2f}")
        print("   By type:")
        for entity_type, count in stats["by_type"].items():
            print(f"      {entity_type}: {count}")


if __name__ == "__main__":
    main()
