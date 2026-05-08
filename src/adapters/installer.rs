use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::{Value, json};

/// Transport configuration for MCP integration across all AI tools.
#[derive(Debug, Clone, PartialEq)]
pub enum Transport {
    Http { port: u16 },
    Stdio,
}

impl Default for Transport {
    fn default() -> Self {
        Transport::Http { port: 43175 }
    }
}

fn mcp_server_config(transport: &Transport) -> Value {
    match transport {
        Transport::Http { port } => json!({
            "type": "http",
            "url": format!("http://127.0.0.1:{port}/mcp")
        }),
        Transport::Stdio => json!({
            "command": "epis",
            "args": ["mcp"]
        }),
    }
}

/// Install MCP config for Claude Code (~/.claude.json).
pub fn install_claude(dry_run: bool, transport: &Transport) -> Result<Vec<String>, String> {
    let home = dirs_home();
    let claude_json = home.join(".claude.json");
    let mut messages = Vec::new();

    let mut config = read_json_file(&claude_json);
    let map = config.as_object_mut().ok_or("config is not an object")?;

    let servers = map
        .entry("mcpServers")
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .ok_or("mcpServers is not an object")?;

    let desired = mcp_server_config(transport);
    let existed = servers.contains_key("episteme");
    let matches = servers.get("episteme") == Some(&desired);

    if matches {
        messages.push("Claude Code: MCP already configured".to_owned());
    } else {
        servers.insert("episteme".to_owned(), desired);
        if !dry_run {
            write_json_file(&claude_json, &config)?;
        }
        let transport_label = match transport {
            Transport::Http { port } => format!("HTTP, port {port}"),
            Transport::Stdio => "stdio".to_owned(),
        };
        if existed {
            messages.push(format!(
                "Claude Code: MCP config updated ({transport_label})"
            ));
        } else {
            messages.push(format!("Claude Code: MCP config added ({transport_label})"));
        }
    }

    // Upsert registry artifacts (agents, skills) into ~/.claude/
    let registry_src = crate::adapters::paths::episteme_home().join("registry");
    if registry_src.is_dir() && !dry_run {
        let claude_dir = home.join(".claude");

        // agents → ~/.claude/agents/
        let agents_src = registry_src.join("agents");
        if agents_src.is_dir() {
            let agents_dst = claude_dir.join("agents");
            let (upserted, skipped) = upsert_dir(&agents_src, &agents_dst)?;
            messages.push(format!(
                "Claude Code: agents — {upserted} updated, {skipped} unchanged"
            ));
        }

        // skills/<name>/ → ~/.claude/skills/<name>/
        let skills_src = registry_src.join("skills");
        if skills_src.is_dir() {
            let skills_dst = claude_dir.join("skills");
            let mut total_upserted = 0usize;
            let mut total_skipped = 0usize;
            for entry in fs::read_dir(&skills_src).map_err(|e| e.to_string())? {
                let entry = entry.map_err(|e| e.to_string())?;
                if !entry.path().is_dir() {
                    continue;
                }
                let name = entry.file_name();
                let src = entry.path();
                let dst = skills_dst.join(&name);
                let (u, s) = upsert_dir(&src, &dst)?;
                total_upserted += u;
                total_skipped += s;
            }
            messages.push(format!(
                "Claude Code: skills — {total_upserted} updated, {total_skipped} unchanged"
            ));
        }

        // hooks/ → ~/.claude/hooks/  (flat files only)
        let hooks_src = registry_src.join("hooks");
        if hooks_src.is_dir() {
            let hooks_dst = claude_dir.join("hooks");
            let (upserted, skipped) = upsert_dir(&hooks_src, &hooks_dst)?;
            messages.push(format!(
                "Claude Code: hooks — {upserted} updated, {skipped} unchanged"
            ));
        }
    }

    Ok(messages)
}

/// Install MCP config for Cursor (~/.cursor/mcp.json).
pub fn install_cursor(dry_run: bool, transport: &Transport) -> Result<Vec<String>, String> {
    let home = dirs_home();
    let cursor_dir = home.join(".cursor");
    let mcp_json = cursor_dir.join("mcp.json");

    fs::create_dir_all(&cursor_dir).map_err(|e| e.to_string())?;

    let mut config = read_json_file(&mcp_json);
    let map = config.as_object_mut().ok_or("config is not an object")?;

    let servers = map
        .entry("mcpServers")
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .ok_or("mcpServers is not an object")?;

    let desired = mcp_server_config(transport);
    let existed = servers.contains_key("episteme");
    let matches = servers.get("episteme") == Some(&desired);

    if matches {
        return Ok(vec!["Cursor: MCP already configured".to_owned()]);
    }

    servers.insert("episteme".to_owned(), desired);
    if !dry_run {
        write_json_file(&mcp_json, &config)?;
    }

    let label = if existed { "updated" } else { "added" };
    Ok(vec![format!("Cursor: MCP config {label}")])
}

