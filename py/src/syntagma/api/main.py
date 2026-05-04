#!/usr/bin/env python3
"""
Syntagma REST API Server
Production-ready FastAPI application for code analysis and knowledge graph queries
"""

import os
import time
from contextlib import asynccontextmanager
from pathlib import Path
from typing import Optional

from fastapi import Depends, FastAPI, HTTPException, Request
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import JSONResponse
from pydantic import BaseModel, Field

from syntagma.api.auth import auth_handler
from syntagma.api.cache import (
    cache_manager,
)
from syntagma.api.logging_config import (
    RequestIDMiddleware,
    get_logger,
    log_analysis,
    log_error,
    log_search,
    setup_logging,
)
from syntagma.api.metrics import (
    analysis_duration,
    graph_query_duration,
    search_duration,
    setup_metrics,
    track_refactoring_suggestion,
    track_search,
    track_smell_detection,
    update_component_health,
    update_memory_usage,
)
from syntagma.api.middleware import RateLimits, limiter, setup_rate_limiting
from syntagma.cli.analyze import CodeSmellDetector
from syntagma.cli.infer import RefactoringInferenceEngine
from syntagma.config import (
    BASE_DIR as CONFIG_BASE_DIR,
)
from syntagma.config import (
    ENABLE_DEBUG_ENDPOINTS,
    ENABLE_JSON_LOGGING,
    LOG_LEVEL,
)
from syntagma.graph.api import KnowledgeGraph
from syntagma.rag.build_v2 import SyntagmaRAG

logger_instance = setup_logging(level=LOG_LEVEL, enable_json=ENABLE_JSON_LOGGING)
logger = get_logger("syntagma.api")


# ===== Global State =====
class AppState:
    """Application state with cached components"""

    def __init__(self):
        self.smell_detector: Optional[CodeSmellDetector] = None
        self.refactoring_engine: Optional[RefactoringInferenceEngine] = None
        self.knowledge_graph: Optional[KnowledgeGraph] = None
        self.rag_system: Optional[SyntagmaRAG] = None
        self.base_dir: Path = CONFIG_BASE_DIR


state = AppState()


@asynccontextmanager
async def lifespan(app: FastAPI):
    """Startup and shutdown events"""
    # Startup: Initialize all components
    logger.info("Starting Syntagma API", extra={"event": "startup"})

    try:
        state.smell_detector = CodeSmellDetector(str(state.base_dir))
        update_component_health("smell_detector", True)
        logger.info("Code smell detector loaded")
    except Exception as e:
        update_component_health("smell_detector", False)
        log_error(logger, e, "loading smell detector")

    try:
        state.refactoring_engine = RefactoringInferenceEngine(str(state.base_dir))
        update_component_health("refactoring_engine", True)
        logger.info("Refactoring engine loaded")
    except Exception as e:
        update_component_health("refactoring_engine", False)
        log_error(logger, e, "loading refactoring engine")

    try:
        state.knowledge_graph = KnowledgeGraph(str(state.base_dir))
        update_component_health("knowledge_graph", True)
        logger.info("Knowledge graph loaded")
    except Exception as e:
        update_component_health("knowledge_graph", False)
        log_error(logger, e, "loading knowledge graph")

    try:
        state.rag_system = SyntagmaRAG(str(state.base_dir))
        update_component_health("rag_system", True)
        logger.info("RAG system loaded")
    except Exception as e:
        update_component_health("rag_system", False)
        log_error(logger, e, "loading RAG system")

    logger.info("All components loaded", extra={"event": "startup_complete"})

    # Connect to Redis cache
    try:
        await cache_manager.connect()
        update_component_health("redis_cache", True)
    except Exception as e:
        update_component_health("redis_cache", False)
        logger.warning(f"Redis cache disabled: {e}")

    # Setup rate limiting
    setup_rate_limiting(app)

    # Update initial memory usage
    update_memory_usage()

    yield

    # Shutdown
    logger.info("Shutting down Syntagma API", extra={"event": "shutdown"})

    # Disconnect Redis
    await cache_manager.disconnect()


