# Guards against: Middle Man FP.
# A context manager that wraps connection lifecycle is a standard Python
# protocol — the delegation is the protocol contract, not a smell.

from __future__ import annotations

import sqlite3
from types import TracebackType
from typing import Any, Optional, Sequence


class DatabaseConnection:
    """Context manager wrapping an SQLite connection.

    Manages the connection lifecycle (open/commit/close) and delegates
    query execution to the underlying sqlite3.Connection. The thin
    delegation is intentional — this class exists to ensure transactions
    are committed on success and rolled back on error.
    """

    def __init__(self, db_path: str) -> None:
        self._db_path = db_path
        self._conn: Optional[sqlite3.Connection] = None

    def __enter__(self) -> DatabaseConnection:
        self._conn = sqlite3.connect(self._db_path)
        self._conn.row_factory = sqlite3.Row
        return self

    def __exit__(
        self,
        exc_type: Optional[type[BaseException]],
        exc_val: Optional[BaseException],
        exc_tb: Optional[TracebackType],
    ) -> None:
        if self._conn is None:
            return
        if exc_type is None:
            self._conn.commit()
        else:
            self._conn.rollback()
        self._conn.close()
        self._conn = None

    def execute(self, query: str, params: Sequence[Any] = ()) -> None:
        """Execute a single SQL statement."""
        if self._conn is None:
            raise RuntimeError("Connection is not open")
        self._conn.execute(query, params)

    def fetchone(self, query: str, params: Sequence[Any] = ()) -> Optional[dict]:
        """Execute a query and return the first row as a dict, or None."""
        if self._conn is None:
            raise RuntimeError("Connection is not open")
        cursor = self._conn.execute(query, params)
        row = cursor.fetchone()
        return dict(row) if row else None

    def fetchall(self, query: str, params: Sequence[Any] = ()) -> list[dict]:
        """Execute a query and return all rows as dicts."""
        if self._conn is None:
            raise RuntimeError("Connection is not open")
        cursor = self._conn.execute(query, params)
        return [dict(row) for row in cursor.fetchall()]
