#!/usr/bin/env python3
"""
Syntagma RAG Builder
Builds SQLite-vec embeddings with relational metadata for knowledge graph queries
"""

import json
import sqlite3
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Dict, List, Optional


@dataclass
class Chunk:
    """Text chunk with metadata for embedding"""

    id: str
    text: str
    file_path: str
    entity_id: str
    entity_type: str
    category: str
    title: str
    section: str
    chunk_index: int
    metadata: Dict


class SyntagmaRAG:
    """Builds and manages Syntagma knowledge graph embeddings"""

    def __init__(self, base_dir: str = "."):
        self.base_dir = Path(base_dir)
        self.raw_dir = self.base_dir / "raw"
        self.meta_dir = self.base_dir / "meta"
        self.db_path = self.meta_dir / "embeddings.db"

        # Load metadata
        self.relations = self._load_json("relations.json")
        self.taxonomy = self._load_json("taxonomy.json")
        self.schema = self._load_json("schema.json")

        # Initialize embedding model
        self.model = None  # Lazy load

    def _load_json(self, filename: str) -> Dict:
        """Load JSON metadata file"""
        path = self.meta_dir / filename
        if not path.exists():
            return {}
        with open(path, "r", encoding="utf-8") as f:
            return dict(json.load(f))

    def _save_json(self, filename: str, data: Dict):
        """Save JSON metadata file"""
        path = self.meta_dir / filename
        with open(path, "w", encoding="utf-8") as f:
            json.dump(data, f, indent=2, ensure_ascii=False)

    def chunk_markdown(self, file_path: Path, entity_id: str) -> List[Chunk]:
        """
        Split markdown file into semantic chunks

        Strategy:
        - Split by ## headers (sections)
        - Keep related content together
        - Max ~500 tokens per chunk
        """
        with open(file_path, "r", encoding="utf-8") as f:
            content = f.read()

        chunks = []
        current_section = "Overview"
        current_text: list[str] = []
        chunk_index = 0

        # Get entity metadata from relations
        entity = self.relations.get(entity_id, {})

        for line in content.split("\n"):
            # Detect section headers
            if line.startswith("## "):
                # Save previous chunk
                if current_text:
                    chunk_id = f"{entity_id}-C{chunk_index:03d}"
                    chunks.append(
                        Chunk(
                            id=chunk_id,
                            text="\n".join(current_text).strip(),
                            file_path=str(file_path.relative_to(self.raw_dir)),
                            entity_id=entity_id,
                            entity_type=entity.get("type", "unknown"),
                            category=entity.get("category", ""),
                            title=entity.get("title", ""),
                            section=current_section,
                            chunk_index=chunk_index,
                            metadata=self._build_metadata(entity_id),
                        )
                    )
                    chunk_index += 1
                    current_text = []

                # Start new section
                current_section = line.replace("## ", "").strip()

            current_text.append(line)

        # Save last chunk
        if current_text:
            chunk_id = f"{entity_id}-C{chunk_index:03d}"
            chunks.append(
                Chunk(
                    id=chunk_id,
                    text="\n".join(current_text).strip(),
                    file_path=str(file_path.relative_to(self.raw_dir)),
                    entity_id=entity_id,
                    entity_type=entity.get("type", "unknown"),
                    category=entity.get("category", ""),
                    title=entity.get("title", ""),
                    section=current_section,
                    chunk_index=chunk_index,
                    metadata=self._build_metadata(entity_id),
                )
            )

        return chunks

    def _build_metadata(self, entity_id: str) -> Dict:
        """Build comprehensive metadata for RAG retrieval"""
        entity = self.relations.get(entity_id, {})

        metadata = {
            "entity_id": entity_id,
            "type": entity.get("type", ""),
            "category": entity.get("category", ""),
            "tags": entity.get("tags", []),
        }

        # Add relational metadata
        relations = entity.get("relations", {})
        metadata.update(
            {
                "solves": relations.get("solves", []),
                "solved_by": relations.get("solved_by", []),
                "enforces": relations.get("enforces", []),
                "violates": relations.get("violates", []),
                "related_to": relations.get("related_to", []),
            }
        )

        # Add context
        context = entity.get("context", {})
        metadata.update(
            {
                "when_to_use": context.get("when_to_use", []),
                "symptoms": context.get("symptoms", []),
                "benefits": context.get("benefits", []),
                "drawbacks": context.get("drawbacks", []),
            }
        )

        return metadata

    def init_database(self):
        """Initialize SQLite database with vec extension"""
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()

        # Main embeddings table
        cursor.execute("""
            CREATE TABLE IF NOT EXISTS embeddings (
                id TEXT PRIMARY KEY,
                text TEXT NOT NULL,
                file_path TEXT NOT NULL,
                entity_id TEXT NOT NULL,
                entity_type TEXT NOT NULL,
                category TEXT,
                title TEXT,
                section TEXT,
                chunk_index INTEGER,
                metadata JSON,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
        """)

        # TODO: Add sqlite-vec virtual table when extension is available
        # cursor.execute("""
        #     CREATE VIRTUAL TABLE IF NOT EXISTS vec_embeddings USING vec0(
        #         id TEXT PRIMARY KEY,
        #         embedding FLOAT[384]
        #     )
        # """)

        # Indexes for fast filtering
        cursor.execute("CREATE INDEX IF NOT EXISTS idx_entity_id ON embeddings(entity_id)")
        cursor.execute("CREATE INDEX IF NOT EXISTS idx_entity_type ON embeddings(entity_type)")
        cursor.execute("CREATE INDEX IF NOT EXISTS idx_category ON embeddings(category)")

        conn.commit()
        conn.close()

        print(f"✅ Database initialized: {self.db_path}")

    def insert_chunks(self, chunks: List[Chunk]):
        """Insert chunks into database"""
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()

        for chunk in chunks:
            cursor.execute(
                """
                INSERT OR REPLACE INTO embeddings
                (id, text, file_path, entity_id, entity_type, category,
                 title, section, chunk_index, metadata)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            """,
                (
                    chunk.id,
                    chunk.text,
                    chunk.file_path,
                    chunk.entity_id,
                    chunk.entity_type,
                    chunk.category,
                    chunk.title,
                    chunk.section,
                    chunk.chunk_index,
                    json.dumps(chunk.metadata, ensure_ascii=False),
                ),
            )

        conn.commit()
        conn.close()

    def scan_and_index(self):
        """Scan all markdown files and build index"""
        print("🔍 Scanning raw directory...")

        # Skip README and Korean translations for now
        md_files = [
            f for f in self.raw_dir.rglob("*.md") if "README" not in f.name and "/ko/" not in str(f)
        ]

        print(f"📄 Found {len(md_files)} markdown files")

        total_chunks = 0

        for md_file in md_files:
            # Derive entity_id from file path
            # This is a placeholder - you'll need to map files to entity IDs
            rel_path = md_file.relative_to(self.raw_dir)
            entity_id = self._derive_entity_id(rel_path)

            if not entity_id:
                print(f"⚠️  Skipping {rel_path} (no entity mapping)")
                continue

            print(f"   Processing: {rel_path} → {entity_id}")
            chunks = self.chunk_markdown(md_file, entity_id)
            self.insert_chunks(chunks)
            total_chunks += len(chunks)

        print(f"✅ Indexed {total_chunks} chunks from {len(md_files)} files")

    def _derive_entity_id(self, file_path: Path) -> Optional[str]:
        """
        Derive entity ID from file path
        TODO: Implement proper mapping or read from relations.json
        """
        # This is a placeholder - implement proper mapping logic
        parts = file_path.parts

        if "design-patterns" in parts:
            # DP-001, DP-002, etc.
            return None  # TODO: Map pattern names to IDs
        elif "refactoring" in parts:
            # RF-001, RF-002, etc.
            return None  # TODO: Map refactoring names to IDs
        elif "software-engineering" in parts:
            # LAW-001, LAW-002, etc.
            return None  # TODO: Map law names to IDs

        return None

    def query(self, query_text: str, filters: Optional[Dict[Any, Any]] = None, limit: int = 5):
        """
        Query embeddings (placeholder for vector search)

        TODO: Implement with sqlite-vec when available
        """
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()

        # Fallback to full-text search for now
        cursor.execute(
            """
            SELECT id, text, entity_id, title, section, metadata
            FROM embeddings
            WHERE text LIKE ?
            LIMIT ?
        """,
            (f"%{query_text}%", limit),
        )

        results = cursor.fetchall()
        conn.close()

        return results

    def stats(self):
        """Print database statistics"""
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()

        cursor.execute("SELECT COUNT(*) FROM embeddings")
        total = cursor.fetchone()[0]

        cursor.execute("SELECT entity_type, COUNT(*) FROM embeddings GROUP BY entity_type")
        by_type = cursor.fetchall()

        conn.close()

        print("\n📊 Database Statistics:")
        print(f"   Total chunks: {total}")
        print("   By type:")
        for entity_type, count in by_type:
            print(f"      {entity_type}: {count}")


def main():
    """Build RAG database"""
    import argparse

    parser = argparse.ArgumentParser(description="Build Syntagma RAG database")
    parser.add_argument("--init", action="store_true", help="Initialize database")
    parser.add_argument("--scan", action="store_true", help="Scan and index files")
    parser.add_argument("--stats", action="store_true", help="Show statistics")
    parser.add_argument("--query", type=str, help="Test query")

    args = parser.parse_args()

    rag = SyntagmaRAG()

    if args.init:
        rag.init_database()

    if args.scan:
        rag.scan_and_index()

    if args.stats:
        rag.stats()

    if args.query:
        results = rag.query(args.query)
        print(f"\n🔍 Query: {args.query}")
        for r in results:
            print(f"   [{r[2]}] {r[3]} - {r[4]}")


if __name__ == "__main__":
    main()
