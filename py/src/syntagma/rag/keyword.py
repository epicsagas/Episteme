import json
import re
import sqlite3
from pathlib import Path

from syntagma import config


def _sanitize_fts_query(query: str) -> str:
    cleaned = query.replace('"', '').replace("'", '').replace('*', '')
    cleaned = re.sub(r'[^\w\s]', ' ', cleaned)
    tokens = cleaned.split()
    return ' '.join(f'"{t}"' for t in tokens if t)


def build_fts_index(db_path: Path | None = None) -> None:
    db_path = db_path or config.DB_PATH
    conn = sqlite3.connect(db_path)
    cursor = conn.cursor()

    cursor.execute("DROP TABLE IF EXISTS chunks_fts")
    cursor.execute("""
        CREATE VIRTUAL TABLE chunks_fts USING fts5(
            text,
            title,
            section,
            content='chunks',
            content_rowid='rowid'
        )
    """)

    cursor.execute("""
        INSERT INTO chunks_fts(rowid, text, title, section)
        SELECT rowid, text, COALESCE(title, ''), COALESCE(section, '')
        FROM chunks
    """)

    conn.commit()
    conn.close()


def keyword_search(
    query: str,
    limit: int = 5,
    db_path: Path | None = None,
    entity_type: str | None = None,
) -> list[dict]:
    db_path = db_path or config.DB_PATH
    fts_query = _sanitize_fts_query(query)

    conn = sqlite3.connect(db_path)
    conn.row_factory = sqlite3.Row
    cursor = conn.cursor()

    if entity_type:
        cursor.execute(
            """
            SELECT
                c.id AS chunk_id,
                c.text,
                c.entity_id,
                c.entity_type,
                c.title,
                c.section,
                c.metadata,
                rank AS relevance_score
            FROM chunks_fts f
            JOIN chunks c ON c.rowid = f.rowid
            WHERE chunks_fts MATCH ? AND c.entity_type = ?
            ORDER BY rank
            LIMIT ?
            """,
            (fts_query, entity_type, limit),
        )
    else:
        cursor.execute(
            """
            SELECT
                c.id AS chunk_id,
                c.text,
                c.entity_id,
                c.entity_type,
                c.title,
                c.section,
                c.metadata,
                rank AS relevance_score
            FROM chunks_fts f
            JOIN chunks c ON c.rowid = f.rowid
            WHERE chunks_fts MATCH ?
            ORDER BY rank
            LIMIT ?
            """,
            (fts_query, limit),
        )

    rows = cursor.fetchall()
    conn.close()

    results = []
    for row in rows:
        results.append({
            'chunk_id': row['chunk_id'],
            'text': row['text'],
            'entity_id': row['entity_id'],
            'entity_type': row['entity_type'],
            'title': row['title'],
            'section': row['section'],
            'metadata': json.loads(row['metadata']),
            'relevance_score': row['relevance_score'],
        })

    return results
