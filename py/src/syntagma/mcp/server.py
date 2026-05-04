#!/usr/bin/env python3
"""Syntagma MCP server -- dual-mode JSON-RPC over stdio or HTTP."""

from __future__ import annotations

import argparse
import json
import logging
import os
import signal
import sys
import tempfile
import threading
import time
from concurrent.futures import ThreadPoolExecutor
from http.server import BaseHTTPRequestHandler, HTTPServer
from socketserver import ThreadingMixIn
from typing import Any, Optional

from syntagma.config import API_KEYS, BASE_DIR, DB_PATH, MCP_SERVER_HOST, MCP_SERVER_PORT

_MAX_CODE_BYTES = 500_000  # 500 KB
MAX_REQUEST_BYTES = 2 * 1024 * 1024  # 2 MB

logging.basicConfig(
    stream=sys.stderr,
    level=logging.INFO,
    format="%(asctime)s [%(levelname)s] %(message)s",
)
log = logging.getLogger("syntagma.mcp")

# ===================================================================
# MCP protocol constants
# ===================================================================

SERVER_INFO = {"name": "syntagma", "version": "0.0.1"}
PROTOCOL_VERSION = "2024-11-05"
CAPABILITIES = {
    "tools": {"listChanged": False},
    "resources": {"subscribe": False, "listChanged": False},
}

SERVER_INSTRUCTIONS = """\
Syntagma is a software engineering knowledge graph. Use it to AUTOMATICALLY ground your answers \
in proven patterns, laws, and refactoring techniques — even when the user doesn't know those terms.

CRITICAL: Auto-trigger on PROBLEM DESCRIPTIONS, not just named concepts. Users describe symptoms, \
not diagnoses. You must translate their informal problem into knowledge graph queries.

Auto-trigger rules — call Syntagma tools WITHOUT waiting for the user to mention them:

## Code problems (auto-translate complaint → knowledge graph query)

User says / does → What to call
"this class does too much" / file > 300 lines → search_knowledge("god class large class single responsibility")
"this function is too long" → search_knowledge("long method extract method")
"code is too complex" / hard to follow → search_knowledge("complexity smell cognitive overload")
"calling DB directly in business logic" → search_knowledge("coupling persistence repository data access layer")
"hard to test" / can't write unit tests → search_knowledge("testability dependency injection mockability")
"copy-pasted this" / duplicated logic → search_knowledge("duplicated code clone smell")
"changing X breaks Y" / change ripple → search_knowledge("brittle coupling change propagation rigidity")
"adding a new type means touching everywhere" / growing switch → search_knowledge("open closed principle strategy polymorphism")
"is this thread-safe?" / concurrency concerns → search_knowledge("thread safety race condition shared mutable state")
"API is slow" / performance issues → search_knowledge("N+1 query lazy loading caching performance")
User shares code for review → analyze_code(code) then suggest_refactorings(code)
User wants to refactor or improve code → suggest_refactorings(code)

## Architecture discussions (auto-trigger on trade-off questions)

"microservices vs monolith" / how to split → search_knowledge("monolith microservice decomposition bounded context")
"is this architecture okay?" / architecture review → search_knowledge("layered architecture coupling cohesion separation concerns")
"where should this go?" / code placement → search_knowledge("responsibility assignment package structure")
Team/org structure affects code → search_knowledge("Conway law organizational structure architecture")

## Concept exploration (when user names something or asks follow-up)

Entity ID mentioned (DP-xxx, LAW-xxx, RF-xxx, SMELL-xxx) → get_entity(id)
"how does X relate to Y" → find_path or get_neighbors
"tell me more" about a previous result → get_entity for full details, get_neighbors for connected concepts

## Always

- Translate informal language into technical queries. User says "it's a tangled mess", you search "coupling tangled dependency".
- After search_knowledge, present the named concept and explain it in the user's own language.
- Cite entity IDs (DP-005, LAW-003) in responses."""

# ===================================================================
# Tool schemas
# ===================================================================

