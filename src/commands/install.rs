//! Install command: install Syntagma into AI tools.

use std::io::Write;
use std::path::PathBuf;

use anyhow::Result;

use syntagma::adapters::config::SyntagmaConfig;

pub fn cmd_install(
    tools: &[String],
    all: bool,
    dry_run: bool,
    local: bool,
) -> Result<()> {
    use syntagma::adapters::installer;
    use std::io::{self, IsTerminal};

    // --- Data seeding ---
    if local {
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
    } else if !dry_run {
        // Default: download from GitHub release
        println!("Fetching data from GitHub Releases...");
        let url = resolve_release_url().map_err(|e| anyhow::anyhow!(e))?;
        println!("  Downloading: {url}");
        for msg in installer::seed_data_from_release(&url, dry_run)
            .map_err(|e| anyhow::anyhow!(e))?
        {
            println!("  {msg}");
        }
    }

    // --- Tool installation ---
    let mut selected: Vec<String> = if all || tools.iter().any(|t| t == "all") {
        vec!["claude", "cursor", "codex", "gemini", "opencode", "cline"]
            .into_iter()
            .map(|s| s.to_owned())
            .collect()
    } else if tools.is_empty() && io::stdin().is_terminal() {
        let installed: Vec<&str> = detect_installed_tools().into_iter().collect();
        let selected = match syntagma::adapters::install_wizard::interactive_select_tools(&installed) {
            Ok(s) if !s.is_empty() => s,
            Ok(_) => anyhow::bail!("install cancelled"),
            Err(e) => {
                eprintln!("Interactive UI failed ({e}); falling back to text prompt.");
                syntagma::adapters::install_wizard::fallback_select_tools()
            }
        };

        println!();
        println!("Step 2/3 \u{00b7} Environment \u{00b7} Redis cache");
        if prompt_yes_no("Configure Redis now? [Y/n]: ", true)? {
            let cfg = SyntagmaConfig::load().unwrap_or_default();
            let redis_enabled = prompt_bool_with_default(
                "  Redis enabled (true/false)",
                cfg.redis_enabled,
            )?;
            let redis_host = prompt_with_default("  host", &cfg.redis_host)?;
            let redis_port = prompt_u16_with_default("  port", cfg.redis_port)?;
            let redis_db = prompt_u16_with_default("  db", cfg.redis_db)?;
            let redis_ttl = prompt_u64_with_default("  ttl (seconds)", cfg.redis_ttl)?;
            upsert_config_yaml(redis_enabled, &redis_host, redis_port, redis_db, redis_ttl)?;
            println!("  Saved: ~/.syntagma/config.yaml");
        } else {
            println!("  Skipped - existing defaults are preserved.");
        }

        println!();
        println!("Step 3/3 \u{00b7} Telemetry");
        let enabled = syntagma::adapters::telemetry::prompt_consent_interactive();
        syntagma::adapters::telemetry::write_consent(enabled).map_err(|e| anyhow::anyhow!(e))?;

        selected
    } else {
        tools.to_vec()
    };
    selected.sort();
    selected.dedup();

    for tool in &selected {
        let result = match tool.as_str() {
            "claude" => installer::install_claude(dry_run),
            "cursor" => installer::install_cursor(dry_run),
            "codex" => installer::install_codex(dry_run),
            "gemini" => installer::install_gemini(dry_run),
            "opencode" => installer::install_opencode(dry_run),
            "cline" => installer::install_cline(dry_run),
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

    syntagma::adapters::telemetry::track_install_completed(selected.len());

    Ok(())
}

/// Find the newest `syntagma-data-*.tar.gz` in `dist/`.
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
        if name_str.starts_with("syntagma-data-") && name_str.ends_with(".tar.gz") {
            if let Ok(meta) = entry.metadata() {
                if let Ok(modified) = meta.modified() {
                    if newest.as_ref().map(|(_, t)| modified > *t).unwrap_or(true) {
                        newest = Some((entry.path(), modified));
                    }
                }
            }
        }
    }
    newest.map(|(p, _)| p)
}

/// Resolve the GitHub release download URL for the current version.
fn resolve_release_url() -> Result<String> {
    let version = get_package_version();
    let repo = "epicsagas/Syntagma";
    let prefix = "syntagma-data-";

    // Try exact version tag first, then latest
    for endpoint in [
        format!("https://api.github.com/repos/{repo}/releases/tags/v{version}"),
        format!("https://api.github.com/repos/{repo}/releases/latest"),
    ] {
        if let Ok(url) = fetch_release_asset_url(&endpoint, prefix) {
            return Ok(url);
        }
    }

    anyhow::bail!(
        "No data asset found for v{version}.\nCheck: https://github.com/{repo}/releases"
    )
}

