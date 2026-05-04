"""
syntagma service — manage the MCP HTTP proxy as a background daemon.

Single-process guarantee:
  - If launchd plist is registered, start/stop/restart delegate to
    `launchctl start/stop` so launchd owns the lifecycle.
  - If launchd is not registered, a PID file is used directly.
  - In both paths, a port check is the final guard: if something is
    already listening on the port, start is a no-op.

Commands:
  serve    Run in the foreground (Ctrl+C to stop)
  start    Launch in the background
  stop     Stop the running daemon
  restart  stop + start
  status   Show running/stopped + PID
  enable   Register as a login item via launchd (macOS LaunchAgent)
  disable  Remove the launchd LaunchAgent
"""

from __future__ import annotations

import os
import platform
import signal
import subprocess
import sys
import time
from pathlib import Path
from typing import Callable, Optional

from syntagma import config as _config

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

_LABEL = "com.epicsagas.syntagma"
_LOG_DIR = _config.LOG_DIR
_PID_FILE = _config.PID_FILE
_PLIST_PATH = Path.home() / "Library" / "LaunchAgents" / f"{_LABEL}.plist"

_PLIST_TEMPLATE = """\
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{label}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{executable}</string>
        <string>--http</string>
        <string>--host</string>
        <string>{host}</string>
        <string>--port</string>
        <string>{port}</string>
    </array>
    <key>RunAtLoad</key>
    <false/>
    <key>KeepAlive</key>
    <false/>
    <key>ProcessType</key>
    <string>Background</string>
    <key>EnvironmentVariables</key>
    <dict>
        <key>PATH</key>
        <string>{env_path}</string>
        <key>DYLD_LIBRARY_PATH</key>
        <string>{dyld_lib_path}</string>
    </dict>
    <key>StandardOutPath</key>
    <string>{log_out}</string>
    <key>StandardErrorPath</key>
    <string>{log_err}</string>
</dict>
</plist>
"""


# ---------------------------------------------------------------------------
# Low-level helpers
# ---------------------------------------------------------------------------


def _mcp_executable() -> str:
    import shutil

    exe = shutil.which("syntagma-mcp")
    if exe:
        return exe
    venv_bin = Path(sys.executable).parent / "syntagma-mcp"
    if venv_bin.exists():
        return str(venv_bin)
    return "syntagma-mcp"


def _mcp_host_port() -> tuple[str, int]:
    from syntagma.config import MCP_SERVER_HOST, MCP_SERVER_PORT

    return MCP_SERVER_HOST, MCP_SERVER_PORT


def _ensure_log_dir() -> None:
    _LOG_DIR.mkdir(parents=True, exist_ok=True)


def _read_pid() -> Optional[int]:
    try:
        return int(_PID_FILE.read_text().strip())
    except (OSError, ValueError):
        return None


def _write_pid(pid: int) -> None:
    _PID_FILE.parent.mkdir(parents=True, exist_ok=True)
    _PID_FILE.write_text(str(pid))


def _clear_pid() -> None:
    _PID_FILE.unlink(missing_ok=True)


def _is_running(pid: int) -> bool:
    try:
        os.kill(pid, 0)
        return True
    except (ProcessLookupError, PermissionError):
        return False


def _find_pid_by_port(port: int) -> Optional[int]:
    """Return the PID listening on *port* via lsof, or None."""
    try:
        out = subprocess.check_output(
            ["lsof", "-ti", f":{port}"],
            stderr=subprocess.DEVNULL,
            text=True,
        ).strip()
        if out:
            return int(out.splitlines()[0])
    except Exception:
        pass
    return None


def _wait_port_free(port: int, timeout: float = 5.0) -> bool:
    """Block until the port is no longer in use. Returns True if free."""
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if not _find_pid_by_port(port):
            return True
        time.sleep(0.25)
    return False


def _wait_port_open(port: int, timeout: float = 5.0) -> bool:
    """Block until something is listening on the port. Returns True if open."""
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if _find_pid_by_port(port):
            return True
        time.sleep(0.25)
    return False


# ---------------------------------------------------------------------------
# launchd helpers (macOS only)
# ---------------------------------------------------------------------------


def _launchd_registered() -> bool:
    """True if our plist is loaded in launchd."""
    try:
        result = subprocess.run(
            ["launchctl", "list", _LABEL],
            capture_output=True,
            text=True,
        )
        return result.returncode == 0
    except Exception:
        return False