/// Install AGENTS.md section for Codex.
pub fn install_codex(dry_run: bool) -> Result<Vec<String>, String> {
    let project_dir = std::env::current_dir().map_err(|e| e.to_string())?;
    let agents_md = project_dir.join("AGENTS.md");

    if agents_md.exists() {
        let content = fs::read_to_string(&agents_md).map_err(|e| e.to_string())?;
        if content.contains("epis mcp") {
            return Ok(vec!["Codex: AGENTS.md already configured".to_owned()]);
        }
    }

    let _ = dry_run;
    // Don't modify AGENTS.md -- just report.
    Ok(vec![
        "Codex: Add 'epis mcp' to AGENTS.md manually".to_owned(),
    ])
}

/// Install MCP config for Gemini CLI (~/.gemini/mcp.json).
pub fn install_gemini(dry_run: bool, transport: &Transport) -> Result<Vec<String>, String> {
    let home = dirs_home();
    let gemini_dir = home.join(".gemini");
    let mcp_json = gemini_dir.join("mcp.json");

    fs::create_dir_all(&gemini_dir).map_err(|e| e.to_string())?;

    let mut config = read_json_file(&mcp_json);
    let map = config.as_object_mut().ok_or("config is not an object")?;

    let servers = map
        .entry("mcpServers")
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .ok_or("mcpServers is not an object")?;

    let desired = mcp_server_config(transport);
    let existed = servers.contains_key("episteme");
    let matches = servers.get("episteme") == Some(&desired);

    if matches {
        return Ok(vec!["Gemini CLI: MCP already configured".to_owned()]);
    }

    servers.insert("episteme".to_owned(), desired);
    if !dry_run {
        write_json_file(&mcp_json, &config)?;
    }

    let label = if existed { "updated" } else { "added" };
    Ok(vec![format!("Gemini CLI: MCP config {label}")])
}

/// Install MCP config for OpenCode (~/.config/opencode/opencode.json).
pub fn install_opencode(dry_run: bool, transport: &Transport) -> Result<Vec<String>, String> {
    let home = dirs_home();
    let opencode_dir = home.join(".config").join("opencode");
    let config_json = opencode_dir.join("opencode.json");

    fs::create_dir_all(&opencode_dir).map_err(|e| e.to_string())?;

    let mut config = read_json_file(&config_json);
    let map = config.as_object_mut().ok_or("config is not an object")?;

    let servers = map
        .entry("mcp")
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .ok_or("mcp is not an object")?;

    let desired = mcp_server_config(transport);
    let existed = servers.contains_key("episteme");
    let matches = servers.get("episteme") == Some(&desired);

    if matches {
        return Ok(vec!["OpenCode: MCP already configured".to_owned()]);
    }

    servers.insert("episteme".to_owned(), desired);
    if !dry_run {
        write_json_file(&config_json, &config)?;
    }

    let label = if existed { "updated" } else { "added" };
    Ok(vec![format!("OpenCode: MCP config {label}")])
}

/// Install MCP config for Cline (~/.cline/mcp.json).
pub fn install_cline(dry_run: bool, transport: &Transport) -> Result<Vec<String>, String> {
    let home = dirs_home();
    let cline_dir = home.join(".cline");
    let mcp_json = cline_dir.join("mcp.json");
    fs::create_dir_all(&cline_dir).map_err(|e| e.to_string())?;

    let mut config = read_json_file(&mcp_json);
    let map = config.as_object_mut().ok_or("config is not an object")?;
    let servers = map
        .entry("mcpServers")
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .ok_or("mcpServers is not an object")?;

    let desired = mcp_server_config(transport);
    let existed = servers.contains_key("episteme");
    let matches = servers.get("episteme") == Some(&desired);

    if matches {
        return Ok(vec!["Cline: MCP already configured".to_owned()]);
    }
    servers.insert("episteme".to_owned(), desired);
    if !dry_run {
        write_json_file(&mcp_json, &config)?;
    }
    let label = if existed { "updated" } else { "added" };
    Ok(vec![format!("Cline: MCP config {label}")])
}

