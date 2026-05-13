//! Daemon management: start / stop / status helpers for MCP HTTP and API servers.
//!
//! Ported from Python `cli/service.py`. Supports dual PID files via
//! [`ServiceKind`] and cross-platform OS service registration (macOS launchd,
//! Linux systemd).

use std::fs;
use std::net::TcpListener;
#[cfg(any(target_os = "macos", target_os = "linux"))]
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crate::adapters::paths;

// ---------------------------------------------------------------------------
// ServiceKind
// ---------------------------------------------------------------------------

/// Discriminates between the two background daemon types Episteme manages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceKind {
    Mcp,
    Api,
}

/// Return the PID file path for the given service kind.
pub fn pid_file_for(kind: ServiceKind) -> PathBuf {
    let name = match kind {
        ServiceKind::Mcp => "mcp.pid",
        ServiceKind::Api => "api.pid",
    };
    paths::episteme_home().join(name)
}

// ---------------------------------------------------------------------------
// PID file helpers (ServiceKind-aware)
// ---------------------------------------------------------------------------

/// Read the PID from the kind-specific PID file, if present and valid.
pub fn read_pid_for(kind: ServiceKind) -> Option<u32> {
    let path = pid_file_for(kind);
    fs::read_to_string(&path)
        .ok()
        .and_then(|s| s.trim().parse().ok())
}

/// Write `pid` to the kind-specific PID file.
pub fn write_pid_for(kind: ServiceKind, pid: u32) -> std::io::Result<()> {
    let path = pid_file_for(kind);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, pid.to_string())
}

