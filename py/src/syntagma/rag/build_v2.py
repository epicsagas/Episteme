#!/usr/bin/env python3
"""
Syntagma RAG Builder v2.0
Complete implementation with sentence-transformers embeddings
"""

import json
import os
import sqlite3
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, List, Optional

import numpy as np

from syntagma import config as _config
from syntagma.embeddings.client import get_client


@dataclass
class Chunk:
    """Text chunk with metadata"""

    id: str
    text: str
    entity_id: str
    entity_type: str
    title: str
    section: str
    chunk_index: int
    metadata: Dict


class SyntagmaRAG:
    """Complete RAG system with embeddings"""

    def __init__(self, base_dir: str | None = None):
        self.base_dir = Path(base_dir) if base_dir else _config.SYNTAGMA_HOME
        self.raw_dir = _config.RAW_DIR
        self.meta_dir = _config.DATA_DIR
        self.db_path = _config.DB_PATH

        # Load metadata
        with open(self.meta_dir / "relations.json", "r") as f:
            data = json.load(f)
            self.relations = {
                k: v for k, v in data.items() if k.startswith(("DP-", "RF-", "LAW-", "SMELL-"))
            }

        with open(self.meta_dir / "file_to_entity.json", "r") as f:
            self.file_to_entity = json.load(f)

        self.client = get_client()
        self.embedding_dim = self.client.embedding_dim

    def init_database(self):
        """Initialize SQLite database"""
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()

        # Main chunks table
        cursor.execute("""
            CREATE TABLE IF NOT EXISTS chunks (
                id TEXT PRIMARY KEY,
                text TEXT NOT NULL,
                entity_id TEXT NOT NULL,
                entity_type TEXT NOT NULL,
                title TEXT,
                section TEXT,
                chunk_index INTEGER,
                metadata JSON,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
        """)

        # Embeddings table (store as BLOB)
        cursor.execute("""
            CREATE TABLE IF NOT EXISTS embeddings (
                chunk_id TEXT PRIMARY KEY,
                embedding BLOB NOT NULL,
                FOREIGN KEY (chunk_id) REFERENCES chunks(id)
            )
        """)

        # Indexes
        cursor.execute("CREATE INDEX IF NOT EXISTS idx_entity_id ON chunks(entity_id)")
        cursor.execute("CREATE INDEX IF NOT EXISTS idx_entity_type ON chunks(entity_type)")

        conn.commit()
        conn.close()

        print(f"✅ Database initialized: {self.db_path}")

    def chunk_markdown(self, file_path: Path, entity_id: str) -> List[Chunk]:
        """Split markdown into semantic chunks"""
        with open(file_path, "r", encoding="utf-8") as f:
            content = f.read()

        entity = self.relations.get(entity_id, {})
        chunks = []
        current_section = "Overview"
        current_text: list[str] = []
        chunk_index = 0

        for line in content.split("\n"):
            if line.startswith("## "):
                # Save previous chunk
                if current_text:
                    text = "\n".join(current_text).strip()
                    if len(text) > 50:  # Skip very short chunks
                        chunks.append(
                            Chunk(
                                id=f"{entity_id}-C{chunk_index:03d}",
                                text=text,
                                entity_id=entity_id,
                                entity_type=entity.get("type", "unknown"),
                                title=entity.get("title", ""),
                                section=current_section,
                                chunk_index=chunk_index,
                                metadata=self._build_metadata(entity_id, entity),
                            )
                        )
                        chunk_index += 1
                    current_text = []

                current_section = line.replace("## ", "").strip()

            current_text.append(line)

        # Save last chunk
        if current_text:
            text = "\n".join(current_text).strip()
            if len(text) > 50:
                chunks.append(
                    Chunk(
                        id=f"{entity_id}-C{chunk_index:03d}",
                        text=text,
                        entity_id=entity_id,
                        entity_type=entity.get("type", "unknown"),
                        title=entity.get("title", ""),
                        section=current_section,
                        chunk_index=chunk_index,
                        metadata=self._build_metadata(entity_id, entity),
                    )
                )

        return chunks

    def _build_metadata(self, entity_id: str, entity: Dict) -> Dict:
        """Build metadata for retrieval"""
        return {
            "entity_id": entity_id,
            "type": entity.get("type", ""),
            "category": entity.get("category", ""),
            "tags": entity.get("tags", []),
            "solves": entity.get("relations", {}).get("solves", []),
            "solved_by": entity.get("relations", {}).get("solved_by", []),
            "enforces": entity.get("relations", {}).get("enforces", []),
            "violates": entity.get("relations", {}).get("violates", []),
            "related_to": entity.get("relations", {}).get("related_to", []),
            "source": entity.get("source"),
        }

    def scan_and_chunk(self) -> List[Chunk]:
        """Scan all files and create chunks"""
        print("🔍 Scanning raw directory...")

        all_chunks = []

        for file_path_str, entity_id in self.file_to_entity.items():
            file_path = self.raw_dir / file_path_str

            if not file_path.exists():
                continue

            if "README" in file_path.name or "/ko/" in str(file_path):
                continue

            print(f"   Processing: {file_path_str} → {entity_id}")

            chunks = self.chunk_markdown(file_path, entity_id)
            all_chunks.extend(chunks)

        print(f"✅ Created {len(all_chunks)} chunks from {len(self.file_to_entity)} entities")
        return all_chunks

    def insert_chunks(self, chunks: List[Chunk]):
        """Insert chunks into database"""
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()

        for chunk in chunks:
            cursor.execute(
                """
                INSERT OR REPLACE INTO chunks
                (id, text, entity_id, entity_type, title, section, chunk_index, metadata)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            """,
                (
                    chunk.id,
                    chunk.text,
                    chunk.entity_id,
                    chunk.entity_type,
                    chunk.title,
                    chunk.section,
                    chunk.chunk_index,
                    json.dumps(chunk.metadata, ensure_ascii=False),
                ),
            )

        conn.commit()
        conn.close()
        print(f"✅ Inserted {len(chunks)} chunks into database")

    def generate_embeddings(self, batch_size: int = 32):
        """Generate embeddings for all chunks"""
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()

        # Get chunks without embeddings
        cursor.execute("""
            SELECT id, text FROM chunks
            WHERE id NOT IN (SELECT chunk_id FROM embeddings)
        """)
        chunks = cursor.fetchall()

        if not chunks:
            print("✅ All chunks already have embeddings")
            conn.close()
            return

        print(f"🧮 Generating embeddings for {len(chunks)} chunks...")

        # Batch encode
        texts = [c[1] for c in chunks]
        embeddings = self.client.embed_batch(texts, batch_size=batch_size)

        # Insert embeddings
        for (chunk_id, _), embedding in zip(chunks, embeddings, strict=False):
            embedding_blob = embedding.astype(np.float32).tobytes()
            cursor.execute(
                """
                INSERT OR REPLACE INTO embeddings (chunk_id, embedding)
                VALUES (?, ?)
            """,
                (chunk_id, embedding_blob),
            )

        conn.commit()
        conn.close()

        print(f"✅ Generated {len(chunks)} embeddings")

    # ------------------------------------------------------------------
    # Re-ranking weights
    # ------------------------------------------------------------------

    # Sections that carry the core meaning of an entity get a boost.
    # Sections like "Implementation Details" or "Examples" are more
    # peripheral — they don't need a penalty, just no bonus.
    _SECTION_BOOST: dict[str, float] = {
        # Universal high-signal sections
        "intent": 1.15,
        "overview": 1.12,
        "when to use": 1.10,
        "motivation": 1.08,
        "definition": 1.08,
        "summary": 1.06,
        "description": 1.05,
        # Pattern-specific
        "applicability": 1.08,
        "structure": 1.04,
        # Law/principle-specific
        "implications": 1.06,
        "origin": 1.03,
    }

    # Query keywords that signal the user wants a specific entity type.
    # Gives a small lift to matching types when the intent is clear.
    _TYPE_QUERY_SIGNALS: dict[str, list[str]] = {
        "pattern": [
            "pattern",
            "design pattern",
            "gof",
            "factory",
            "singleton",
            "observer",
            "strategy",
            "decorator",
            "adapter",
            "facade",
            "proxy",
            "composite",
            "bridge",
            "flyweight",
            "template",
            "iterator",
            "mediator",
            "memento",
            "visitor",
            "command",
            "chain",
            "state",
            "abstract factory",
            "builder",
            "prototype",
        ],
        "refactoring": [
            "refactor",
            "refactoring",
            "extract",
            "inline",
            "move",
            "rename",
            "replace",
            "split",
            "encapsulate",
            "decompose",
            "consolidate",
            "pull up",
            "push down",
        ],
        "smell": [
            "smell",
            "code smell",
            "anti-pattern",
            "antipattern",
            "bloat",
            "long method",
            "large class",
            "duplicate",
            "coupling",
        ],
        "law": [
            "law",
            "principle",
            "rule",
            "theorem",
            "effect",
            "bias",
            "conway",
            "brooks",
            "solid",
            "dry",
            "kiss",
            "yagni",
            "cap",
            "amdahl",
            "dunning",
            "occam",
            "pareto",
        ],
    }
    _TYPE_SIGNAL_BOOST = 1.05  # modest — don't override semantic similarity

    def _section_boost(self, section: str) -> float:
        """Return boost multiplier for the given section name."""
        key = section.strip().lower()
        # Exact match first, then prefix match for compound section names
        if key in self._SECTION_BOOST:
            return self._SECTION_BOOST[key]
        for pattern, boost in self._SECTION_BOOST.items():
            if key.startswith(pattern):
                return boost
        return 1.0

    def _type_boost(self, entity_type: str, query_lower: str) -> float:
        """Return boost multiplier when the query signals a specific entity type."""
        signals = self._TYPE_QUERY_SIGNALS.get(entity_type, [])
        if any(sig in query_lower for sig in signals):
            return self._TYPE_SIGNAL_BOOST
        return 1.0

    def search(self, query: str, top_k: int = 5, filters: Optional[Dict] = None) -> List[Dict]:
        """Semantic search with section- and type-aware re-ranking."""
        query_embedding = self.client.embed(query)
        query_lower = query.lower()

        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()

        where_clauses = []
        params = []

        if filters:
            if "entity_type" in filters:
                where_clauses.append("c.entity_type = ?")
                params.append(filters["entity_type"])
            if "entity_id" in filters:
                where_clauses.append("c.entity_id = ?")
                params.append(filters["entity_id"])

        where_clause = " AND " + " AND ".join(where_clauses) if where_clauses else ""

        cursor.execute(
            f"""
            SELECT c.id, c.text, c.entity_id, c.entity_type, c.title, c.section, c.metadata, e.embedding
            FROM chunks c
            JOIN embeddings e ON c.id = e.chunk_id
            {where_clause}
            """,
            params,
        )

        results = []

        for row in cursor.fetchall():
            (
                chunk_id,
                text,
                entity_id,
                entity_type,
                title,
                section,
                metadata_json,
                embedding_blob,
            ) = row

            embedding = np.frombuffer(embedding_blob, dtype=np.float32)

            similarity = float(
                np.dot(query_embedding, embedding)
                / (np.linalg.norm(query_embedding) * np.linalg.norm(embedding))
            )

            score = (
                similarity
                * self._section_boost(section or "")
                * self._type_boost(entity_type or "", query_lower)
            )

            results.append(
                {
                    "chunk_id": chunk_id,
                    "text": text,
                    "entity_id": entity_id,
                    "entity_type": entity_type,
                    "title": title,
                    "section": section,
                    "metadata": json.loads(metadata_json),
                    "similarity": similarity,  # raw cosine — unchanged
                    "score": score,  # re-ranked score used for ordering
                }
            )

        conn.close()

        results.sort(key=lambda x: x["score"], reverse=True)
        return results[:top_k]

    def stats(self):
        """Print database statistics"""
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()

        cursor.execute("SELECT COUNT(*) FROM chunks")
        total_chunks = cursor.fetchone()[0]

        cursor.execute("SELECT COUNT(*) FROM embeddings")
        total_embeddings = cursor.fetchone()[0]

        cursor.execute("SELECT entity_type, COUNT(*) FROM chunks GROUP BY entity_type")
        by_type = cursor.fetchall()

        conn.close()

        print("\n📊 Database Statistics:")
        print(f"   Total chunks: {total_chunks}")
        print(f"   Total embeddings: {total_embeddings}")
        print("   By type:")
        for entity_type, count in by_type:
            print(f"      {entity_type}: {count}")


