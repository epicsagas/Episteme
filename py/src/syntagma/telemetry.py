# Public API — most functions called by CLI and MCP entry points.
"""Syntagma telemetry — PostHog (product analytics) + Sentry (error monitoring).

Architecture: epiccounty/reports/telemetry_sentry_posthog_architecture_2026-04-29.md
  - PostHog: single project, ``product=syntagma`` property
  - Sentry:  SENTRY_DSN_SYNTAGMA secret
  - Consent: opt-out (on by default), on/off only
  - PII: strictly forbidden — enum values only, no code / file paths / query text

Consent flow:
  - New users:     ``syntagma install`` wizard sets consent explicitly
  - Existing users / MCP clients: first binary invocation auto-enables
    telemetry and prints a one-time opt-out notice
"""

from __future__ import annotations

import hashlib
import importlib.metadata
import os
import platform
import subprocess
import sys
import time
import uuid
from enum import Enum
from pathlib import Path
from typing import Optional

# ── Enum-gated event values (no free strings) ────────────────────────────────


class Tool(str, Enum):
    SearchKnowledge = "search_knowledge"
    GetEntity = "get_entity"
    GetNeighbors = "get_neighbors"
    FindPath = "find_path"
    AnalyzeCode = "analyze_code"
    SuggestRefactorings = "suggest_refactorings"


class Command(str, Enum):
    Install = "install"
    Build = "build"
    Analyze = "analyze"
    Infer = "infer"
    Explore = "explore"
    Api = "api"
    Mcp = "mcp"
    Service = "service"
    Telemetry = "telemetry"


class FailureClass(str, Enum):
    GraphLoadError = "graph_load_error"
    EmbeddingError = "embedding_error"
    DatabaseError = "database_error"
    PermissionDenied = "permission_denied"
    NetworkError = "network_error"
    Timeout = "timeout"
    Unknown = "unknown"

    def should_capture_sentry(self) -> bool:
        return self in (FailureClass.GraphLoadError, FailureClass.DatabaseError)


class ResultSizeBucket(str, Enum):
    Empty = "empty"
    Lt5 = "<5"
    Lt20 = "<20"
    Gte20 = ">=20"

    @classmethod
    def from_count(cls, n: int) -> "ResultSizeBucket":
        if n == 0:
            return cls.Empty
        if n < 5:
            return cls.Lt5
        if n < 20:
            return cls.Lt20
        return cls.Gte20


# ── Consent ───────────────────────────────────────────────────────────────────

_SYNTAGMA_HOME = Path(os.getenv("SYNTAGMA_HOME", Path.home() / ".syntagma"))
_CONSENT_FILE = _SYNTAGMA_HOME / "telemetry-consent"
_INSTALL_ID_FILE = _SYNTAGMA_HOME / "install-id"


def read_consent_raw() -> Optional[bool]:
    """Return True/False if set, None if unset."""
    try:
        val = _CONSENT_FILE.read_text(encoding="utf-8").strip()
        if val == "off":
            return False
        if val in ("on", "true", "1"):
            return True
    except OSError:
        pass
    return None


def read_consent() -> bool:
    val = read_consent_raw()
    return val if val is not None else False


def write_consent(enabled: bool) -> None:
    _SYNTAGMA_HOME.mkdir(parents=True, exist_ok=True)
    _CONSENT_FILE.write_text("on" if enabled else "off", encoding="utf-8")


def ensure_consent_or_set_default() -> None:
    """Auto-enable on first run; print a one-time opt-out notice."""
    if read_consent_raw() is None:
        write_consent(True)
        print("[syntagma] Telemetry enabled (anonymous install ID).", file=sys.stderr)
        print("[syntagma] To opt out: syntagma telemetry off", file=sys.stderr)
        print(
            "[syntagma] Details: https://github.com/epicsagas/Syntagma#telemetry",
            file=sys.stderr,
        )


def prompt_consent_interactive() -> bool:
    """Print consent box and prompt. Returns True = enabled."""
    print()
    print("  ┌─ Telemetry ──────────────────────────────────────────────────────────┐")
    print("  │ Syntagma collects anonymous usage data to improve detection quality. │")
    print("  │                                                                      │")
    print("  │  What we send:    tool name, duration, outcome, version, OS          │")
    print("  │  What we never:   code content, file paths, search queries           │")
    print("  │  Identifier:      random install ID (not linked to you or machine)   │")
    print("  │  Opt out anytime: syntagma telemetry off                             │")
    print("  └──────────────────────────────────────────────────────────────────────┘")
    print()
    try:
        ans = input("  Enable telemetry? [Y/n]: ").strip().lower()
    except (EOFError, KeyboardInterrupt):
        ans = ""
    enabled = ans not in ("n", "no")
    return enabled


# ── Install ID ───────────────────────────────────────────────────────────────


def _load_or_create_install_id() -> str:
    try:
        val = _INSTALL_ID_FILE.read_text(encoding="utf-8").strip()
        if val:
            return val
    except OSError:
        pass
    new_id = str(uuid.uuid4())
    try:
        _SYNTAGMA_HOME.mkdir(parents=True, exist_ok=True)
        _INSTALL_ID_FILE.write_text(new_id, encoding="utf-8")
    except OSError:
        pass
    return new_id


# ── Keys (injected at build time via hatchling hook) ─────────────────────────


def _posthog_key() -> Optional[str]:
    try:
        from syntagma._keys import POSTHOG_KEY  # type: ignore[import]

        return POSTHOG_KEY or None
    except ImportError:
        pass
    return None


def _sentry_dsn() -> Optional[str]:
    try:
        from syntagma._keys import SENTRY_DSN_SYNTAGMA  # type: ignore[import]

        return SENTRY_DSN_SYNTAGMA or None
    except ImportError:
        pass
    return None