/// Remove the kind-specific PID file (idempotent).
pub fn clear_pid_for(kind: ServiceKind) -> std::io::Result<()> {
    let path = pid_file_for(kind);
    if path.exists() {
        fs::remove_file(path)
    } else {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// PID file helpers (backward-compat wrappers defaulting to Mcp)
// ---------------------------------------------------------------------------

/// Read the PID from the MCP PID file. Backward-compatible wrapper.
pub fn read_pid() -> Option<u32> {
    read_pid_for(ServiceKind::Mcp)
}

/// Write `pid` to the MCP PID file. Backward-compatible wrapper.
pub fn write_pid(pid: u32) -> std::io::Result<()> {
    write_pid_for(ServiceKind::Mcp, pid)
}

/// Remove the MCP PID file. Backward-compatible wrapper.
pub fn clear_pid() -> std::io::Result<()> {
    clear_pid_for(ServiceKind::Mcp)
}

// ---------------------------------------------------------------------------
// Process helpers
// ---------------------------------------------------------------------------

/// Check whether a process with the given PID is still alive.
///
/// On non-Unix targets this always returns `false`.
pub fn is_running(pid: u32) -> bool {
    #[cfg(unix)]
    {
        // SAFETY: `kill` with signal 0 performs no action; it only checks
        // process existence and permissions. No state is mutated.
        unsafe { libc::kill(pid as libc::pid_t, 0) == 0 }
    }
    #[cfg(not(unix))]
    {
        let _ = pid;
        false
    }
}

/// Find the PID of the process listening on `port` using `lsof`.
pub fn find_pid_by_port(port: u16) -> Option<u32> {
    let output = Command::new("lsof")
        .args(["-ti", &format!(":{port}")])
        .output()
        .ok()?;
    if output.status.success() {
        String::from_utf8_lossy(&output.stdout).trim().parse().ok()
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Port helpers
// ---------------------------------------------------------------------------

/// Returns `true` when something is already bound on `127.0.0.1:port`.
pub fn is_port_in_use(port: u16) -> bool {
    TcpListener::bind(("127.0.0.1", port)).is_err()
}

/// Poll until the port is free, up to `timeout_secs`.
/// Returns `true` if the port became free within the deadline.
pub fn wait_port_free(port: u16, timeout_secs: u64) -> bool {
    let start = Instant::now();
    let deadline = Duration::from_secs(timeout_secs);
    while start.elapsed() < deadline {
        if !is_port_in_use(port) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    false
}

/// Poll until something is listening on `port`, up to `timeout_secs`.
/// Returns `true` if the port came up within the deadline.
pub fn wait_port_open(port: u16, timeout_secs: u64) -> bool {
    let start = Instant::now();
    let deadline = Duration::from_secs(timeout_secs);
    while start.elapsed() < deadline {
        if is_port_in_use(port) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    false
}

// ---------------------------------------------------------------------------
// Config helpers per ServiceKind
// ---------------------------------------------------------------------------

/// Resolve the configured host and port for the given service kind.
/// Loads config once.
fn get_host_port(kind: ServiceKind) -> (String, u16) {
    let cfg = crate::adapters::config::EpistemeConfig::load().ok();
    match kind {
        ServiceKind::Mcp => {
            let host = cfg
                .as_ref()
                .map(|c| c.mcp_host.clone())
                .unwrap_or_else(|| "127.0.0.1".to_owned());
            let port = cfg.as_ref().map(|c| c.mcp_port).unwrap_or(43175);
            (host, port)
        }
        ServiceKind::Api => {
            let host = cfg
                .as_ref()
                .map(|c| c.api_host.clone())
                .unwrap_or_else(|| "0.0.0.0".to_owned());
            let port = cfg.as_ref().map(|c| c.api_port).unwrap_or(8000);
            (host, port)
        }
    }
}

/// Resolve the configured port for the given service kind.
fn get_port(kind: ServiceKind) -> u16 {
    get_host_port(kind).1
}

/// Resolve the configured bearer token for MCP, if any.
fn get_token(kind: ServiceKind) -> Option<String> {
    match kind {
        ServiceKind::Mcp => {
            let cfg = crate::adapters::config::EpistemeConfig::load().ok()?;
            if cfg.mcp_token.is_empty() {
                None
            } else {
                Some(cfg.mcp_token)
            }
        }
        ServiceKind::Api => None,
    }
}

/// Return a human-readable label for the service kind.
pub fn kind_label(kind: ServiceKind) -> &'static str {
    match kind {
        ServiceKind::Mcp => "MCP",
        ServiceKind::Api => "API",
    }
}

// ---------------------------------------------------------------------------
// High-level service commands (ServiceKind-aware)
// ---------------------------------------------------------------------------

/// Spawn the server in the background for the given `kind`.
///
/// Returns the child PID on success, or `Ok(0)` if the server was already running.
pub fn cmd_start(kind: ServiceKind, host: &str, port: u16) -> Result<u32, String> {
    if is_port_in_use(port) {
        return Ok(0);
    }

    let exe = std::env::current_exe().map_err(|e| format!("cannot determine current exe: {e}"))?;

    let log_dir = paths::log_dir();
    fs::create_dir_all(&log_dir).map_err(|e| format!("cannot create log dir: {e}"))?;

    let (log_name, err_name) = match kind {
        ServiceKind::Mcp => ("mcp.log", "mcp.err"),
        ServiceKind::Api => ("api.log", "api.err"),
    };
    let stdout_path = log_dir.join(log_name);
    let stderr_path = log_dir.join(err_name);

    let stdout = fs::File::create(&stdout_path)
        .map_err(|e| format!("cannot create {}: {e}", stdout_path.display()))?;
    let stderr = fs::File::create(&stderr_path)
        .map_err(|e| format!("cannot create {}: {e}", stderr_path.display()))?;

    let args: Vec<String> = match kind {
        ServiceKind::Mcp => vec![
            "mcp".into(),
            "serve".into(),
            "--host".into(),
            host.into(),
            "--port".into(),
            port.to_string(),
        ],
        ServiceKind::Api => vec![
            "api".into(),
            "serve".into(),
            "--host".into(),
            host.into(),
            "--port".into(),
            port.to_string(),
        ],
    };

    let child = Command::new(&exe)
        .args(&args)
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr))
        .spawn()
        .map_err(|e| format!("failed to spawn server: {e}"))?;

    let pid = child.id();
    write_pid_for(kind, pid).map_err(|e| format!("failed to write PID file: {e}"))?;

    if wait_port_open(port, 10) {
        Ok(pid)
    } else {
        // Best-effort cleanup.
        let _ = clear_pid_for(kind);
        Err("Server failed to start within 10 seconds".to_owned())
    }
}

/// Stop the running server for the given `kind` (SIGTERM, then SIGKILL on timeout).
///
/// When a launchd agent is loaded for this service kind, unloads it first so that
/// macOS does not immediately re-spawn the process.
///
/// Works even without a PID file by discovering the process via the port it binds.
pub fn cmd_stop(kind: ServiceKind) -> Result<(), String> {
    let port = get_port(kind);

    // Unload launchd agent first to prevent immediate re-spawn.
    #[cfg(target_os = "macos")]
    {
        let label = launch_agent_label(kind);
        let loaded = Command::new("launchctl")
            .args(["list", label])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if loaded {
            let uid = Command::new("id")
                .arg("-u")
                .output()
                .ok()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned())
                .unwrap_or_default();
            let _ = Command::new("launchctl")
                .args(["bootout", &format!("gui/{uid}/{label}")])
                .status();
        }
    }

    // Resolve PID: prefer PID file, fall back to port-based discovery.
    let pid = match read_pid_for(kind) {
        Some(p) => p,
        None => match find_pid_by_port(port) {
            Some(p) => p,
            None => {
                // Nothing running — just clean up stale PID file.
                clear_pid_for(kind).ok();
                return Ok(());
            }
        },
    };

    if !is_running(pid) {
        clear_pid_for(kind).ok();
        return Ok(());
    }

    // Graceful SIGTERM.
    #[cfg(unix)]
    {
        // SAFETY: Sending SIGTERM is a standard POSIX signal delivery. The
        // kernel validates `pid`; no memory-safety invariants are violated.
        unsafe {
            libc::kill(pid as libc::pid_t, libc::SIGTERM);
        }
    }
    #[cfg(not(unix))]
    {
        // On non-Unix, try to kill via the OS command.
        let _ = Command::new("kill").arg(pid.to_string()).output();
    }

    if wait_port_free(port, 5) {
        clear_pid_for(kind).ok();
        return Ok(());
    }

    // Force SIGKILL.
    #[cfg(unix)]
    {
        // SAFETY: SIGKILL is immediate process termination. Same reasoning as
        // SIGTERM above.
        unsafe {
            libc::kill(pid as libc::pid_t, libc::SIGKILL);
        }
    }
    #[cfg(windows)]
    {
        let _ = Command::new("taskkill")
            .args(["/F", "/PID", &pid.to_string()])
            .output();
    }
    #[cfg(all(not(unix), not(windows)))]
    {
        let _ = Command::new("kill").args(["-9", &pid.to_string()]).output();
    }

    // Give it a moment, then clean up PID file regardless.
    std::thread::sleep(Duration::from_millis(500));
    clear_pid_for(kind).ok();
    Ok(())
}

/// Print a human-readable status line for `kind` and return `true` if up.
pub fn cmd_status(kind: ServiceKind) -> bool {
    let port = get_port(kind);
    let label = kind_label(kind);

    match read_pid_for(kind) {
        Some(pid) => {
            if is_running(pid) {
                let port_status = if is_port_in_use(port) {
                    format!("listening on port {port}")
                } else {
                    format!("not yet listening on port {port}")
                };
                println!("{label} server is running (PID {pid}, {port_status})");
                true
            } else {
                println!("{label} server is NOT running (stale PID {pid})");
                clear_pid_for(kind).ok();
                false
            }
        }
        None => {
            // No PID file -- but maybe the server is running from another source.
            if is_port_in_use(port)
                && let Some(pid) = find_pid_by_port(port)
            {
                println!(
                    "{label} server appears to be running (PID {pid} on port {port}), but no PID file"
                );
                return true;
            }
            println!("{label} server is stopped");
            false
        }
    }
}

// ---------------------------------------------------------------------------
// Backward-compat wrappers (no ServiceKind arg)
// ---------------------------------------------------------------------------

/// Spawn the MCP HTTP server in the background. Backward-compatible wrapper.
pub fn cmd_start_mcp(host: &str, port: u16) -> Result<u32, String> {
    cmd_start(ServiceKind::Mcp, host, port)
}

/// Stop the running MCP server. Backward-compatible wrapper.
pub fn cmd_stop_mcp() -> Result<(), String> {
    cmd_stop(ServiceKind::Mcp)
}

/// Print MCP status. Backward-compatible wrapper.
pub fn cmd_status_mcp() -> bool {
    cmd_status(ServiceKind::Mcp)
}

// ---------------------------------------------------------------------------
// macOS launchd integration
// ---------------------------------------------------------------------------

#[cfg(target_os = "macos")]
fn launch_agent_label(kind: ServiceKind) -> &'static str {
    match kind {
        ServiceKind::Mcp => "io.episteme.mcp",
        ServiceKind::Api => "io.episteme.api",
    }
}

#[cfg(target_os = "macos")]
fn launch_agent_plist_path(kind: ServiceKind) -> PathBuf {
    let home = paths::episteme_home()
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/tmp"));
    home.join("Library")
        .join("LaunchAgents")
        .join(format!("{}.plist", launch_agent_label(kind)))
}

/// Install a launchd plist for the given service kind (macOS only).
pub fn install_launchd_agent_for(
    kind: ServiceKind,
    host: &str,
    port: u16,
    token: Option<&str>,
) -> Result<String, String> {
    validate_host_port(host, port)?;
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (kind, host, port, token);
        Err("launchd integration is only supported on macOS".to_owned())
    }
    #[cfg(target_os = "macos")]
    {
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        let plist_path = launch_agent_plist_path(kind);
        if let Some(parent) = plist_path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        let label = launch_agent_label(kind);
        let (log_name, err_name) = match kind {
            ServiceKind::Mcp => ("launchd-mcp.log", "launchd-mcp.err"),
            ServiceKind::Api => ("launchd-api.log", "launchd-api.err"),
        };
        let log_dir = paths::log_dir();
        let stdout_log = log_dir.join(log_name);
        let stderr_log = log_dir.join(err_name);

        let exec_args = match kind {
            ServiceKind::Mcp => format!(
                r#"<string>{exe}</string>
    <string>mcp</string>
    <string>serve</string>
    <string>--host</string>
    <string>{host}</string>
    <string>--port</string>
    <string>{port}</string>"#,
                exe = exe.display(),
            ),
            ServiceKind::Api => format!(
                r#"<string>{exe}</string>
    <string>api</string>
    <string>serve</string>
    <string>--host</string>
    <string>{host}</string>
    <string>--port</string>
    <string>{port}</string>"#,
                exe = exe.display(),
            ),
        };

        let env_vars_section = match token {
            Some(t) => format!(
                r#"  <key>EnvironmentVariables</key>
  <dict>
    <key>EPISTEME_MCP_TOKEN</key>
    <string>{t}</string>
  </dict>
"#
            ),
            None => String::new(),
        };

        let plist = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key><string>{label}</string>
  <key>ProgramArguments</key>
  <array>
    {exec_args}
  </array>
  <key>RunAtLoad</key><true/>
  <key>KeepAlive</key><true/>
  {env_vars_section}<key>StandardOutPath</key><string>{stdout}</string>
  <key>StandardErrorPath</key><string>{stderr}</string>
</dict>
</plist>"#,
            label = label,
            stdout = stdout_log.display(),
            stderr = stderr_log.display(),
        );
        let uid = Command::new("id")
            .arg("-u")
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned())
            .unwrap_or_else(|| "501".to_owned());

        // Check if the agent is already loaded — if so, bootout first so we can
        // re-bootstrap with the updated plist.
        let already_loaded = Command::new("launchctl")
            .args(["list", label])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if already_loaded {
            let _ = Command::new("launchctl")
                .args(["bootout", &format!("gui/{uid}/{label}")])
                .status();
        }

        fs::write(&plist_path, plist).map_err(|e| e.to_string())?;

        let st = Command::new("launchctl")
            .args([
                "bootstrap",
                &format!("gui/{uid}"),
                plist_path.to_string_lossy().as_ref(),
            ])
            .status()
            .map_err(|e| e.to_string())?;
        if st.success() {
            if already_loaded {
                Ok(format!(
                    "{kind} launchd agent reloaded: {path}",
                    kind = kind_label(kind),
                    path = plist_path.display(),
                ))
            } else {
                Ok(format!(
                    "{kind} launchd agent installed: {path}",
                    kind = kind_label(kind),
                    path = plist_path.display(),
                ))
            }
        } else {
            Err("failed to bootstrap launchd agent".to_owned())
        }
    }
}

