//! Daemon management: start / stop / status helpers for MCP HTTP.
//!
//! Ported from Python `cli/service.py`.

use std::fs;
use std::net::TcpListener;
#[cfg(target_os = "macos")]
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crate::adapters::paths;

// ---------------------------------------------------------------------------
// PID file helpers
// ---------------------------------------------------------------------------

/// Read the PID from the standard PID file, if present and valid.
pub fn read_pid() -> Option<u32> {
    let path = paths::pid_file();
    fs::read_to_string(&path)
        .ok()
        .and_then(|s| s.trim().parse().ok())
}

/// Write `pid` to the PID file.
pub fn write_pid(pid: u32) -> std::io::Result<()> {
    // Ensure the parent directory exists.
    if let Some(parent) = paths::pid_file().parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(paths::pid_file(), pid.to_string())
}

/// Remove the PID file (idempotent).
pub fn clear_pid() -> std::io::Result<()> {
    let path = paths::pid_file();
    if path.exists() {
        fs::remove_file(path)
    } else {
        Ok(())
    }
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
        String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse()
            .ok()
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
// High-level service commands
// ---------------------------------------------------------------------------

/// Spawn the MCP HTTP server in the background.
///
/// Returns the child PID on success.
pub fn cmd_start(host: &str, port: u16) -> Result<u32, String> {
    if is_port_in_use(port) {
        if let Some(existing) = find_pid_by_port(port) {
            return Err(format!(
                "Port {port} is already in use (PID {existing})"
            ));
        }
        return Err(format!("Port {port} is already in use"));
    }

    let exe = std::env::current_exe().map_err(|e| format!("cannot determine current exe: {e}"))?;

    let log_dir = paths::log_dir();
    fs::create_dir_all(&log_dir).map_err(|e| format!("cannot create log dir: {e}"))?;

    let stdout_path = log_dir.join("mcp.log");
    let stderr_path = log_dir.join("mcp.err");

    let stdout = fs::File::create(&stdout_path)
        .map_err(|e| format!("cannot create {}: {e}", stdout_path.display()))?;
    let stderr = fs::File::create(&stderr_path)
        .map_err(|e| format!("cannot create {}: {e}", stderr_path.display()))?;

    let child = Command::new(&exe)
        .args(["mcp", "--http", "--host", host, "--port", &port.to_string()])
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr))
        .spawn()
        .map_err(|e| format!("failed to spawn server: {e}"))?;

    let pid = child.id();
    write_pid(pid).map_err(|e| format!("failed to write PID file: {e}"))?;

    if wait_port_open(port, 10) {
        Ok(pid)
    } else {
        // Best-effort cleanup.
        let _ = clear_pid();
        Err("Server failed to start within 10 seconds".to_owned())
    }
}

/// Stop the running MCP HTTP server (SIGTERM, then SIGKILL on timeout).
pub fn cmd_stop() -> Result<(), String> {
    let pid = read_pid().ok_or("No PID file found -- is the server running?")?;

    if !is_running(pid) {
        clear_pid().ok();
        return Ok(());
    }

    let port = get_mcp_port();

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
        let _ = Command::new("kill")
            .arg(pid.to_string())
            .output();
    }

    if wait_port_free(port, 5) {
        clear_pid().ok();
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
    #[cfg(not(unix))]
    {
        let _ = Command::new("kill")
            .args(["-9", &pid.to_string()])
            .output();
    }

    // Give it a moment, then clean up PID file regardless.
    std::thread::sleep(Duration::from_millis(500));
    clear_pid().ok();
    Ok(())
}

/// Return the MCP port from config (or the default).
pub fn get_mcp_port() -> u16 {
    crate::adapters::config::SyntagmaConfig::load()
        .map(|c| c.mcp_port)
        .unwrap_or(43175)
}

/// Print a human-readable status line and return `true` if the server is up.
pub fn cmd_status() -> bool {
    let port = get_mcp_port();

    match read_pid() {
        Some(pid) => {
            if is_running(pid) {
                let port_status = if is_port_in_use(port) {
                    format!("listening on port {port}")
                } else {
                    format!("not yet listening on port {port}")
                };
                println!("syntagma server is running (PID {pid}, {port_status})");
                true
            } else {
                println!("syntagma server is NOT running (stale PID {pid})");
                clear_pid().ok();
                false
            }
        }
        None => {
            // No PID file -- but maybe the server is running from another source.
            if is_port_in_use(port) && let Some(pid) = find_pid_by_port(port) {
                println!(
                    "syntagma server appears to be running (PID {pid} on port {port}), but no PID file"
                );
                return true;
            }
            println!("syntagma server is stopped");
            false
        }
    }
}