def _version() -> str:
    try:
        return importlib.metadata.version("syntagma")
    except importlib.metadata.PackageNotFoundError:
        return "unknown"


def _os_name() -> str:
    s = platform.system().lower()
    if s == "darwin":
        return "macos"
    if s == "windows":
        return "windows"
    return "linux"


# ── PostHog transport (curl, fire-and-forget) ─────────────────────────────────

_POSTHOG_HOST = "app.posthog.com"
_POSTHOG_PORT = 443


def _posthog_send(payload: str) -> None:
    key = _posthog_key()
    if not key:
        return
    body = f'{{"api_key":"{key}","batch":[{payload}]}}'
    url = f"https://{_POSTHOG_HOST}/batch/"
    try:
        subprocess.Popen(
            [
                "curl", "-s", "--max-time", "5",
                "-X", "POST",
                "-H", "Content-Type: application/json",
                "-d", body,
                url,
            ],
            stdin=subprocess.DEVNULL,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
    except OSError:
        pass


def _track(event: str, props: dict) -> None:
    if not read_consent():
        return
    install_id = _load_or_create_install_id()
    ts = time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())
    base_props = {
        "product": "syntagma",
        "version": _version(),
        "os": _os_name(),
        **props,
    }
    props_json = ", ".join(
        f'"{k}": "{v}"' for k, v in base_props.items()
    )
    payload = (
        f'{{"event": "{event}", "distinct_id": "{install_id}", '
        f'"timestamp": "{ts}", "properties": {{{props_json}}}}}'
    )
    _posthog_send(payload)


# ── Sentry transport ──────────────────────────────────────────────────────────


def _parse_sentry_dsn(dsn: str) -> tuple[str, str, str]:
    """Return (host, project_path, public_key) or ('', '', '')."""
    try:
        without_scheme = dsn.removeprefix("https://")
        at = without_scheme.index("@")
        public_key = without_scheme[:at]
        rest = without_scheme[at + 1:]
        slash = rest.index("/")
        host = rest[:slash]
        project = rest[slash:]
        return host, f"/api{project}envelope/", public_key
    except (ValueError, IndexError):
        return "", "", ""


def _sentry_send(message: str, failure_class: str) -> None:
    dsn = _sentry_dsn()
    if not dsn:
        return
    host, path, public_key = _parse_sentry_dsn(dsn)
    if not host:
        return
    ver = _version()
    event_id = hashlib.md5(f"{message}{time.time()}".encode()).hexdigest()
    ts = time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())
    envelope = (
        "{}\n"
        '{"type":"event"}\n'
        f'{{"event_id":"{event_id}","timestamp":"{ts}",'
        f'"message":"{message}",'
        f'"level":"error",'
        f'"tags":{{"failure_class":"{failure_class}","product":"syntagma"}},'
        f'"release":"{ver}","platform":"python"}}\n'
    )
    auth = (
        f"Sentry sentry_version=7,"
        f"sentry_key={public_key},"
        f"sentry_client=syntagma/{ver}"
    )
    try:
        subprocess.Popen(
            [
                "curl", "-s", "--max-time", "5",
                "-X", "POST",
                "-H", f"X-Sentry-Auth: {auth}",
                "-H", "Content-Type: application/x-sentry-envelope",
                "-d", envelope,
                f"https://{host}{path}",
            ],
            stdin=subprocess.DEVNULL,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
    except OSError:
        pass


# ── Typed event helpers ───────────────────────────────────────────────────────


def track_session_started() -> None:
    _track("session_started", {})


def track_command_invoked(command: Command) -> None:
    _track("command_invoked", {"command": command.value})


def track_command_completed(command: Command, duration_ms: int) -> None:
    _track("command_completed", {
        "command": command.value,
        "duration_ms": str(duration_ms),
    })


def track_command_failed(command: Command, failure: FailureClass) -> None:
    _track("command_failed", {
        "command": command.value,
        "failure_class": failure.value,
    })
    if failure.should_capture_sentry():
        _sentry_send(f"command_failed:{command.value}", failure.value)


def track_tool_called(tool: Tool) -> None:
    _track("tool_called", {"tool": tool.value})


def track_tool_completed(tool: Tool, duration_ms: int, result_size: ResultSizeBucket) -> None:
    _track("tool_completed", {
        "tool": tool.value,
        "duration_ms": str(duration_ms),
        "result_size": result_size.value,
    })


def track_tool_failed(tool: Tool, failure: FailureClass) -> None:
    _track("tool_failed", {
        "tool": tool.value,
        "failure_class": failure.value,
    })
    if failure.should_capture_sentry():
        _sentry_send(f"tool_failed:{tool.value}", failure.value)


def track_install_completed(tool_count: int) -> None:
    _track("install_completed", {"tool_count": str(tool_count)})


# ── CLI (syntagma telemetry on|off|status) ────────────────────────────────────


def run_cli(action: str = "status") -> int:
    action = action.strip().lower()
    if action == "on":
        write_consent(True)
        print("[syntagma] Telemetry enabled.")
        print("[syntagma] To opt out: syntagma telemetry off")
        return 0
    if action == "off":
        write_consent(False)
        print("[syntagma] Telemetry disabled.")
        print("[syntagma] To re-enable: syntagma telemetry on")
        return 0
    # status (default)
    val = read_consent_raw()
    if val is None:
        state = "unset (will auto-enable on next command)"
    elif val:
        state = "enabled"
    else:
        state = "disabled"
    print(f"[syntagma] Telemetry: {state}")
    print(f"[syntagma] Consent file: {_CONSENT_FILE}")
    print(f"[syntagma] Install ID:   {_INSTALL_ID_FILE}")
    return 0
