use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, Json},
    routing::get,
    Router,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::server::mcp_handler::SyntagmaMCP;

/// Build the web viewer router.
pub fn web_router(handler: Arc<SyntagmaMCP>) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/api/graph/full", get(graph_full))
        .route("/api/graph/entity/{id}", get(graph_entity))
        .route("/api/graph/path/{from}/{to}", get(graph_path))
        .route("/api/entities/search", get(entities_search))
        .with_state(handler)
}

async fn index() -> Html<&'static str> {
    Html(GRAPH_HTML)
}

/// Full graph as Cytoscape.js elements.
async fn graph_full(State(mcp): State<Arc<SyntagmaMCP>>) -> Json<serde_json::Value> {
    let graph = mcp.graph();
    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    for id in graph.all_entity_ids() {
        let Some(entity) = graph.get_entity(&id) else {
            continue;
        };

        nodes.push(serde_json::json!({
            "data": {
                "id": id,
                "label": entity.title,
                "type": entity.r#type,
                "category": entity.category,
            }
        }));

        for edge in graph.get_all_edges(&id) {
            if !graph.get_entity(&edge.to_id).is_some() {
                continue;
            }
            edges.push(serde_json::json!({
                "data": {
                    "id": format!("{}-{}-{}", edge.from_id, edge.relation_type, edge.to_id),
                    "source": edge.from_id,
                    "target": edge.to_id,
                    "label": edge.relation_type,
                }
            }));
        }
    }

    Json(serde_json::json!({"nodes": nodes, "edges": edges}))
}

#[derive(Deserialize)]
struct EntityParams {
    #[serde(default = "default_radius")]
    radius: usize,
}

fn default_radius() -> usize {
    2
}

async fn graph_entity(
    Path(id): Path<String>,
    Query(params): Query<EntityParams>,
    State(mcp): State<Arc<SyntagmaMCP>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let graph = mcp.graph();
    let (ids, edges) = graph.extract_subgraph(&id, params.radius);

    if ids.is_empty() {
        return Err(StatusCode::NOT_FOUND);
    }

    let nodes: Vec<_> = ids
        .iter()
        .filter_map(|eid| {
            graph.get_entity(eid).map(|e| {
                serde_json::json!({
                    "data": {
                        "id": eid,
                        "label": e.title,
                        "type": e.r#type,
                    }
                })
            })
        })
        .collect();

    let edge_data: Vec<_> = edges
        .iter()
        .map(|e| {
            serde_json::json!({
                "data": {
                    "id": format!("{}-{}-{}", e.from_id, e.relation_type, e.to_id),
                    "source": e.from_id,
                    "target": e.to_id,
                    "label": e.relation_type,
                }
            })
        })
        .collect();

    Ok(Json(serde_json::json!({"nodes": nodes, "edges": edge_data})))
}

#[derive(Deserialize)]
struct PathQueryParams {
    #[serde(default = "default_max_depth")]
    max_depth: usize,
}

fn default_max_depth() -> usize {
    5
}

async fn graph_path(
    Path((from, to)): Path<(String, String)>,
    Query(params): Query<PathQueryParams>,
    State(mcp): State<Arc<SyntagmaMCP>>,
) -> Json<serde_json::Value> {
    let graph = mcp.graph();
    match graph.find_shortest_path(&from, &to, params.max_depth) {
        Some(path) => {
            let nodes: Vec<_> = path
                .iter()
                .filter_map(|id| {
                    graph.get_entity(id).map(|e| {
                        serde_json::json!({
                            "data": { "id": id, "label": e.title }
                        })
                    })
                })
                .collect();
            let edges: Vec<_> = path
                .windows(2)
                .map(|w| {
                    serde_json::json!({
                        "data": { "source": w[0], "target": w[1] }
                    })
                })
                .collect();
            Json(serde_json::json!({"nodes": nodes, "edges": edges, "path": path}))
        }
        None => Json(serde_json::json!({"nodes": [], "edges": [], "path": []})),
    }
}

#[derive(Deserialize)]
struct SearchParams {
    q: String,
}

async fn entities_search(
    Query(params): Query<SearchParams>,
    State(mcp): State<Arc<SyntagmaMCP>>,
) -> Json<Vec<serde_json::Value>> {
    let graph = mcp.graph();
    let query = params.q.to_lowercase();
    let results: Vec<_> = graph
        .all_entity_ids()
        .iter()
        .filter_map(|id| {
            graph.get_entity(id).and_then(|e| {
                let title_match = e.title.to_lowercase().contains(&query);
                let name_match = e.name.to_lowercase().contains(&query);
                if title_match || name_match {
                    Some(serde_json::json!({
                        "id": id,
                        "title": e.title,
                        "type": e.r#type,
                    }))
                } else {
                    None
                }
            })
        })
        .take(20)
        .collect();
    Json(results)
}