/// Uninstall the launchd plist for the given service kind (macOS only).
pub fn uninstall_launchd_agent_for(kind: ServiceKind) -> Result<String, String> {
    #[cfg(not(target_os = "macos"))]
    {
        let _ = kind;
        Err("launchd integration is only supported on macOS".to_owned())
    }
    #[cfg(target_os = "macos")]
    {
        let plist_path = launch_agent_plist_path(kind);
        let uid = Command::new("id")
            .arg("-u")
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned())
            .unwrap_or_default();
        let _ = Command::new("launchctl")
            .args([
                "bootout",
                &format!("gui/{uid}"),
                plist_path.to_string_lossy().as_ref(),
            ])
            .status();
        if plist_path.exists() {
            fs::remove_file(&plist_path).map_err(|e| e.to_string())?;
        }
        Ok(format!(
            "{kind} launchd agent removed: {path}",
            kind = kind_label(kind),
            path = plist_path.display(),
        ))
    }
}

/// Return the launchd status for the given service kind (macOS only).
pub fn launchd_status_for(kind: ServiceKind) -> Result<String, String> {
    #[cfg(not(target_os = "macos"))]
    {
        let _ = kind;
        Err("launchd integration is only supported on macOS".to_owned())
    }
    #[cfg(target_os = "macos")]
    {
        let label = launch_agent_label(kind);
        let output = Command::new("launchctl")
            .args(["list", label])
            .output()
            .map_err(|e| e.to_string())?;
        if output.status.success() {
            Ok(format!(
                "{kind} launchd active: {detail}",
                kind = kind_label(kind),
                detail = String::from_utf8_lossy(&output.stdout).trim(),
            ))
        } else {
            Ok(format!(
                "{kind} launchd agent not loaded",
                kind = kind_label(kind),
            ))
        }
    }
}