def main():
    """Main CLI"""
    import argparse

    parser = argparse.ArgumentParser(description="Syntagma RAG Builder v2")
    parser.add_argument("--init", action="store_true", help="Initialize database")
    parser.add_argument("--build", action="store_true", help="Build complete RAG (scan + embed)")
    parser.add_argument("--scan", action="store_true", help="Scan and chunk files")
    parser.add_argument("--embed", action="store_true", help="Generate embeddings")
    parser.add_argument("--search", type=str, help="Search query")
    parser.add_argument("--filter-type", type=str, help="Filter by entity type")
    parser.add_argument("--top-k", type=int, default=5, help="Number of results")
    parser.add_argument("--stats", action="store_true", help="Show statistics")

    args = parser.parse_args()

    rag = SyntagmaRAG()

    if args.init:
        rag.init_database()

    if args.build:
        print("\n🚀 Building complete RAG system...\n")
        rag.init_database()
        chunks = rag.scan_and_chunk()
        rag.insert_chunks(chunks)
        rag.generate_embeddings()

        print(" Building FTS5 keyword index...")
        try:
            # Add project root to path for module imports
            import sys

            sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
            from retrieval.keyword import build_fts_index

            build_fts_index()
        except ImportError as e:
            print(f"⚠️  FTS5 index build skipped: {e}")
            print("   (Semantic search via embeddings still available)")

        rag.stats()

    if args.scan:
        chunks = rag.scan_and_chunk()
        rag.insert_chunks(chunks)

    if args.embed:
        rag.generate_embeddings()

    if args.search:
        filters = {}
        if args.filter_type:
            filters["entity_type"] = args.filter_type

        print(f"\n🔍 Searching: {args.search}")
        if filters:
            print(f"   Filters: {filters}")

        results = rag.search(args.search, top_k=args.top_k, filters=filters)

        for i, result in enumerate(results, 1):
            print(f"\n{i}. [{result['entity_id']}] {result['title']} - {result['section']}")
            print(f"   Similarity: {result['similarity']:.4f}")
            print(f"   Type: {result['entity_type']}")
            print(f"   Text: {result['text'][:150]}...")

    if args.stats:
        rag.stats()


if __name__ == "__main__":
    main()
