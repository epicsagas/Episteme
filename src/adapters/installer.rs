use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::{json, Value};

/// stdio config for Cursor, Codex, Gemini, OpenCode, Cline.
fn mcp_server_config() -> Value {
    json!({
        "command": "syntagma-mcp",
        "args": []
    })
}

/// HTTP transport config for Claude Code.
/// Runs `syntagma mcp --http --port <port>` as a subprocess; Claude Code connects via HTTP.
/// Port is read from config (default 43175, overridable via SYNTAGMA_MCP_PORT or config.yaml).
fn claude_mcp_server_config(port: u16) -> Value {
    json!({
        "command": "syntagma",
        "args": ["mcp", "--http", "--port", port.to_string()],
        "type": "http"
    })
}

/// Install MCP config for Claude Code (~/.claude.json).
pub fn install_claude(dry_run: bool) -> Result<Vec<String>, String> {
    let home = dirs_home();
    let claude_json = home.join(".claude.json");
    let mut messages = Vec::new();

    let port = crate::adapters::config::SyntagmaConfig::load()
        .map(|c| c.mcp_port)
        .unwrap_or(43175);

    let mut config = read_json_file(&claude_json);
    let map = config
        .as_object_mut()
        .ok_or("config is not an object")?;

    let servers = map
        .entry("mcpServers")
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .ok_or("mcpServers is not an object")?;

    if servers.contains_key("syntagma") {
        messages.push("Claude Code: MCP already configured".to_owned());
    } else {
        servers.insert("syntagma".to_owned(), claude_mcp_server_config(port));
        if !dry_run {
            write_json_file(&claude_json, &config)?;
        }
        messages.push(format!("Claude Code: MCP config added (HTTP, port {port})"));
    }

    Ok(messages)
}

/// Install MCP config for Cursor (~/.cursor/mcp.json).
pub fn install_cursor(dry_run: bool) -> Result<Vec<String>, String> {
    let home = dirs_home();
    let cursor_dir = home.join(".cursor");
    let mcp_json = cursor_dir.join("mcp.json");

    fs::create_dir_all(&cursor_dir).map_err(|e| e.to_string())?;

    let mut config = read_json_file(&mcp_json);
    let map = config
        .as_object_mut()
        .ok_or("config is not an object")?;

    let servers = map
        .entry("mcpServers")
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .ok_or("mcpServers is not an object")?;

    if servers.contains_key("syntagma") {
        return Ok(vec!["Cursor: MCP already configured".to_owned()]);
    }

    servers.insert("syntagma".to_owned(), mcp_server_config());
    if !dry_run {
        write_json_file(&mcp_json, &config)?;
    }

    Ok(vec!["Cursor: MCP config added".to_owned()])
}

/// Install AGENTS.md section for Codex.
pub fn install_codex(dry_run: bool) -> Result<Vec<String>, String> {
    let project_dir = std::env::current_dir().map_err(|e| e.to_string())?;
    let agents_md = project_dir.join("AGENTS.md");

    if agents_md.exists() {
        let content = fs::read_to_string(&agents_md).map_err(|e| e.to_string())?;
        if content.contains("syntagma-mcp") {
            return Ok(vec!["Codex: AGENTS.md already configured".to_owned()]);
        }
    }

    let _ = dry_run;
    // Don't modify AGENTS.md -- just report.
    Ok(vec![
        "Codex: Add 'syntagma-mcp' to AGENTS.md manually".to_owned(),
    ])
}

/// Install MCP config for Gemini CLI (~/.gemini/mcp.json).
pub fn install_gemini(dry_run: bool) -> Result<Vec<String>, String> {
    let home = dirs_home();
    let gemini_dir = home.join(".gemini");
    let mcp_json = gemini_dir.join("mcp.json");

    fs::create_dir_all(&gemini_dir).map_err(|e| e.to_string())?;

    let mut config = read_json_file(&mcp_json);
    let map = config
        .as_object_mut()
        .ok_or("config is not an object")?;

    let servers = map
        .entry("mcpServers")
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .ok_or("mcpServers is not an object")?;

    if servers.contains_key("syntagma") {
        return Ok(vec!["Gemini CLI: MCP already configured".to_owned()]);
    }

    servers.insert("syntagma".to_owned(), mcp_server_config());
    if !dry_run {
        write_json_file(&mcp_json, &config)?;
    }

    Ok(vec!["Gemini CLI: MCP config added".to_owned()])
}

/// Install MCP config for OpenCode (~/.config/opencode/opencode.json).
pub fn install_opencode(dry_run: bool) -> Result<Vec<String>, String> {
    let home = dirs_home();
    let opencode_dir = home.join(".config").join("opencode");
    let config_json = opencode_dir.join("opencode.json");

    fs::create_dir_all(&opencode_dir).map_err(|e| e.to_string())?;

    let mut config = read_json_file(&config_json);
    let map = config
        .as_object_mut()
        .ok_or("config is not an object")?;

    let servers = map
        .entry("mcp")
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .ok_or("mcp is not an object")?;

    if servers.contains_key("syntagma") {
        return Ok(vec!["OpenCode: MCP already configured".to_owned()]);
    }

    servers.insert("syntagma".to_owned(), mcp_server_config());
    if !dry_run {
        write_json_file(&config_json, &config)?;
    }

    Ok(vec!["OpenCode: MCP config added".to_owned()])
}