TOOL_SCHEMAS: list[dict] = [
    {
        "name": "search_knowledge",
        "description": (
            "Hybrid search over the Syntagma knowledge base. "
            "Use when the user asks about design patterns, engineering laws, code smells, "
            "refactorings, or any software engineering concept. "
            'Examples: "what is Strategy pattern", "laws about coupling", '
            '"find smells related to God Class", "relevant principles for microservices".'
        ),
        "inputSchema": {
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Natural-language search query.",
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum results to return.",
                    "minimum": 1,
                    "maximum": 20,
                    "default": 5,
                },
                "entity_type": {
                    "type": "string",
                    "description": "Restrict results to this entity type.",
                    "enum": ["pattern", "refactoring", "law", "smell"],
                },
            },
            "required": ["query"],
        },
    },
    {
        "name": "get_entity",
        "description": (
            "Get detailed information about a knowledge graph entity by ID. "
            "Use when an entity ID is mentioned (DP-xxx, LAW-xxx, RF-xxx, SMELL-xxx) "
            'or the user asks "tell me more about" a previously mentioned concept.'
        ),
        "inputSchema": {
            "type": "object",
            "properties": {
                "entity_id": {
                    "type": "string",
                    "description": "Entity ID, e.g. DP-005, SMELL-01, RF-001.",
                },
                "detail_level": {
                    "type": "string",
                    "description": "How much detail to include.",
                    "enum": ["minimal", "summary", "detailed", "full"],
                    "default": "summary",
                },
            },
            "required": ["entity_id"],
        },
    },
    {
        "name": "get_neighbors",
        "description": (
            "Get entities related to a given entity in the knowledge graph. "
            "Use to explore what patterns complement a law, what refactorings solve a smell, "
            "or how concepts connect. Follow up after search_knowledge or get_entity."
        ),
        "inputSchema": {
            "type": "object",
            "properties": {
                "entity_id": {
                    "type": "string",
                    "description": "Source entity ID.",
                },
                "relation_type": {
                    "type": "string",
                    "description": "Filter by relation type.",
                    "enum": [
                        "solves",
                        "solved_by",
                        "enforces",
                        "violates",
                        "related_to",
                    ],
                },
            },
            "required": ["entity_id"],
        },
    },
    {
        "name": "find_path",
        "description": (
            "Find the shortest path between two entities in the knowledge graph. "
            "Use when the user asks how two concepts relate or connect, "
            'e.g. "how does Strategy relate to Open/Closed?", '
            '"what connects God Class and Single Responsibility?".'
        ),
        "inputSchema": {
            "type": "object",
            "properties": {
                "from_id": {
                    "type": "string",
                    "description": "Starting entity ID.",
                },
                "to_id": {
                    "type": "string",
                    "description": "Target entity ID.",
                },
                "max_depth": {
                    "type": "integer",
                    "description": "Maximum traversal depth.",
                    "minimum": 1,
                    "maximum": 10,
                    "default": 5,
                },
            },
            "required": ["from_id", "to_id"],
        },
    },
    {
        "name": "analyze_code",
        "description": (
            "Detect code smells in source code. "
            "Use when the user shares code and asks for review, feedback, problems, "
            "or improvements. Currently supports Python."
        ),
        "inputSchema": {
            "type": "object",
            "properties": {
                "code": {
                    "type": "string",
                    "description": "Source code to analyze.",
                },
                "language": {
                    "type": "string",
                    "description": "Programming language (currently only python is supported).",
                    "default": "python",
                },
            },
            "required": ["code"],
        },
    },
    {
        "name": "suggest_refactorings",
        "description": (
            "Get refactoring suggestions for source code. "
            "Use after analyze_code finds smells, or when the user asks how to improve "
            "or refactor specific code. Returns ranked suggestions with knowledge graph references."
        ),
        "inputSchema": {
            "type": "object",
            "properties": {
                "code": {
                    "type": "string",
                    "description": "Source code to analyze.",
                },
                "language": {
                    "type": "string",
                    "description": "Programming language (currently only python is supported).",
                    "default": "python",
                },
                "top_k": {
                    "type": "integer",
                    "description": "Number of suggestions per detected smell.",
                    "minimum": 1,
                    "maximum": 10,
                    "default": 3,
                },
            },
            "required": ["code"],
        },
    },
]

