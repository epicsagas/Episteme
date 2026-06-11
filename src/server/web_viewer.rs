use axum::{
    Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::get,
};
use serde::Deserialize;
use std::sync::Arc;
use tower_http::services::ServeDir;

use crate::server::mcp_handler::EpistemeMCP;

/// Build the web viewer router.
///
/// Serves the Vite-built SPA from `web/dist/` on `/`, falling back to `index.html`
/// for client-side routing. API endpoints remain unchanged.
pub fn web_router(handler: Arc<EpistemeMCP>) -> Router {
    let dist_dir = std::env::var("EPISTEME_WEB_DIST").unwrap_or_else(|_| "web/dist".to_owned());

    let api_routes = Router::new()
        .route("/api/graph/full", get(graph_full))
        .route("/api/graph/entity/{id}", get(graph_entity))
        .route("/api/graph/path/{from}/{to}", get(graph_path))
        .route("/api/graph/sankey", get(graph_sankey))
        .route("/api/graph/schema", get(graph_schema))
        .route("/api/graph/tree", get(graph_tree))
        .route("/api/entities/search", get(entities_search))
        .with_state(handler);

    Router::new().merge(api_routes).fallback_service(
        ServeDir::new(&dist_dir).fallback(ServeDir::new(format!("{}/index.html", dist_dir))),
    )
}

/// Full graph as Cytoscape.js elements — includes canonical + user (insight) entities.
async fn graph_full(State(mcp): State<Arc<EpistemeMCP>>) -> Json<serde_json::Value> {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let all_ids = mcp.all_entity_ids();

    for id in &all_ids {
        let Some(entity) = mcp.get_entity_merged(id) else {
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

        for edge in mcp.get_all_edges_merged(id) {
            // Only include edges where the target is also in our merged set
            if mcp.get_entity_merged(&edge.to_id).is_none() {
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
    // Try canonical subgraph first, fall back to merged lookup for user entities
    let graph = mcp.graph();
    let (ids, edges) = graph.extract_subgraph(&id, params.radius);

    // For user entities not in canonical, include just the entity itself
    let (final_ids, final_edges) = if ids.is_empty() {
        if mcp.get_entity_merged(&id).is_some() {
            (vec![id.clone()], mcp.get_all_edges_merged(&id))
        } else {
            return Err(StatusCode::NOT_FOUND);
        }
    } else {
        // extract_subgraph returns HashSet — convert to Vec
        (ids.into_iter().collect::<Vec<_>>(), edges)
    };

    let nodes: Vec<_> = final_ids
        .iter()
        .filter_map(|eid| {
            mcp.get_entity_merged(eid).map(|e| {
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

    let edge_data: Vec<_> = final_edges
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
    let query = params.q.to_lowercase();
    let results: Vec<_> = mcp
        .all_entity_ids()
        .iter()
        .filter_map(|id| {
            mcp.get_entity_merged(id).and_then(|e| {
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
// Schema API endpoint
// ---------------------------------------------------------------------------

/// GET /api/graph/schema -- entity types, relation types, and data sources with counts.
///
/// Returns type keys and counts only. Colors/icons/labels are the frontend's concern.
async fn graph_schema(State(mcp): State<Arc<EpistemeMCP>>) -> Json<serde_json::Value> {
    // Count across canonical + user entities
    let mut type_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    for id in mcp.all_entity_ids() {
        if let Some(entity) = mcp.get_entity_merged(&id) {
            let t = if entity.r#type.is_empty() {
                "unknown".to_owned()
            } else {
                entity.r#type.clone()
            };
            *type_counts.entry(t).or_insert(0) += 1;
        }
    }

    let entity_types: Vec<serde_json::Value> = type_counts
        .iter()
        .map(|(t, count)| {
            serde_json::json!({
                "key": t,
                "count": count,
            })
        })
        .collect();

    let relation_types = serde_json::json!([
        { "key": "solves", "inverse": "solved_by" },
        { "key": "solved_by", "inverse": "solves" },
        { "key": "enforces", "inverse": "enforced_by" },
        { "key": "enforced_by", "inverse": "enforces" },
        { "key": "violates", "inverse": "violated_by" },
        { "key": "violated_by", "inverse": "violates" },
        { "key": "related_to", "inverse": null },
        { "key": "derives_from", "inverse": null },
        { "key": "applies_to", "inverse": null },
        { "key": "supersedes", "inverse": null }
    ]);

    Json(serde_json::json!({
        "entity_types": entity_types,
        "relation_types": relation_types,
    }))
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
        "insight" => "Insights",
        _ => t,
    }
}

/// GET /api/graph/sankey -- aggregated flow data for a sankey diagram.
async fn graph_sankey(State(mcp): State<Arc<EpistemeMCP>>) -> Json<serde_json::Value> {
    // Count entities per type — canonical + user.
    let mut type_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    for id in mcp.all_entity_ids() {
        if let Some(entity) = mcp.get_entity_merged(&id) {
            let t = if entity.r#type.is_empty() {
                "unknown".to_owned()
            } else {
                entity.r#type.clone()
            };
            *type_counts.entry(t).or_insert(0) += 1;
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

    for id in mcp.all_entity_ids() {
        let Some(entity) = mcp.get_entity_merged(&id) else {
            continue;
        };
        let source_type = if entity.r#type.is_empty() {
            "unknown".to_owned()
        } else {
            entity.r#type.clone()
        };

        for edge in mcp.get_all_edges_merged(&id) {
            if !SANKEY_RELATIONS.contains(&edge.relation_type.as_str()) {
                continue;
            }
            let Some(target) = mcp.get_entity_merged(&edge.to_id) else {
                continue;
            };
            let target_type = if target.r#type.is_empty() {
                "unknown".to_owned()
            } else {
                target.r#type.clone()
            };

            *link_counts
                .entry((source_type.clone(), edge.relation_type.clone(), target_type))
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

    // Group: type -> category -> Vec of (id, title) — includes user insights.
    let mut by_type: HashMap<String, HashMap<String, Vec<(String, String)>>> = HashMap::new();

    for id in mcp.all_entity_ids() {
        let Some(entity) = mcp.get_entity_merged(&id) else {
            continue;
        };
        let t = if entity.r#type.is_empty() {
            "unknown".to_owned()
        } else {
            entity.r#type.clone()
        };
        let cat = if entity.category.is_empty() {
            "uncategorized".to_owned()
        } else {
            entity.category.clone()
        };

        by_type
            .entry(t)
            .or_default()
            .entry(cat)
            .or_default()
            .push((id, entity.title.clone()));
    }

    // Order types consistently — insight appears after canonical types.
    let type_order = ["pattern", "refactoring", "law", "smell", "insight"];
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
                            let desc = mcp
                                .get_entity_merged(id)
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