/// Data seeding: copy raw data from project source tree to ~/.episteme/.
pub fn seed_data(dry_run: bool) -> Result<Vec<String>, String> {
    let data_dir = crate::adapters::paths::data_dir();
    let raw_dir = crate::adapters::paths::raw_dir();

    fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;
    fs::create_dir_all(&raw_dir).map_err(|e| e.to_string())?;

    let mut messages = Vec::new();

    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;

    // Seed registry/ (agent prompts etc.) alongside data.
    let registry_src = cwd.join("registry");
    let registry_dst = crate::adapters::paths::episteme_home().join("registry");
    if registry_src.exists() && !dry_run {
        fs::create_dir_all(&registry_dst).map_err(|e| e.to_string())?;
        copy_dir_recursive(&registry_src, &registry_dst)?;
    }

    let source_dirs: Vec<PathBuf> = vec![cwd.join("raw"), cwd.join("data"), cwd.join("meta")]
        .into_iter()
        .filter(|p| p.exists() && p.is_dir())
        .collect();

    for source in source_dirs {
        let target = if source.file_name() == Some(std::ffi::OsStr::new("raw")) {
            raw_dir.clone()
        } else {
            data_dir.clone()
        };
        if !dry_run {
            copy_dir_recursive(&source, &target)?;
        }
        messages.push(format!("Seeded data from {}", source.display()));
    }

    if messages.is_empty() {
        messages.push(
            "No local data found to seed. Run 'epis build' after providing data.".to_owned(),
        );
    }

    Ok(messages)
}

/// Seed data from a remote `.tar.gz` archive (typically GitHub Releases).
pub fn seed_data_from_release(url: &str, dry_run: bool) -> Result<Vec<String>, String> {
    let mut messages = Vec::new();
    if dry_run {
        messages.push(format!(
            "Would download and extract release archive from {url}"
        ));
        return Ok(messages);
    }

    let tmp_dir = std::env::temp_dir().join(format!("episteme-install-{}", std::process::id()));
    fs::create_dir_all(&tmp_dir).map_err(|e| e.to_string())?;
    let archive_path = tmp_dir.join("release.tar.gz");

    let status = Command::new("curl")
        .args(["-LfsS", url, "-o", archive_path.to_string_lossy().as_ref()])
        .status()
        .map_err(|e| format!("failed to execute curl: {e}"))?;
    if !status.success() {
        return Err(format!("failed to download archive from {url}"));
    }
    messages.push(format!("Downloaded archive from {url}"));

    let extract_dir = tmp_dir.join("extract");
    fs::create_dir_all(&extract_dir).map_err(|e| e.to_string())?;
    let tar_file = fs::File::open(&archive_path).map_err(|e| e.to_string())?;
    let gz = flate2::read::GzDecoder::new(tar_file);
    let mut archive = tar::Archive::new(gz);
    archive.unpack(&extract_dir).map_err(|e| e.to_string())?;
    messages.push("Extracted release archive".to_owned());

    // Copy discovered data folders into ~/.episteme
    let mut copied = false;
    for entry in fs::read_dir(&extract_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let root = entry.path();
        if !root.is_dir() {
            continue;
        }
        for dir in ["raw", "data", "meta", "db", "registry"] {
            let src = root.join(dir);
            if src.exists() && src.is_dir() {
                let target = match dir {
                    "raw" => crate::adapters::paths::raw_dir(),
                    "db" => crate::adapters::paths::db_path()
                        .parent()
                        .map(|p| p.to_path_buf())
                        .unwrap_or_else(|| crate::adapters::paths::episteme_home().join("db")),
                    "registry" => crate::adapters::paths::episteme_home().join("registry"),
                    _ => crate::adapters::paths::data_dir(),
                };
                fs::create_dir_all(&target).map_err(|e| e.to_string())?;
                copy_dir_recursive(&src, &target)?;
                messages.push(format!("Seeded {dir} from release archive"));
                copied = true;
            }
        }
    }
    if !copied {
        messages.push("Archive extracted but no raw/data/meta/db directories found".to_owned());
    }
    Ok(messages)
}