def _launchd_start() -> bool:
    result = subprocess.run(
        ["launchctl", "start", _LABEL],
        capture_output=True,
        text=True,
    )
    return result.returncode == 0


def _launchd_stop() -> bool:
    result = subprocess.run(
        ["launchctl", "stop", _LABEL],
        capture_output=True,
        text=True,
    )
    return result.returncode == 0


# ---------------------------------------------------------------------------
# Commands
# ---------------------------------------------------------------------------


def cmd_start() -> int:
    host, port = _mcp_host_port()

    # Single-process guard: port already in use → already running
    existing = _find_pid_by_port(port)
    if existing:
        print(f"Already running (PID {existing})")
        _write_pid(existing)
        return 0

    _ensure_log_dir()
    log_out = _LOG_DIR / "mcp.out.log"
    log_err = _LOG_DIR / "mcp.err.log"

    # Delegate to launchd if registered — it owns the lifecycle
    if _launchd_registered():
        if not _launchd_start():
            print("launchctl start failed — check logs", file=sys.stderr)
            return 1
        if not _wait_port_open(port):
            print(f"Timed out waiting for server — check logs: {log_err}", file=sys.stderr)
            return 1
        pid = _find_pid_by_port(port)
        if pid:
            _write_pid(pid)
        print(f"Started (PID {pid}) — http://{host}:{port}")
        print(f"Logs: {log_out}")
        return 0

    # Direct launch (launchd not registered)
    exe = _mcp_executable()
    lib_path = _resolve_dyld_lib_path(exe)
    env = os.environ.copy()
    if lib_path:
        env["DYLD_LIBRARY_PATH"] = lib_path
    f_out = open(log_out, "a")
    f_err = open(log_err, "a")
    proc = subprocess.Popen(
        [exe, "--http", "--host", host, "--port", str(port)],
        stdout=f_out,
        stderr=f_err,
        start_new_session=True,
        env=env,
    )
    f_out.close()
    f_err.close()

    if not _wait_port_open(port):
        _clear_pid()
        print(f"Failed to start — check logs: {log_err}", file=sys.stderr)
        return 1

    # uv shims may fork: resolve actual listening PID
    pid = _find_pid_by_port(port) or proc.pid
    _write_pid(pid)
    print(f"Started (PID {pid}) — http://{host}:{port}")
    print(f"Logs: {log_out}")
    return 0


def cmd_stop() -> int:
    _, port = _mcp_host_port()
    pid = _find_pid_by_port(port)

    if not pid:
        print("Not running")
        _clear_pid()
        return 0

    # Delegate to launchd if registered
    if _launchd_registered():
        if not _launchd_stop():
            print(
                "Warning: launchctl stop failed — falling back to SIGTERM/SIGKILL", file=sys.stderr
            )
    else:
        os.kill(pid, signal.SIGTERM)

    # Wait for process to die
    for _ in range(20):
        time.sleep(0.25)
        if not _is_running(pid):
            break
    else:
        os.kill(pid, signal.SIGKILL)

    _wait_port_free(port)
    _clear_pid()
    print(f"Stopped (PID {pid})")
    return 0


def cmd_restart() -> int:
    cmd_stop()
    return cmd_start()


def cmd_status() -> int:
    host, port = _mcp_host_port()
    pid = _find_pid_by_port(port)

    if pid:
        _write_pid(pid)
        managed = " [launchd]" if _launchd_registered() else ""
        print(f"Running{managed}  — PID {pid}  http://{host}:{port}")
        print(f"Logs:    {_LOG_DIR}/mcp.out.log")
    else:
        _clear_pid()
        registered = " (login item registered)" if _launchd_registered() else ""
        print(f"Stopped{registered}")
    return 0


def _resolve_dyld_lib_path(exe: str) -> str:
    """Find the lib directory containing libpython for the given executable."""
    exe_path = Path(exe).resolve()
    # shebang points to venv python → venv/bin/python → venv/lib/
    if exe_path.is_file():
        shebang = exe_path.read_text(encoding="utf-8", errors="ignore").split("\n", 1)[0]
        if shebang.startswith("#!"):
            python_path = Path(shebang[2:].strip()).resolve()
            lib_dir = python_path.parent.parent / "lib"
            if lib_dir.exists():
                return str(lib_dir)
    # Fallback: executable's own parent/../lib
    lib_dir = exe_path.parent.parent / "lib"
    return str(lib_dir) if lib_dir.exists() else ""