// ---------------------------------------------------------------------------
// macOS launchd backward-compat wrappers (default to Mcp)
// ---------------------------------------------------------------------------

/// Install the MCP launchd agent. Backward-compatible wrapper.
pub fn install_launchd_agent(host: &str, port: u16) -> Result<String, String> {
    install_launchd_agent_for(ServiceKind::Mcp, host, port, None)
}

/// Uninstall the MCP launchd agent. Backward-compatible wrapper.
pub fn uninstall_launchd_agent() -> Result<String, String> {
    uninstall_launchd_agent_for(ServiceKind::Mcp)
}

/// Return MCP launchd status. Backward-compatible wrapper.
pub fn launchd_status() -> Result<String, String> {
    launchd_status_for(ServiceKind::Mcp)
}

/// Enable the MCP launchd agent. Backward-compatible wrapper.
pub fn enable_launchd(now: bool) -> Result<String, String> {
    enable_service(ServiceKind::Mcp, now)
}

/// Disable the MCP launchd agent. Backward-compatible wrapper.
pub fn disable_launchd(now: bool) -> Result<String, String> {
    disable_service(ServiceKind::Mcp, now)
}

// ---------------------------------------------------------------------------
// Linux systemd integration
// ---------------------------------------------------------------------------

#[cfg(target_os = "linux")]
fn systemd_unit_path(kind: ServiceKind) -> Result<PathBuf, String> {
    let home = std::env::var("HOME").map_err(|_| {
        "HOME environment variable not set; cannot determine systemd user unit path".to_owned()
    })?;
    Ok(PathBuf::from(home)
        .join(".config")
        .join("systemd")
        .join("user")
        .join(format!("{}.service", systemd_unit_label(kind))))
}