pub fn seed_data_from_local_archive(path: &Path, dry_run: bool) -> Result<Vec<String>, String> {
    if !path.exists() {
        return Err(format!("archive not found: {}", path.display()));
    }
    if dry_run {
        return Ok(vec![format!(
            "Would extract local archive from {}",
            path.display()
        )]);
    }
    let mut messages = Vec::new();
    let tmp_dir = std::env::temp_dir().join(format!("episteme-install-{}", std::process::id()));
    fs::create_dir_all(&tmp_dir).map_err(|e| e.to_string())?;
    let extract_dir = tmp_dir.join("extract-local");
    fs::create_dir_all(&extract_dir).map_err(|e| e.to_string())?;
    let tar_file = fs::File::open(path).map_err(|e| e.to_string())?;
    let gz = flate2::read::GzDecoder::new(tar_file);
    let mut archive = tar::Archive::new(gz);
    archive.unpack(&extract_dir).map_err(|e| e.to_string())?;
    messages.push(format!("Extracted local archive {}", path.display()));

    let mut copied = false;
    for entry in fs::read_dir(&extract_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let root = entry.path();
        if !root.is_dir() {
            continue;
        }
        for dir in ["raw", "data", "meta", "db", "registry"] {
            let src = root.join(dir);
            if src.exists() && src.is_dir() {
                let target = match dir {
                    "raw" => crate::adapters::paths::raw_dir(),
                    "db" => crate::adapters::paths::db_path()
                        .parent()
                        .map(|p| p.to_path_buf())
                        .unwrap_or_else(|| crate::adapters::paths::episteme_home().join("db")),
                    "registry" => crate::adapters::paths::episteme_home().join("registry"),
                    _ => crate::adapters::paths::data_dir(),
                };
                fs::create_dir_all(&target).map_err(|e| e.to_string())?;
                copy_dir_recursive(&src, &target)?;
                messages.push(format!("Seeded {dir} from local archive"));
                copied = true;
            }
        }
    }
    if !copied {
        messages.push("Archive extracted but no raw/data/meta directories found".to_owned());
    }
    Ok(messages)
}