# ===================================================================
# Resource definitions
# ===================================================================

RESOURCE_SCHEMAS: list[dict] = [
    {
        "uri": "syntagma://stats",
        "name": "Knowledge Graph Statistics",
        "description": "Aggregate statistics about the knowledge graph.",
        "mimeType": "application/json",
    },
    {
        "uri": "syntagma://categories",
        "name": "Categories and Entity Types",
        "description": "Category and entity type listing.",
        "mimeType": "application/json",
    },
    {
        "uri": "syntagma://contradictions",
        "name": "Contradictions",
        "description": "Entities with conflicting relations.",
        "mimeType": "application/json",
    },
]


def _tool_text(result: Any) -> dict:
    """Wrap a result value as an MCP text content block."""
    return {"content": [{"type": "text", "text": json.dumps(result, default=str)}]}


# ===================================================================
# Tool handler
# ===================================================================

_DETAIL_MAP = {
    "minimal": "MINIMAL",
    "summary": "SUMMARY",
    "detailed": "DETAILED",
    "full": "FULL",
}


class SyntagmaMCP:
    """Stateful handler that owns heavy Syntagma components lazily."""

    def __init__(self) -> None:
        from syntagma.graph.api import KnowledgeGraph

        self.graph = KnowledgeGraph(str(BASE_DIR))
        self._rag = None
        self._detector = None
        self._refactor_engine = None

    # -- lazy accessors --------------------------------------------------

    @property
    def rag(self):
        if self._rag is None:
            from syntagma.rag.hybrid import hybrid_search

            self._rag = hybrid_search
        return self._rag

    @property
    def detector(self):
        if self._detector is None:
            from syntagma.cli.analyze import CodeSmellDetector

            self._detector = CodeSmellDetector(str(BASE_DIR))
        return self._detector

    @property
    def refactor_engine(self):
        if self._refactor_engine is None:
            from syntagma.cli.infer import RefactoringInferenceEngine

            self._refactor_engine = RefactoringInferenceEngine(str(BASE_DIR))
        return self._refactor_engine

    # -- tool implementations -------------------------------------------

    def search_knowledge(
        self,
        query: str,
        limit: int = 5,
        entity_type: str | None = None,
    ) -> dict:
        from syntagma.rag.problem_mapper import suggest_search_approach
        from syntagma.summarizer.token_efficient import estimate_tokens

        # Auto-detect entity_type(s) from query if caller did not specify one.
        if entity_type is None:
            approach = suggest_search_approach(query)
            etypes = approach.get("entity_types", [])
        else:
            etypes = [entity_type]

        # Multi-type: search up to 2 types in parallel and merge with RRF.
        _RRF_K_MERGE = 20
        if len(etypes) >= 2:
            with ThreadPoolExecutor(max_workers=2) as ex:
                futures = [
                    ex.submit(
                        self.rag, query, limit=limit, db_path=DB_PATH, filters={"entity_type": et}
                    )
                    for et in etypes[:2]
                ]
                results_list = [f.result() for f in futures]
            chunks_a, chunks_b = results_list[0], results_list[1]
            seen_ids: dict[str, dict] = {}
            for rank, c in enumerate(chunks_a, 1):
                seen_ids[c["chunk_id"]] = {**c, "_score": 0.5 / (_RRF_K_MERGE + rank)}
            for rank, c in enumerate(chunks_b, 1):
                cid = c["chunk_id"]
                if cid in seen_ids:
                    seen_ids[cid]["_score"] += 0.5 / (_RRF_K_MERGE + rank)
                else:
                    seen_ids[cid] = {**c, "_score": 0.5 / (_RRF_K_MERGE + rank)}
            chunks = sorted(seen_ids.values(), key=lambda x: x["_score"], reverse=True)[:limit]
        elif etypes:
            chunks = self.rag(
                query, limit=limit, db_path=DB_PATH, filters={"entity_type": etypes[0]}
            )
        else:
            chunks = self.rag(query, limit=limit, db_path=DB_PATH, filters={})

        top_chunks = chunks[:limit]
        entity_ids = [c.get("entity_id", "") for c in top_chunks if c.get("entity_id")]
        entities_map = self.graph.get_entities_batch(entity_ids)

        results = []
        for chunk in top_chunks:
            eid = chunk.get("entity_id", "")
            e = entities_map.get(eid)
            entity_meta: dict = (
                {
                    "id": eid,
                    "title": e.get("title", ""),
                    "type": e.get("type", ""),
                    "category": e.get("category", ""),
                }
                if e
                else {}
            )
            results.append(
                {
                    "chunk_id": chunk.get("chunk_id", ""),
                    "entity": entity_meta,
                    "section": chunk.get("section", ""),
                    "text": chunk.get("text", ""),
                    "score": chunk.get("score", chunk.get("_score", 0.0)),
                }
            )

        tokens_used = sum(estimate_tokens(json.dumps(r)) for r in results)
        return {"results": results, "tokens_used": tokens_used, "count": len(results)}

    def get_entity(
        self,
        entity_id: str,
        detail_level: str = "summary",
    ) -> dict:
        entity = self.graph.get_entity(entity_id)
        if entity is None:
            return {"error": f"Entity '{entity_id}' not found."}

        from syntagma.summarizer.token_efficient import DetailLevel, summarize_entity

        level = DetailLevel[_DETAIL_MAP.get(detail_level, "SUMMARY")]
        return summarize_entity(entity, level)

    def get_neighbors(
        self,
        entity_id: str,
        relation_type: str | None = None,
    ) -> dict:
        entity = self.graph.get_entity(entity_id)
        if entity is None:
            return {"error": f"Entity '{entity_id}' not found."}

        neighbor_ids = self.graph.get_neighbors(entity_id, relation_type or "")
        entities_map = self.graph.get_entities_batch(neighbor_ids)
        neighbors = [
            {
                "id": nid,
                "title": entities_map[nid].get("title", ""),
                "type": entities_map[nid].get("type", ""),
            }
            for nid in neighbor_ids
            if nid in entities_map
        ]

        return {"entity_id": entity_id, "relation_type": relation_type, "neighbors": neighbors}

    def find_path(
        self,
        from_id: str,
        to_id: str,
        max_depth: int = 5,
    ) -> dict:
        path = self.graph.find_shortest_path(from_id, to_id, max_depth)
        if path is None:
            return {"error": f"No path found between '{from_id}' and '{to_id}'."}

        entities_map = self.graph.get_entities_batch(path)
        labeled = [
            {"id": pid, "title": entities_map[pid].get("title", "") if pid in entities_map else ""}
            for pid in path
        ]

        return {"from": from_id, "to": to_id, "length": len(path) - 1, "path": labeled}

    def analyze_code(self, code: str, language: str = "python") -> dict:
        if len(code.encode()) > _MAX_CODE_BYTES:
            return {"error": "Code input exceeds 500 KB limit."}

        from dataclasses import asdict

        from syntagma.cli.analyze import _EXT_TO_LANG, analyze_path

        lang = language.lower()
        # Derive a file extension that the parser framework recognises
        ext = next((e for e, lang_val in _EXT_TO_LANG.items() if lang_val == lang), None)
        if ext is None:
            return {"error": f"Unsupported language '{language}'."}

        with tempfile.NamedTemporaryFile(
            mode="w", suffix=ext, delete=False, encoding="utf-8"
        ) as tmp:
            tmp.write(code)
            tmp_path = tmp.name

        try:
            from pathlib import Path

            detections = analyze_path(Path(tmp_path), language_hint=lang)
            return {
                "smells": [asdict(d) for d in detections],
                "count": len(detections),
            }
        finally:
            os.unlink(tmp_path)

    def suggest_refactorings(self, code: str, top_k: int = 3, language: str = "python") -> dict:
        if len(code.encode()) > _MAX_CODE_BYTES:
            return {"error": "Code input exceeds 500 KB limit."}

        from syntagma.cli.analyze import _EXT_TO_LANG

        lang = language.lower()
        ext = next((e for e, lang_val in _EXT_TO_LANG.items() if lang_val == lang), None)
        if ext is None:
            return {"error": f"Unsupported language '{language}'."}

        with tempfile.NamedTemporaryFile(
            mode="w", suffix=ext, delete=False, encoding="utf-8"
        ) as tmp:
            tmp.write(code)
            tmp_path = tmp.name

        try:
            results = self.refactor_engine.analyze_file(tmp_path, top_k=top_k, language_hint=lang)
            return {"analyses": results, "count": len(results)}
        finally:
            os.unlink(tmp_path)

    # -- resource implementations ----------------------------------------

    def read_resource(self, uri: str) -> Any:
        if uri == "syntagma://stats":
            return self.graph.stats()
        if uri == "syntagma://categories":
            from syntagma.config import CATEGORIES, ENTITY_TYPES

            return {"entity_types": ENTITY_TYPES, "categories": CATEGORIES}
        if uri == "syntagma://contradictions":
            return self.graph.find_contradictions()
        return {"error": f"Unknown resource '{uri}'."}