#[cfg(target_os = "linux")]
fn systemd_unit_label(kind: ServiceKind) -> &'static str {
    match kind {
        ServiceKind::Mcp => "episteme-mcp",
        ServiceKind::Api => "episteme-api",
    }
}

/// Install a systemd user unit for the given service kind (Linux only).
#[cfg(target_os = "linux")]
pub fn install_systemd_unit(
    kind: ServiceKind,
    host: &str,
    port: u16,
    token: Option<&str>,
) -> Result<String, String> {
    validate_host_port(host, port)?;
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let unit_path = systemd_unit_path(kind)?;
    if let Some(parent) = unit_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let cmd = match kind {
        ServiceKind::Mcp => "mcp",
        ServiceKind::Api => "api",
    };

    let log_dir = paths::log_dir();
    let stdout_log = log_dir.join(format!("systemd-{}.log", cmd));
    let stderr_log = log_dir.join(format!("systemd-{}.err", cmd));

    // Ensure log directory exists.
    fs::create_dir_all(&log_dir).map_err(|e| e.to_string())?;

    let description = kind_label(kind);

    let env_line = match token {
        Some(t) => format!("Environment=\"EPISTEME_MCP_TOKEN={t}\"\n"),
        None => String::new(),
    };

    let unit = format!(
        "[Unit]\n\
Description=Episteme {description} Server\n\
After=network.target\n\
\n\
[Service]\n\
Type=simple\n\
ExecStart=\"{exe}\" {cmd} serve --host \"{host}\" --port {port}\n\
{env_line}Restart=on-failure\n\
RestartSec=5\n\
StandardOutput=append:{stdout}\n\
StandardError=append:{stderr}\n\
\n\
[Install]\n\
WantedBy=default.target\n",
        exe = exe.display(),
        cmd = cmd,
        host = host,
        port = port,
        env_line = env_line,
        stdout = stdout_log.display(),
        stderr = stderr_log.display(),
    );

    fs::write(&unit_path, unit).map_err(|e| e.to_string())?;

    // Reload systemd user daemon.
    let reload_st = Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .status()
        .map_err(|e| format!("failed to run systemctl daemon-reload: {e}"))?;
    if !reload_st.success() {
        return Err("systemctl --user daemon-reload failed".to_owned());
    }

    // Enable the unit so it starts automatically on login.
    let label = systemd_unit_label(kind);
    let enable_st = Command::new("systemctl")
        .args(["--user", "enable", &format!("{label}.service")])
        .status()
        .map_err(|e| format!("failed to enable systemd unit: {e}"))?;
    if !enable_st.success() {
        return Err(format!("failed to enable {label}.service"));
    }

    Ok(format!(
        "{kind} systemd unit installed: {path}",
        kind = kind_label(kind),
        path = unit_path.display(),
    ))
}