/// Install MCP config for Cline (~/.cline/mcp.json).
pub fn install_cline(dry_run: bool) -> Result<Vec<String>, String> {
    let home = dirs_home();
    let cline_dir = home.join(".cline");
    let mcp_json = cline_dir.join("mcp.json");
    fs::create_dir_all(&cline_dir).map_err(|e| e.to_string())?;

    let mut config = read_json_file(&mcp_json);
    let map = config
        .as_object_mut()
        .ok_or("config is not an object")?;
    let servers = map
        .entry("mcpServers")
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .ok_or("mcpServers is not an object")?;
    if servers.contains_key("syntagma") {
        return Ok(vec!["Cline: MCP already configured".to_owned()]);
    }
    servers.insert("syntagma".to_owned(), mcp_server_config());
    if !dry_run {
        write_json_file(&mcp_json, &config)?;
    }
    Ok(vec!["Cline: MCP config added".to_owned()])
}

/// Data seeding: copy raw data from project source tree to ~/.syntagma/.
pub fn seed_data(dry_run: bool) -> Result<Vec<String>, String> {
    let data_dir = crate::adapters::paths::data_dir();
    let raw_dir = crate::adapters::paths::raw_dir();

    fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;
    fs::create_dir_all(&raw_dir).map_err(|e| e.to_string())?;

    let mut messages = Vec::new();

    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let source_dirs: Vec<PathBuf> = vec![
        cwd.join("raw"),
        cwd.join("data"),
        cwd.join("meta"),
    ]
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
            "No local data found to seed. Run 'syntagma build' after providing data.".to_owned(),
        );
    }

    Ok(messages)
}

/// Seed data from a remote `.tar.gz` archive (typically GitHub Releases).
pub fn seed_data_from_release(url: &str, dry_run: bool) -> Result<Vec<String>, String> {
    let mut messages = Vec::new();
    if dry_run {
        messages.push(format!("Would download and extract release archive from {url}"));
        return Ok(messages);
    }

    let tmp_dir = std::env::temp_dir().join(format!("syntagma-install-{}", std::process::id()));
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

    // Copy discovered data folders into ~/.syntagma
    let mut copied = false;
    for entry in fs::read_dir(&extract_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let root = entry.path();
        if !root.is_dir() {
            continue;
        }
        for dir in ["raw", "data", "meta", "db"] {
            let src = root.join(dir);
            if src.exists() && src.is_dir() {
                let target = match dir {
                    "raw" => crate::adapters::paths::raw_dir(),
                    "db" => crate::adapters::paths::db_path()
                        .parent()
                        .map(|p| p.to_path_buf())
                        .unwrap_or_else(|| crate::adapters::paths::syntagma_home().join("db")),
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
    let tmp_dir = std::env::temp_dir().join(format!("syntagma-install-{}", std::process::id()));
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
        for dir in ["raw", "data", "meta", "db"] {
            let src = root.join(dir);
            if src.exists() && src.is_dir() {
                let target = match dir {
                    "raw" => crate::adapters::paths::raw_dir(),
                    "db" => crate::adapters::paths::db_path()
                        .parent()
                        .map(|p| p.to_path_buf())
                        .unwrap_or_else(|| crate::adapters::paths::syntagma_home().join("db")),
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
pub fn install_all(dry_run: bool) -> Result<Vec<String>, String> {
    let mut messages = Vec::new();

    for result in [
        install_claude(dry_run),
        install_cursor(dry_run),
        install_codex(dry_run),
        install_gemini(dry_run),
        install_opencode(dry_run),
        install_cline(dry_run),
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
    crate::adapters::paths::syntagma_home()
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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn mcp_server_config_has_expected_shape() {
        let config = mcp_server_config();
        assert_eq!(config["command"], "syntagma-mcp");
        assert!(config["args"].is_array());
    }

    #[test]
    fn claude_mcp_server_config_uses_http_transport() {
        let config = claude_mcp_server_config(43175);
        assert_eq!(config["command"], "syntagma");
        assert_eq!(config["type"], "http");
        assert_eq!(config["args"][0], "mcp");
        assert_eq!(config["args"][1], "--http");
        assert_eq!(config["args"][2], "--port");
        assert_eq!(config["args"][3], "43175");
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
        let mcp_servers = map
            .entry("mcpServers")
            .or_insert_with(|| json!({}));
        let servers = mcp_servers.as_object_mut().unwrap();
        assert!(!servers.contains_key("syntagma"));
        servers.insert("syntagma".to_owned(), claude_mcp_server_config(43175));
        write_json_file(&path, &config).unwrap();

        let reloaded = read_json_file(&path);
        assert_eq!(reloaded["mcpServers"]["syntagma"]["command"], "syntagma");
        assert_eq!(reloaded["mcpServers"]["syntagma"]["type"], "http");
    }

    #[test]
    fn install_claude_idempotent() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join(".claude.json");

        // First install.
        let mut config = json!({});
        let map = config.as_object_mut().unwrap();
        let mcp_servers = map
            .entry("mcpServers")
            .or_insert_with(|| json!({}));
        let servers = mcp_servers.as_object_mut().unwrap();
        servers.insert("syntagma".to_owned(), mcp_server_config());
        write_json_file(&path, &config).unwrap();

        // Second pass should detect existing.
        let config = read_json_file(&path);
        let servers = config["mcpServers"].as_object().unwrap();
        assert!(servers.contains_key("syntagma"));
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
}