# ===== FastAPI App =====
app = FastAPI(
    title="Syntagma API",
    description="Knowledge Graph for Software Engineering - Code Analysis & Semantic Search",
    version="0.0.1",
    lifespan=lifespan,
)

# Request ID middleware (must be first)
app.add_middleware(RequestIDMiddleware)

# CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],  # Configure appropriately for production
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Setup Prometheus metrics
instrumentator = setup_metrics(app)


# Expose /metrics endpoint
@app.on_event("startup")
async def expose_metrics():
    """Expose Prometheus metrics endpoint"""
    instrumentator.expose(app, endpoint="/metrics", include_in_schema=False)


# ===== Request/Response Models =====


class CodeAnalysisRequest(BaseModel):
    """Request for code smell detection"""

    code: str = Field(..., description="Source code to analyze")
    language: str = Field(
        "python", description="Language of the source code (e.g. python, java, go)"
    )
    min_confidence: float = Field(0.5, ge=0.0, le=1.0, description="Minimum confidence threshold")


class RefactoringRequest(BaseModel):
    """Request for refactoring suggestions"""

    code: str = Field(..., description="Source code to analyze")
    language: str = Field(
        "python", description="Language of the source code (e.g. python, java, go)"
    )
    top_k: int = Field(3, ge=1, le=10, description="Number of suggestions per smell")
    min_confidence: float = Field(0.5, ge=0.0, le=1.0, description="Minimum smell confidence")


class SearchRequest(BaseModel):
    """Request for semantic search"""

    query: str = Field(..., description="Search query (English or Korean)")
    top_k: int = Field(5, ge=1, le=20, description="Number of results")
    entity_type: Optional[str] = Field(
        None, description="Filter by entity type (pattern/refactoring/law/smell)"
    )
    entity_id: Optional[str] = Field(None, description="Filter by specific entity ID")


class GraphQueryRequest(BaseModel):
    """Request for graph query"""

    entity_id: str = Field(..., description="Entity ID to query")
    relation_type: Optional[str] = Field(None, description="Filter by relation type")


class PathRequest(BaseModel):
    """Request for shortest path"""

    from_id: str = Field(..., description="Source entity ID")
    to_id: str = Field(..., description="Target entity ID")
    max_depth: int = Field(5, ge=1, le=10, description="Maximum search depth")


class SubgraphRequest(BaseModel):
    """Request for subgraph extraction"""

    center_id: str = Field(..., description="Center entity ID")
    radius: int = Field(2, ge=1, le=5, description="Subgraph radius")


# Note: Request logging is now handled by RequestIDMiddleware in logging_config.py


# ===== Health & Info Endpoints =====


@app.get("/")
async def root():
    """Root endpoint"""
    return {
        "service": "Syntagma API",
        "version": "0.0.1",
        "status": "operational",
        "endpoints": {
            "health": "/health",
            "stats": "/stats",
            "analyze": "/analyze",
            "refactor": "/refactor",
            "search": "/search",
            "graph": "/graph/*",
        },
    }


@app.get("/health")
async def health_check():
    """
    Comprehensive health check with component status and resource usage
    """
    import psutil

    # Check component health
    components = {
        "smell_detector": state.smell_detector is not None,
        "refactoring_engine": state.refactoring_engine is not None,
        "knowledge_graph": state.knowledge_graph is not None,
        "rag_system": state.rag_system is not None,
        "redis_cache": cache_manager.enabled and cache_manager.redis is not None,
    }

    # Get cache stats if available
    cache_stats = await cache_manager.get_stats()

    # Get memory and disk usage
    process = psutil.Process()
    memory_info = process.memory_info()
    disk_usage = psutil.disk_usage("/")

    # Determine overall status
    all_healthy = all(components.values())
    status = "healthy" if all_healthy else "degraded"

    # Update memory metric
    update_memory_usage()

    return {
        "status": status,
        "components": components,
        "cache": cache_stats,
        "resources": {
            "memory": {
                "rss_bytes": memory_info.rss,
                "rss_mb": round(memory_info.rss / 1024 / 1024, 2),
                "percent": process.memory_percent(),
            },
            "disk": {
                "total_gb": round(disk_usage.total / 1024 / 1024 / 1024, 2),
                "used_gb": round(disk_usage.used / 1024 / 1024 / 1024, 2),
                "free_gb": round(disk_usage.free / 1024 / 1024 / 1024, 2),
                "percent": disk_usage.percent,
            },
        },
    }