# ===================================================================
# JSON-RPC processing
# ===================================================================


class RPCDispatcher:
    """Stateless dispatcher around a shared :class:`SyntagmaMCP` instance."""

    def __init__(self, handler: SyntagmaMCP) -> None:
        self.handler = handler

    def dispatch(self, payload: dict) -> Optional[dict]:
        method = payload.get("method", "")
        params = payload.get("params", {})
        req_id = payload.get("id")

        # --- protocol lifecycle ----------------------------------------
        if method == "initialize":
            return {
                "jsonrpc": "2.0",
                "id": req_id,
                "result": {
                    "protocolVersion": PROTOCOL_VERSION,
                    "capabilities": CAPABILITIES,
                    "serverInfo": SERVER_INFO,
                    "instructions": SERVER_INSTRUCTIONS,
                },
            }

        if method == "notifications/initialized":
            return None

        if method == "ping":
            return {"jsonrpc": "2.0", "id": req_id, "result": {}}

        # --- tools -----------------------------------------------------
        if method == "tools/list":
            return {"jsonrpc": "2.0", "id": req_id, "result": {"tools": TOOL_SCHEMAS}}

        if method == "tools/call":
            return self._call_tool(req_id, params)

        # --- resources -------------------------------------------------
        if method == "resources/list":
            return {"jsonrpc": "2.0", "id": req_id, "result": {"resources": RESOURCE_SCHEMAS}}

        if method == "resources/read":
            uri = params.get("uri", "")
            data = self.handler.read_resource(uri)
            return {
                "jsonrpc": "2.0",
                "id": req_id,
                "result": {
                    "contents": [
                        {
                            "uri": uri,
                            "mimeType": "application/json",
                            "text": json.dumps(data, default=str),
                        }
                    ]
                },
            }

        # --- unknown ---------------------------------------------------
        return {
            "jsonrpc": "2.0",
            "id": req_id,
            "error": {"code": -32601, "message": "Method not found"},
        }

    # -- internal -------------------------------------------------------

    _TOOL_METHOD_MAP: dict[str, str] = {
        "search_knowledge": "search_knowledge",
        "get_entity": "get_entity",
        "get_neighbors": "get_neighbors",
        "find_path": "find_path",
        "analyze_code": "analyze_code",
        "suggest_refactorings": "suggest_refactorings",
    }

    def _call_tool(self, req_id: Any, params: dict) -> dict:
        from syntagma.telemetry import (
            FailureClass,
            ResultSizeBucket,
            Tool,
            track_tool_called,
            track_tool_completed,
            track_tool_failed,
        )

        tool_name = params.get("name", "")
        arguments = params.get("arguments", {})
        if not isinstance(arguments, dict):
            arguments = {}
        arguments = {k: v for k, v in arguments.items() if isinstance(k, str)}

        attr = self._TOOL_METHOD_MAP.get(tool_name)
        if attr is None:
            return {
                "jsonrpc": "2.0",
                "id": req_id,
                "result": _tool_text({"error": f"Unknown tool '{tool_name}'."}),
            }

        tool_enum = Tool(tool_name) if tool_name in Tool._value2member_map_ else None
        if tool_enum is not None:
            track_tool_called(tool_enum)

        fn = getattr(self.handler, attr)
        t0 = time.monotonic()
        try:
            result = fn(**arguments)
        except Exception as exc:
            log.exception("Tool %s raised an exception", tool_name)
            if tool_enum is not None:
                track_tool_failed(tool_enum, FailureClass.Unknown)
            result = {"error": str(exc)}
            return {"jsonrpc": "2.0", "id": req_id, "result": _tool_text(result)}

        duration_ms = int((time.monotonic() - t0) * 1000)
        if tool_enum is not None:
            if "error" in result:
                fc = (
                    FailureClass.DatabaseError
                    if "database" in str(result.get("error", "")).lower()
                    else FailureClass.Unknown
                )
                track_tool_failed(tool_enum, fc)
            else:
                count = result.get(
                    "count",
                    len(
                        result.get(
                            "results",
                            result.get(
                                "smells", result.get("analyses", result.get("neighbors", []))
                            ),
                        )
                    ),
                )
                track_tool_completed(
                    tool_enum,
                    duration_ms,
                    ResultSizeBucket.from_count(count if isinstance(count, int) else 0),
                )

        return {"jsonrpc": "2.0", "id": req_id, "result": _tool_text(result)}