/// Resolve the configured API host.
pub fn get_mcp_host() -> String {
    crate::adapters::config::SyntagmaConfig::load()
        .map(|c| c.mcp_host)
        .unwrap_or_else(|_| "127.0.0.1".to_owned())
}

// ---------------------------------------------------------------------------
// macOS launchd integration
// ---------------------------------------------------------------------------

#[cfg(target_os = "macos")]
fn launch_agent_label() -> &'static str {
    "io.syntagma.api"
}

#[cfg(target_os = "macos")]
fn launch_agent_plist_path() -> PathBuf {
    let home = crate::adapters::paths::syntagma_home()
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/tmp"));
    home.join("Library")
        .join("LaunchAgents")
        .join(format!("{}.plist", launch_agent_label()))
}

pub fn install_launchd_agent(host: &str, port: u16) -> Result<String, String> {
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (host, port);
        Err("launchd integration is only supported on macOS".to_owned())
    }
    #[cfg(target_os = "macos")]
    {
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        let plist_path = launch_agent_plist_path();
        if let Some(parent) = plist_path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let plist = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key><string>{label}</string>
  <key>ProgramArguments</key>
  <array>
    <string>{exe}</string>
    <string>mcp</string>
    <string>--http</string>
    <string>--host</string>
    <string>{host}</string>
    <string>--port</string>
    <string>{port}</string>
  </array>
  <key>RunAtLoad</key><true/>
  <key>KeepAlive</key><true/>
  <key>StandardOutPath</key><string>{stdout}</string>
  <key>StandardErrorPath</key><string>{stderr}</string>
</dict>
</plist>"#,
            label = launch_agent_label(),
            exe = exe.display(),
            host = host,
            port = port,
            stdout = crate::adapters::paths::log_dir().join("launchd.log").display(),
            stderr = crate::adapters::paths::log_dir().join("launchd.err").display(),
        );
        fs::write(&plist_path, plist).map_err(|e| e.to_string())?;
        let uid = std::process::Command::new("id")
            .arg("-u")
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned())
            .unwrap_or_else(|| "501".to_owned());
        let st = Command::new("launchctl")
            .args(["bootstrap", &format!("gui/{uid}"), plist_path.to_string_lossy().as_ref()])
            .status()
            .map_err(|e| e.to_string())?;
        if st.success() {
            Ok(format!("launchd agent installed: {}", plist_path.display()))
        } else {
            Err("failed to bootstrap launchd agent".to_owned())
        }
    }
}

pub fn uninstall_launchd_agent() -> Result<String, String> {
    #[cfg(not(target_os = "macos"))]
    {
        Err("launchd integration is only supported on macOS".to_owned())
    }
    #[cfg(target_os = "macos")]
    {
        let plist_path = launch_agent_plist_path();
        let uid = std::process::Command::new("id")
            .arg("-u")
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned())
            .unwrap_or_default();
        let _ = Command::new("launchctl")
            .args(["bootout", &format!("gui/{uid}"), plist_path.to_string_lossy().as_ref()])
            .status();
        if plist_path.exists() {
            fs::remove_file(&plist_path).map_err(|e| e.to_string())?;
        }
        Ok(format!("launchd agent removed: {}", plist_path.display()))
    }
}

pub fn launchd_status() -> Result<String, String> {
    #[cfg(not(target_os = "macos"))]
    {
        Err("launchd integration is only supported on macOS".to_owned())
    }
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("launchctl")
            .args(["list", launch_agent_label()])
            .output()
            .map_err(|e| e.to_string())?;
        if output.status.success() {
            Ok(format!(
                "launchd active: {}",
                String::from_utf8_lossy(&output.stdout).trim()
            ))
        } else {
            Ok("launchd agent not loaded".to_owned())
        }
    }
}

pub fn enable_launchd(now: bool) -> Result<String, String> {
    let host = get_mcp_host();
    let port = get_mcp_port();
    let mut msg = install_launchd_agent(&host, port)?;
    if now {
        let _ = cmd_stop();
        match cmd_start(&host, port) {
            Ok(pid) => msg.push_str(&format!("\nServer started (PID {pid})")),
            Err(e) if e.contains("already in use") => {
                msg.push_str(&format!("\nServer already running on port {port}"));
            }
            Err(e) => return Err(e),
        }
    }
    Ok(msg)
}

pub fn disable_launchd(now: bool) -> Result<String, String> {
    let mut msg = String::new();
    if now {
        let _ = cmd_stop();
        msg.push_str("Server stopped.\n");
    }
    let uninstall = uninstall_launchd_agent()?;
    msg.push_str(&uninstall);
    Ok(msg)
}