@app.get("/health/live")
async def liveness_probe():
    """
    Kubernetes liveness probe - checks if the app is running
    """
    return {"status": "alive"}


@app.get("/health/ready")
async def readiness_probe():
    """
    Kubernetes readiness probe - checks if the app is ready to serve traffic
    """
    # Check if critical components are loaded
    critical_components = [
        state.smell_detector is not None,
        state.knowledge_graph is not None,
    ]

    if all(critical_components):
        return {"status": "ready"}
    else:
        raise HTTPException(status_code=503, detail="Not ready")


@app.get("/stats")
@limiter.limit(RateLimits.DEFAULT)
async def get_stats(request: Request, api_key: str = Depends(auth_handler.verify_api_key)):
    """Get system statistics (requires API key)"""
    if state.knowledge_graph is None:
        raise HTTPException(status_code=503, detail="Not ready")
    graph_stats = state.knowledge_graph.stats()

    return {
        "knowledge_graph": graph_stats,
        "rag": {
            "embedding_model": "all-MiniLM-L6-v2",
            "embedding_dim": 384,
            "database": "syntagma.db",
        },
    }


# ===== Code Analysis Endpoints =====


@app.post("/analyze")
@limiter.limit(RateLimits.ANALYZE)
async def analyze_code(
    req: Request, request: CodeAnalysisRequest, api_key: str = Depends(auth_handler.verify_api_key)
):
    """
    Detect code smells in source code.

    Returns list of detected smells with confidence scores.

    Rate limit: 20 requests/minute
    """
    start_time = time.time()

    try:
        import tempfile

        from syntagma.cli.analyze import _EXT_TO_LANG, analyze_path

        lang = request.language.lower()
        ext = next((e for e, lang_val in _EXT_TO_LANG.items() if lang_val == lang), ".py")

        with tempfile.NamedTemporaryFile(mode="w", suffix=ext, delete=False) as f:
            f.write(request.code)
            temp_path = f.name

        try:
            detections = analyze_path(
                Path(temp_path), language_hint=lang, min_confidence=request.min_confidence
            )

            # Convert to dict and track metrics
            results = []
            for d in detections:
                results.append(
                    {
                        "smell_id": d.smell_id,
                        "smell_name": d.smell_name,
                        "confidence": d.confidence,
                        "location": d.location,
                        "function_name": d.function_name,
                        "metrics": {
                            "loc": d.metrics.loc,
                            "cyclomatic_complexity": d.metrics.cyclomatic_complexity,
                            "nesting_depth": d.metrics.nesting_depth,
                            "parameter_count": d.metrics.parameter_count,
                            "local_variables": d.metrics.local_variables,
                            "return_statements": d.metrics.return_statements,
                        },
                        "reasons": d.reasons,
                    }
                )

                # Track smell detection metric
                track_smell_detection(d.smell_id, d.smell_name)

            # Track analysis duration
            duration_ms = (time.time() - start_time) * 1000
            analysis_duration.observe(duration_ms / 1000)

            # Log analysis
            log_analysis(logger, len(results), duration_ms, len(request.code))

            return {"smells_detected": len(results), "detections": results}

        finally:
            # Cleanup temp file
            os.unlink(temp_path)

    except Exception as e:
        log_error(logger, e, "code analysis", code_length=len(request.code))
        raise HTTPException(status_code=500, detail=str(e)) from e


