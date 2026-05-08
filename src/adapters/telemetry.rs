use std::time::Duration;
use std::{fs, io::Write};

use tokio::sync::mpsc;

#[derive(Debug, Clone, Copy)]
pub enum Tool {
    SearchKnowledge,
    GetEntity,
    GetNeighbors,
    FindPath,
    AnalyzeCode,
    SuggestRefactorings,
}

impl Tool {
    fn as_str(self) -> &'static str {
        match self {
            Self::SearchKnowledge => "search_knowledge",
            Self::GetEntity => "get_entity",
            Self::GetNeighbors => "get_neighbors",
            Self::FindPath => "find_path",
            Self::AnalyzeCode => "analyze_code",
            Self::SuggestRefactorings => "suggest_refactorings",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Command {
    Install,
    Build,
    Analyze,
    Infer,
    Explore,
    Api,
    Mcp,
    Service,
    Telemetry,
}

impl Command {
    fn as_str(self) -> &'static str {
        match self {
            Self::Install => "install",
            Self::Build => "build",
            Self::Analyze => "analyze",
            Self::Infer => "infer",
            Self::Explore => "explore",
            Self::Api => "api",
            Self::Mcp => "mcp",
            Self::Service => "service",
            Self::Telemetry => "telemetry",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum FailureClass {
    GraphLoadError,
    EmbeddingError,
    DatabaseError,
    PermissionDenied,
    NetworkError,
    Timeout,
    Unknown,
}

impl FailureClass {
    fn as_str(self) -> &'static str {
        match self {
            Self::GraphLoadError => "graph_load_error",
            Self::EmbeddingError => "embedding_error",
            Self::DatabaseError => "database_error",
            Self::PermissionDenied => "permission_denied",
            Self::NetworkError => "network_error",
            Self::Timeout => "timeout",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ResultSizeBucket {
    Empty,
    Lt5,
    Lt20,
    Gte20,
}

impl ResultSizeBucket {
    fn as_str(self) -> &'static str {
        match self {
            Self::Empty => "empty",
            Self::Lt5 => "<5",
            Self::Lt20 => "<20",
            Self::Gte20 => ">=20",
        }
    }
}

impl From<usize> for ResultSizeBucket {
    fn from(value: usize) -> Self {
        if value == 0 {
            Self::Empty
        } else if value < 5 {
            Self::Lt5
        } else if value < 20 {
            Self::Lt20
        } else {
            Self::Gte20
        }
    }
}

#[derive(Clone)]
pub struct Telemetry {
    enabled: bool,
    sender: Option<mpsc::UnboundedSender<TelemetryEvent>>,
}

#[derive(Debug, Clone)]
struct TelemetryEvent {
    event: String,
    properties: serde_json::Value,
}

impl Telemetry {
    pub fn new(
        enabled: bool,
        posthog_api_key: String,
        posthog_host: String,
        sentry_dsn: String,
    ) -> Self {
        if !enabled {
            return Self {
                enabled: false,
                sender: None,
            };
        }
        let (tx, mut rx) = mpsc::unbounded_channel::<TelemetryEvent>();
        let ph_key = posthog_api_key.clone();
        let ph_host = posthog_host.clone();
        let sentry = sentry_dsn.clone();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("telemetry runtime");
            rt.block_on(async move {
                let client = reqwest::Client::new();
                while let Some(ev) = rx.recv().await {
                    let _ = send_with_retry(&client, &ph_key, &ph_host, &sentry, &ev).await;
                }
            });
        });
        Self {
            enabled,
            sender: Some(tx),
        }
    }

    pub fn disabled() -> Self {
        Self::new(false, String::new(), String::new(), String::new())
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn track_event(&self, event: &'static str, properties: serde_json::Value) {
        if !self.enabled || !read_consent() {
            return;
        }
        let Some(tx) = &self.sender else {
            return;
        };
        let _ = tx.send(TelemetryEvent {
            event: event.to_owned(),
            properties: serde_json::json!({
                "install_id": load_or_create_install_id(),
                "payload": properties,
            }),
        });
    }
}

fn consent_file() -> std::path::PathBuf {
    crate::adapters::paths::episteme_home().join("telemetry-consent")
}

fn install_id_file() -> std::path::PathBuf {
    crate::adapters::paths::episteme_home().join("install-id")
}

pub fn read_consent_raw() -> Option<bool> {
    let path = consent_file();
    let text = fs::read_to_string(path).ok()?;
    match text.trim() {
        "on" | "true" | "1" => Some(true),
        "off" | "false" | "0" => Some(false),
        _ => None,
    }
}

pub fn read_consent() -> bool {
    read_consent_raw().unwrap_or(true)
}

pub fn write_consent(enabled: bool) -> Result<(), String> {
    let home = crate::adapters::paths::episteme_home();
    fs::create_dir_all(&home).map_err(|e| e.to_string())?;
    fs::write(consent_file(), if enabled { "on" } else { "off" }).map_err(|e| e.to_string())
}

pub fn ensure_consent_or_set_default() -> Result<(), String> {
    if read_consent_raw().is_none() {
        write_consent(true)?;
        eprintln!("[episteme] Telemetry enabled (anonymous install ID).");
        eprintln!("[episteme] To opt out: epis telemetry off");
        eprintln!("[episteme] Details: https://github.com/epicsagas/Episteme#telemetry");
    }
    Ok(())
}

pub fn prompt_consent_interactive() -> bool {
    println!();
    println!("  ┌─ Telemetry ──────────────────────────────────────────────────────────┐");
    println!("  │ Episteme collects anonymous usage data to improve detection quality. │");
    println!("  │                                                                      │");
    println!("  │  What we send:    tool name, duration, outcome, version, OS          │");
    println!("  │  What we never:   code content, file paths, search queries           │");
    println!("  │  Identifier:      random install ID (not linked to you or machine)   │");
    println!("  │  Opt out anytime: epis telemetry off                             │");
    println!("  └──────────────────────────────────────────────────────────────────────┘");
    println!();
    print!("  Enable telemetry? [Y/n]: ");
    let _ = std::io::stdout().flush();
    let mut line = String::new();
    if std::io::stdin().read_line(&mut line).is_err() {
        return true;
    }
    let ans = line.trim().to_lowercase();
    !(ans == "n" || ans == "no")
}

pub fn load_or_create_install_id() -> String {
    let path = install_id_file();
    if let Ok(s) = fs::read_to_string(&path) {
        let v = s.trim();
        if !v.is_empty() {
            return v.to_owned();
        }
    }
    let id = uuid::Uuid::new_v4().to_string();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(mut f) = fs::File::create(&path) {
        let _ = f.write_all(id.as_bytes());
    }
    id
}

async fn send_with_retry(
    client: &reqwest::Client,
    posthog_api_key: &str,
    posthog_host: &str,
    sentry_dsn: &str,
    ev: &TelemetryEvent,
) -> Result<(), ()> {
    const MAX_RETRIES: usize = 3;
    let mut attempt = 0usize;
    loop {
        let mut ok = true;
        if !posthog_api_key.is_empty() {
            let posthog_url = format!("{}/capture/", posthog_host.trim_end_matches('/'));
            let res = client
                .post(posthog_url)
                .json(&serde_json::json!({
                    "api_key": posthog_api_key,
                    "event": ev.event,
                    "distinct_id": "episteme-server",
                    "properties": ev.properties,
                }))
                .send()
                .await;
            ok &= res
                .as_ref()
                .map(|r| r.status().is_success())
                .unwrap_or(false);
        }
        if !sentry_dsn.is_empty() {
            let res = client
                .post(sentry_dsn)
                .json(&serde_json::json!({
                    "message": ev.event,
                    "level": "info",
                    "extra": ev.properties,
                }))
                .send()
                .await;
            ok &= res
                .as_ref()
                .map(|r| r.status().is_success())
                .unwrap_or(false);
        }
        if ok {
            return Ok(());
        }
        attempt += 1;
        if attempt >= MAX_RETRIES {
            tracing::warn!(event = ev.event, "telemetry delivery failed after retries");
            return Err(());
        }
        tokio::time::sleep(Duration::from_millis(250 * (1 << attempt))).await;
    }
}

fn telemetry_client() -> Option<Telemetry> {
    let cfg = crate::adapters::config::EpistemeConfig::load().ok()?;
    if !cfg.telemetry_enabled {
        return None;
    }
    Some(Telemetry::new(
        true,
        cfg.posthog_api_key,
        cfg.posthog_host,
        cfg.sentry_dsn,
    ))
}

pub fn track_session_started() {
    if let Some(t) = telemetry_client() {
        t.track_event("session_started", serde_json::json!({}));
    }
}

pub fn track_command_invoked(command: Command) {
    if let Some(t) = telemetry_client() {
        t.track_event(
            "command_invoked",
            serde_json::json!({"command": command.as_str()}),
        );
    }
}

pub fn track_command_completed(command: Command, duration_ms: u128) {
    if let Some(t) = telemetry_client() {
        t.track_event(
            "command_completed",
            serde_json::json!({"command": command.as_str(), "duration_ms": duration_ms.to_string()}),
        );
    }
}

pub fn track_command_failed(command: Command, failure: FailureClass) {
    if let Some(t) = telemetry_client() {
        t.track_event(
            "command_failed",
            serde_json::json!({"command": command.as_str(), "failure_class": failure.as_str()}),
        );
    }
}

pub fn track_tool_called(tool: Tool) {
    if let Some(t) = telemetry_client() {
        t.track_event("tool_called", serde_json::json!({"tool": tool.as_str()}));
    }
}

pub fn track_tool_completed(tool: Tool, duration_ms: u128, result_size: ResultSizeBucket) {
    if let Some(t) = telemetry_client() {
        t.track_event(
            "tool_completed",
            serde_json::json!({
                "tool": tool.as_str(),
                "duration_ms": duration_ms.to_string(),
                "result_size": result_size.as_str()
            }),
        );
    }
}

pub fn track_tool_failed(tool: Tool, failure: FailureClass) {
    if let Some(t) = telemetry_client() {
        t.track_event(
            "tool_failed",
            serde_json::json!({"tool": tool.as_str(), "failure_class": failure.as_str()}),
        );
    }
}

pub fn track_install_completed(tool_count: usize) {
    if let Some(t) = telemetry_client() {
        t.track_event(
            "install_completed",
            serde_json::json!({"tool_count": tool_count.to_string()}),
        );
    }
}