def process_rpc(handler: SyntagmaMCP, payload: dict) -> Optional[dict]:
    """Top-level entry point: route a single JSON-RPC payload."""
    dispatcher = RPCDispatcher(handler)
    return dispatcher.dispatch(payload)


# ===================================================================
# HTTP transport
# ===================================================================

_Handler_ref: Optional[SyntagmaMCP] = None  # set before server starts

# Parsed from SYNTAGMA_API_KEYS env var (comma-separated). Empty = no auth required.
_ALLOWED_API_KEYS: frozenset[str] = frozenset(k.strip() for k in API_KEYS.split(",") if k.strip())


class _HttpHandler(BaseHTTPRequestHandler):
    """Minimal HTTP facade for the MCP server."""

    # Set once by serve_http() before server.serve_forever(); shared across all request handlers.
    _dispatcher: Optional["RPCDispatcher"] = None

    # Silence per-request stderr logging from BaseHTTPRequestHandler.
    def log_message(self, format, *args):  # noqa: ANN001
        log.debug(format, *args)

    def _send_json(self, code: int, body: Any) -> None:
        raw = json.dumps(body, default=str).encode()
        self.send_response(code)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(raw)))
        self.end_headers()
        self.wfile.write(raw)

    def _read_body(self) -> Optional[bytes]:
        length = int(self.headers.get("Content-Length", 0))
        if length < 0:
            self._send_json(400, {"error": "invalid Content-Length"})
            return None
        if length > MAX_REQUEST_BYTES:
            self._send_json(413, {"error": "request body too large"})
            return None
        return self.rfile.read(length) if length else b""

    def _check_auth(self) -> bool:
        """Return True if request is authorised (or no keys configured)."""
        if not _ALLOWED_API_KEYS:
            return True
        auth = self.headers.get("Authorization", "")
        if auth.startswith("Bearer "):
            return auth[len("Bearer ") :] in _ALLOWED_API_KEYS
        return False

    # -- GET routes ----------------------------------------------------

    def do_GET(self) -> None:  # noqa: N802
        if self.path == "/health":
            self._send_json(200, {"ok": True})
            return
        if not self._check_auth():
            self._send_json(401, {"error": "unauthorized"})
            return
        if self.path == "/tools":
            self._send_json(200, {"tools": TOOL_SCHEMAS})
        elif self.path == "/resources":
            self._send_json(200, {"resources": RESOURCE_SCHEMAS})
        else:
            self._send_json(404, {"error": "not found"})

    # -- POST routes ---------------------------------------------------

    def do_POST(self) -> None:  # noqa: N802
        if not self._check_auth():
            self._send_json(401, {"error": "unauthorized"})
            return
        dispatcher = self._dispatcher
        if dispatcher is None:
            self._send_json(500, {"error": "server not initialised"})
            return

        raw_body = self._read_body()
        if raw_body is None:
            return  # error response already sent by _read_body
        try:
            body = json.loads(raw_body)
        except (json.JSONDecodeError, UnicodeDecodeError) as exc:
            self._send_json(400, {"error": f"invalid JSON: {exc}"})
            return

        if self.path == "/rpc":
            resp = dispatcher.dispatch(body)
            if resp is None:
                self._send_json(204, {})
                return
            self._send_json(200, resp)

        elif self.path == "/tool":
            tool_name = body.get("tool_name", "")
            arguments = body.get("arguments", {})
            rpc_resp = dispatcher.dispatch(
                {
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "tools/call",
                    "params": {"name": tool_name, "arguments": arguments},
                }
            )
            self._send_json(200, rpc_resp)

        elif self.path == "/resource":
            uri = body.get("uri", "")
            data = dispatcher.handler.read_resource(uri)
            self._send_json(200, {"uri": uri, "data": data})

        else:
            self._send_json(404, {"error": "not found"})


