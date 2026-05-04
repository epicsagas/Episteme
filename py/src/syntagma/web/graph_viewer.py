#!/usr/bin/env python3
"""
Graph Visualization Server
Interactive knowledge graph viewer with Cytoscape.js
"""

from pathlib import Path

from fastapi import FastAPI, Request
from fastapi.responses import HTMLResponse, JSONResponse
from fastapi.staticfiles import StaticFiles
from fastapi.templating import Jinja2Templates

from syntagma.graph.api import KnowledgeGraph

app = FastAPI(title="Syntagma Graph Viewer", version="0.0.1")

# Setup static files and templates
static_path = Path(__file__).parent / "static"
templates_path = Path(__file__).parent / "templates"
static_path.mkdir(exist_ok=True)
templates_path.mkdir(exist_ok=True)

app.mount("/static", StaticFiles(directory=str(static_path)), name="static")
templates = Jinja2Templates(directory=str(templates_path))

# Initialize knowledge graph (after app/mount setup)
from syntagma.config import BASE_DIR  # noqa: E402

kg = KnowledgeGraph(str(BASE_DIR))


@app.get("/", response_class=HTMLResponse)
async def index(request: Request):
    """Main graph viewer page"""
    return templates.TemplateResponse(
        "index.html",
        {
            "request": request,
            "title": "Syntagma Knowledge Graph",
            "stats": kg.stats(),
        },
    )


@app.get("/api/graph/full")
async def get_full_graph() -> JSONResponse:
    """Get complete graph data in Cytoscape.js format"""
    entities = kg.entities

    # Build nodes
    nodes = []
    for entity_id, entity_data in entities.items():
        node = {
            "data": {
                "id": entity_id,
                "label": entity_data.get("title", entity_id),
                "type": entity_data.get("type", "unknown"),
                "category": entity_data.get("category", ""),
            }
        }
        nodes.append(node)

    # Build edges from relations
    edges = []
    edge_id = 0
    for entity_id, entity_data in entities.items():
        relations = entity_data.get("relations", {})

        for rel_type, targets in relations.items():
            if not isinstance(targets, list):
                continue

            for target_id in targets:
                if target_id in entities:
                    edge = {
                        "data": {
                            "id": f"e{edge_id}",
                            "source": entity_id,
                            "target": target_id,
                            "type": rel_type,
                        }
                    }
                    edges.append(edge)
                    edge_id += 1

    return JSONResponse(
        {
            "elements": {
                "nodes": nodes,
                "edges": edges,
            },
            "stats": {
                "total_nodes": len(nodes),
                "total_edges": len(edges),
            },
        }
    )


@app.get("/api/graph/entity/{entity_id}")
async def get_entity_subgraph(entity_id: str, radius: int = 2) -> JSONResponse:
    """Get subgraph centered on specific entity"""
    try:
        node_ids, graph_edges = kg.extract_subgraph(entity_id, radius=radius)

        # Convert to Cytoscape.js format
        nodes = []
        for eid in node_ids:
            entity_data = kg.entities.get(eid, {})
            nodes.append(
                {
                    "data": {
                        "id": eid,
                        "label": entity_data.get("title", eid),
                        "type": entity_data.get("type", "unknown"),
                        "category": entity_data.get("category", ""),
                        "is_center": eid == entity_id,
                    }
                }
            )

        edges = []
        for edge_id, edge_data in enumerate(graph_edges):
            edges.append(
                {
                    "data": {
                        "id": f"e{edge_id}",
                        "source": edge_data.from_id,
                        "target": edge_data.to_id,
                        "type": edge_data.relation_type,
                    }
                }
            )

        return JSONResponse(
            {
                "elements": {
                    "nodes": nodes,
                    "edges": edges,
                },
                "center": entity_id,
            }
        )

    except Exception as e:
        return JSONResponse({"error": str(e)}, status_code=404)


@app.get("/api/graph/path/{from_id}/{to_id}")
async def get_shortest_path(from_id: str, to_id: str) -> JSONResponse:
    """Get shortest path between two entities"""
    try:
        path_nodes = kg.find_shortest_path(from_id, to_id)

        if path_nodes is None:
            return JSONResponse({"error": "No path found"}, status_code=404)

        # Convert to Cytoscape.js format
        nodes = []
        for eid in path_nodes:
            entity_data = kg.entities.get(eid, {})
            nodes.append(
                {
                    "data": {
                        "id": eid,
                        "label": entity_data.get("title", eid),
                        "type": entity_data.get("type", "unknown"),
                        "is_path": True,
                    }
                }
            )

        edges = []
        for i in range(len(path_nodes) - 1):
            edges.append(
                {
                    "data": {
                        "id": f"path_e{i}",
                        "source": path_nodes[i],
                        "target": path_nodes[i + 1],
                        "type": "path",
                    }
                }
            )

        return JSONResponse(
            {
                "elements": {
                    "nodes": nodes,
                    "edges": edges,
                },
                "path": path_nodes,
                "length": len(path_nodes) - 1,
            }
        )

    except Exception as e:
        return JSONResponse({"error": str(e)}, status_code=404)


@app.get("/api/entities/search")
async def search_entities(q: str) -> JSONResponse:
    """Search entities by title"""
    results = []
    query_lower = q.lower()

    for entity_id, entity_data in kg.entities.items():
        title = entity_data.get("title", "")
        if query_lower in title.lower():
            results.append(
                {
                    "id": entity_id,
                    "title": title,
                    "type": entity_data.get("type", "unknown"),
                }
            )

    return JSONResponse({"results": results[:20]})


def start(host: str = "127.0.0.1", port: int = 8001) -> None:
    import uvicorn

    print(f"Graph Viewer running at http://{host}:{port}")
    uvicorn.run(app, host=host, port=port, log_level="info")


if __name__ == "__main__":
    start()