@app.post("/refactor")
@limiter.limit(RateLimits.REFACTOR)
async def get_refactoring_suggestions(
    req: Request, request: RefactoringRequest, api_key: str = Depends(auth_handler.verify_api_key)
):
    """
    Get refactoring suggestions for source code.

    Returns ranked refactoring suggestions based on detected smells.

    Rate limit: 20 requests/minute
    """
    try:
        import tempfile

        from syntagma.cli.analyze import _EXT_TO_LANG

        lang = request.language.lower()
        ext = next((e for e, lang_val in _EXT_TO_LANG.items() if lang_val == lang), ".py")

        with tempfile.NamedTemporaryFile(mode="w", suffix=ext, delete=False) as f:
            f.write(request.code)
            temp_path = f.name

        try:
            if state.refactoring_engine is None:
                raise HTTPException(status_code=503, detail="Not ready")
            results = state.refactoring_engine.analyze_file(
                temp_path, top_k=request.top_k, language_hint=lang
            )

            filtered_results = []
            for result in results:
                smell = result["smell"]
                if smell["confidence"] >= request.min_confidence:
                    filtered_results.append(result)

                    for suggestion in result.get("suggestions", []):
                        track_refactoring_suggestion(suggestion["refactoring_id"])

            logger.info(
                "Refactoring suggestions generated",
                extra={
                    "smells_analyzed": len(filtered_results),
                    "total_suggestions": sum(
                        len(r.get("suggestions", [])) for r in filtered_results
                    ),
                },
            )

            return {"smells_analyzed": len(filtered_results), "results": filtered_results}

        finally:
            # Cleanup temp file
            os.unlink(temp_path)

    except Exception as e:
        log_error(logger, e, "refactoring analysis")
        raise HTTPException(status_code=500, detail=str(e)) from e


# ===== Semantic Search Endpoint =====


@app.post("/search")
@limiter.limit(RateLimits.SEARCH)
async def semantic_search(
    req: Request, request: SearchRequest, api_key: str = Depends(auth_handler.verify_api_key)
):
    """
    Semantic search over knowledge base

    Supports English and Korean queries

    Rate limit: 50 requests/minute
    """
    start_time = time.time()

    try:
        if state.rag_system is None:
            raise HTTPException(status_code=503, detail="Not ready")
        filters = {}
        if request.entity_type:
            filters["entity_type"] = request.entity_type
        if request.entity_id:
            filters["entity_id"] = request.entity_id

        results = state.rag_system.search(
            request.query, top_k=request.top_k, filters=filters if filters else None
        )

        # Track search metrics
        duration_ms = (time.time() - start_time) * 1000
        search_duration.observe(duration_ms / 1000)
        track_search(entity_type=request.entity_type or "", has_filter=bool(filters))

        # Log search
        log_search(logger, request.query, len(results), duration_ms, request.entity_type)

        return {"query": request.query, "results_count": len(results), "results": results}

    except Exception as e:
        log_error(logger, e, "semantic search", query_length=len(request.query))
        raise HTTPException(status_code=500, detail=str(e)) from e


# ===== Knowledge Graph Endpoints =====


@app.get("/graph/{entity_id}")
@limiter.limit(RateLimits.DEFAULT)
async def get_entity(
    entity_id: str, request: Request, api_key: str = Depends(auth_handler.verify_api_key)
):
    """Get entity details by ID"""
    start_time = time.time()

    if state.knowledge_graph is None:
        raise HTTPException(status_code=503, detail="Not ready")
    entity = state.knowledge_graph.get_entity(entity_id)

    if not entity:
        raise HTTPException(status_code=404, detail=f"Entity not found: {entity_id}")

    graph_query_duration.observe(time.time() - start_time)
    return entity


