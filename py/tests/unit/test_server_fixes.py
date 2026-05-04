"""Tests for security and performance fixes in mcp/server.py."""

from __future__ import annotations

import ast
import importlib
import inspect
import sys
import textwrap
import types
from io import BytesIO
from pathlib import Path
from unittest.mock import MagicMock, patch

import pytest

SERVER_PATH = Path(__file__).parent.parent.parent / "src" / "syntagma" / "mcp" / "server.py"


# ---------------------------------------------------------------------------
# Helper: parse server source once
# ---------------------------------------------------------------------------


@pytest.fixture(scope="module")
def server_source() -> str:
    return SERVER_PATH.read_text(encoding="utf-8")


@pytest.fixture(scope="module")
def server_ast(server_source) -> ast.Module:
    return ast.parse(server_source)


# ---------------------------------------------------------------------------
# Fix 1: MAX_REQUEST_BYTES constant exists and _read_body enforces limits
# ---------------------------------------------------------------------------


class TestMaxRequestBytes:
    """Fix 1: _read_body must enforce a 2 MB cap and reject negative lengths."""

    def test_constant_defined(self, server_source):
        assert "MAX_REQUEST_BYTES" in server_source, (
            "MAX_REQUEST_BYTES constant is missing from server.py"
        )

    def test_constant_value(self, server_ast):
        """MAX_REQUEST_BYTES must equal 2 * 1024 * 1024 (2097152)."""
        for node in ast.walk(server_ast):
            if isinstance(node, ast.Assign):
                for target in node.targets:
                    if isinstance(target, ast.Name) and target.id == "MAX_REQUEST_BYTES":
                        # Compile the RHS expression and evaluate it safely
                        expr_code = compile(ast.Expression(body=node.value), "<ast>", "eval")
                        val = eval(expr_code, {"__builtins__": {}})  # noqa: S307
                        assert val == 2 * 1024 * 1024, (
                            f"MAX_REQUEST_BYTES should be 2097152, got {val}"
                        )
                        return
        pytest.fail("MAX_REQUEST_BYTES assignment not found in AST")

    def _make_handler(self, content_length_header: str, body: bytes):
        """Return a minimal _HttpHandler instance with mocked rfile/headers."""
        # We import here to get the live module after fixes are applied.
        from syntagma.mcp import server as srv

        handler = srv._HttpHandler.__new__(srv._HttpHandler)
        handler.rfile = BytesIO(body)
        handler.headers = {"Content-Length": content_length_header}
        # Capture _send_json calls
        handler._responses = []

        def fake_send_json(code, body):
            handler._responses.append((code, body))

        handler._send_json = fake_send_json
        return handler

    def test_read_body_normal(self):
        """A body within the limit is returned as-is."""
        from syntagma.mcp import server as srv

        data = b'{"hello": "world"}'
        h = self._make_handler(str(len(data)), data)
        result = h._read_body()
        assert result == data

    def test_read_body_over_limit_returns_none_and_sends_413(self):
        """A body exceeding MAX_REQUEST_BYTES must return None and send 413."""
        from syntagma.mcp import server as srv

        big = srv.MAX_REQUEST_BYTES + 1
        h = self._make_handler(str(big), b"x" * 10)
        result = h._read_body()
        assert result is None, "_read_body must return None when body exceeds limit"
        assert len(h._responses) == 1
        assert h._responses[0][0] == 413, f"Expected 413, got {h._responses[0][0]}"

    def test_read_body_negative_length_returns_none_and_sends_400(self):
        """A negative Content-Length must return None and send 400."""
        h = self._make_handler("-1", b"")
        result = h._read_body()
        assert result is None, "_read_body must return None for negative Content-Length"
        assert len(h._responses) == 1
        assert h._responses[0][0] == 400, f"Expected 400, got {h._responses[0][0]}"

    def test_do_post_handles_none_body_gracefully(self):
        """If _read_body returns None, do_POST must not raise."""
        from syntagma.mcp import server as srv

        handler = srv._HttpHandler.__new__(srv._HttpHandler)
        handler._responses = []

        def fake_send_json(code, body):
            handler._responses.append((code, body))

        handler._send_json = fake_send_json
        # _read_body returns None (simulates over-limit / already sent error response)
        handler._read_body = lambda: None
        handler._check_auth = lambda: True
        handler.path = "/rpc"
        # _Handler_ref must be set
        srv._Handler_ref = MagicMock()

        # Should not raise
        handler.do_POST()
        # Either a response was already queued by _read_body or do_POST returns early.
        # The key invariant: no unhandled exception.


# ---------------------------------------------------------------------------
# Fix 2: No local `import json` inside search_knowledge
# ---------------------------------------------------------------------------


class TestNoDuplicateImportJson:
    """Fix 2: `import json` must not appear inside search_knowledge()."""

    def test_no_local_import_json_in_search_knowledge(self, server_ast):
        for node in ast.walk(server_ast):
            if (
                isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef))
                and node.name == "search_knowledge"
            ):
                for child in ast.walk(node):
                    if isinstance(child, ast.Import):
                        for alias in child.names:
                            assert alias.name != "json", (
                                "Local `import json` found inside search_knowledge(); "
                                "json is already imported at module level."
                            )
                return
        pytest.fail("search_knowledge function not found in AST")


