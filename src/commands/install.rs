//! Install command: install Episteme into AI tools.

use std::io::Write;
use std::path::PathBuf;

use anyhow::Result;

use episteme::adapters::config::EpistemeConfig;

pub fn cmd_install(tools: &[String], all: bool, dry_run: bool, local: bool) -> Result<()> {
    use episteme::adapters::installer;
    use std::io::{self, IsTerminal};

    // --- Data seeding ---
    let seeded = if local {
        // --local: try dist/ archive first, fallback to raw/meta/ source tree
        println!("Seeding data (local)...");
        let cwd = std::env::current_dir().map_err(|e| anyhow::anyhow!(e))?;
        let dist_archive = find_dist_archive(&cwd);

        if let Some(archive_path) = dist_archive {
            println!("  Using dist archive: {}", archive_path.display());
            for msg in installer::seed_data_from_local_archive(&archive_path, dry_run)
                .map_err(|e| anyhow::anyhow!(e))?
            {
                println!("  {msg}");
            }
        } else {
            println!("  No dist archive found, seeding from source tree...");
            for msg in installer::seed_data(dry_run).map_err(|e| anyhow::anyhow!(e))? {
                println!("  {msg}");
            }
        }
        true
    } else if !dry_run {
        // Default: download from GitHub release
        println!("Fetching data from GitHub Releases...");
        let url = resolve_release_url().map_err(|e| anyhow::anyhow!(e))?;
        println!("  Downloading: {url}");
        for msg in
            installer::seed_data_from_release(&url, dry_run).map_err(|e| anyhow::anyhow!(e))?
        {
            println!("  {msg}");
        }
        true
    } else {
        false
    };

    // --- Build RAG index (skip if DB already provided by archive) ---
    if seeded && !dry_run && !episteme::adapters::paths::db_path().exists() {
        println!("\nBuilding RAG index...");
        super::build::cmd_build(None, None, false, false, 64, true, false)?;
    }

    // --- Tool installation ---
    use episteme::adapters::installer::Transport;

    let mut selected: Vec<String> = if all || tools.iter().any(|t| t == "all") {
        vec!["claude", "cursor", "codex", "gemini", "opencode", "cline"]
            .into_iter()
            .map(|s| s.to_owned())
            .collect()
    } else if tools.is_empty() && io::stdin().is_terminal() {
        let installed: Vec<&str> = detect_installed_tools().into_iter().collect();
        let selected =
            match episteme::adapters::install_wizard::interactive_select_tools(&installed) {
                Ok(s) if !s.is_empty() => s,
                Ok(_) => anyhow::bail!("install cancelled"),
                Err(e) => {
                    eprintln!("Interactive UI failed ({e}); falling back to text prompt.");
                    episteme::adapters::install_wizard::fallback_select_tools()
                }
            };

        let cfg = EpistemeConfig::load().unwrap_or_default();
        if let Some(redis) = episteme::adapters::install_wizard::configure_redis_tui(
            episteme::adapters::install_wizard::RedisConfig {
                enabled: cfg.redis_enabled,
                host: cfg.redis_host.clone(),
                port: cfg.redis_port,
                db: cfg.redis_db,
                ttl: cfg.redis_ttl,
            },
        )
        .map_err(|e| anyhow::anyhow!(e))?
        {
            upsert_config_yaml(redis.enabled, &redis.host, redis.port, redis.db, redis.ttl)?;
        }

        let telemetry_enabled = episteme::adapters::install_wizard::configure_telemetry_tui()
            .map_err(|e| anyhow::anyhow!(e))?;
        episteme::adapters::telemetry::write_consent(telemetry_enabled)
            .map_err(|e| anyhow::anyhow!(e))?;

        selected
    } else {
        tools.to_vec()
    };
    selected.sort();
    selected.dedup();

    // --- Transport selection (TTY only; non-TTY defaults to HTTP + 43175) ---
    let transport: Transport = if io::stdin().is_terminal() {
        episteme::adapters::install_wizard::select_transport().map_err(|e| anyhow::anyhow!(e))?
    } else {
        Transport::default()
    };

    for tool in &selected {
        let result = match tool.as_str() {
            "claude" => installer::install_claude(dry_run, &transport),
            "cursor" => installer::install_cursor(dry_run, &transport),
            "codex" => installer::install_codex(dry_run),
            "gemini" => installer::install_gemini(dry_run, &transport),
            "opencode" => installer::install_opencode(dry_run, &transport),
            "cline" => installer::install_cline(dry_run, &transport),
            _ => Err(format!("Unknown tool: {tool}")),
        };
        match result {
            Ok(msgs) => {
                for msg in msgs {
                    println!("  {msg}");
                }
            }
            Err(e) => eprintln!("  Error ({tool}): {e}"),
        }
    }

    // --- Enable launchd for HTTP transport ---
    if matches!(transport, Transport::Http { .. }) && !dry_run && io::stdin().is_terminal() {
        print!("\nEnable episteme MCP as login item and start now? [Y/n]: ");
        io::stdout().flush().ok();
        let mut line = String::new();
        io::stdin().read_line(&mut line).ok();
        if !line.trim().eq_ignore_ascii_case("n") {
            match episteme::adapters::service::enable_launchd(true) {
                Ok(msg) => println!("  {msg}"),
                Err(e) => eprintln!("  Warning: {e}"),
            }
        }
    }

    episteme::adapters::telemetry::track_install_completed(selected.len());

    Ok(())
}

