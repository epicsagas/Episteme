#!/usr/bin/env python3
"""
Interactive CLI for Knowledge Graph Exploration
Navigate relationships, find paths, and explore subgraphs
"""

from typing import Optional

from syntagma.graph.api import KnowledgeGraph


class GraphExplorer:
    """Interactive knowledge graph explorer"""

    def __init__(self, base_dir: str = "."):
        self.graph = KnowledgeGraph(base_dir)
        self.current_entity_id: Optional[str] = None
        self.history: list[str] = []

    def start(self, initial_entity: Optional[str] = None):
        """Start interactive session"""
        print("\n🌐 Syntagma Knowledge Graph Explorer")
        print("=" * 60)

        if initial_entity:
            self.navigate_to(initial_entity)
        else:
            self.show_help()

        # Interactive loop
        while True:
            try:
                command = input("\n> ").strip()

                if not command:
                    continue

                if command.lower() in ["q", "quit", "exit"]:
                    print("\nGoodbye! 👋")
                    break

                self.execute_command(command)

            except KeyboardInterrupt:
                print("\n\nGoodbye! 👋")
                break
            except Exception as e:
                print(f"❌ Error: {e}")

    def execute_command(self, command: str):
        """Execute a user command"""
        parts = command.split()
        cmd = parts[0].lower()

        if cmd in ["h", "help", "?"]:
            self.show_help()

        elif cmd in ["s", "stats"]:
            self.show_stats()

        elif cmd in ["n", "neighbors"]:
            if len(parts) > 1:
                relation_type = parts[1]
                self.show_neighbors(relation_type)
            else:
                self.show_neighbors()

        elif cmd in ["e", "enforces"]:
            self.show_neighbors("enforces")

        elif cmd in ["v", "violates"]:
            self.show_neighbors("violates")

        elif cmd in ["r", "related"]:
            self.show_neighbors("related_to")

        elif cmd in ["so", "solves"]:
            self.show_neighbors("solves")

        elif cmd in ["sb", "solved_by"]:
            self.show_neighbors("solved_by")

        elif cmd in ["p", "path"]:
            if len(parts) >= 2:
                if len(parts) == 2:
                    # Path from current to target
                    target = parts[1]
                    if self.current_entity_id is not None:
                        self.find_path(self.current_entity_id, target)
                    else:
                        print("No current entity. Navigate to an entity first.")
                else:
                    # Path from source to target
                    source = parts[1]
                    target = parts[2]
                    self.find_path(source, target)
            else:
                print("Usage: path <target> or path <source> <target>")

        elif cmd in ["g", "subgraph"]:
            radius = int(parts[1]) if len(parts) > 1 else 2
            self.show_subgraph(radius)

        elif cmd in ["b", "back"]:
            self.go_back()

        elif cmd in ["c", "contradictions"]:
            self.show_contradictions()

        elif cmd in ["i", "infer"]:
            self.show_inferred_relations()

        elif cmd in ["sim", "similar"]:
            threshold = float(parts[1]) if len(parts) > 1 else 0.5
            self.show_similar(threshold)

        elif cmd in ["go", "goto", "nav", "navigate"]:
            if len(parts) > 1:
                entity_id = parts[1]
                self.navigate_to(entity_id)
            else:
                print("Usage: goto <entity_id>")

        elif cmd in ["search", "find"]:
            if len(parts) > 1:
                query = " ".join(parts[1:])
                self.search_entities(query)
            else:
                print("Usage: search <query>")

        else:
            # Try to navigate to entity
            if cmd.startswith(("DP-", "RF-", "LAW-", "SMELL-")):
                self.navigate_to(cmd.upper())
            else:
                print(f"Unknown command: {cmd}. Type 'help' for commands.")

    def navigate_to(self, entity_id: str):
        """Navigate to an entity"""
        entity = self.graph.get_entity(entity_id)

        if not entity:
            print(f"❌ Entity not found: {entity_id}")
            return

        # Save to history
        if self.current_entity_id:
            self.history.append(self.current_entity_id)

        self.current_entity_id = entity_id

        # Display entity
        self.show_current_entity()

    def show_current_entity(self):
        """Display current entity details"""
        if not self.current_entity_id:
            print("❌ No entity selected")
            return

        entity = self.graph.get_entity(self.current_entity_id)
        if not entity:
            return

        print(f"\n{'=' * 60}")
        print(f"📍 {self.current_entity_id}: {entity.get('title', entity.get('name', 'Unknown'))}")
        print(
            f"   Type: {entity.get('type', 'unknown')} | Category: {entity.get('category', 'N/A')}"
        )
        print(f"{'=' * 60}")

        # Show relationships
        relations = entity.get("relations", {})

        if relations.get("solves"):
            print("\n✅ Solves:")
            for smell_id in relations["solves"]:
                smell = self.graph.get_entity(smell_id)
                name = smell.get("name", "Unknown") if smell else "Unknown"
                print(f"   - {smell_id}: {name}")

        if relations.get("solved_by"):
            print("\n🔧 Solved By:")
            for rf_id in relations["solved_by"]:
                rf = self.graph.get_entity(rf_id)
                title = rf.get("title", "Unknown") if rf else "Unknown"
                print(f"   - {rf_id}: {title}")

        if relations.get("enforces"):
            print("\n✅ Enforces:")
            for law_id in relations["enforces"]:
                law = self.graph.get_entity(law_id)
                title = law.get("title", "Unknown") if law else "Unknown"
                print(f"   - {law_id}: {title}")

        if relations.get("violates"):
            print("\n⚠️  Violates:")
            for law_id in relations["violates"]:
                law = self.graph.get_entity(law_id)
                title = law.get("title", "Unknown") if law else "Unknown"
                print(f"   - {law_id}: {title}")

        if relations.get("related_to"):
            print("\n🔗 Related To:")
            for related_id in relations["related_to"][:5]:  # Limit to 5
                related = self.graph.get_entity(related_id)
                title = (
                    related.get("title", related.get("name", "Unknown")) if related else "Unknown"
                )
                print(f"   - {related_id}: {title}")

            if len(relations["related_to"]) > 5:
                print(f"   ... and {len(relations['related_to']) - 5} more")

    def show_neighbors(self, relation_type: Optional[str] = None):
        """Show neighbors of current entity"""
        if not self.current_entity_id:
            print("❌ No entity selected")
            return

        neighbors = self.graph.get_neighbors(self.current_entity_id, relation_type)

        if not neighbors:
            rel_text = f" (relation: {relation_type})" if relation_type else ""
            print(f"ℹ️  No neighbors found{rel_text}")
            return

        rel_text = f" via '{relation_type}'" if relation_type else ""
        print(f"\n📋 Neighbors{rel_text}:")

        for i, neighbor_id in enumerate(neighbors, 1):
            neighbor = self.graph.get_entity(neighbor_id)
            title = (
                neighbor.get("title", neighbor.get("name", "Unknown")) if neighbor else "Unknown"
            )
            print(f"{i}. {neighbor_id}: {title}")

        # Prompt to navigate
        print("\nType 'goto <entity_id>' or just '<entity_id>' to navigate")

    def find_path(self, from_id: str, to_id: str):
        """Find shortest path between entities"""
        path = self.graph.find_shortest_path(from_id, to_id)

        if not path:
            print(f"❌ No path found between {from_id} and {to_id}")
            return

        print(f"\n🛤️  Shortest path ({len(path) - 1} hops):")

        for i, entity_id in enumerate(path):
            entity = self.graph.get_entity(entity_id)
            title = entity.get("title", entity.get("name", "Unknown")) if entity else "Unknown"

            if i == 0:
                print(f"  {entity_id}: {title}")
            else:
                print("  ↓")
                print(f"  {entity_id}: {title}")

    def show_subgraph(self, radius: int = 2):
        """Show subgraph around current entity"""
        if not self.current_entity_id:
            print("❌ No entity selected")
            return

        nodes, edges = self.graph.extract_subgraph(self.current_entity_id, radius)

        print(f"\n🕸️  Subgraph (radius {radius}):")
        print(f"   Nodes: {len(nodes)}")
        print(f"   Edges: {len(edges)}")

        print("\n   Entities:")
        for node_id in sorted(nodes)[:20]:  # Limit to 20
            entity = self.graph.get_entity(node_id)
            title = entity.get("title", entity.get("name", "Unknown")) if entity else "Unknown"
            marker = "📍" if node_id == self.current_entity_id else "  "
            print(f"   {marker} {node_id}: {title}")

        if len(nodes) > 20:
            print(f"   ... and {len(nodes) - 20} more")

    def go_back(self):
        """Go back to previous entity"""
        if not self.history:
            print("ℹ️  No history to go back to")
            return

        entity_id = self.history.pop()
        self.current_entity_id = entity_id
        self.show_current_entity()

    def show_contradictions(self):
        """Show contradictions in the graph"""
        contradictions = self.graph.find_contradictions()

        if not contradictions:
            print("✅ No contradictions found")
            return

        print(f"\n⚠️  Found {len(contradictions)} contradictions:")

        for c in contradictions:
            print(f"\n  {c['entity_id']} ({c['title']}):")
            for conflict_id in c["conflicts"]:
                conflict = self.graph.get_entity(conflict_id)
                title = conflict.get("title", "Unknown") if conflict else "Unknown"
                print(f"    - {conflict_id}: {title}")

    def show_inferred_relations(self):
        """Show inferred transitive relations"""
        inferred = self.graph.infer_transitive_enforcements()

        if not inferred:
            print("ℹ️  No new relations to infer")
            return

        print(f"\n🔮 Inferred {len(inferred)} transitive enforcements:")

        for rf_id, smell_id, law_id in inferred[:10]:
            rf = self.graph.get_entity(rf_id)
            smell = self.graph.get_entity(smell_id)
            law = self.graph.get_entity(law_id)

            rf_title = rf.get("title", "Unknown") if rf else "Unknown"
            smell_name = smell.get("name", "Unknown") if smell else "Unknown"
            law_title = law.get("title", "Unknown") if law else "Unknown"

            print(f"  {rf_id} ({rf_title})")
            print(f"    → solves {smell_id} ({smell_name})")
            print(f"    → enforces {law_id} ({law_title})")

        if len(inferred) > 10:
            print(f"  ... and {len(inferred) - 10} more")

    def show_similar(self, threshold: float = 0.5):
        """Show similar entities to current"""
        if not self.current_entity_id:
            print("❌ No entity selected")
            return

        similar = self.graph.find_similar_entities(self.current_entity_id, threshold)

        if not similar:
            print(f"ℹ️  No similar entities found (threshold: {threshold})")
            return

        print(f"\n🔍 Similar entities (threshold: {threshold}):")

        for entity_id, score in similar[:10]:
            entity = self.graph.get_entity(entity_id)
            title = entity.get("title", entity.get("name", "Unknown")) if entity else "Unknown"
            print(f"  {entity_id} ({title}): {score:.3f}")

    def search_entities(self, query: str):
        """Search entities by title/name"""
        query_lower = query.lower()
        matches = []

        for entity_id, entity in self.graph.entities.items():
            title = entity.get("title", entity.get("name", "")).lower()
            if query_lower in title or query_lower in entity_id.lower():
                matches.append((entity_id, entity))

        if not matches:
            print(f"❌ No matches found for: {query}")
            return

        print(f"\n🔎 Found {len(matches)} matches:")

        for entity_id, entity in matches[:20]:
            title = entity.get("title", entity.get("name", "Unknown"))
            entity_type = entity.get("type", "unknown")
            print(f"  {entity_id} ({entity_type}): {title}")

        if len(matches) > 20:
            print(f"  ... and {len(matches) - 20} more")

    def show_stats(self):
        """Show graph statistics"""
        stats = self.graph.stats()

        print("\n📊 Graph Statistics:")
        print(f"   Total entities: {stats['total_entities']}")
        print(f"   Total edges: {stats['total_edges']}")
        print(f"   Entities with relations: {stats['entities_with_relations']}")
        print(f"   Avg edges per entity: {stats['avg_edges_per_entity']:.2f}")
        print("\n   By type:")
        for entity_type, count in stats["by_type"].items():
            print(f"      {entity_type}: {count}")

    def show_help(self):
        """Show help message"""
        print("""
Commands:
  <entity_id>          Navigate to entity (e.g., DP-005, RF-001)
  goto <entity_id>     Navigate to entity
  search <query>       Search entities by name

  n, neighbors [type]  Show neighbors (optionally filter by relation type)
  e, enforces          Show enforces relations
  v, violates          Show violates relations
  r, related           Show related_to relations
  so, solves           Show solves relations
  sb, solved_by        Show solved_by relations

  p, path <target>     Find shortest path from current to target
  path <src> <dst>     Find shortest path between two entities
  g, subgraph [N]      Show subgraph (default radius: 2)

  c, contradictions    Find entities with conflicting relations
  i, infer             Show inferred transitive relations
  sim, similar [N]     Find similar entities (default threshold: 0.5)

  b, back              Go back to previous entity
  s, stats             Show graph statistics
  h, help              Show this help
  q, quit              Exit
""")


def main():
    """CLI entry point"""
    import argparse

    parser = argparse.ArgumentParser(description="Interactive Knowledge Graph Explorer")
    parser.add_argument("entity", nargs="?", help="Initial entity to explore")
    parser.add_argument("--base-dir", default=".", help="Base directory (default: current)")

    args = parser.parse_args()

    explorer = GraphExplorer(args.base_dir)
    explorer.start(args.entity)


if __name__ == "__main__":
    main()
