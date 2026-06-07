# Guards against: Message Chains FP.
# Fluent API / method chaining is an intentional design pattern — each method
# returns self to enable readable query construction. This is not a smell.

from __future__ import annotations

from typing import Optional, Sequence


class QueryBuilder:
    """Fluent SQL query builder.

    Each method returns self to support chaining. The resulting SQL
    is built only when `.build()` is called. This is a standard pattern
    for constructing queries programmatically.
    """

    def __init__(self) -> None:
        self._table: Optional[str] = None
        self._columns: list[str] = ["*"]
        self._where_clauses: list[str] = []
        self._where_params: list[object] = []
        self._order_by: Optional[str] = None
        self._limit_value: Optional[int] = None

    def select(self, *cols: str) -> QueryBuilder:
        """Specify columns to select. Defaults to '*' if not called."""
        if cols:
            self._columns = list(cols)
        return self

    def from_table(self, table: str) -> QueryBuilder:
        """Set the target table."""
        self._table = table
        return self

    def where(self, condition: str, *params: object) -> QueryBuilder:
        """Add a WHERE clause (AND-combined). Params are positionally bound."""
        self._where_clauses.append(condition)
        self._where_params.extend(params)
        return self

    def order_by(self, column: str, descending: bool = False) -> QueryBuilder:
        """Set ORDER BY column."""
        direction = "DESC" if descending else "ASC"
        self._order_by = f"{column} {direction}"
        return self

    def limit(self, n: int) -> QueryBuilder:
        """Set LIMIT on result count."""
        self._limit_value = n
        return self

    def build(self) -> tuple[str, Sequence[object]]:
        """Construct the SQL string and parameter list."""
        if self._table is None:
            raise ValueError("Table must be specified via from_table()")

        cols = ", ".join(self._columns)
        sql = f"SELECT {cols} FROM {self._table}"

        if self._where_clauses:
            clauses = " AND ".join(self._where_clauses)
            sql += f" WHERE {clauses}"

        if self._order_by is not None:
            sql += f" ORDER BY {self._order_by}"

        if self._limit_value is not None:
            sql += f" LIMIT {self._limit_value}"

        return sql, self._where_params