/// Find the newest `episteme-data-*.tar.gz` in `dist/`.
fn find_dist_archive(cwd: &std::path::Path) -> Option<PathBuf> {
    let dist_dir = cwd.join("dist");
    if !dist_dir.is_dir() {
        return None;
    }
    let mut newest: Option<(PathBuf, std::time::SystemTime)> = None;
    let Ok(entries) = std::fs::read_dir(&dist_dir) else {
        return None;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.starts_with("episteme-data-")
            && name_str.ends_with(".tar.gz")
            && let Ok(meta) = entry.metadata()
            && let Ok(modified) = meta.modified()
            && newest.as_ref().map(|(_, t)| modified > *t).unwrap_or(true)
        {
            newest = Some((entry.path(), modified));
        }
    }
    newest.map(|(p, _)| p)
}

/// Resolve the GitHub release download URL for the current version.
fn resolve_release_url() -> Result<String> {
    let version = get_package_version();
    let repo = "epicsagas/Episteme";
    let prefix = "episteme-data-";

    // Try exact version tag first, then latest
    for endpoint in [
        format!("https://api.github.com/repos/{repo}/releases/tags/v{version}"),
        format!("https://api.github.com/repos/{repo}/releases/latest"),
    ] {
        if let Ok(url) = fetch_release_asset_url(&endpoint, prefix) {
            return Ok(url);
        }
    }

    anyhow::bail!("No data asset found for v{version}.\nCheck: https://github.com/{repo}/releases")
}

fn get_package_version() -> String {
    std::env::var("EPISTEME_VERSION").unwrap_or_else(|_| env!("CARGO_PKG_VERSION").to_owned())
}

fn fetch_release_asset_url(api_url: &str, prefix: &str) -> Result<String> {
    let output = std::process::Command::new("curl")
        .args([
            "-LfsS",
            "-H",
            "Accept: application/vnd.github+json",
            api_url,
        ])
        .output()
        .map_err(|e| anyhow::anyhow!("curl failed: {e}"))?;

    if !output.status.success() {
        anyhow::bail!("curl returned non-zero");
    }

    let json_str = String::from_utf8_lossy(&output.stdout);
    let val: serde_json::Value =
        serde_json::from_str(&json_str).map_err(|e| anyhow::anyhow!("JSON parse: {e}"))?;

    if let Some(assets) = val.get("assets").and_then(|a| a.as_array()) {
        for asset in assets {
            let name = asset.get("name").and_then(|n| n.as_str()).unwrap_or("");
            if name.starts_with(prefix)
                && name.ends_with(".tar.gz")
                && let Some(url) = asset.get("browser_download_url").and_then(|u| u.as_str())
            {
                return Ok(url.to_owned());
            }
        }
    }

    anyhow::bail!("no matching asset in release")
}

