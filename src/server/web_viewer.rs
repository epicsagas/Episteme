use axum::{
    Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, Json},
    routing::get,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::server::mcp_handler::EpistemeMCP;

/// Build the web viewer router.
pub fn web_router(handler: Arc<EpistemeMCP>) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/api/graph/full", get(graph_full))
        .route("/api/graph/entity/{id}", get(graph_entity))
        .route("/api/graph/path/{from}/{to}", get(graph_path))
        .route("/api/graph/sankey", get(graph_sankey))
        .route("/api/graph/tree", get(graph_tree))
        .route("/api/entities/search", get(entities_search))
        .with_state(handler)
}

async fn index() -> Html<&'static str> {
    Html(GRAPH_HTML)
}

/// Full graph as Cytoscape.js elements.
async fn graph_full(State(mcp): State<Arc<EpistemeMCP>>) -> Json<serde_json::Value> {
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
                "description": entity.description,
                "type": entity.r#type,
                "category": entity.category,
            }
        }));

        for edge in graph.get_all_edges(&id) {
            if graph.get_entity(&edge.to_id).is_none() {
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
    State(mcp): State<Arc<EpistemeMCP>>,
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

    Ok(Json(
        serde_json::json!({"nodes": nodes, "edges": edge_data}),
    ))
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
    State(mcp): State<Arc<EpistemeMCP>>,
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
    State(mcp): State<Arc<EpistemeMCP>>,
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

// ---------------------------------------------------------------------------
// Sankey & Tree API endpoints
// ---------------------------------------------------------------------------

/// Relation types included in the sankey diagram.
const SANKEY_RELATIONS: &[&str] = &["solves", "solved_by", "enforces", "violates"];

/// Human-readable labels for entity types.
fn type_label(t: &str) -> &str {
    match t {
        "smell" => "Code Smells",
        "refactoring" => "Refactorings",
        "law" => "Laws & Principles",
        "pattern" => "Design Patterns",
        _ => t,
    }
}

/// GET /api/graph/sankey -- aggregated flow data for a sankey diagram.
async fn graph_sankey(State(mcp): State<Arc<EpistemeMCP>>) -> Json<serde_json::Value> {
    let graph = mcp.graph();

    // Count entities per type.
    let mut type_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    for id in graph.all_entity_ids() {
        if let Some(entity) = graph.get_entity(&id) {
            let t = if entity.r#type.is_empty() {
                "unknown"
            } else {
                &entity.r#type
            };
            *type_counts.entry(t.to_owned()).or_insert(0) += 1;
        }
    }

    let nodes: Vec<serde_json::Value> = type_counts
        .iter()
        .map(|(t, count)| {
            serde_json::json!({
                "id": t,
                "label": type_label(t),
                "count": count,
            })
        })
        .collect();

    // Aggregate edges by (source_type, relation_type, target_type).
    let mut link_counts: std::collections::HashMap<(String, String, String), usize> =
        std::collections::HashMap::new();

    for id in graph.all_entity_ids() {
        let Some(entity) = graph.get_entity(&id) else {
            continue;
        };
        let source_type = if entity.r#type.is_empty() {
            "unknown"
        } else {
            &entity.r#type
        };

        for edge in graph.get_all_edges(&id) {
            if !SANKEY_RELATIONS.contains(&edge.relation_type.as_str()) {
                continue;
            }
            let Some(target) = graph.get_entity(&edge.to_id) else {
                continue;
            };
            let target_type = if target.r#type.is_empty() {
                "unknown"
            } else {
                &target.r#type
            };

            *link_counts
                .entry((
                    source_type.to_owned(),
                    edge.relation_type.clone(),
                    target_type.to_owned(),
                ))
                .or_insert(0) += 1;
        }
    }

    let links: Vec<serde_json::Value> = link_counts
        .iter()
        .map(|((src, rel, tgt), count)| {
            serde_json::json!({
                "source": src,
                "target": tgt,
                "relation": rel,
                "value": count,
            })
        })
        .collect();

    Json(serde_json::json!({"nodes": nodes, "links": links}))
}