def cmd_enable(now: bool = False) -> int:
    """Register as a login item (launchd). With --now, also start immediately."""
    if platform.system() != "Darwin":
        print("enable/disable is only supported on macOS (launchd)", file=sys.stderr)
        return 1

    host, port = _mcp_host_port()
    _ensure_log_dir()
    exe = _mcp_executable()

    plist = _PLIST_TEMPLATE.format(
        label=_LABEL,
        executable=exe,
        host=host,
        port=port,
        env_path=os.environ.get("PATH", "/usr/local/bin:/usr/bin:/bin"),
        dyld_lib_path=_resolve_dyld_lib_path(exe),
        log_out=_LOG_DIR / "mcp.out.log",
        log_err=_LOG_DIR / "mcp.err.log",
    )

    _PLIST_PATH.parent.mkdir(parents=True, exist_ok=True)
    _PLIST_PATH.write_text(plist)

    # Unload first if already registered (idempotent re-enable)
    if _launchd_registered():
        subprocess.run(["launchctl", "unload", str(_PLIST_PATH)], capture_output=True)

    result = subprocess.run(
        ["launchctl", "load", str(_PLIST_PATH)],
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        print(f"launchctl load failed: {result.stderr.strip()}", file=sys.stderr)
        return 1

    print(f"Enabled as login item — {_LABEL}")
    print(f"Plist: {_PLIST_PATH}")

    if now:
        print("Starting now...")
        return cmd_start()

    print("Tip: run `syntagma service start` to start now.")
    return 0


def cmd_disable(now: bool = False) -> int:
    """Remove login item. With --now, also stop the running process."""
    if platform.system() != "Darwin":
        print("enable/disable is only supported on macOS (launchd)", file=sys.stderr)
        return 1

    if now:
        cmd_stop()

    if not _PLIST_PATH.exists():
        print("Not enabled (no plist found)")
        return 0

    result = subprocess.run(
        ["launchctl", "unload", str(_PLIST_PATH)],
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        print(f"launchctl unload failed: {result.stderr.strip()}", file=sys.stderr)
        return 1

    _PLIST_PATH.unlink(missing_ok=True)
    print(f"Disabled and removed — {_LABEL}")
    return 0


def cmd_serve() -> int:
    """Run in the foreground (Ctrl+C to stop)."""
    host, port = _mcp_host_port()

    # Single-process guard
    existing = _find_pid_by_port(port)
    if existing:
        print(
            f"Error: port {port} already in use (PID {existing}). "
            "Stop the background daemon first.",
            file=sys.stderr,
        )
        return 1

    _ensure_log_dir()
    from syntagma.mcp.server import serve_http

    print(f"Serving MCP HTTP on http://{host}:{port}  (Ctrl+C to stop)")
    serve_http(host, port)
    return 0


# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------

_COMMANDS: dict[str, Callable[..., int]] = {
    "serve": cmd_serve,
    "start": cmd_start,
    "stop": cmd_stop,
    "restart": cmd_restart,
    "status": cmd_status,
    "enable": cmd_enable,
    "disable": cmd_disable,
}


def build_parser():
    import argparse

    parser = argparse.ArgumentParser(
        prog="syntagma service",
        description=(
            "Manage the Syntagma MCP HTTP proxy as a background daemon.\n\n"
            "  serve    Run in the foreground (Ctrl+C to stop)\n"
            "  start    Launch in the background\n"
            "  stop     Stop the running daemon\n"
            "  restart  Stop then start\n"
            "  status   Show running state and PID\n"
            "  enable   Register as a login item (macOS LaunchAgent)\n"
            "  disable  Remove the login item"
        ),
        epilog=(
            "Examples:\n"
            "  syntagma service serve           # foreground\n"
            "  syntagma service start           # background\n"
            "  syntagma service status\n"
            "  syntagma service enable          # register login item only\n"
            "  syntagma service enable --now    # register + start immediately\n"
            "  syntagma service disable         # unregister only\n"
            "  syntagma service disable --now   # stop + unregister"
        ),
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument(
        "command",
        choices=list(_COMMANDS),
        metavar="COMMAND",
        help=f"One of: {', '.join(_COMMANDS)}",
    )
    parser.add_argument(
        "--now",
        action="store_true",
        default=False,
        help="For enable: also start now. For disable: also stop now.",
    )
    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    cmd = args.command
    if cmd in ("enable", "disable"):
        return _COMMANDS[cmd](now=args.now)
    return _COMMANDS[cmd]()