/// Uninstall the systemd user unit for the given service kind (Linux only).
#[cfg(target_os = "linux")]
pub fn uninstall_systemd_unit(kind: ServiceKind) -> Result<String, String> {
    let label = systemd_unit_label(kind);
    let _ = Command::new("systemctl")
        .args(["--user", "stop", &format!("{label}.service")])
        .status();
    let _ = Command::new("systemctl")
        .args(["--user", "disable", &format!("{label}.service")])
        .status();

    let unit_path = systemd_unit_path(kind)?;
    if unit_path.exists() {
        fs::remove_file(&unit_path).map_err(|e| e.to_string())?;
    }

    let _ = Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .status();

    Ok(format!(
        "{kind} systemd unit removed: {path}",
        kind = kind_label(kind),
        path = unit_path.display(),
    ))
}

// Stubs for non-Linux targets so callers can compile unconditionally.
#[cfg(not(target_os = "linux"))]
pub fn install_systemd_unit(
    kind: ServiceKind,
    host: &str,
    port: u16,
    token: Option<&str>,
) -> Result<String, String> {
    let _ = (kind, host, port, token);
    Err("systemd integration is only supported on Linux".to_owned())
}

#[cfg(not(target_os = "linux"))]
pub fn uninstall_systemd_unit(kind: ServiceKind) -> Result<String, String> {
    let _ = kind;
    Err("systemd integration is only supported on Linux".to_owned())
}