/// GET /api/graph/tree -- entity tree grouped by type then category.
async fn graph_tree(State(mcp): State<Arc<EpistemeMCP>>) -> Json<serde_json::Value> {
    use std::collections::HashMap;

    let graph = mcp.graph();

    // Group: type -> category -> Vec of (id, title).
    let mut by_type: HashMap<String, HashMap<String, Vec<(String, String)>>> = HashMap::new();

    for id in graph.all_entity_ids() {
        let Some(entity) = graph.get_entity(&id) else {
            continue;
        };
        let t = if entity.r#type.is_empty() {
            "unknown"
        } else {
            &entity.r#type
        };
        let cat = if entity.category.is_empty() {
            "uncategorized"
        } else {
            &entity.category
        };

        by_type
            .entry(t.to_owned())
            .or_default()
            .entry(cat.to_owned())
            .or_default()
            .push((id, entity.title.clone()));
    }

    // Order types consistently.
    let type_order = ["pattern", "refactoring", "law", "smell"];
    let mut sorted_types: Vec<String> = by_type.keys().cloned().collect();
    sorted_types.sort_by(|a, b| {
        let ai = type_order
            .iter()
            .position(|t| *t == a.as_str())
            .unwrap_or(999);
        let bi = type_order
            .iter()
            .position(|t| *t == b.as_str())
            .unwrap_or(999);
        ai.cmp(&bi)
    });

    let tree: Vec<serde_json::Value> = sorted_types
        .iter()
        .map(|t| {
            let categories = by_type.get(t).unwrap();
            let mut sorted_cats: Vec<&String> = categories.keys().collect();
            sorted_cats.sort();

            let children: Vec<serde_json::Value> = sorted_cats
                .iter()
                .map(|cat| {
                    let mut items = categories.get(*cat).unwrap().clone();
                    items.sort_by(|a, b| a.0.cmp(&b.0));

                    let leaf_children: Vec<serde_json::Value> = items
                        .iter()
                        .map(|(id, title)| {
                            let desc = graph
                                .get_entity(id)
                                .map(|e| e.description.clone())
                                .unwrap_or_default();
                            serde_json::json!({"id": id, "title": title, "description": desc})
                        })
                        .collect();

                    let cat_display = if cat.as_str() == "uncategorized" {
                        "Uncategorized".to_owned()
                    } else {
                        // Capitalize first letter
                        let mut c = cat.chars();
                        match c.next() {
                            None => String::new(),
                            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                        }
                    };

                    serde_json::json!({
                        "category": cat,
                        "label": cat_display,
                        "children": leaf_children,
                    })
                })
                .collect();

            serde_json::json!({
                "type": t,
                "label": type_label(t),
                "children": children,
            })
        })
        .collect();

    Json(serde_json::json!({"tree": tree}))
}