# ---------------------------------------------------------------------------
# Fix 3: RPCDispatcher created once, not per request
# ---------------------------------------------------------------------------


class TestRPCDispatcherCreatedOnce:
    """Fix 3: RPCDispatcher must not be instantiated inside do_POST."""

    def test_rpc_dispatcher_not_inside_do_post(self, server_ast):
        """RPCDispatcher() constructor call must not appear inside do_POST."""
        for node in ast.walk(server_ast):
            if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)) and node.name == "do_POST":
                for child in ast.walk(node):
                    if isinstance(child, ast.Call):
                        func = child.func
                        name = (
                            func.id
                            if isinstance(func, ast.Name)
                            else (func.attr if isinstance(func, ast.Attribute) else "")
                        )
                        assert name != "RPCDispatcher", (
                            "RPCDispatcher() must not be instantiated inside do_POST; "
                            "create it once before serve_forever()."
                        )
                return
        # do_POST not found at top level — it may be a method; walk all classes
        for node in ast.walk(server_ast):
            if isinstance(node, ast.ClassDef):
                for item in node.body:
                    if (
                        isinstance(item, (ast.FunctionDef, ast.AsyncFunctionDef))
                        and item.name == "do_POST"
                    ):
                        for child in ast.walk(item):
                            if isinstance(child, ast.Call):
                                func = child.func
                                name = (
                                    func.id
                                    if isinstance(func, ast.Name)
                                    else (func.attr if isinstance(func, ast.Attribute) else "")
                                )
                                assert name != "RPCDispatcher", (
                                    "RPCDispatcher() must not be instantiated inside do_POST."
                                )
                        return

    def test_process_rpc_not_creates_dispatcher_per_call(self, server_ast, server_source):
        """process_rpc should reuse a dispatcher; do_POST must not contain RPCDispatcher()."""
        # Use AST to extract the exact line range of do_POST across all class bodies,
        # then verify no RPCDispatcher() call appears in those lines.
        lines = server_source.splitlines()
        for node in ast.walk(server_ast):
            if isinstance(node, ast.ClassDef):
                for item in node.body:
                    if (
                        isinstance(item, (ast.FunctionDef, ast.AsyncFunctionDef))
                        and item.name == "do_POST"
                    ):
                        start = item.lineno - 1  # 0-indexed
                        end = item.end_lineno  # exclusive
                        method_lines = lines[start:end]
                        for ln in method_lines:
                            if "RPCDispatcher(" in ln:
                                pytest.fail(
                                    f"RPCDispatcher() instantiation found inside do_POST: {ln!r}"
                                )
                        return


# ---------------------------------------------------------------------------
# Fix 4: Parallel RAG calls using ThreadPoolExecutor
# ---------------------------------------------------------------------------


class TestParallelRagCalls:
    """Fix 4: When etypes >= 2, rag() calls must run in parallel via ThreadPoolExecutor."""

    def test_thread_pool_executor_imported(self, server_source):
        assert "ThreadPoolExecutor" in server_source, (
            "ThreadPoolExecutor is not imported/used in server.py"
        )

    def test_concurrent_futures_import(self, server_source):
        assert (
            "concurrent.futures" in server_source or "from concurrent.futures" in server_source
        ), "concurrent.futures is not imported in server.py"

    def test_rag_calls_are_parallel(self):
        """When etypes has 2 values, both rag() calls must happen concurrently."""
        import threading
        from syntagma.mcp import server as srv

        call_threads: list[int] = []
        barrier = threading.Barrier(2, timeout=2)

        def fake_rag(query, *, limit, db_path, filters):
            tid = threading.get_ident()
            call_threads.append(tid)
            try:
                barrier.wait()  # both calls must be in-flight simultaneously
            except threading.BrokenBarrierError:
                pass
            return []

        mock_graph = MagicMock()
        mock_graph.get_entities_batch.return_value = {}

        with patch("syntagma.mcp.server.DB_PATH", "dummy.db"):
            instance = srv.SyntagmaMCP.__new__(srv.SyntagmaMCP)
            instance.graph = mock_graph
            instance._rag = fake_rag
            instance._detector = None
            instance._refactor_engine = None

            with (
                patch(
                    "syntagma.rag.problem_mapper.suggest_search_approach",
                    return_value={"entity_types": ["smell", "pattern"]},
                ),
                patch(
                    "syntagma.summarizer.token_efficient.estimate_tokens",
                    return_value=10,
                ),
            ):
                instance.search_knowledge("god class")

        # Barrier was reached — both rag() calls were in-flight at the same time.
        assert len(call_threads) == 2, f"Expected 2 rag() calls, got {len(call_threads)}"
        assert len(set(call_threads)) == 2, (
            "Both rag() calls ran on the same thread — they are NOT parallel."
        )