@app.post("/graph/neighbors")
@limiter.limit(RateLimits.DEFAULT)
async def get_neighbors(
    req: Request, request: GraphQueryRequest, api_key: str = Depends(auth_handler.verify_api_key)
):
    """Get neighbors of an entity"""
    start_time = time.time()

    if state.knowledge_graph is None:
        raise HTTPException(status_code=503, detail="Not ready")
    neighbors = state.knowledge_graph.get_neighbors(request.entity_id, request.relation_type or "")

    # Enrich with entity details
    neighbor_details = []
    for neighbor_id in neighbors:
        entity = state.knowledge_graph.get_entity(neighbor_id)
        if entity:
            neighbor_details.append(
                {
                    "id": neighbor_id,
                    "title": entity.get("title", entity.get("name", "Unknown")),
                    "type": entity.get("type", "unknown"),
                }
            )

    graph_query_duration.observe(time.time() - start_time)

    return {
        "entity_id": request.entity_id,
        "relation_type": request.relation_type,
        "neighbor_count": len(neighbor_details),
        "neighbors": neighbor_details,
    }


@app.post("/graph/path")
@limiter.limit(RateLimits.DEFAULT)
async def find_shortest_path(
    req: Request, request: PathRequest, api_key: str = Depends(auth_handler.verify_api_key)
):
    """Find shortest path between two entities"""
    if state.knowledge_graph is None:
        raise HTTPException(status_code=503, detail="Not ready")
    path = state.knowledge_graph.find_shortest_path(
        request.from_id, request.to_id, max_depth=request.max_depth
    )

    if not path:
        return {"from_id": request.from_id, "to_id": request.to_id, "path_found": False, "path": []}

    # Enrich path with entity details
    path_details = []
    for entity_id in path:
        entity = state.knowledge_graph.get_entity(entity_id)
        if entity:
            path_details.append(
                {
                    "id": entity_id,
                    "title": entity.get("title", entity.get("name", "Unknown")),
                    "type": entity.get("type", "unknown"),
                }
            )

    return {
        "from_id": request.from_id,
        "to_id": request.to_id,
        "path_found": True,
        "hops": len(path) - 1,
        "path": path_details,
    }


@app.post("/graph/subgraph")
@limiter.limit(RateLimits.DEFAULT)
async def extract_subgraph(
    req: Request, request: SubgraphRequest, api_key: str = Depends(auth_handler.verify_api_key)
):
    """Extract subgraph around an entity"""
    if state.knowledge_graph is None:
        raise HTTPException(status_code=503, detail="Not ready")
    nodes, edges = state.knowledge_graph.extract_subgraph(request.center_id, radius=request.radius)

    # Enrich nodes with entity details
    node_details = []
    for node_id in nodes:
        entity = state.knowledge_graph.get_entity(node_id)
        if entity:
            node_details.append(
                {
                    "id": node_id,
                    "title": entity.get("title", entity.get("name", "Unknown")),
                    "type": entity.get("type", "unknown"),
                }
            )

    # Convert edges to dict
    edge_details = []
    for edge in edges:
        edge_details.append(
            {"from": edge.from_id, "to": edge.to_id, "relation": edge.relation_type}
        )

    return {
        "center_id": request.center_id,
        "radius": request.radius,
        "node_count": len(node_details),
        "edge_count": len(edge_details),
        "nodes": node_details,
        "edges": edge_details,
    }


@app.get("/graph/contradictions")
@limiter.limit(RateLimits.DEFAULT)
async def find_contradictions(
    request: Request, api_key: str = Depends(auth_handler.verify_api_key)
):
    """Find entities with conflicting relations"""
    if state.knowledge_graph is None:
        raise HTTPException(status_code=503, detail="Not ready")
    contradictions = state.knowledge_graph.find_contradictions()

    return {"contradiction_count": len(contradictions), "contradictions": contradictions}


