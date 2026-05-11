use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::{Value, json};

/// Transport configuration for MCP integration across all AI tools.
#[derive(Debug, Clone, PartialEq)]
pub enum Transport {
    Http {
        port: u16,
        token: Option<String>,
    },
    Stdio,
}

impl Default for Transport {
    fn default() -> Self {
        Transport::Http {
            port: 43175,
            token: None,
        }
    }
}

fn mcp_server_config(transport: &Transport) -> Value {
    match transport {
        Transport::Http { port, token } => {
            let mut config = json!({
                "type": "http",
                "url": format!("http://127.0.0.1:{port}/mcp")
            });
            if let Some(t) = token {
                config
                    .as_object_mut()
                    .unwrap()
                    .insert(
                        "headers".to_owned(),
                        json!({ "Authorization": format!("Bearer {t}") }),
                    );
            }
            config
        }
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
    let legacy_removed = remove_legacy_keys(servers);

    if matches {
        messages.push("Claude Code: MCP already configured".to_owned());
    } else {
        servers.insert("episteme".to_owned(), desired);
        if !dry_run {
            write_json_file(&claude_json, &config)?;
        }
        let transport_label = match transport {
            Transport::Http { port, .. } => format!("HTTP, port {port}"),
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

    if !legacy_removed.is_empty() {
        if !dry_run {
            write_json_file(&claude_json, &config)?;
        }
        messages.push(format!(
            "Claude Code: removed legacy key(s): {}",
            legacy_removed.join(", ")
        ));
    }

    // Upsert registry artifacts (agents, skills) into ~/.claude/
    let claude_dir = home.join(".claude");
    let registry_msgs = upsert_registry_artifacts(&claude_dir, dry_run, "Claude Code")?;
    messages.extend(registry_msgs);

    Ok(messages)
}

/// Remove legacy MCP server keys (e.g. "syntagma") from a servers map.
/// Returns a list of removed key names.
fn remove_legacy_keys(servers: &mut serde_json::Map<String, Value>) -> Vec<String> {
    let legacy_keys = ["syntagma".to_owned()];
    let removed: Vec<String> = legacy_keys
        .iter()
        .filter(|k| servers.remove(k.as_str()).is_some())
        .cloned()
        .collect();
    removed
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
    let legacy_removed = remove_legacy_keys(servers);

    let mut msgs = Vec::new();

    if matches && legacy_removed.is_empty() {
        msgs.push("Cursor: MCP already configured".to_owned());
    } else {
        servers.insert("episteme".to_owned(), desired);
        if !dry_run {
            write_json_file(&mcp_json, &config)?;
        }
        let label = if existed { "updated" } else { "added" };
        msgs.push(format!("Cursor: MCP config {label}"));
        if !legacy_removed.is_empty() {
            msgs.push(format!(
                "Cursor: removed legacy key(s): {}",
                legacy_removed.join(", ")
            ));
        }
    }

    // Seed Episteme rules as .cursor/rules/episteme.mdc
    let rules_msgs = upsert_cursor_rules(dry_run, "Cursor")?;
    msgs.extend(rules_msgs);

    Ok(msgs)
}

/// Install AGENTS.md section for Codex and seed skills to ~/.codex/.
pub fn install_codex(dry_run: bool) -> Result<Vec<String>, String> {
    let home = dirs_home();
    let mut msgs = Vec::new();

    // Seed registry artifacts (agents + skills) into ~/.codex/
    let codex_dir = home.join(".codex");
    let registry_msgs = upsert_registry_artifacts(&codex_dir, dry_run, "Codex")?;
    msgs.extend(registry_msgs);

    // Also handle AGENTS.md in CWD
    let project_dir = std::env::current_dir().map_err(|e| e.to_string())?;
    let agents_md = project_dir.join("AGENTS.md");

    if agents_md.exists() {
        let content = fs::read_to_string(&agents_md).map_err(|e| e.to_string())?;
        if content.contains("epis mcp") || content.contains(EPISTEME_BEGIN) {
            msgs.push("Codex: AGENTS.md already configured".to_owned());
            let skill_msgs = upsert_skill_to_file(&agents_md, dry_run, "Codex")?;
            msgs.extend(skill_msgs);
            return Ok(msgs);
        }
    }

    if msgs.is_empty() {
        msgs.push("Codex: Add 'epis mcp' to AGENTS.md manually".to_owned());
    }
    Ok(msgs)
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
    let legacy_removed = remove_legacy_keys(servers);

    let mut msgs = Vec::new();

    if matches && legacy_removed.is_empty() {
        msgs.push("Gemini CLI: MCP already configured".to_owned());
    } else {
        servers.insert("episteme".to_owned(), desired);
        if !dry_run {
            write_json_file(&mcp_json, &config)?;
        }
        let label = if existed { "updated" } else { "added" };
        msgs.push(format!("Gemini CLI: MCP config {label}"));
        if !legacy_removed.is_empty() {
            msgs.push(format!(
                "Gemini CLI: removed legacy key(s): {}",
                legacy_removed.join(", ")
            ));
        }
    }

    // Seed registry artifacts (agents + skills) into ~/.gemini/
    let registry_msgs = upsert_registry_artifacts(&gemini_dir, dry_run, "Gemini CLI")?;
    msgs.extend(registry_msgs);

    // Also seed skill content into ~/.gemini/GEMINI.md for legacy compatibility
    let gemini_md = gemini_dir.join("GEMINI.md");
    let skill_msgs = upsert_skill_to_file(&gemini_md, dry_run, "Gemini CLI")?;
    msgs.extend(skill_msgs);

    Ok(msgs)
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
    let legacy_removed = remove_legacy_keys(servers);

    if matches && legacy_removed.is_empty() {
        return Ok(vec!["OpenCode: MCP already configured".to_owned()]);
    }

    servers.insert("episteme".to_owned(), desired);
    if !dry_run {
        write_json_file(&config_json, &config)?;
    }

    let label = if existed { "updated" } else { "added" };
    let mut msgs = vec![format!("OpenCode: MCP config {label}")];
    if !legacy_removed.is_empty() {
        msgs.push(format!(
            "OpenCode: removed legacy key(s): {}",
            legacy_removed.join(", ")
        ));
    }

    // Seed agents + skills to ~/.config/opencode/ (OpenCode supports agents/ natively)
    let registry_msgs = upsert_registry_artifacts(&opencode_dir, dry_run, "OpenCode")?;
    msgs.extend(registry_msgs);

    Ok(msgs)
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
    let legacy_removed = remove_legacy_keys(servers);

    let mut msgs = Vec::new();

    if matches && legacy_removed.is_empty() {
        msgs.push("Cline: MCP already configured".to_owned());
    } else {
        servers.insert("episteme".to_owned(), desired);
        if !dry_run {
            write_json_file(&mcp_json, &config)?;
        }
        let label = if existed { "updated" } else { "added" };
        msgs.push(format!("Cline: MCP config {label}"));
        if !legacy_removed.is_empty() {
            msgs.push(format!(
                "Cline: removed legacy key(s): {}",
                legacy_removed.join(", ")
            ));
        }
    }

    // Seed registry artifacts (agents + skills) into ~/.cline/
    let registry_msgs = upsert_registry_artifacts(&cline_dir, dry_run, "Cline")?;
    msgs.extend(registry_msgs);

    // Also seed skill content to ~/Documents/Cline/Rules/episteme.md
    let cline_rules = home
        .join("Documents")
        .join("Cline")
        .join("Rules")
        .join("episteme.md");
    let skill_msgs = upsert_skill_to_file(&cline_rules, dry_run, "Cline")?;
    msgs.extend(skill_msgs);

    Ok(msgs)
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
        messages
            .push("No local data found to seed. Run 'epis build' after providing data.".to_owned());
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

/// Upsert registry artifacts (agents + skills + hooks) into `dest_dir`.
/// Copies `~/.episteme/registry/agents/` → `dest_dir/agents/`,
/// `~/.episteme/registry/skills/<name>/` → `dest_dir/skills/<name>/`,
/// `~/.episteme/registry/hooks/` → `dest_dir/hooks/`.
fn upsert_registry_artifacts(
    dest_dir: &Path,
    dry_run: bool,
    label: &str,
) -> Result<Vec<String>, String> {
    let mut messages = Vec::new();
    let registry_src = crate::adapters::paths::episteme_home().join("registry");
    if !registry_src.is_dir() || dry_run {
        return Ok(messages);
    }

    // agents → dest_dir/agents/
    let agents_src = registry_src.join("agents");
    if agents_src.is_dir() {
        let agents_dst = dest_dir.join("agents");
        let (upserted, skipped) = upsert_dir(&agents_src, &agents_dst)?;
        messages.push(format!(
            "{label}: agents — {upserted} updated, {skipped} unchanged"
        ));
    }

    // skills/<name>/ → dest_dir/skills/<name>/
    let skills_src = registry_src.join("skills");
    if skills_src.is_dir() {
        let skills_dst = dest_dir.join("skills");
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
            "{label}: skills — {total_upserted} updated, {total_skipped} unchanged"
        ));
    }

    // hooks/ → dest_dir/hooks/  (flat files only)
    let hooks_src = registry_src.join("hooks");
    if hooks_src.is_dir() {
        let hooks_dst = dest_dir.join("hooks");
        let (upserted, skipped) = upsert_dir(&hooks_src, &hooks_dst)?;
        messages.push(format!(
            "{label}: hooks — {upserted} updated, {skipped} unchanged"
        ));
    }

    Ok(messages)
}

/// Read the skill content from `~/.episteme/registry/skills/episteme/SKILL.md`.
fn read_skill_content() -> Result<String, String> {
    let skill_path = crate::adapters::paths::episteme_home()
        .join("registry")
        .join("skills")
        .join("episteme")
        .join("SKILL.md");
    fs::read_to_string(&skill_path).map_err(|e| format!("skill not found: {e}"))
}

const EPISTEME_BEGIN: &str = "<!-- EPISTEME-BEGIN -->";
const EPISTEME_END: &str = "<!-- EPISTEME-END -->";

/// Upsert the Episteme skill content into a single file using section markers.
/// Creates the file if it doesn't exist. Only replaces the EPISTEME section.
fn upsert_skill_to_file(
    dest_file: &Path,
    dry_run: bool,
    label: &str,
) -> Result<Vec<String>, String> {
    let skill_content = read_skill_content()?;
    let section = format!("{EPISTEME_BEGIN}\n# Episteme\n\n{skill_content}\n{EPISTEME_END}\n");

    let existing = fs::read_to_string(dest_file).unwrap_or_default();
    let new_content = if existing.contains(EPISTEME_BEGIN) && existing.contains(EPISTEME_END) {
        let start = existing.find(EPISTEME_BEGIN).unwrap();
        let end = existing.find(EPISTEME_END).unwrap() + EPISTEME_END.len();
        format!("{}{}{}", &existing[..start], section, &existing[end..])
    } else {
        let separator = if existing.is_empty() { "" } else { "\n" };
        format!("{existing}{separator}{section}")
    };

    if new_content == existing {
        return Ok(vec![format!("{label}: skill already up-to-date")]);
    }

    if !dry_run {
        if let Some(parent) = dest_file.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        fs::write(dest_file, &new_content).map_err(|e| e.to_string())?;
    }
    Ok(vec![format!("{label}: skill seeded")])
}

/// Upsert the Episteme skill content as a Cursor `.mdc` rule file.
fn upsert_cursor_rules(dry_run: bool, label: &str) -> Result<Vec<String>, String> {
    let home = dirs_home();
    let rules_dir = home.join(".cursor").join("rules");
    let rule_file = rules_dir.join("episteme.mdc");

    let skill_content = read_skill_content()?;
    let mdc_content = format!(
        "---\ndescription: Episteme knowledge graph auto-trigger rules\nalwaysApply: true\n---\n\n{skill_content}\n"
    );

    if rule_file.exists() {
        let existing = fs::read_to_string(&rule_file).map_err(|e| e.to_string())?;
        if existing == mdc_content {
            return Ok(vec![format!("{label}: rules already up-to-date")]);
        }
    }

    if !dry_run {
        fs::create_dir_all(&rules_dir).map_err(|e| e.to_string())?;
        fs::write(&rule_file, &mdc_content).map_err(|e| e.to_string())?;
    }
    Ok(vec![format!(
        "{label}: rules seeded to .cursor/rules/episteme.mdc"
    )])
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
        let config = mcp_server_config(&Transport::Http {
            port: 43175,
            token: None,
        });
        assert_eq!(config["type"], "http");
        assert_eq!(config["url"], "http://127.0.0.1:43175/mcp");
        assert!(!config.as_object().unwrap().contains_key("headers"));
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
            Transport::Http {
                port: 43175,
                token: None,
            }
        );
    }

    #[test]
    fn mcp_server_config_http_custom_port() {
        let cfg = mcp_server_config(&Transport::Http {
            port: 8080,
            token: None,
        });
        assert_eq!(cfg["type"], "http");
        assert_eq!(cfg["url"], "http://127.0.0.1:8080/mcp");
    }

    #[test]
    fn mcp_server_config_http_with_token() {
        let cfg = mcp_server_config(&Transport::Http {
            port: 43175,
            token: Some("epis-abc123".to_owned()),
        });
        assert_eq!(cfg["type"], "http");
        assert_eq!(cfg["url"], "http://127.0.0.1:43175/mcp");
        assert_eq!(
            cfg["headers"]["Authorization"],
            "Bearer epis-abc123"
        );
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
            mcp_server_config(&Transport::Http {
                port: 43175,
                token: None,
            }),
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
        servers.insert("episteme".to_owned(), mcp_server_config(&Transport::Stdio));
        write_json_file(&path, &config).unwrap();

        let reloaded = read_json_file(&path);
        assert_eq!(reloaded["mcpServers"]["episteme"]["command"], "epis");
        assert_eq!(reloaded["mcpServers"]["episteme"]["args"], json!(["mcp"]));
    }

    #[test]
    fn upsert_skill_to_file_creates_new() {
        let dir = TempDir::new().unwrap();
        let _dest = dir.path().join("AGENTS.md");

        // Create a fake skill source
        let skill_dir = dir.path().join("registry").join("skills").join("episteme");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(skill_dir.join("SKILL.md"), "test skill content").unwrap();

        // upsert_skill_to_file reads from episteme home, so we can't easily test
        // without mocking. Instead test the marker logic directly.
        let existing = String::new();
        let section =
            format!("{EPISTEME_BEGIN}\n# Episteme\n\ntest skill content\n{EPISTEME_END}\n");
        let result = format!("{existing}{section}");
        assert!(result.contains(EPISTEME_BEGIN));
        assert!(result.contains("test skill content"));
        assert!(result.contains(EPISTEME_END));
    }

    #[test]
    fn upsert_skill_to_file_replaces_existing_section() {
        let old_section = format!("{EPISTEME_BEGIN}\n# Episteme\n\nold content\n{EPISTEME_END}");
        let existing = format!("Some header\n\n{old_section}\n\nOther content");

        let new_section = format!("{EPISTEME_BEGIN}\n# Episteme\n\nnew content\n{EPISTEME_END}\n");

        let start = existing.find(EPISTEME_BEGIN).unwrap();
        let end = existing.find(EPISTEME_END).unwrap() + EPISTEME_END.len();
        let result = format!("{}{}{}", &existing[..start], new_section, &existing[end..]);

        assert!(result.contains("Some header"));
        assert!(result.contains("new content"));
        assert!(!result.contains("old content"));
        assert!(result.contains("Other content"));
    }

    #[test]
    fn upsert_dir_skips_identical_files() {
        let src_dir = TempDir::new().unwrap();
        let dst_dir = TempDir::new().unwrap();

        fs::write(src_dir.path().join("a.txt"), "hello").unwrap();
        fs::write(dst_dir.path().join("a.txt"), "hello").unwrap();

        let (upserted, skipped) = upsert_dir(src_dir.path(), dst_dir.path()).unwrap();
        assert_eq!(upserted, 0);
        assert_eq!(skipped, 1);
    }

    #[test]
    fn upsert_dir_updates_changed_files() {
        let src_dir = TempDir::new().unwrap();
        let dst_dir = TempDir::new().unwrap();

        fs::write(src_dir.path().join("a.txt"), "new").unwrap();
        fs::write(dst_dir.path().join("a.txt"), "old").unwrap();

        let (upserted, skipped) = upsert_dir(src_dir.path(), dst_dir.path()).unwrap();
        assert_eq!(upserted, 1);
        assert_eq!(skipped, 0);
        assert_eq!(
            fs::read_to_string(dst_dir.path().join("a.txt")).unwrap(),
            "new"
        );
    }
}