// ---------------------------------------------------------------------------
// Cross-platform enable / disable
// ---------------------------------------------------------------------------

/// Validate host and port for security and correctness.
fn validate_host_port(host: &str, port: u16) -> Result<(), String> {
    if host.contains(|c: char| !c.is_alphanumeric() && c != '.' && c != ':' && c != '-') {
        return Err(format!("invalid host: {}", host));
    }
    if port == 0 {
        return Err("port cannot be 0".to_owned());
    }
    Ok(())
}

/// Enable (install OS service unit and optionally start) the given service.
pub fn enable_service(kind: ServiceKind, now: bool) -> Result<String, String> {
    let (host, port) = get_host_port(kind);
    let token = get_token(kind);
    validate_host_port(&host, port)?;

    let mut msg = String::new();

    #[cfg(target_os = "macos")]
    {
        let install_msg = install_launchd_agent_for(kind, &host, port, token.as_deref())?;
        msg.push_str(&install_msg);
    }
    #[cfg(target_os = "linux")]
    {
        let install_msg = install_systemd_unit(kind, &host, port, token.as_deref())?;
        msg.push_str(&install_msg);
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        msg.push_str("OS service registration (persistence) is not supported on this platform.");
    }

    if now {
        let _ = cmd_stop(kind);
        match cmd_start(kind, &host, port) {
            Ok(pid) => {
                if msg.is_empty() {
                    msg.push_str(&format!(
                        "{label} started (PID {pid})",
                        label = kind_label(kind),
                    ));
                } else {
                    msg.push_str(&format!(
                        "\n{label} started in background (PID {pid})",
                        label = kind_label(kind),
                    ));
                }
            }
            Err(e) if e.contains("already in use") => {
                msg.push_str(&format!(
                    "\n{label} already running on port {port}",
                    label = kind_label(kind),
                ));
            }
            Err(e) => return Err(e),
        }
    }
    Ok(msg)
}

/// Disable (stop and uninstall OS service unit) the given service.
pub fn disable_service(kind: ServiceKind, now: bool) -> Result<String, String> {
    let mut msg = String::new();
    if now {
        let _ = cmd_stop(kind);
        msg.push_str(&format!("{} stopped.\n", kind_label(kind)));
    }

    #[cfg(target_os = "macos")]
    {
        let uninstall_msg = uninstall_launchd_agent_for(kind)?;
        msg.push_str(&uninstall_msg);
    }
    #[cfg(target_os = "linux")]
    {
        let uninstall_msg = uninstall_systemd_unit(kind)?;
        msg.push_str(&uninstall_msg);
    }

    Ok(msg)
}
