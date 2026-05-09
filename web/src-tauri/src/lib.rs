use std::sync::Arc;
use tauri::{Manager, State};

use episteme::adapters::user_graph_store::UserGraphStore;
use episteme::domain::composite_graph::CompositeGraph;
use episteme::EpistemeMCP;
use episteme::KnowledgeGraph;

type ServerState = Arc<tokio::sync::Mutex<Option<ServerHandles>>>;

struct ServerHandles {
    api_shutdown: tokio::sync::oneshot::Sender<()>,
    web_shutdown: tokio::sync::oneshot::Sender<()>,
}

fn build_mcp(graph: KnowledgeGraph) -> EpistemeMCP {
    let db_path = episteme::paths::episteme_home().join("user_knowledge.db");
    match UserGraphStore::open(&db_path) {
        Ok(store) => {
            let composite = CompositeGraph::new(graph.clone(), Box::new(store));
            EpistemeMCP::with_composite(graph, composite)
        }
        Err(_) => EpistemeMCP::new(graph),
    }
}

#[tauri::command]
async fn start_backend(state: State<'_, ServerState>) -> Result<String, String> {
    let mut guard = state.lock().await;

    if guard.is_some() {
        return Ok("already running".into());
    }

    let config = episteme::EpistemeConfig::load().map_err(|e| e.to_string())?;

    let graph = episteme::json_loader::load_graph(&episteme::paths::data_dir())
        .map_err(|e| e.to_string())?;

    let handler = Arc::new(build_mcp(graph.clone()));

    // --- Start REST API server (port 8000) ---
    let api_config = config.clone();
    let api_graph = graph.clone();
    let (api_tx, api_rx) = tokio::sync::oneshot::channel::<()>();
    tokio::spawn(async move {
        let addr = format!("{}:{}", api_config.api_host, api_config.api_port);
        let api_keys = episteme::mcp_auth::parse_api_keys(&api_config.api_keys);
        let app = episteme::api_app::create_app(
            api_graph,
            api_keys,
            &api_config.cors_origins,
            &api_config.redis_host,
            api_config.redis_port,
            api_config.redis_db,
            api_config.redis_ttl,
            api_config.redis_enabled,
            api_config.enable_debug_endpoints,
            api_config.telemetry_enabled,
            api_config.posthog_api_key.clone(),
            api_config.posthog_host.clone(),
            api_config.sentry_dsn.clone(),
        )
        .await;

        let listener = tokio::net::TcpListener::bind(&addr)
            .await
            .expect("failed to bind API server");

        tracing::info!("API server listening on {}", addr);

        axum::serve(listener, app)
            .with_graceful_shutdown(async {
                let _ = api_rx.await;
            })
            .await
            .ok();
    });

    // --- Start Web Viewer server (port 8080) ---
    let (web_tx, web_rx) = tokio::sync::oneshot::channel::<()>();
    let web_handler = handler.clone();
    let web_host = config.api_host.clone();
    let web_port = 8080u16;
    tokio::spawn(async move {
        let app = episteme::web_viewer::web_router(web_handler);
        let addr = format!("{}:{}", web_host, web_port);
        let listener = tokio::net::TcpListener::bind(&addr)
            .await
            .expect("failed to bind web viewer");

        tracing::info!("Web viewer listening on {}", addr);

        axum::serve(listener, app)
            .with_graceful_shutdown(async {
                let _ = web_rx.await;
            })
            .await
            .ok();
    });

    *guard = Some(ServerHandles {
        api_shutdown: api_tx,
        web_shutdown: web_tx,
    });

    Ok("started".into())
}

#[tauri::command]
async fn stop_backend(state: State<'_, ServerState>) -> Result<String, String> {
    let mut guard = state.lock().await;
    if let Some(handles) = guard.take() {
        let _ = handles.api_shutdown.send(());
        let _ = handles.web_shutdown.send(());
        Ok("stopped".into())
    } else {
        Ok("not running".into())
    }
}

#[tauri::command]
async fn backend_status(state: State<'_, ServerState>) -> Result<String, String> {
    let guard = state.lock().await;
    Ok(if guard.is_some() {
        "running".into()
    } else {
        "stopped".into()
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage::<ServerState>(Arc::new(tokio::sync::Mutex::new(None)))
        .invoke_handler(tauri::generate_handler![
            start_backend,
            stop_backend,
            backend_status,
        ])
        .setup(|app| {
            let handle = app.handle().clone();
            tokio::spawn(async move {
                let state: State<'_, ServerState> = handle.state();
                if let Err(e) = start_backend(state).await {
                    tracing::error!("failed to start backend: {e}");
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