@app.get("/graph/infer")
@limiter.limit(RateLimits.DEFAULT)
async def infer_transitive_relations(
    request: Request, api_key: str = Depends(auth_handler.verify_api_key)
):
    """Infer transitive enforcement relationships"""
    if state.knowledge_graph is None:
        raise HTTPException(status_code=503, detail="Not ready")
    inferred = state.knowledge_graph.infer_transitive_enforcements()

    # Enrich with entity details
    inferred_details = []
    for rf_id, smell_id, law_id in inferred:
        rf = state.knowledge_graph.get_entity(rf_id)
        smell = state.knowledge_graph.get_entity(smell_id)
        law = state.knowledge_graph.get_entity(law_id)

        inferred_details.append(
            {
                "refactoring": {
                    "id": rf_id,
                    "title": rf.get("title", "Unknown") if rf else "Unknown",
                },
                "smell": {
                    "id": smell_id,
                    "name": smell.get("name", "Unknown") if smell else "Unknown",
                },
                "law": {"id": law_id, "title": law.get("title", "Unknown") if law else "Unknown"},
            }
        )

    return {"inferred_count": len(inferred_details), "inferred_relations": inferred_details}


# ===== Debug Endpoints (Development Only) =====


@app.get("/debug/profile")
async def performance_profile(request: Request):
    """
    Get performance profiling information (development only)

    Requires ENABLE_DEBUG_ENDPOINTS=true environment variable
    """
    if not ENABLE_DEBUG_ENDPOINTS:
        raise HTTPException(status_code=404, detail="Not found")

    import psutil

    # Get process info
    process = psutil.Process()
    memory_info = process.memory_info()

    # Get component stats
    graph_stats = state.knowledge_graph.stats() if state.knowledge_graph else {}

    profile_data = {
        "process": {
            "pid": process.pid,
            "cpu_percent": process.cpu_percent(interval=0.1),
            "memory_rss_mb": round(memory_info.rss / 1024 / 1024, 2),
            "memory_vms_mb": round(memory_info.vms / 1024 / 1024, 2),
            "memory_percent": process.memory_percent(),
            "num_threads": process.num_threads(),
            "create_time": process.create_time(),
        },
        "system": {
            "cpu_count": psutil.cpu_count(),
            "cpu_percent": psutil.cpu_percent(interval=0.1, percpu=True),
            "memory_total_gb": round(psutil.virtual_memory().total / 1024 / 1024 / 1024, 2),
            "memory_available_gb": round(psutil.virtual_memory().available / 1024 / 1024 / 1024, 2),
            "memory_percent": psutil.virtual_memory().percent,
        },
        "components": {
            "knowledge_graph": graph_stats,
            "loaded": {
                "smell_detector": state.smell_detector is not None,
                "refactoring_engine": state.refactoring_engine is not None,
                "knowledge_graph": state.knowledge_graph is not None,
                "rag_system": state.rag_system is not None,
            },
        },
    }

    return profile_data


# ===== Error Handlers =====


@app.exception_handler(HTTPException)
async def http_exception_handler(request: Request, exc: HTTPException):
    """Custom HTTP exception handler"""
    logger.warning(
        f"HTTP {exc.status_code} error",
        extra={"status_code": exc.status_code, "path": str(request.url.path), "detail": exc.detail},
    )

    return JSONResponse(
        status_code=exc.status_code,
        content={
            "error": exc.detail,
            "status_code": exc.status_code,
            "path": str(request.url.path),
        },
    )


@app.exception_handler(Exception)
async def general_exception_handler(request: Request, exc: Exception):
    """General exception handler"""
    log_error(logger, exc, "unhandled exception", path=str(request.url.path))

    return JSONResponse(
        status_code=500,
        content={
            "error": "Internal server error",
            "detail": str(exc),
            "path": str(request.url.path),
        },
    )


# ===== Main =====


def start():
    """Start the API server (entry point for syntagma-api command)"""
    import uvicorn

    from syntagma.config import API_HOST, API_PORT, LOG_LEVEL

    uvicorn.run(
        "syntagma.api.main:app",
        host=API_HOST,
        port=API_PORT,
        log_level=LOG_LEVEL.lower(),
    )


if __name__ == "__main__":
    import uvicorn

    uvicorn.run("syntagma.api.main:app", host="0.0.0.0", port=8000, reload=True, log_level="info")