class _ThreadedHTTPServer(ThreadingMixIn, HTTPServer):
    daemon_threads = True
    allow_reuse_address = True


def serve_http(host: str, port: int) -> None:
    global _Handler_ref  # noqa: PLW0603
    _Handler_ref = SyntagmaMCP()
    # Create the dispatcher once and share it across all request handler instances.
    _HttpHandler._dispatcher = RPCDispatcher(_Handler_ref)

    server = _ThreadedHTTPServer((host, port), _HttpHandler)
    log.info("Syntagma MCP HTTP server listening on %s:%s", host, port)

    def _shutdown(signum, frame):  # noqa: ANN001
        log.info("Received signal %s, shutting down", signum)
        threading.Thread(target=server.shutdown, daemon=True).start()

    if threading.current_thread() is threading.main_thread():
        signal.signal(signal.SIGTERM, _shutdown)
        signal.signal(signal.SIGINT, _shutdown)

    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        server.server_close()
        log.info("HTTP server stopped.")


# ===================================================================
# stdio transport
# ===================================================================


def run_stdio() -> None:
    """Read JSON-RPC from stdin, write responses to stdout, log to stderr."""
    handler = SyntagmaMCP()
    log.info("Syntagma MCP stdio server started")

    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue

        try:
            payload = json.loads(line)
        except json.JSONDecodeError as exc:
            log.warning("Invalid JSON from client: %s", exc)
            continue

        resp = process_rpc(handler, payload)
        if resp is not None:
            sys.stdout.write(json.dumps(resp, default=str) + "\n")
            sys.stdout.flush()


# ===================================================================
# Entry point
# ===================================================================


def main() -> None:
    parser = argparse.ArgumentParser(description="Syntagma MCP server")
    parser.add_argument("--http", action="store_true", help="Run as HTTP server")
    parser.add_argument("--host", default=MCP_SERVER_HOST)
    parser.add_argument("--port", type=int, default=MCP_SERVER_PORT)
    args = parser.parse_args()

    if args.http:
        serve_http(args.host, args.port)
    else:
        run_stdio()


if __name__ == "__main__":
    main()