/// Run all installers in sequence, collecting messages and tolerating
/// individual failures without aborting the whole operation.
pub fn install_all(dry_run: bool, transport: &Transport) -> Result<Vec<String>, String> {
    let mut messages = Vec::new();

    for result in [
        install_claude(dry_run, transport),
        install_cursor(dry_run, transport),
        install_codex(dry_run),
        install_gemini(dry_run, transport),
        install_opencode(dry_run, transport),
        install_cline(dry_run, transport),
    ] {
        match result {
            Ok(msgs) => messages.extend(msgs),
            Err(e) => messages.push(format!("Error: {e}")),
        }
    }

    Ok(messages)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn dirs_home() -> PathBuf {
    crate::adapters::paths::episteme_home()
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/tmp"))
}

fn read_json_file(path: &Path) -> Value {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(json!({}))
}

fn write_json_file(path: &Path, value: &Value) -> Result<(), String> {
    let content = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
    fs::write(path, content).map_err(|e| e.to_string())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    if !dst.exists() {
        fs::create_dir_all(dst).map_err(|e| e.to_string())?;
    }
    for entry in fs::read_dir(src).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

/// Copy files from `src` into `dst`, only overwriting when content differs.
/// Returns (upserted_count, skipped_count).
fn upsert_dir(src: &Path, dst: &Path) -> Result<(usize, usize), String> {
    if !dst.exists() {
        fs::create_dir_all(dst).map_err(|e| e.to_string())?;
    }
    let mut upserted = 0usize;
    let mut skipped = 0usize;
    for entry in fs::read_dir(src).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            let (u, s) = upsert_dir(&src_path, &dst_path)?;
            upserted += u;
            skipped += s;
        } else {
            let src_content = fs::read(&src_path).map_err(|e| e.to_string())?;
            let needs_write = match fs::read(&dst_path) {
                Ok(dst_content) => dst_content != src_content,
                Err(_) => true,
            };
            if needs_write {
                fs::write(&dst_path, &src_content).map_err(|e| e.to_string())?;
                upserted += 1;
            } else {
                skipped += 1;
            }
        }
    }
    Ok((upserted, skipped))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn mcp_server_config_stdio_has_expected_shape() {
        let config = mcp_server_config(&Transport::Stdio);
        assert_eq!(config["command"], "epis");
        assert_eq!(config["args"], json!(["mcp"]));
    }

    #[test]
    fn mcp_server_config_http_has_expected_shape() {
        let config = mcp_server_config(&Transport::Http { port: 43175 });
        assert_eq!(config["type"], "http");
        assert_eq!(config["url"], "http://127.0.0.1:43175/mcp");
    }

    #[test]
    fn read_json_file_missing_returns_empty_object() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("nonexistent.json");
        let value = read_json_file(&path);
        assert!(value.is_object());
        assert!(value.as_object().unwrap().is_empty());
    }

    #[test]
    fn write_and_read_roundtrip() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.json");
        let original = json!({"key": "value", "num": 42});
        write_json_file(&path, &original).unwrap();
        let loaded = read_json_file(&path);
        assert_eq!(original, loaded);
    }

    #[test]
    fn install_claude_fresh() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join(".claude.json");
        // Write an empty object so read finds it.
        fs::write(&path, "{}").unwrap();

        // We cannot easily redirect dirs_home(), so test the config logic directly.
        let mut config = read_json_file(&path);
        let map = config.as_object_mut().unwrap();
        let mcp_servers = map.entry("mcpServers").or_insert_with(|| json!({}));
        let servers = mcp_servers.as_object_mut().unwrap();
        assert!(!servers.contains_key("episteme"));
        servers.insert("episteme".to_owned(), mcp_server_config(&Transport::Stdio));
        write_json_file(&path, &config).unwrap();

        let reloaded = read_json_file(&path);
        assert!(reloaded["mcpServers"]["episteme"]["command"] == "epis");
    }

    #[test]
    fn install_claude_idempotent() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join(".claude.json");

        // First install.
        let mut config = json!({});
        let map = config.as_object_mut().unwrap();
        let mcp_servers = map.entry("mcpServers").or_insert_with(|| json!({}));
        let servers = mcp_servers.as_object_mut().unwrap();
        servers.insert("episteme".to_owned(), mcp_server_config(&Transport::Stdio));
        write_json_file(&path, &config).unwrap();

        // Second pass should detect existing.
        let config = read_json_file(&path);
        let servers = config["mcpServers"].as_object().unwrap();
        assert!(servers.contains_key("episteme"));
    }

    #[test]
    fn copy_dir_recursive_copies_files() {
        let src_dir = TempDir::new().unwrap();
        let dst_dir = TempDir::new().unwrap();

        fs::write(src_dir.path().join("a.txt"), "hello").unwrap();
        fs::create_dir_all(src_dir.path().join("sub")).unwrap();
        fs::write(src_dir.path().join("sub").join("b.txt"), "world").unwrap();

        copy_dir_recursive(src_dir.path(), dst_dir.path()).unwrap();

        assert_eq!(
            fs::read_to_string(dst_dir.path().join("a.txt")).unwrap(),
            "hello"
        );
        assert_eq!(
            fs::read_to_string(dst_dir.path().join("sub").join("b.txt")).unwrap(),
            "world"
        );
    }

    #[test]
    fn seed_data_no_sources() {
        // Use a temp dir as cwd so there are no raw/data/meta dirs.
        let dir = TempDir::new().unwrap();
        let original = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        // dry_run=true avoids actual file ops but should still produce message.
        let msgs = seed_data(true).unwrap();
        assert!(msgs.iter().any(|m| m.contains("No local data found")));

        std::env::set_current_dir(original).unwrap();
    }

    #[test]
    fn transport_default_is_http_43175() {
        assert_eq!(
            Transport::default(),
            Transport::Http { port: 43175 }
        );
    }

    #[test]
    fn mcp_server_config_http_custom_port() {
        let cfg = mcp_server_config(&Transport::Http { port: 8080 });
        assert_eq!(cfg["type"], "http");
        assert_eq!(cfg["url"], "http://127.0.0.1:8080/mcp");
    }

    #[test]
    fn mcp_server_config_stdio() {
        let cfg = mcp_server_config(&Transport::Stdio);
        assert_eq!(cfg["command"], "epis");
        assert_eq!(cfg["args"], json!(["mcp"]));
    }

    #[test]
    fn install_claude_http_writes_url() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("claude.json");
        fs::write(&path, "{}").unwrap();

        let mut config = read_json_file(&path);
        let map = config.as_object_mut().unwrap();
        let servers = map
            .entry("mcpServers")
            .or_insert_with(|| json!({}))
            .as_object_mut()
            .unwrap();
        servers.insert(
            "episteme".to_owned(),
            mcp_server_config(&Transport::Http { port: 43175 }),
        );
        write_json_file(&path, &config).unwrap();

        let reloaded = read_json_file(&path);
        assert_eq!(reloaded["mcpServers"]["episteme"]["type"], "http");
        assert_eq!(
            reloaded["mcpServers"]["episteme"]["url"],
            "http://127.0.0.1:43175/mcp"
        );
    }

    #[test]
    fn install_claude_stdio_writes_command() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("claude.json");
        fs::write(&path, "{}").unwrap();

        let mut config = read_json_file(&path);
        let map = config.as_object_mut().unwrap();
        let servers = map
            .entry("mcpServers")
            .or_insert_with(|| json!({}))
            .as_object_mut()
            .unwrap();
        servers.insert(
            "episteme".to_owned(),
            mcp_server_config(&Transport::Stdio),
        );
        write_json_file(&path, &config).unwrap();

        let reloaded = read_json_file(&path);
        assert_eq!(reloaded["mcpServers"]["episteme"]["command"], "epis");
        assert_eq!(reloaded["mcpServers"]["episteme"]["args"], json!(["mcp"]));
    }
}