const GRAPH_HTML: &str = r#"<!DOCTYPE html>
<html><head><meta charset="utf-8"><title>Syntagma Knowledge Graph</title>
<script src="https://cdnjs.cloudflare.com/ajax/libs/cytoscape/3.28.1/cytoscape.min.js"></script>
<style>
body { margin: 0; font-family: system-ui; overflow: hidden; }
#controls { position: fixed; top: 10px; left: 10px; z-index: 100; background: white; padding: 10px; border-radius: 8px; box-shadow: 0 2px 8px rgba(0,0,0,0.15); }
#cy { position: absolute; top: 0; left: 0; width: 100%; height: 100%; }
input { padding: 6px; margin: 4px; border: 1px solid #ccc; border-radius: 4px; }
button { padding: 6px 12px; margin: 4px; border: 1px solid #ccc; border-radius: 4px; cursor: pointer; background: #f8f8f8; }
button:hover { background: #e8e8e8; }
#status { font-size: 12px; color: #666; }
#status.error { color: #d32f2f; }
</style></head><body>
<div id="controls">
  <input id="search" placeholder="Search entities..." size="30">
  <button onclick="loadFull()">Full Graph</button>
  <button onclick="loadEntity()">Subgraph</button>
  <button onclick="findPath()">Find Path</button>
  <br><span id="status">Initializing...</span>
</div>
<div id="cy"></div>
<script>
(function(){
  const statusEl = document.getElementById('status');
  function setStatus(msg, isError) {
    statusEl.textContent = msg;
    statusEl.className = isError ? 'error' : '';
  }

  if (typeof cytoscape === 'undefined') {
    setStatus('Failed to load Cytoscape.js — check your network connection', true);
    return;
  }

  const typeColors = {pattern:'#4CAF50',refactoring:'#2196F3',law:'#FF9800',smell:'#f44336'};
  let cy = cytoscape({container: document.getElementById('cy'),
    style:[
      {selector:'node', style:{'label':'data(label)','text-wrap':'wrap','text-max-width':'120px',
        'background-color': function(ele){ var c=typeColors[ele.data('type')]; return c||'#9E9E9E'; },
        'font-size':'11px','width':40,'height':40}},
      {selector:'edge', style:{'width':2,'line-color':'#aaa','target-arrow-color':'#aaa',
        'target-arrow-shape':'triangle','opacity':0.6,'label':'data(label)','font-size':'9px'}}
    ]
  });

  function addElements(data) {
    if (!data || !data.nodes || !data.edges) {
      throw new Error('Invalid response format');
    }
    cy.elements().remove();
    cy.add({nodes: data.nodes, edges: data.edges});
  }

  window.loadFull = function() {
    setStatus('Loading full graph...');
    fetch('/api/graph/full')
      .then(function(r) {
        if (!r.ok) throw new Error('Server returned ' + r.status);
        return r.json();
      })
      .then(function(d) {
        addElements(d);
        cy.layout({name:'cose', animate:true, randomize:true}).run();
        setStatus(d.nodes.length + ' nodes, ' + d.edges.length + ' edges');
      })
      .catch(function(e) { setStatus('Error: ' + e.message, true); });
  };

  window.loadEntity = function() {
    var id = prompt('Entity ID:','DP-005');
    if (!id) return;
    setStatus('Loading subgraph...');
    fetch('/api/graph/entity/' + encodeURIComponent(id) + '?radius=2')
      .then(function(r) {
        if (!r.ok) throw new Error('Not found (status ' + r.status + ')');
        return r.json();
      })
      .then(function(d) {
        addElements(d);
        cy.layout({name:'cose', animate:true}).run();
        setStatus('Subgraph: ' + d.nodes.length + ' nodes');
      })
      .catch(function(e) { setStatus('Error: ' + e.message, true); });
  };

  window.findPath = function() {
    var f = prompt('From:','SMELL-01'), t = prompt('To:','RF-001');
    if (!f || !t) return;
    setStatus('Finding path...');
    fetch('/api/graph/path/' + encodeURIComponent(f) + '/' + encodeURIComponent(t))
      .then(function(r) {
        if (!r.ok) throw new Error('Server returned ' + r.status);
        return r.json();
      })
      .then(function(d) {
        addElements(d);
        cy.layout({name:'breadthfirst', animate:true}).run();
        setStatus('Path length: ' + (d.path ? d.path.length - 1 : 0));
      })
      .catch(function(e) { setStatus('Error: ' + e.message, true); });
  };

  document.getElementById('search').addEventListener('input', function(e) {
    var q = e.target.value.toLowerCase();
    if (q.length < 2) { cy.elements().style('opacity', 1); return; }
    cy.nodes().forEach(function(n) {
      var label = n.data('label');
      n.style('opacity', label && label.toLowerCase().includes(q) ? 1 : 0.15);
    });
  });

  loadFull();
})();
</script></body></html>"#;