const GRAPH_HTML: &str = r#"<!DOCTYPE html>
<html><head><meta charset="utf-8"><title>Episteme Knowledge Graph</title>
<script src="https://cdnjs.cloudflare.com/ajax/libs/cytoscape/3.28.1/cytoscape.min.js"></script>
<style>
*{box-sizing:border-box;margin:0;padding:0}
body{font-family:system-ui,-apple-system,sans-serif;background:#0f1117;color:#e0e0e0;overflow:hidden;height:100vh;display:flex;flex-direction:column}
#toolbar{display:flex;align-items:center;gap:8px;padding:8px 16px;background:#1a1b23;border-bottom:1px solid #2a2b35;z-index:10;flex-shrink:0}
#toolbar h1{font-size:14px;font-weight:600;color:#8b8fa3;margin-right:12px;white-space:nowrap}
#toolbar input{flex:1;max-width:320px;padding:6px 10px;background:#12131a;border:1px solid #2a2b35;border-radius:6px;color:#e0e0e0;font-size:13px;outline:none}
#toolbar input:focus{border-color:#4a9eff}
.filter-btn{padding:4px 10px;border-radius:4px;border:1px solid #2a2b35;background:#12131a;color:#8b8fa3;font-size:11px;cursor:pointer;transition:all .15s}
.filter-btn.active{border-color:#4a9eff;color:#4a9eff;background:#1a2333}
.filter-btn:hover{border-color:#4a9eff80}
#content{display:flex;flex:1;overflow:hidden}
#sidebar{width:240px;min-width:240px;background:#1a1b23;border-right:1px solid #2a2b35;overflow-y:auto;flex-shrink:0}
.tree-section{padding:8px 0}
.tree-header{padding:6px 12px;font-size:11px;font-weight:600;text-transform:uppercase;letter-spacing:.5px;cursor:pointer;display:flex;align-items:center;gap:6px;user-select:none}
.tree-header:hover{background:#ffffff08}
.tree-header .arrow{transition:transform .15s;font-size:8px}
.tree-header .arrow.open{transform:rotate(90deg)}
.tree-children{display:none}
.tree-children.open{display:block}
.tree-cat{padding:4px 12px 4px 24px;font-size:12px;color:#8b8fa3;cursor:pointer;display:flex;align-items:center;gap:6px}
.tree-cat:hover{background:#ffffff08;color:#c0c4d8}
.tree-cat .arrow{font-size:8px;transition:transform .15s}
.tree-cat .arrow.open{transform:rotate(90deg)}
.tree-leaf{padding:3px 12px 3px 40px;font-size:12px;color:#6b7080;cursor:pointer;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
.tree-leaf:hover{background:#ffffff08;color:#c0c4d8}
.tree-leaf.active{color:#4a9eff;background:#1a233380}
.type-dot{width:8px;height:8px;border-radius:50%;flex-shrink:0}
#main{flex:1;position:relative;overflow:hidden}
#sankey-view,#cy-view{position:absolute;top:0;left:0;width:100%;height:100%}
#cy-view{display:none}
#cy{width:100%;height:100%}
#detail{position:absolute;right:0;top:0;width:300px;height:100%;background:#1a1b23;border-left:1px solid #2a2b35;overflow-y:auto;transform:translateX(100%);transition:transform .2s ease;z-index:5}
#detail.open{transform:translateX(0)}
#detail .close{position:absolute;top:8px;right:8px;cursor:pointer;font-size:18px;color:#6b7080;width:24px;height:24px;display:flex;align-items:center;justify-content:center;border-radius:4px}
#detail .close:hover{background:#ffffff10;color:#e0e0e0}
.detail-content{padding:16px}
.detail-id{font-size:11px;color:#6b7080;font-family:monospace}
.detail-title{font-size:16px;font-weight:600;margin:4px 0 8px;color:#e0e0e0}
.detail-meta{display:flex;gap:8px;margin-bottom:12px}
.detail-badge{padding:2px 8px;border-radius:4px;font-size:11px;background:#2a2b35;color:#8b8fa3}
.detail-section{margin-bottom:12px}
.detail-section h3{font-size:11px;text-transform:uppercase;letter-spacing:.5px;color:#6b7080;margin-bottom:6px}
.detail-rel{padding:4px 8px;margin:2px 0;background:#12131a;border-radius:4px;font-size:12px;cursor:pointer;display:flex;justify-content:space-between;align-items:center}
.detail-rel:hover{background:#1a2333}
.detail-rel .rel-type{color:#6b7080;font-size:10px}
#status{font-size:11px;color:#6b7080;margin-left:auto;white-space:nowrap}
svg text{font-family:system-ui,-apple-system,sans-serif}
</style></head><body>
<div id="toolbar">
  <h1>Episteme</h1>
  <input id="search" placeholder="Search entities..." autocomplete="off">
  <button class="filter-btn active" data-rel="solves">solves</button>
  <button class="filter-btn active" data-rel="solved_by">solved_by</button>
  <button class="filter-btn active" data-rel="enforces">enforces</button>
  <button class="filter-btn active" data-rel="violates">violates</button>
  <button class="filter-btn active" data-rel="related_to">related_to</button>
  <button id="home-btn" class="filter-btn" style="margin-left:4px">Home</button>
  <span id="status">Loading...</span>
</div>
<div id="content">
  <div id="sidebar"></div>
  <div id="main">
    <div id="sankey-view"><svg id="sankey-svg" width="100%" height="100%"></svg></div>
    <div id="cy-view"><div id="cy"></div></div>
    <div id="detail">
      <div class="close" onclick="closeDetail()">&times;</div>
      <div class="detail-content" id="detail-content"></div>
    </div>
  </div>
</div>
<script>
(function(){
var typeColors={pattern:'#4CAF50',refactoring:'#2196F3',law:'#FF9800',smell:'#f44336'};
var typeColorsDark={pattern:'#2E7D32',refactoring:'#1565C0',law:'#E65100',smell:'#C62828'};
var typeIcons={pattern:'◆',refactoring:'⟳',law:'§',smell:'⚠'};
var entityMap={};
var activeRelFilters=new Set(['solves','solved_by','enforces','violates','related_to']);
var sankeyData=null,treeData=null;
var cy=null;

document.querySelectorAll('.filter-btn[data-rel]').forEach(function(btn){
  btn.addEventListener('click',function(){
    var rel=this.dataset.rel;
    if(activeRelFilters.has(rel)){activeRelFilters.delete(rel);this.classList.remove('active')}
    else{activeRelFilters.add(rel);this.classList.add('active')}
    if(sankeyData)renderSankey(sankeyData);
    if(cy)filterCyEdges();
  });
});
document.getElementById('home-btn').addEventListener('click',function(){showSankey()});

function setStatus(msg){document.getElementById('status').textContent=msg}

// ---- Data loading ----
function loadAll(){
  Promise.all([
    fetch('/api/graph/sankey').then(function(r){return r.json()}),
    fetch('/api/graph/tree').then(function(r){return r.json()}),
    fetch('/api/graph/full').then(function(r){return r.json()})
  ]).then(function(results){
    sankeyData=results[0];
    treeData=results[1];
    var full=results[2];
    full.nodes.forEach(function(n){
      var d=n.data;
      entityMap[d.id]={id:d.id,title:d.label,description:d.description||'',type:d.type,category:d.category||''};
    });
    // Store edges for detail panel
    full.edges.forEach(function(e){
      var d=e.data;
      var src=entityMap[d.source];
      if(src){
        if(!src._rels)src._rels=[];
        src._rels.push({type:d.label,target:d.target});
      }
    });
    renderTree(treeData.tree);
    renderSankey(sankeyData);
    setStatus(sankeyData.nodes.reduce(function(s,n){return s+n.count},0)+' entities loaded');
  }).catch(function(e){setStatus('Error: '+e.message)});
}

// ---- Sidebar tree ----
function renderTree(tree){
  var sb=document.getElementById('sidebar');
  sb.innerHTML='';
  tree.forEach(function(typeNode){
    var section=el('div','tree-section');
    var header=el('div','tree-header');
    header.innerHTML='<span class="type-dot" style="background:'+typeColors[typeNode.type]+'"></span>'
      +'<span>'+typeIcons[typeNode.type]+' '+typeNode.label+' ('+countLeaves(typeNode)+')</span>'
      +'<span class="arrow">▶</span>';
    var childrenWrap=el('div','tree-children');
    header.onclick=function(){
      var arr=this.querySelector('.arrow');
      arr.classList.toggle('open');
      childrenWrap.classList.toggle('open');
    };
    typeNode.children.forEach(function(catNode){
      var catEl=el('div','tree-cat');
      catEl.innerHTML='<span class="arrow">▶</span> '+catNode.label+' ('+catNode.children.length+')';
      var catChildren=el('div','tree-children');
      catEl.onclick=function(e){
        e.stopPropagation();
        this.querySelector('.arrow').classList.toggle('open');
        catChildren.classList.toggle('open');
      };
      catNode.children.forEach(function(leaf){
        var le=el('div','tree-leaf');
        le.textContent=leaf.id+' '+leaf.title;
        le.dataset.id=leaf.id;
        le.onclick=function(e){e.stopPropagation();navigateToEntity(leaf.id)};
        catChildren.appendChild(le);
      });
      childrenWrap.appendChild(catEl);
      childrenWrap.appendChild(catChildren);
    });
    section.appendChild(header);
    section.appendChild(childrenWrap);
    sb.appendChild(section);
  });
}

function countLeaves(typeNode){
  var c=0;typeNode.children.forEach(function(cat){c+=cat.children.length});return c;
}
function el(tag,cls){var e=document.createElement(tag);if(cls)e.className=cls;return e}

// ---- Sankey rendering ----
// Fixed layout: 3 columns, nodes at predetermined positions.
// Links always flow left→right: source.right → target.left via bezier.
// Slots on each node side are pre-counted then distributed evenly.
function renderSankey(data){
  var svg=document.getElementById('sankey-svg');
  svg.innerHTML='';
  var rect=svg.getBoundingClientRect();
  var W=rect.width||800,H=rect.height||600;
  if(W<200||H<200)return;
  var nodes=data.nodes;
  if(!nodes.length)return;

  // Fixed column x positions (3 columns)
  var NW=Math.min(160,Math.floor((W-80)/4));   // node width
  var GAP=Math.max(60,Math.floor((W-80-3*NW)/2)); // gap between cols
  var padL=40;
  var colX=[padL, padL+NW+GAP, padL+2*(NW+GAP)];

  // Fixed node positions by type
  var nodeH={smell:Math.min(160,H*0.28), pattern:Math.min(120,H*0.22),
             refactoring:Math.min(300,H*0.55), law:Math.min(280,H*0.52)};
  var nodeY={
    smell:   H*0.12,
    pattern: H*0.62,
    refactoring: (H - nodeH.refactoring)/2,
    law:         (H - nodeH.law)/2
  };
  var nodeCol={smell:0, pattern:0, refactoring:1, law:2};

  // Build position map
  var pos={};
  nodes.forEach(function(node){
    var col=nodeCol[node.id]!=null?nodeCol[node.id]:1;
    var x=colX[col];
    var h=nodeH[node.id]||100;
    var y=nodeY[node.id]!=null?nodeY[node.id]:(H-h)/2;
    pos[node.id]={x:x,y:y,w:NW,h:h};

    // Draw node block
    var g=document.createElementNS('http://www.w3.org/2000/svg','g');
    var bg=svgRect(x,y,NW,h,10);
    bg.setAttribute('fill',typeColorsDark[node.id]||'#222');
    bg.setAttribute('opacity','0.55');
    g.appendChild(bg);
    var border=svgRect(x,y,NW,h,10);
    border.setAttribute('fill','none');
    border.setAttribute('stroke',typeColors[node.id]||'#555');
    border.setAttribute('stroke-width','2');
    g.appendChild(border);
    // Icon
    var mid=y+h/2;
    g.appendChild(svgText(x+NW/2, mid-18, typeIcons[node.id]||'●', 22, typeColors[node.id]||'#aaa'));
    g.appendChild(svgText(x+NW/2, mid+2,  node.label, 12, '#e8e8e8'));
    g.appendChild(svgText(x+NW/2, mid+18, node.count+' entities', 10, '#6b7080'));
    g.style.cursor='pointer';
    g.addEventListener('click',function(){showTypeSubgraph(node.id)});
    g.addEventListener('mouseenter',function(){bg.setAttribute('opacity','0.8')});
    g.addEventListener('mouseleave',function(){bg.setAttribute('opacity','0.55')});
    svg.appendChild(g);
  });

  // Filter and sort links (left-col sources first)
  var links=data.links.filter(function(l){
    var sp=pos[l.source],tp=pos[l.target];
    return sp&&tp&&activeRelFilters.has(l.relation)&&sp.x<tp.x;
  });
  if(!links.length){svg.appendChild(makeLegend(W,40));return}

  var maxVal=Math.max.apply(null,links.map(function(l){return l.value}));

  // Pre-count slots per node side
  var srcCount={},tgtCount={};
  links.forEach(function(l){
    srcCount[l.source]=(srcCount[l.source]||0)+1;
    tgtCount[l.target]=(tgtCount[l.target]||0)+1;
  });
  var srcUsed={},tgtUsed={};

  // Draw links — links drawn before nodes so nodes appear on top
  var linkLayer=document.createElementNS('http://www.w3.org/2000/svg','g');
  linkLayer.setAttribute('opacity','1');
  links.forEach(function(link){
    var sp=pos[link.source],tp=pos[link.target];
    var thick=Math.max(3,Math.round(link.value/maxVal*48));

    // Source slot: right edge, distributed over 20%-80% of node height
    var si=srcUsed[link.source]||0; srcUsed[link.source]=si+1;
    var sn=srcCount[link.source];
    var sy=sp.y+sp.h*0.2 + (sp.h*0.6)*(sn>1?si/(sn-1):0.5);
    var sx=sp.x+sp.w;

    // Target slot: left edge
    var ti=tgtUsed[link.target]||0; tgtUsed[link.target]=ti+1;
    var tn=tgtCount[link.target];
    var ty=tp.y+tp.h*0.2 + (tp.h*0.6)*(tn>1?ti/(tn-1):0.5);
    var tx=tp.x;

    // Horizontal bezier — control points pulled toward source/target x
    var cpx1=sx+(tx-sx)*0.42;
    var cpx2=sx+(tx-sx)*0.58;
    var color=relColor(link.relation);

    var path=document.createElementNS('http://www.w3.org/2000/svg','path');
    path.setAttribute('d','M'+sx+','+sy+' C'+cpx1+','+sy+' '+cpx2+','+ty+' '+tx+','+ty);
    path.setAttribute('fill','none');
    path.setAttribute('stroke',color);
    path.setAttribute('stroke-width',thick);
    path.setAttribute('opacity','0.38');
    path.setAttribute('stroke-linecap','butt');
    var title=document.createElementNS('http://www.w3.org/2000/svg','title');
    title.textContent=link.relation+': '+link.value+' ('+link.source+' → '+link.target+')';
    path.appendChild(title);
    path.addEventListener('mouseenter',function(){this.setAttribute('opacity','0.85')});
    path.addEventListener('mouseleave',function(){this.setAttribute('opacity','0.38')});
    linkLayer.appendChild(path);
  });
  // Insert link layer before first child so nodes render on top
  svg.insertBefore(linkLayer,svg.firstChild);

  // Legend
  svg.appendChild(makeLegend(W,40));
}

function makeLegend(W,topY){
  var g=document.createElementNS('http://www.w3.org/2000/svg','g');
  var relTypes=['solves','solved_by','enforces','violates','related_to'];
  var lx=W-170;
  relTypes.forEach(function(rel,i){
    var ly=topY+i*20;
    var dot=document.createElementNS('http://www.w3.org/2000/svg','circle');
    dot.setAttribute('cx',lx+5);dot.setAttribute('cy',ly+6);dot.setAttribute('r',5);
    dot.setAttribute('fill',relColor(rel));dot.setAttribute('opacity','0.8');
    g.appendChild(dot);
    g.appendChild(svgText(lx+14,ly+10,rel,11,'#6b7080'));
  });
  return g;
}

function relColor(rel){
  return{solves:'#66BB6A',solved_by:'#81C784',enforces:'#42A5F5',violates:'#EF5350',related_to:'#78909C'}[rel]||'#78909C';
}
function svgRect(x,y,w,h,rx,fill,stroke){
  var r=document.createElementNS('http://www.w3.org/2000/svg','rect');
  r.setAttribute('x',x);r.setAttribute('y',y);r.setAttribute('width',w);r.setAttribute('height',h);
  if(rx)r.setAttribute('rx',rx);if(fill)r.setAttribute('fill',fill);if(stroke)r.setAttribute('stroke',stroke);
  return r;
}
function svgText(x,y,text,size,fill){
  var t=document.createElementNS('http://www.w3.org/2000/svg','text');
  t.setAttribute('x',x);t.setAttribute('y',y);t.setAttribute('text-anchor','middle');
  t.setAttribute('font-size',size||12);t.setAttribute('fill',fill||'#e0e0e0');
  t.textContent=text;return t;
}

// ---- Cytoscape subgraph view ----
function initCy(){
  cy=cytoscape({container:document.getElementById('cy'),
    style:[
      {selector:'node',style:{
        'label':'data(label)','text-wrap':'wrap','text-max-width':'140px',
        'background-color':function(ele){return typeColors[ele.data('type')]||'#9E9E9E'},
        'font-size':'12px','color':'#e0e0e0',
        'text-outline-color':'#0a0b10','text-outline-width':'3px',
        'text-valign':'bottom','text-margin-y':'8px',
        'width':56,'height':56,
        'border-width':2,'border-color':'#0f1117',
        'border-opacity':0.6}},
      {selector:'node:active',style:{'overlay-opacity':0}},
      {selector:'node.selected',style:{'border-color':'#ffffff','border-width':3,'border-opacity':1,'width':70,'height':70}},
      {selector:'edge',style:{
        'width':2,
        'line-color':function(ele){return relColor(ele.data('label'))},
        'target-arrow-color':function(ele){return relColor(ele.data('label'))},
        'target-arrow-shape':'triangle',
        'label':'',
        'opacity':0.6,'curve-style':'unbundled-bezier',
        'control-point-distances':40,'control-point-weights':0.5}},
      {selector:'edge.hovered',style:{'opacity':1,'width':3}},
      {selector:'edge.filtered',style:{'display':'none'}}
    ]
  });
  cy.on('tap','node',function(e){
    var id=e.target.id();
    showDetail(id);
    cy.nodes().removeClass('selected');
    e.target.addClass('selected');
  });
  cy.on('mouseover','edge',function(e){
    var el=e.target;
    el.addClass('hovered');
  });
  cy.on('mouseout','edge',function(e){e.target.removeClass('hovered')});
}

function loadSubgraph(entityId,radius){
  document.getElementById('sankey-view').style.display='none';
  document.getElementById('cy-view').style.display='block';
  if(!cy)initCy();
  setStatus('Loading subgraph...');
  fetch('/api/graph/entity/'+encodeURIComponent(entityId)+'?radius='+radius)
    .then(function(r){if(!r.ok)throw new Error('Not found');return r.json()})
    .then(function(d){
      cy.elements().remove();
      cy.add({nodes:d.nodes,edges:d.edges});
      cy.nodes().forEach(function(n){
        if(!n.data('type')){
          var e=entityMap[n.id()];
          if(e)n.data('type',e.type);
        }
      });
      cy.layout({name:'cose',animate:true,padding:80,
        nodeRepulsion:function(){return 120000},
        idealEdgeLength:function(){return 200},
        nodeOverlap:40,
        gravity:0.15,
        numIter:2000,
        coolingFactor:0.97
      }).run();
      filterCyEdges();
      setStatus(d.nodes.length+' nodes, '+d.edges.length+' edges');
      // Highlight center
      var center=cy.getElementById(entityId);
      if(center.length)center.addClass('selected');
      highlightLeaf(entityId);
    })
    .catch(function(e){setStatus('Error: '+e.message)});
}

function filterCyEdges(){
  if(!cy)return;
  cy.edges().forEach(function(e){
    var label=e.data('label')||'';
    if(activeRelFilters.has(label))e.removeClass('filtered');
    else e.addClass('filtered');
  });
}

function showTypeSubgraph(typeId){
  // Find first entity of this type and show subgraph
  var found=null;
  for(var id in entityMap){
    if(entityMap[id].type===typeId){found=id;break}
  }
  if(found)loadSubgraph(found,1);
}

function showSankey(){
  document.getElementById('sankey-view').style.display='block';
  document.getElementById('cy-view').style.display='none';
  closeDetail();
  if(sankeyData)renderSankey(sankeyData);
}

function navigateToEntity(id){
  loadSubgraph(id,2);
  showDetail(id);
  highlightLeaf(id);
}

function highlightLeaf(id){
  document.querySelectorAll('.tree-leaf').forEach(function(el){
    el.classList.toggle('active',el.dataset.id===id);
  });
}

// ---- Detail panel ----
function escapeHtml(text){
  return text.replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;').replace(/"/g,'&quot;').replace(/'/g,'&#39;');
}
function showDetail(id){
  var panel=document.getElementById('detail');
  var content=document.getElementById('detail-content');
  var entity=entityMap[id];
  if(!entity){content.innerHTML='<p style="color:#6b7080">Entity not found</p>';panel.classList.add('open');return}
  var color=typeColors[entity.type]||'#9E9E9E';
  var html='<div class="detail-id">'+id+'</div>';
  html+='<div class="detail-title" style="color:'+color+'">'+entity.title+'</div>';
  html+='<div class="detail-meta">';
  html+='<span class="detail-badge" style="border-color:'+color+'40;color:'+color+'">'+entity.type+'</span>';
  if(entity.category)html+='<span class="detail-badge">'+entity.category+'</span>';
  html+='</div>';
  if(entity.description){
    html+='<div class="detail-section"><h3>Description</h3>';
    html+='<p style="font-size:13px;line-height:1.5;color:#c0c4d8">'+escapeHtml(entity.description)+'</p>';
    html+='</div>';
  }
  // Relations
  var rels=entity._rels||[];
  var byType={};
  rels.forEach(function(r){
    if(!byType[r.type])byType[r.type]=[];
    byType[r.type].push(r.target);
  });
  html+='<div class="detail-section"><h3>Relations ('+rels.length+')</h3>';
  for(var rt in byType){
    html+='<div style="margin-bottom:8px"><div style="font-size:10px;color:'+relColor(rt)+';margin-bottom:2px">'+rt+'</div>';
    byType[rt].forEach(function(tid){
      var te=entityMap[tid];
      var label=te?te.title:tid;
      html+='<div class="detail-rel" onclick="navigateToEntity(\''+tid+'\')">'
        +'<span style="color:'+(typeColors[te?te.type:'unknown']||'#999')+'">'+tid+'</span> '
        +label
        +'<span class="rel-type">'+(te?te.type:'')+'</span></div>';
    });
    html+='</div>';
  }
  html+='</div>';
  content.innerHTML=html;
  panel.classList.add('open');
}

window.closeDetail=function(){document.getElementById('detail').classList.remove('open')};
window.navigateToEntity=navigateToEntity;

// ---- Search ----
document.getElementById('search').addEventListener('input',function(e){
  var q=e.target.value.toLowerCase().trim();
  if(q.length<2){
    if(document.getElementById('sankey-view').style.display!=='none')return;
    if(cy)cy.elements().style('opacity',1);
    return;
  }
  // Search in sidebar
  document.querySelectorAll('.tree-leaf').forEach(function(el){
    var match=el.textContent.toLowerCase().includes(q);
    el.style.display=match?'':'none';
    if(match){
      el.parentElement.classList.add('open');
      el.parentElement.previousElementSibling&&el.parentElement.previousElementSibling.querySelector('.arrow')&&el.parentElement.previousElementSibling.querySelector('.arrow').classList.add('open');
      var sec=el.closest('.tree-section');
      if(sec){sec.querySelector('.tree-children').classList.add('open');sec.querySelector('.arrow').classList.add('open')}
    }
  });
  // Filter cy nodes if visible
  if(cy&&document.getElementById('cy-view').style.display!=='none'){
    cy.nodes().forEach(function(n){
      var label=n.data('label')||'';
      n.style('opacity',label.toLowerCase().includes(q)?1:0.15);
    });
  }
});

// Handle resize for sankey
window.addEventListener('resize',function(){if(sankeyData&&document.getElementById('sankey-view').style.display!=='none')renderSankey(sankeyData)});

loadAll();
})();
</script></body></html>"#;

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::graph::KnowledgeGraph;
    use crate::domain::types::Entity;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use std::collections::HashMap;
    use tower::ServiceExt;

    /// Build a test entity with all fields.
    fn make_entity(id: &str, r#type: &str, category: &str, title: &str) -> Entity {
        Entity {
            id: id.to_owned(),
            r#type: r#type.to_owned(),
            title: title.to_owned(),
            description: String::new(),
            name: String::new(),
            category: category.to_owned(),
            tags: vec![],
            relations: HashMap::new(),
            context: HashMap::new(),
            file_path: String::new(),
            source: serde_json::Value::Null,
        }
    }

    /// Build an EpistemeMCP handler from a vec of entities.
    fn make_mcp(entities: Vec<Entity>) -> Arc<EpistemeMCP> {
        let map: HashMap<String, Entity> =
            entities.into_iter().map(|e| (e.id.clone(), e)).collect();
        let kg = KnowledgeGraph::from_entities(map);
        Arc::new(EpistemeMCP::new(kg))
    }

    fn test_app(mcp: Arc<EpistemeMCP>) -> Router {
        web_router(mcp)
    }

    #[tokio::test]
    async fn sankey_returns_nodes_and_links() {
        let mut smell = make_entity("SMELL-01", "smell", "quality", "Long Method");
        smell
            .relations
            .insert("solved_by".into(), vec!["RF-001".into()]);
        let mut rf = make_entity("RF-001", "refactoring", "design", "Extract Method");
        rf.relations
            .insert("enforces".into(), vec!["LAW-001".into()]);
        let law = make_entity("LAW-001", "law", "quality", "Single Responsibility");

        let mcp = make_mcp(vec![smell, rf, law]);
        let app = test_app(mcp);

        let req = Request::builder()
            .uri("/api/graph/sankey")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let val: serde_json::Value = serde_json::from_slice(&body).unwrap();

        let nodes = val["nodes"].as_array().unwrap();
        assert!(!nodes.is_empty());

        // Check that smell node has count 1
        let smell_node = nodes.iter().find(|n| n["id"] == "smell").unwrap();
        assert_eq!(smell_node["count"], 1);
    }

    #[tokio::test]
    async fn sankey_excludes_filtered_relations() {
        let mut e1 = make_entity("DP-001", "pattern", "creational", "Factory");
        e1.relations
            .insert("related_to".into(), vec!["DP-002".into()]);
        let e2 = make_entity("DP-002", "pattern", "structural", "Adapter");

        let mcp = make_mcp(vec![e1, e2]);
        let app = test_app(mcp);

        let req = Request::builder()
            .uri("/api/graph/sankey")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let val: serde_json::Value = serde_json::from_slice(&body).unwrap();

        let links = val["links"].as_array().unwrap();
        assert!(links.is_empty(), "related_to should be excluded");
    }

    #[tokio::test]
    async fn sankey_includes_allowed_relations() {
        let mut smell = make_entity("SMELL-01", "smell", "quality", "Long Method");
        smell
            .relations
            .insert("solved_by".into(), vec!["RF-001".into()]);
        let rf = make_entity("RF-001", "refactoring", "design", "Extract Method");

        let mcp = make_mcp(vec![smell, rf]);
        let app = test_app(mcp);

        let req = Request::builder()
            .uri("/api/graph/sankey")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let val: serde_json::Value = serde_json::from_slice(&body).unwrap();

        let links = val["links"].as_array().unwrap();
        assert_eq!(links.len(), 1);
        assert_eq!(links[0]["source"], "smell");
        assert_eq!(links[0]["target"], "refactoring");
        assert_eq!(links[0]["relation"], "solved_by");
        assert_eq!(links[0]["value"], 1);
    }

    #[tokio::test]
    async fn sankey_aggregates_multiple_edges() {
        let mut smell1 = make_entity("SMELL-01", "smell", "quality", "Long Method");
        smell1
            .relations
            .insert("solved_by".into(), vec!["RF-001".into(), "RF-002".into()]);
        let rf1 = make_entity("RF-001", "refactoring", "design", "Extract Method");
        let rf2 = make_entity("RF-002", "refactoring", "design", "Decompose Conditional");

        let mcp = make_mcp(vec![smell1, rf1, rf2]);
        let app = test_app(mcp);

        let req = Request::builder()
            .uri("/api/graph/sankey")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let val: serde_json::Value = serde_json::from_slice(&body).unwrap();

        let links = val["links"].as_array().unwrap();
        let link = links.iter().find(|l| l["relation"] == "solved_by").unwrap();
        assert_eq!(link["value"], 2);
    }

    #[tokio::test]
    async fn tree_returns_grouped_structure() {
        let dp1 = make_entity("DP-001", "pattern", "creational", "Abstract Factory");
        let dp2 = make_entity("DP-002", "pattern", "creational", "Builder");
        let dp3 = make_entity("DP-003", "pattern", "structural", "Adapter");
        let rf1 = make_entity("RF-001", "refactoring", "design", "Extract Method");

        let mcp = make_mcp(vec![dp1, dp2, dp3, rf1]);
        let app = test_app(mcp);

        let req = Request::builder()
            .uri("/api/graph/tree")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let val: serde_json::Value = serde_json::from_slice(&body).unwrap();

        let tree = val["tree"].as_array().unwrap();
        assert!(tree.len() >= 2);

        // Find pattern type node
        let pattern_node = tree.iter().find(|n| n["type"] == "pattern").unwrap();
        assert_eq!(pattern_node["label"], "Design Patterns");
        let children = pattern_node["children"].as_array().unwrap();
        assert_eq!(children.len(), 2); // creational + structural

        // Find creational category
        let creational = children
            .iter()
            .find(|c| c["category"] == "creational")
            .unwrap();
        assert_eq!(creational["label"], "Creational");
        let leaf_items = creational["children"].as_array().unwrap();
        assert_eq!(leaf_items.len(), 2);
        assert_eq!(leaf_items[0]["id"], "DP-001");
        assert_eq!(leaf_items[1]["id"], "DP-002");
    }

    #[tokio::test]
    async fn tree_handles_empty_category() {
        let e = make_entity("LAW-001", "law", "", "DRY");
        let mcp = make_mcp(vec![e]);
        let app = test_app(mcp);

        let req = Request::builder()
            .uri("/api/graph/tree")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let val: serde_json::Value = serde_json::from_slice(&body).unwrap();

        let tree = val["tree"].as_array().unwrap();
        let law_node = tree.iter().find(|n| n["type"] == "law").unwrap();
        let cats = law_node["children"].as_array().unwrap();
        assert_eq!(cats[0]["category"], "uncategorized");
    }

    #[tokio::test]
    async fn tree_types_are_ordered_consistently() {
        let e1 = make_entity("SMELL-01", "smell", "quality", "S1");
        let e2 = make_entity("LAW-001", "law", "quality", "L1");
        let e3 = make_entity("DP-001", "pattern", "design", "P1");
        let e4 = make_entity("RF-001", "refactoring", "design", "R1");

        let mcp = make_mcp(vec![e1, e2, e3, e4]);
        let app = test_app(mcp);

        let req = Request::builder()
            .uri("/api/graph/tree")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let val: serde_json::Value = serde_json::from_slice(&body).unwrap();

        let tree = val["tree"].as_array().unwrap();
        let types: Vec<&str> = tree.iter().map(|n| n["type"].as_str().unwrap()).collect();
        assert_eq!(types, vec!["pattern", "refactoring", "law", "smell"]);
    }

    #[tokio::test]
    async fn sankey_empty_graph_returns_empty() {
        let mcp = make_mcp(vec![]);
        let app = test_app(mcp);

        let req = Request::builder()
            .uri("/api/graph/sankey")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let val: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(val["nodes"].as_array().unwrap().len(), 0);
        assert_eq!(val["links"].as_array().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn tree_empty_graph_returns_empty() {
        let mcp = make_mcp(vec![]);
        let app = test_app(mcp);

        let req = Request::builder()
            .uri("/api/graph/tree")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let val: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(val["tree"].as_array().unwrap().len(), 0);
    }
}