pub fn detect_installed_tools() -> std::collections::HashSet<&'static str> {
    use serde_json::Value;
    use std::collections::HashSet;
    use std::path::Path;

    fn has_json_path(path: &Path, parent: &str, child: &str) -> bool {
        let Ok(text) = std::fs::read_to_string(path) else {
            return false;
        };
        let Ok(v) = serde_json::from_str::<Value>(&text) else {
            return false;
        };
        v.get(parent).and_then(|m| m.get(child)).is_some()
    }

    let mut installed = HashSet::new();
    let home = std::env::var("HOME").unwrap_or_default();
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    if has_json_path(
        &PathBuf::from(&home).join(".claude.json"),
        "mcpServers",
        "epis",
    ) {
        installed.insert("claude");
    }
    if has_json_path(
        &PathBuf::from(&home).join(".cursor").join("mcp.json"),
        "mcpServers",
        "epis",
    ) {
        installed.insert("cursor");
    }
    if has_json_path(
        &PathBuf::from(&home).join(".gemini").join("mcp.json"),
        "mcpServers",
        "epis",
    ) {
        installed.insert("gemini");
    }
    if has_json_path(
        &PathBuf::from(&home)
            .join(".config")
            .join("opencode")
            .join("opencode.json"),
        "mcp",
        "epis",
    ) {
        installed.insert("opencode");
    }
    if PathBuf::from(&home)
        .join("Documents")
        .join("Cline")
        .join("Hooks")
        .join("TaskStart")
        .exists()
    {
        installed.insert("cline");
    }
    if let Ok(content) = std::fs::read_to_string(cwd.join("AGENTS.md"))
        && (content.contains("EPISTEME-BEGIN") || content.contains("epis mcp"))
    {
        installed.insert("codex");
    }
    installed
}

fn upsert_config_yaml(
    redis_enabled: bool,
    redis_host: &str,
    redis_port: u16,
    redis_db: u16,
    redis_ttl: u64,
) -> Result<()> {
    use serde_yaml::{Mapping, Value};
    let path = episteme::adapters::paths::episteme_home().join("config.yaml");
    let mut root = if path.exists() {
        let text = std::fs::read_to_string(&path)?;
        serde_yaml::from_str::<Value>(&text).unwrap_or_else(|_| Value::Mapping(Mapping::new()))
    } else {
        Value::Mapping(Mapping::new())
    };

    if !root.is_mapping() {
        root = Value::Mapping(Mapping::new());
    }
    let root_map = root.as_mapping_mut().expect("mapping checked above");

    let mut redis_map = Mapping::new();
    redis_map.insert(
        Value::String("enabled".to_owned()),
        Value::Bool(redis_enabled),
    );
    redis_map.insert(
        Value::String("host".to_owned()),
        Value::String(redis_host.to_owned()),
    );
    redis_map.insert(
        Value::String("port".to_owned()),
        Value::Number(serde_yaml::Number::from(redis_port)),
    );
    redis_map.insert(
        Value::String("db".to_owned()),
        Value::Number(serde_yaml::Number::from(redis_db)),
    );
    redis_map.insert(
        Value::String("ttl".to_owned()),
        Value::Number(serde_yaml::Number::from(redis_ttl)),
    );
    root_map.insert(Value::String("redis".to_owned()), Value::Mapping(redis_map));

    std::fs::create_dir_all(episteme::adapters::paths::episteme_home())?;
    let yaml = serde_yaml::to_string(&root)?;
    std::fs::write(path, yaml)?;
    Ok(())
}