fn get_package_version() -> String {
    std::env::var("SYNTAGMA_VERSION")
        .unwrap_or_else(|_| env!("CARGO_PKG_VERSION").to_owned())
}

fn fetch_release_asset_url(api_url: &str, prefix: &str) -> Result<String> {
    let output = std::process::Command::new("curl")
        .args(["-LfsS", "-H", "Accept: application/vnd.github+json", api_url])
        .output()
        .map_err(|e| anyhow::anyhow!("curl failed: {e}"))?;

    if !output.status.success() {
        anyhow::bail!("curl returned non-zero");
    }

    let json_str = String::from_utf8_lossy(&output.stdout);
    let val: serde_json::Value = serde_json::from_str(&json_str)
        .map_err(|e| anyhow::anyhow!("JSON parse: {e}"))?;

    if let Some(assets) = val.get("assets").and_then(|a| a.as_array()) {
        for asset in assets {
            let name = asset.get("name").and_then(|n| n.as_str()).unwrap_or("");
            if name.starts_with(prefix) && name.ends_with(".tar.gz") {
                if let Some(url) = asset.get("browser_download_url").and_then(|u| u.as_str()) {
                    return Ok(url.to_owned());
                }
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
        v.get(parent)
            .and_then(|m| m.get(child))
            .is_some()
    }

    let mut installed = HashSet::new();
    let home = std::env::var("HOME").unwrap_or_default();
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    if has_json_path(&PathBuf::from(&home).join(".claude.json"), "mcpServers", "syntagma") {
        installed.insert("claude");
    }
    if has_json_path(
        &PathBuf::from(&home).join(".cursor").join("mcp.json"),
        "mcpServers",
        "syntagma",
    ) {
        installed.insert("cursor");
    }
    if has_json_path(
        &PathBuf::from(&home).join(".gemini").join("mcp.json"),
        "mcpServers",
        "syntagma",
    ) {
        installed.insert("gemini");
    }
    if has_json_path(
        &PathBuf::from(&home)
            .join(".config")
            .join("opencode")
            .join("opencode.json"),
        "mcp",
        "syntagma",
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
        && (content.contains("SYNTAGMA-BEGIN") || content.contains("syntagma-mcp"))
    {
        installed.insert("codex");
    }
    installed
}

fn prompt_yes_no(prompt: &str, default_yes: bool) -> Result<bool> {
    print!("{prompt}");
    std::io::stdout().flush()?;
    let mut line = String::new();
    std::io::stdin().read_line(&mut line)?;
    let answer = line.trim().to_ascii_lowercase();
    if answer.is_empty() {
        return Ok(default_yes);
    }
    Ok(matches!(answer.as_str(), "y" | "yes"))
}

fn prompt_with_default(label: &str, default: &str) -> Result<String> {
    print!("{label} [{default}]: ");
    std::io::stdout().flush()?;
    let mut line = String::new();
    std::io::stdin().read_line(&mut line)?;
    let value = line.trim();
    if value.is_empty() {
        Ok(default.to_owned())
    } else {
        Ok(value.to_owned())
    }
}

fn prompt_bool_with_default(label: &str, default: bool) -> Result<bool> {
    let value = prompt_with_default(label, if default { "true" } else { "false" })?;
    Ok(!matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "false" | "0" | "no" | "n"
    ))
}

fn prompt_u16_with_default(label: &str, default: u16) -> Result<u16> {
    let value = prompt_with_default(label, &default.to_string())?;
    Ok(value.parse::<u16>().unwrap_or(default))
}

fn prompt_u64_with_default(label: &str, default: u64) -> Result<u64> {
    let value = prompt_with_default(label, &default.to_string())?;
    Ok(value.parse::<u64>().unwrap_or(default))
}

fn upsert_config_yaml(
    redis_enabled: bool,
    redis_host: &str,
    redis_port: u16,
    redis_db: u16,
    redis_ttl: u64,
) -> Result<()> {
    use serde_yaml::{Mapping, Value};
    let path = syntagma::adapters::paths::syntagma_home().join("config.yaml");
    let mut root = if path.exists() {
        let text = std::fs::read_to_string(&path)?;
        serde_yaml::from_str::<Value>(&text).unwrap_or_else(|_| Value::Mapping(Mapping::new()))
    } else {
        Value::Mapping(Mapping::new())
    };

    if !root.is_mapping() {
        root = Value::Mapping(Mapping::new());
    }
    let root_map = root
        .as_mapping_mut()
        .expect("mapping checked above");

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
    root_map.insert(
        Value::String("redis".to_owned()),
        Value::Mapping(redis_map),
    );

    std::fs::create_dir_all(syntagma::adapters::paths::syntagma_home())?;
    let yaml = serde_yaml::to_string(&root)?;
    std::fs::write(path, yaml)?;
    Ok(())
}
