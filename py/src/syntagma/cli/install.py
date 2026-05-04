"""
syntagma install — Interactive onboarding command for AI tool integration.

Supported tools: claude-code, cursor, codex, gemini, opencode
"""

from __future__ import annotations

import argparse
import json
import os
import shutil
import sys
import tarfile
import tempfile
import urllib.request
from pathlib import Path
from typing import Any

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

SUPPORTED_TOOLS = ["claude", "cursor", "codex", "gemini", "cline", "opencode", "all"]
MANIFEST_FILENAME = ".syntagma-agents"

_TOOL_ALIASES: dict[str, str] = {
    "claude": "claude",
    "claude-code": "claude",
    "claudecode": "claude",
    "cline": "cline",
    "claude-dev": "cline",
}

_AGENT_FILES = [
    "syntagma-advisor.md",
    "syntagma-researcher.md",
    "code-reviewer.md",
    "architecture-analyst.md",
]

_AGENT_DISPLAY = {
    "syntagma-advisor.md": "syntagma-advisor    — Engineering decisions advisor",
    "syntagma-researcher.md": "syntagma-researcher — Knowledge graph researcher",
    "code-reviewer.md": "code-reviewer       — Code quality reviewer",
    "architecture-analyst.md": "architecture-analyst — Architecture evaluator",
}

_TOOL_DISPLAY = {
    "claude": "Claude Code",
    "cursor": "Cursor",
    "codex": "Codex (OpenAI)",
    "gemini": "Gemini CLI",
    "cline": "Cline (Claude Dev)",
    "opencode": "OpenCode",
    "all": "All",
}

_MCP_ENTRY = {
    "command": "syntagma-mcp",
    "args": [],
}

_SYNTAGMA_AGENTS_MD_SECTION = """\

<!-- SYNTAGMA-BEGIN -->
# AGENTS.md — Syntagma

이 프로젝트는 Syntagma MCP 서버를 통해 AI 에이전트와 통합됩니다.

## MCP 서버 연결

MCP 서버를 시작하려면 다음을 사용하세요:

```bash
syntagma-mcp
```

설정 파일에 아래와 같이 등록합니다:

```json
{
  "mcpServers": {
    "syntagma": {
      "command": "syntagma-mcp",
      "args": []
    }
  }
}
```

## 사용 가능한 에이전트

| 에이전트 | 역할 |
|---------|------|
| syntagma-advisor | 엔지니어링 결정에 대한 실행 가능한 조언 제공 |
| syntagma-researcher | 소프트웨어 엔지니어링 지식 그래프 탐색 및 연구 |
| code-reviewer | 코드 스멜 감지 및 리팩토링 제안 기반 리뷰 |
| architecture-analyst | 시스템 아키텍처 평가 및 구조적 위험 분석 |

## 사용 가능한 MCP 도구

| 도구 | 설명 |
|------|------|
| `search_knowledge` | 지식 그래프에서 패턴, 법칙, 리팩토링, 스멜 검색 |
| `get_entity` | 특정 엔티티의 상세 정보 조회 (ID 기준) |
| `get_neighbors` | 엔티티의 관련 연결 노드 탐색 |
| `find_path` | 두 엔티티 사이의 연결 경로 탐색 |
| `analyze_code` | 코드 스멜 자동 감지 분석 |
| `suggest_refactorings` | 감지된 스멜에 대한 리팩토링 제안 |
<!-- SYNTAGMA-END -->
"""


# ---------------------------------------------------------------------------
# Argument parser
# ---------------------------------------------------------------------------


def build_parser() -> argparse.ArgumentParser:
    tool_list = ", ".join(SUPPORTED_TOOLS)
    parser = argparse.ArgumentParser(
        prog="syntagma install",
        description="Install Syntagma agents and MCP server into your AI tool",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=(
            "Examples:\n"
            "  syntagma install                   # interactive mode\n"
            "  syntagma install all               # install for all tools\n"
            "  syntagma install claude            # install for Claude Code only\n"
            "  syntagma install cursor codex      # install for multiple tools\n"
            "  syntagma install claude --dry-run  # preview without writing files\n"
            f"\nTools: {tool_list}\n"
        ),
    )
    parser.add_argument(
        "tools",
        nargs="*",
        metavar="TOOL",
        help=f"AI tools to install for ({tool_list})",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        default=False,
        help="Show what would be installed without writing files",
    )
    parser.add_argument(
        "--offline",
        metavar="PATH",
        default=None,
        help="Use a local syntagma-data-*.tar.gz instead of downloading from GitHub Releases",
    )
    parser.add_argument(
        "--local",
        action="store_true",
        default=False,
        help="Seed from source tree (meta/ + raw/) without downloading — for dev/build use",
    )
    return parser


# ---------------------------------------------------------------------------
# Interactive checkbox UI (stdlib only, POSIX raw-mode)
# ---------------------------------------------------------------------------

# ---------------------------------------------------------------------------
# ANSI helpers
# ---------------------------------------------------------------------------

_RESET = "\033[0m"
_BOLD = "\033[1m"
_DIM = "\033[2m"
_CYAN = "\033[36m"
_GREEN = "\033[32m"
_YELLOW = "\033[33m"

_KEY_UP = "\x1b[A"
_KEY_DOWN = "\x1b[B"
_KEY_ESC = "\x1b"
_KEY_SPACE = " "
_KEY_ENTER = "\r"
_KEY_ENTER_LF = "\n"
_KEY_CTRL_C = "\x03"

# Sentinel returned by _checkbox_select when ESC pressed
_BACK = object()


def _cls_line() -> None:
    sys.stdout.write("\r\033[2K")


def _move_up(n: int) -> None:
    if n > 0:
        sys.stdout.write(f"\033[{n}A")


def _read_key(fd: int) -> str:
    ch = os.read(fd, 1).decode("utf-8", errors="replace")
    if ch == "\x1b":
        # Try to read escape sequence (non-blocking peek)
        try:
            import select

            r, _, _ = select.select([fd], [], [], 0.05)
            if r:
                rest = os.read(fd, 3).decode("utf-8", errors="replace")
                return ch + rest
        except Exception:
            pass
        return _KEY_ESC  # bare ESC (no following bytes)
    return ch


def _header(step: int, total: int, title: str) -> None:
    bar = ("━" * step) + ("─" * (total - step))
    pct = f"{step}/{total}"
    sys.stdout.write(
        f"\n{_BOLD}{_CYAN}  Syntagma Setup{_RESET}  "
        f"{_DIM}{bar}{_RESET}  {_DIM}{pct}{_RESET}\n"
        f"  {_BOLD}{title}{_RESET}\n\n"
    )
    sys.stdout.flush()


def _hint(*parts: str) -> None:
    sys.stdout.write(f"  {_DIM}" + "  ·  ".join(parts) + f"{_RESET}\n\n")
    sys.stdout.flush()


def _checkbox_select(
    title: str,
    options: list[str],
    defaults: list[bool],
    allow_toggle_all: bool = False,
) -> list[bool] | object:
    """
    Interactive checkbox list.

    Returns list[bool] on Enter, or _BACK sentinel on ESC.
    Falls back to defaults when no TTY is available.
    """
    if not sys.stdin.isatty():
        return list(defaults)

    selected = list(defaults)
    cursor = 0
    fd = sys.stdin.fileno()
    n = len(options)

    # hint line count: 1 hint line + 1 blank
    HINT_LINES = 2
    # total lines rendered: title(already printed above) + options + hint
    BODY_LINES = n + HINT_LINES

    def _render_body(first: bool = False) -> None:
        if not first:
            _move_up(BODY_LINES)
        for i, opt in enumerate(options):
            _cls_line()
            checked = f"{_GREEN}●{_RESET}" if selected[i] else f"{_DIM}○{_RESET}"
            if i == cursor:
                row = f"  {_CYAN}>{_RESET} {checked}  {_BOLD}{_CYAN}{opt}{_RESET}"
            else:
                row = f"    {checked}  {_DIM}{opt}{_RESET}"
            sys.stdout.write(row + "\n")

        # hint bar
        _cls_line()
        hints = ["↑↓jk move", "Space select"]
        if allow_toggle_all:
            hints.append("a toggle all")
        hints += ["n clear", "Enter confirm", "Esc/Q back"]
        sys.stdout.write(f"\n  {_DIM}" + "  ·  ".join(hints) + f"{_RESET}\n")
        sys.stdout.flush()

    _render_body(first=True)

    try:
        import termios
        import tty as _tty
    except ImportError as exc:
        raise RuntimeError(
            "Interactive checkbox is not supported on this platform (termios/tty unavailable)."
        ) from exc

    old_settings = termios.tcgetattr(fd)
    result: list[bool] | object = list(selected)
    try:
        _tty.setraw(fd)
        while True:
            key = _read_key(fd)
            if key in (_KEY_ENTER, _KEY_ENTER_LF):
                result = list(selected)
                break
            elif key == _KEY_ESC:
                result = _BACK
                break
            elif key == _KEY_UP:
                cursor = (cursor - 1) % n
                _render_body()
            elif key == _KEY_DOWN:
                cursor = (cursor + 1) % n
                _render_body()
            elif key in ("k", "K"):
                cursor = (cursor - 1) % n
                _render_body()
            elif key in ("j", "J"):
                cursor = (cursor + 1) % n
                _render_body()
            elif key == _KEY_SPACE:
                selected[cursor] = not selected[cursor]
                _render_body()
            elif allow_toggle_all and key in ("a", "A"):
                all_on = all(selected)
                selected = [not all_on] * n
                _render_body()
            elif key in ("n", "N"):
                selected = [False] * n
                _render_body()
            elif key in ("q", "Q"):
                result = _BACK
                break
            elif key == _KEY_CTRL_C:
                sys.stdout.write("\n")
                sys.stdout.flush()
                sys.exit(130)
    finally:
        termios.tcsetattr(fd, termios.TCSADRAIN, old_settings)

    sys.stdout.write("\n")
    sys.stdout.flush()
    return result


def _prompt_confirm(message: str = "Install? [Y/n] ") -> bool:
    """Simple Y/n confirmation. Returns True for Y/Enter, False for n."""
    if not sys.stdin.isatty():
        return True
    try:
        sys.stdout.write(f"  {message}")
        sys.stdout.flush()
        answer = input("").strip().lower()
        return answer in ("", "y", "yes")
    except (EOFError, KeyboardInterrupt):
        return False


# ---------------------------------------------------------------------------
# SyntagmaInstaller
# ---------------------------------------------------------------------------


class SyntagmaInstaller:
    """Core installer logic — separated from UI for testability."""

    def __init__(self, registry_dir: Path, skills_dir: Path, dry_run: bool = False):
        self.registry_dir = registry_dir
        self.skills_dir = skills_dir
        self.dry_run = dry_run
        self._results: list[str] = []

    # ------------------------------------------------------------------
    # Low-level helpers
    # ------------------------------------------------------------------

    def _log(self, msg: str) -> None:
        prefix = "[dry-run] " if self.dry_run else ""
        print(f"  {prefix}{msg}")
        self._results.append(msg)

    def _ensure_dir(self, path: Path) -> None:
        if not self.dry_run:
            path.mkdir(parents=True, exist_ok=True)

    def _write_text(self, path: Path, content: str) -> None:
        if not self.dry_run:
            self._ensure_dir(path.parent)
            path.write_text(content, encoding="utf-8")

    def _merge_mcp_json(self, target: Path, server_name: str, server_config: dict[str, Any]) -> str:
        """
        Merge a server entry into an mcp.json file.
        Returns: "created" | "merged" | "already_installed"
        """
        existing: dict[str, Any] = {}
        if target.exists():
            try:
                existing = json.loads(target.read_text(encoding="utf-8"))
            except (json.JSONDecodeError, OSError):
                existing = {}

        servers: dict[str, Any] = existing.get("mcpServers", {})
        if server_name in servers and servers[server_name] == server_config:
            return "already_installed"

        was_new = not target.exists()
        servers[server_name] = server_config
        existing["mcpServers"] = servers

        if not self.dry_run:
            self._ensure_dir(target.parent)
            target.write_text(
                json.dumps(existing, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
            )

        return "created" if was_new else "merged"

    # MCP tool names used in Claude-format agent frontmatter
    _MCP_TOOLS = {
        "search_knowledge",
        "get_entity",
        "get_neighbors",
        "find_path",
        "analyze_code",
        "suggest_refactorings",
    }

    # Gemini-compatible native tools (list-style tools:)
    _GEMINI_NATIVE_TOOLS = ["read_file", "grep_search", "glob", "run_shell_command"]

    # OpenCode capability map (dict-style tools:)
    _OPENCODE_TOOLS = {"read": True, "bash": True}

    _MCP_NOTE = {
        "gemini": (
            "\n> **Gemini CLI**: This agent uses the Syntagma MCP server for knowledge graph"
            " access.\n"
            "> MCP tools (`search_knowledge`, `get_entity`, etc.) are available via the"
            " configured\n"
            "> MCP server. Use `run_shell_command` to invoke `syntagma-mcp` if direct CLI"
            " access\n"
            "> is needed.\n"
        ),
        "opencode": (
            "\n> **OpenCode**: This agent uses the Syntagma MCP server for knowledge graph"
            " access.\n"
            "> MCP tools (`search_knowledge`, `get_entity`, etc.) are available via the"
            " configured\n"
            "> MCP server registered in `~/.config/opencode/opencode.json`.\n"
        ),
    }

    def _transform_agent(self, content: str, fmt: str) -> str:
        """
        Convert a Claude-format agent .md to the target tool format.

        fmt: "gemini" | "opencode"
        - Removes 'model' and 'version' frontmatter fields (both tools)
        - Replaces MCP list-tools with native equivalents (gemini)
          or capability map (opencode)
        - Injects an MCP usage note after the first heading
        """
        import re

        match = re.match(r"^---\n(.*?)\n---\n(.*)", content, re.DOTALL)
        if not match:
            return content
        fm_raw, body = match.group(1), match.group(2)

        # Strip model/version and collect non-tools lines
        base_lines: list[str] = []
        in_tools = False
        for line in fm_raw.splitlines():
            stripped = line.strip()
            if re.match(r"^(model|version):", stripped):
                in_tools = False
                continue
            if stripped == "tools:":
                in_tools = True
                continue  # drop the tools: block entirely; we'll rebuild it
            if in_tools and stripped.startswith("- "):
                continue  # drop all MCP tool entries
            elif in_tools and line and line[0] == " ":
                continue  # skip indented continuation lines inside the tools block
            else:
                in_tools = False
            base_lines.append(line)

        # Build the new tools block
        if fmt == "gemini":
            tools_block = ["tools:"] + [f"  - {t}" for t in self._GEMINI_NATIVE_TOOLS]
        else:  # opencode
            tools_block = ["tools:"] + [
                f"  {k}: {'true' if v else 'false'}" for k, v in self._OPENCODE_TOOLS.items()
            ]

        result_fm = base_lines + tools_block

        note = self._MCP_NOTE.get(fmt, "")
        body_with_note = re.sub(r"(# .+\n)", r"\1" + note, body, count=1)

        return "---\n" + "\n".join(result_fm) + "\n---\n" + body_with_note

    def _remove_stale_agents(self, src_base: Path, dst_base: Path, agent_files: list[str]) -> None:
        """Remove previously installed syntagma agent files that are no longer in agent_files.
        Only touches files whose stem matches the syntagma source agent names."""
        if self.dry_run or not dst_base.exists():
            return
        # Derive the full set of names syntagma has previously installed
        manifest = dst_base / MANIFEST_FILENAME
        if manifest.exists():
            managed = {
                line.strip().lower()
                for line in manifest.read_text(encoding="utf-8").splitlines()
                if line.strip()
            }
        else:
            managed = (
                {f.stem.lower() for f in src_base.glob("*.md")} if src_base.exists() else set()
            )
        managed |= {Path(f).stem.lower() for f in agent_files}
        current = {Path(f).stem.lower() for f in agent_files}
        for existing in dst_base.glob("*.md"):
            if existing.stem.lower() in managed and existing.stem.lower() not in current:
                if existing.is_symlink():
                    self._log(f"  skipping symlink: {existing}")
                    continue
                existing.unlink()
                self._log(f"stale agent removed: {existing}")

    def _copy_agents(
        self,
        src_base: Path,
        dst_base: Path,
        agent_files: list[str],
        fmt: str | None = None,
    ) -> None:
        """
        Replace syntagma agent files in dst_base/ (flat structure).
        Removes stale files from previous installs before copying.
        fmt: None (raw copy) | "gemini" | "opencode" — triggers frontmatter transform.
        """
        self._remove_stale_agents(src_base, dst_base, agent_files)
        for filename in agent_files:
            src = src_base / filename
            if not src.exists():
                self._log(f"  warning: agent source not found: {src}")
                continue
            dst = dst_base / filename
            if not self.dry_run:
                self._ensure_dir(dst_base)
                if fmt:
                    content = src.read_text(encoding="utf-8")
                    dst.write_text(self._transform_agent(content, fmt), encoding="utf-8")
                else:
                    shutil.copy2(src, dst)
            self._log(f"agent copied: {dst}")
        if not self.dry_run and dst_base.exists():
            stems = [Path(f).stem for f in agent_files]
            (dst_base / MANIFEST_FILENAME).write_text("\n".join(stems) + "\n", encoding="utf-8")

    def _copy_skills(self, src_base: Path, dst_base: Path, namespace: str | None = None) -> None:
        """
        Replace skills under dst_base/ with skills from src_base/.
        Removes stale skills left over from previous installs.
        """
        if not src_base.exists():
            return
        if not self.dry_run and dst_base.exists():
            shutil.rmtree(dst_base)
        for skill_dir in sorted(src_base.iterdir()):
            if not skill_dir.is_dir():
                continue
            src = skill_dir / "SKILL.md"
            if not src.exists():
                continue
            dst = dst_base / skill_dir.name / "SKILL.md"
            if not self.dry_run:
                self._ensure_dir(dst.parent)
                shutil.copy2(src, dst)
            self._log(f"skill copied: {dst}")

    # ------------------------------------------------------------------
    # Tool-specific installers
    # ------------------------------------------------------------------

    def _merge_claude_json(
        self, claude_json: Path, server_name: str, server_config: dict[str, Any]
    ) -> str:
        """
        Merge a server entry into ~/.claude.json mcpServers key.
        Returns: "added" | "already_installed"
        """
        existing: dict[str, Any] = {}
        if claude_json.exists():
            try:
                existing = json.loads(claude_json.read_text(encoding="utf-8"))
            except (json.JSONDecodeError, OSError):
                existing = {}

        servers: dict[str, Any] = existing.get("mcpServers", {})
        if server_name in servers and servers[server_name] == server_config:
            return "already_installed"

        servers[server_name] = server_config
        existing["mcpServers"] = servers

        if not self.dry_run:
            claude_json.write_text(
                json.dumps(existing, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
            )

        return "added"

    def install_claude_code(
        self,
        agent_files: list[str],
        claude_home: Path | None = None,
        claude_json: Path | None = None,
    ) -> None:
        """Install agents, skills, and MCP config for Claude Code."""
        if claude_home is None:
            claude_home = Path.home() / ".claude"
        if claude_json is None:
            claude_json = Path.home() / ".claude.json"

        agents_dst = claude_home / "agents"
        skills_dst = claude_home / "skills"
        agents_src = self.registry_dir / "agents"

        self._copy_agents(agents_src, agents_dst, agent_files)
        self._copy_skills(self.skills_dir, skills_dst)

        status = self._merge_claude_json(claude_json, "syntagma", _MCP_ENTRY)
        if status == "already_installed":
            self._log(f"MCP already installed: {claude_json}")
        else:
            self._log(f"MCP config added to: {claude_json}")

    def _copy_rules(self, src_base: Path, dst_base: Path) -> None:
        """Copy *.mdc rule files from src_base/ to dst_base/."""
        if not src_base.exists():
            return
        for src in sorted(src_base.glob("*.mdc")):
            dst = dst_base / src.name
            if not self.dry_run:
                self._ensure_dir(dst_base)
                shutil.copy2(src, dst)
            self._log(f"rule copied: {dst}")

    def install_cursor(self) -> None:
        """Install MCP config, agents, skills, and rules for Cursor (global)."""
        cursor_home = Path.home() / ".cursor"
        agents_src = self.registry_dir / "agents"
        rules_src = self.registry_dir / "rules"

        # Cursor natively supports Claude-format agents (model/version/MCP tools all OK)
        self._copy_agents(agents_src, cursor_home / "agents", _AGENT_FILES)
        self._copy_skills(self.skills_dir, cursor_home / "skills")
        self._copy_rules(rules_src, cursor_home / "rules")

        # Global MCP config (~/.cursor/mcp.json) — applies across all projects
        global_mcp = cursor_home / "mcp.json"
        status = self._merge_mcp_json(global_mcp, "syntagma", _MCP_ENTRY)
        if status == "already_installed":
            self._log(f"Cursor MCP already installed: {global_mcp}")
        else:
            self._log(f"Cursor MCP config {status}: {global_mcp}")

    def install_codex(self, project_dir: Path | None = None) -> None:
        """Create or append Syntagma section to AGENTS.md for Codex."""
        if project_dir is None:
            project_dir = Path.cwd()

        agents_md = project_dir / "AGENTS.md"
        if agents_md.exists():
            content = agents_md.read_text(encoding="utf-8")
            if "SYNTAGMA-BEGIN" in content or "AGENTS.md — Syntagma" in content:
                self._log(f"Codex AGENTS.md already contains Syntagma section: {agents_md}")
                return
            if not self.dry_run:
                with agents_md.open("a", encoding="utf-8") as f:
                    f.write(_SYNTAGMA_AGENTS_MD_SECTION)
            self._log(f"Syntagma section appended: {agents_md}")
        else:
            if not self.dry_run:
                self._ensure_dir(agents_md.parent)
                agents_md.write_text(
                    "# AGENTS.md\n" + _SYNTAGMA_AGENTS_MD_SECTION, encoding="utf-8"
                )
            self._log(f"AGENTS.md created: {agents_md}")

    def install_gemini(self, gemini_home: Path | None = None) -> None:
        """Install MCP config, agents, and skills for Gemini CLI."""
        if gemini_home is None:
            gemini_home = Path.home() / ".gemini"

        agents_dst = gemini_home / "agents"
        skills_dst = gemini_home / "skills"
        agents_src = self.registry_dir / "agents"

        self._copy_agents(agents_src, agents_dst, _AGENT_FILES, fmt="gemini")
        self._copy_skills(self.skills_dir, skills_dst)

        mcp_json = gemini_home / "mcp.json"
        status = self._merge_mcp_json(mcp_json, "syntagma", _MCP_ENTRY)
        if status == "already_installed":
            self._log(f"Gemini MCP already installed: {mcp_json}")
        else:
            self._log(f"Gemini MCP config {status}: {mcp_json}")

    def _merge_opencode_json(self, config_json: Path, server_name: str) -> str:
        """
        Merge a Syntagma MCP entry into ~/.config/opencode/opencode.json.
        OpenCode format: {"mcp": {"<name>": {"type": "local", "command": [...]}}}
        Returns: "added" | "already_installed"
        """
        entry = {"type": "local", "command": ["syntagma-mcp"]}
        existing: dict[str, Any] = {}
        if config_json.exists():
            try:
                existing = json.loads(config_json.read_text(encoding="utf-8"))
            except (json.JSONDecodeError, OSError):
                existing = {}

        mcp: dict[str, Any] = existing.get("mcp", {})
        if server_name in mcp and mcp[server_name] == entry:
            return "already_installed"

        mcp[server_name] = entry
        existing["mcp"] = mcp

        if not self.dry_run:
            self._ensure_dir(config_json.parent)
            config_json.write_text(
                json.dumps(existing, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
            )
        return "added"

    def install_opencode(self, opencode_config: Path | None = None) -> None:
        """Install MCP config, agents, and skills for OpenCode."""
        if opencode_config is None:
            opencode_config = Path.home() / ".config" / "opencode"

        agents_dst = opencode_config / "agents"
        skills_dst = opencode_config / "skills"
        agents_src = self.registry_dir / "agents"

        self._copy_agents(agents_src, agents_dst, _AGENT_FILES, fmt="opencode")
        self._copy_skills(self.skills_dir, skills_dst)

        config_json = opencode_config / "opencode.json"
        status = self._merge_opencode_json(config_json, "syntagma")
        if status == "already_installed":
            self._log(f"OpenCode MCP already installed: {config_json}")
        else:
            self._log(f"OpenCode MCP config added: {config_json}")

    # ------------------------------------------------------------------
    # Hook Installers (Standardized per Research Spec)
    # ------------------------------------------------------------------

    @staticmethod
    def _hook_command(item: dict[str, Any]) -> str | None:
        """Extract the command string from either flat or nested hook schema."""
        # nested: {"matcher": ..., "hooks": [{"type": "command", "command": "..."}]}
        hooks = item.get("hooks")
        if isinstance(hooks, list) and hooks:
            return str(hooks[0].get("command")) if hooks[0].get("command") is not None else None
        # flat (legacy): {"matcher": ..., "type": "command", "command": "..."}
        return item.get("command")

    def _merge_json_array(self, path: Path, key_path: list[str], new_item: dict[str, Any]) -> None:
        """Upsert a hook item into a nested JSON array.

        Replaces any existing item whose resolved command matches new_item's command,
        regardless of whether the existing item uses the legacy flat schema or the
        current nested-hooks schema. Appends if no match is found.
        """
        data: dict[str, Any] = {}
        if path.exists():
            try:
                data = json.loads(path.read_text(encoding="utf-8"))
            except Exception:
                data = {}

        # Navigate to the target array
        curr = data
        for key in key_path[:-1]:
            if key not in curr or not isinstance(curr[key], dict):
                curr[key] = {}
            curr = curr[key]

        target_key = key_path[-1]
        if target_key not in curr or not isinstance(curr[target_key], list):
            curr[target_key] = []

        target_list: list[dict[str, Any]] = curr[target_key]
        new_cmd = self._hook_command(new_item)

        for i, item in enumerate(target_list):
            if self._hook_command(item) == new_cmd:
                if item == new_item:
                    self._log(f"hook already up-to-date ({new_cmd}): {path}")
                    return
                # Replace legacy flat schema (or any stale entry) with the new item
                target_list[i] = new_item
                if not self.dry_run:
                    self._ensure_dir(path.parent)
                    path.write_text(
                        json.dumps(data, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
                    )
                self._log(f"hook migrated to nested schema ({new_cmd}): {path}")
                return

        target_list.append(new_item)
        if not self.dry_run:
            self._ensure_dir(path.parent)
            path.write_text(json.dumps(data, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
        self._log(f"hooks updated: {path}")

    def install_claude_hooks(self) -> None:
        """Claude Code Hooks: ~/.claude/settings.json (User Global Scope only)."""
        # SECURITY NOTE: "$PROMPT" is expanded by Claude Code's hook runner via its own shell
        # environment, not by a shell we invoke. Claude Code is responsible for sanitising the
        # value before expanding it. We cannot pass arguments as an array here because the
        # settings.json "command" field is a plain string interpreted by Claude Code's runner.
        events = [
            {
                "event": "SessionStart",
                "matcher": "startup|resume",
                "cmd": 'syntagma-hook ground "$PROMPT"',
            },
            {
                "event": "PreToolUse",
                "matcher": "Bash(git commit*)",
                "cmd": "syntagma-hook sniff --staged",
            },
            {
                "event": "PostToolUse",
                "matcher": "Edit|Write",
                "cmd": "syntagma-hook audit",
            },
        ]

        cfg_path = Path.home() / ".claude" / "settings.json"
        for e in events:
            hook_item = {
                "matcher": e["matcher"],
                "hooks": [
                    {
                        "type": "command",
                        "command": e["cmd"],
                        "timeout": 30,  # seconds — hooks must complete within 30 s (osn-hooks policy)
                    }
                ],
            }
            self._merge_json_array(cfg_path, ["hooks", e["event"]], hook_item)

    def install_gemini_hooks(self, gemini_home: Path) -> None:
        """Gemini CLI Hooks: ~/.gemini/settings.json (User Global Scope only)."""
        # SECURITY NOTE: "$PROMPT" is expanded by Gemini CLI's hook runner. The "command" field
        # is a plain string; array-style argument passing is not supported by this API.
        # Gemini CLI is responsible for sanitising prompt content before shell expansion.
        for cfg_path in [gemini_home / "settings.json"]:
            for e in [
                {
                    "event": "BeforeAgent",
                    "matcher": "startup",
                    "cmd": 'syntagma-hook ground "$PROMPT"',
                },
                {
                    "event": "BeforeTool",
                    "matcher": "run_shell_command(git commit*)",
                    "cmd": "syntagma-hook sniff --staged",
                },
                {
                    "event": "AfterTool",
                    "matcher": "write_file|replace_in_file",
                    "cmd": "syntagma-hook audit",
                },
            ]:
                hook_item = {
                    "name": f"syntagma-{e['event'].lower()}",
                    "matcher": e["matcher"],
                    "type": "command",
                    "command": e["cmd"],
                    "timeout": 10000,  # milliseconds
                }
                self._merge_json_array(cfg_path, ["hooks", e["event"]], hook_item)

    def install_codex_hooks(self) -> None:
        """Codex Hooks: ~/.codex/hooks.json + config.toml feature flag (User Global Scope only)."""
        # 1. Enable feature flag in global config
        toml_path = Path.home() / ".codex" / "config.toml"
        if not self.dry_run:
            self._ensure_dir(toml_path.parent)
        if toml_path.exists():
            content = toml_path.read_text(encoding="utf-8")
        else:
            content = ""
        if "codex_hooks = true" not in content:
            if "[features]" in content:
                content = content.replace("[features]", "[features]\ncodex_hooks = true")
            else:
                content = "[features]\ncodex_hooks = true\n\n" + content
            if not self.dry_run:
                toml_path.write_text(content, encoding="utf-8")
            self._log(f"Codex feature flag enabled: {toml_path}")

        # 2. Seed global hooks.json
        # SECURITY NOTE: "$PROMPT" is expanded by Codex's hook runner. The hooks.json "command"
        # field accepts only a plain string; array-style argument passing is not supported.
        # Codex is responsible for sanitising prompt content before shell expansion.
        events = [
            {
                "event": "SessionStart",
                "matcher": "startup",
                "cmd": 'syntagma-hook ground "$PROMPT"',
            },
            {
                "event": "PreToolUse",
                "matcher": "Bash(git commit*)",
                "cmd": "syntagma-hook sniff --staged",
            },
            {
                "event": "PostToolUse",
                "matcher": "apply_patch|write_file",
                "cmd": "syntagma-hook audit",
            },
        ]
        cfg_path = Path.home() / ".codex" / "hooks.json"
        for e in events:
            hook_item = {
                "matcher": e["matcher"],
                "type": "command",
                "command": e["cmd"],
                "timeout": 30,  # seconds — hooks must complete within 30 s (osn-hooks policy)
            }
            self._merge_json_array(cfg_path, ["hooks", e["event"]], hook_item)

    def install_cline_hooks(self) -> None:
        """Cline Hooks: File-based in ~/Documents/Cline/Hooks/ (User Global Scope only)."""
        # SECURITY NOTE: Cline executes hook scripts via shell. "$PROMPT" will be expanded by
        # bash at runtime. Cline does not support passing arguments as an array, so the string
        # form is unavoidable. Cline is responsible for sanitising the prompt value before
        # executing the hook script.
        hook_map = {
            "TaskStart": 'syntagma-hook ground "$PROMPT"',
            "PostToolUse": "syntagma-hook audit",
        }

        is_win = os.name == "nt"
        ext = ".ps1" if is_win else ""

        base = Path.home() / "Documents" / "Cline" / "Hooks"
        for name, cmd in hook_map.items():
            hook_file = base / f"{name}{ext}"
            content = cmd if is_win else f"#!/bin/bash\n{cmd}\n"
            if not self.dry_run:
                self._ensure_dir(base)
                hook_file.write_text(content, encoding="utf-8")
                if not is_win:
                    os.chmod(hook_file, 0o750)  # owner+group only; others cannot read/execute
            self._log(f"Cline hook created: {hook_file}")

    def install_cursor_hooks(self) -> None:
        """Cursor Hooks: ~/.cursor/hooks.json (User Global Scope only)."""
        # SECURITY NOTE: "$PROMPT" is expanded by Cursor's hook runner. The "command" field is a
        # plain string; array-style argument passing is not supported by this API. Cursor is
        # responsible for sanitising prompt content before shell expansion.
        events = [
            {"event": "sessionStart", "matcher": "*", "cmd": 'syntagma-hook ground "$PROMPT"'},
            {
                "event": "preToolUse",
                "matcher": "beforeShellExecution(git commit*)",
                "cmd": "syntagma-hook sniff --staged",
            },
            {
                "event": "postToolUse",
                "matcher": "afterFileEdit|Write",
                "cmd": "syntagma-hook audit",
            },
        ]

        cfg_path = Path.home() / ".cursor" / "hooks.json"
        for e in events:
            hook_item = {
                "matcher": e["matcher"],
                "type": "command",
                "command": e["cmd"],
                "timeout": 300,  # seconds
            }
            self._merge_json_array(cfg_path, ["hooks", e["event"]], hook_item)

    def install_tool(
        self,
        tool: str,
        agent_files: list[str],
        project_dir: Path | None = None,
    ) -> None:
        """Dispatch to the appropriate installer based on tool name."""
        if project_dir is None:
            project_dir = Path.cwd()

        if tool == "claude":
            self.install_claude_code(agent_files=agent_files)
            self.install_claude_hooks()
        elif tool == "cursor":
            self.install_cursor()
            self.install_cursor_hooks()
        elif tool == "codex":
            self.install_codex(project_dir=project_dir)
            self.install_codex_hooks()
        elif tool == "gemini":
            gemini_home = Path.home() / ".gemini"
            self.install_gemini(gemini_home=gemini_home)
            self.install_gemini_hooks(gemini_home)
        elif tool == "cline":
            self.install_cline_hooks()
        elif tool == "opencode":
            self.install_opencode()
        else:
            print(f"  Unknown tool: {tool}", file=sys.stderr)


# ---------------------------------------------------------------------------
# Interactive flow
# ---------------------------------------------------------------------------


def _detect_installed_tools() -> set[str]:
    """Return set of tool keys that already have Syntagma MCP registered."""
    installed: set[str] = set()

    # claude: ~/.claude.json mcpServers.syntagma
    claude_json = Path.home() / ".claude.json"
    if claude_json.exists():
        try:
            data = json.loads(claude_json.read_text(encoding="utf-8"))
            if "syntagma" in data.get("mcpServers", {}):
                installed.add("claude")
        except Exception:
            pass

    # cursor: ~/.cursor/mcp.json (global)
    cursor_mcp = Path.home() / ".cursor" / "mcp.json"
    if cursor_mcp.exists():
        try:
            data = json.loads(cursor_mcp.read_text(encoding="utf-8"))
            if "syntagma" in data.get("mcpServers", {}):
                installed.add("cursor")
        except Exception:
            pass

    # gemini: ~/.gemini/mcp.json
    gemini_mcp = Path.home() / ".gemini" / "mcp.json"
    if gemini_mcp.exists():
        try:
            data = json.loads(gemini_mcp.read_text(encoding="utf-8"))
            if "syntagma" in data.get("mcpServers", {}):
                installed.add("gemini")
        except Exception:
            pass

    # opencode: ~/.config/opencode/opencode.json mcp.syntagma
    opencode_cfg = Path.home() / ".config" / "opencode" / "opencode.json"
    if opencode_cfg.exists():
        try:
            data = json.loads(opencode_cfg.read_text(encoding="utf-8"))
            if "syntagma" in data.get("mcp", {}):
                installed.add("opencode")
        except Exception:
            pass

    # codex: AGENTS.md in cwd — accept SYNTAGMA-BEGIN marker or syntagma-mcp reference
    agents_md = Path.cwd() / "AGENTS.md"
    if agents_md.exists():
        try:
            content = agents_md.read_text(encoding="utf-8")
            if "SYNTAGMA-BEGIN" in content or "syntagma-mcp" in content:
                installed.add("codex")
        except Exception:
            pass

    # cline: check for the Syntagma hook file, not just the directory
    global_cline = Path.home() / "Documents" / "Cline" / "Hooks"
    local_cline = Path.cwd() / ".clinerules" / "hooks"
    if (global_cline / "TaskStart").exists() or (local_cline / "TaskStart").exists():
        installed.add("cline")

    return installed


def _prompt_input(prompt: str, default: str) -> str:
    if not sys.stdin.isatty():
        return default
    try:
        val = input(f"  {_BOLD}{prompt}{_RESET} {_DIM}[{default}]{_RESET}: ").strip()
        return val if val else default
    except (EOFError, KeyboardInterrupt):
        return default


def _save_yaml_section(config_path: Path, section: str, data: dict, dry_run: bool) -> None:
    try:
        import yaml  # type: ignore[import-untyped]
    except ImportError:
        return
    existing: dict = {}
    if config_path.exists():
        try:
            existing = yaml.safe_load(config_path.read_text(encoding="utf-8")) or {}
        except Exception:
            existing = {}
    existing[section] = data
    if not dry_run:
        config_path.parent.mkdir(parents=True, exist_ok=True)
        config_path.write_text(
            yaml.dump(existing, default_flow_style=False, allow_unicode=True), encoding="utf-8"
        )


def _step_redis(config_path: Path, dry_run: bool) -> dict:
    """Render Redis config form. Returns collected values."""
    try:
        import yaml  # type: ignore[import-untyped]

        existing = (
            yaml.safe_load(config_path.read_text(encoding="utf-8")) or {}
            if config_path.exists()
            else {}
        )
    except Exception:
        existing = {}
    cur = existing.get("redis", {})

    # Show current state if already configured
    if cur:
        enabled_label = (
            f"{_GREEN}enabled{_RESET}" if cur.get("enabled", True) else f"{_DIM}disabled{_RESET}"
        )
        sys.stdout.write(
            f"  Current: {enabled_label}  "
            f"{_DIM}{cur.get('host', 'localhost')}:{cur.get('port', 6379)}"
            f"  db={cur.get('db', 0)}  ttl={cur.get('ttl', 3600)}s{_RESET}\n\n"
        )
    else:
        sys.stdout.write(f"  {_DIM}Not configured yet.{_RESET}\n\n")
    sys.stdout.write(f"  {_DIM}Press Enter to keep the current value.{_RESET}\n\n")
    sys.stdout.flush()

    enabled_s = _prompt_input("Redis enabled  (true/false)", str(cur.get("enabled", True)).lower())
    host = _prompt_input("host          ", str(cur.get("host", "localhost")))
    port = _prompt_input("port          ", str(cur.get("port", 6379)))
    db = _prompt_input("db            ", str(cur.get("db", 0)))
    ttl = _prompt_input("ttl (seconds) ", str(cur.get("ttl", 3600)))

    data = {
        "enabled": enabled_s.lower() not in ("false", "no", "0"),
        "host": host,
        "port": int(port),
        "db": int(db),
        "ttl": int(ttl),
    }
    _save_yaml_section(config_path, "redis", data, dry_run)
    sys.stdout.write(f"\n  {_GREEN}✓{_RESET} Saved to {_DIM}{config_path}{_RESET}\n")
    sys.stdout.flush()
    return data


def _step_telemetry(config_path: Path, dry_run: bool) -> bool:
    """Render telemetry consent form. Returns consent bool."""
    # Show current state if already set
    try:
        import yaml  # type: ignore[import-untyped]

        existing = (
            yaml.safe_load(config_path.read_text(encoding="utf-8")) or {}
            if config_path.exists()
            else {}
        )
        cur_consent = existing.get("telemetry", {}).get("enabled")
    except Exception:
        cur_consent = None

    if cur_consent is not None:
        cur_label = f"{_GREEN}enabled{_RESET}" if cur_consent else f"{_DIM}disabled{_RESET}"
        sys.stdout.write(f"  Current: {cur_label}\n\n")

    sys.stdout.write(
        f"  Syntagma may collect {_BOLD}anonymous{_RESET} usage data\n"
        f"  to improve detection quality. No code is ever sent.\n\n"
    )
    sys.stdout.flush()
    default_yn = "Y/n" if cur_consent is not False else "y/N"
    consent = _prompt_confirm(f"Allow anonymous telemetry? [{default_yn}] ")
    _save_yaml_section(config_path, "telemetry", {"enabled": consent}, dry_run)
    if not dry_run:
        from syntagma.telemetry import write_consent

        write_consent(consent)
    label = f"{_GREEN}enabled{_RESET}" if consent else f"{_DIM}disabled{_RESET}"
    sys.stdout.write(f"\n  {_GREEN}✓{_RESET} Telemetry {label}\n")
    if consent:
        sys.stdout.write(f"  {_DIM}To opt out: syntagma telemetry off{_RESET}\n")
    sys.stdout.flush()
    return consent


def _run_interactive(installer: SyntagmaInstaller) -> int:
    """Full interactive installation wizard with ESC-to-back navigation."""
    from syntagma import config as _config

    TOTAL = 3
    config_path = _config.SYNTAGMA_HOME / "config.yaml"

    tool_options = list(_TOOL_DISPLAY.values())
    tool_keys = list(_TOOL_DISPLAY.keys())

    already_installed = _detect_installed_tools()
    # Pre-select already-installed tools
    tool_sel: list[bool] = [k in already_installed for k in tool_keys]

    sys.stdout.write(
        f"\n{_BOLD}{_CYAN}  ╔══════════════════════════╗{_RESET}\n"
        f"{_BOLD}{_CYAN}  ║   Syntagma  Setup  v1    ║{_RESET}\n"
        f"{_BOLD}{_CYAN}  ╚══════════════════════════╝{_RESET}\n"
    )
    sys.stdout.flush()

    chosen_tools: list[str] = []
    step = 1
    while step <= TOTAL:
        if step == 1:
            # Annotate already-installed tools
            annotated = [
                f"{label}  {_DIM}(installed){_RESET}" if k in already_installed else label
                for k, label in zip(tool_keys, tool_options, strict=False)
            ]
            _header(step, TOTAL, "Select your AI tool")
            _hint("↑↓ move", "Space select", "a toggle all", "Enter next")
            result = _checkbox_select("", annotated, tool_sel, allow_toggle_all=True)
            if result is _BACK:
                sys.stdout.write(f"  {_DIM}Already at first step.{_RESET}\n\n")
                sys.stdout.flush()
                continue
            tool_sel = result  # type: ignore[assignment]
            chosen_tools = []
            for i, sel in enumerate(tool_sel):
                if sel:
                    key = tool_keys[i]
                    if key == "all":
                        chosen_tools = [t for t in tool_keys if t != "all"]
                        break
                    chosen_tools.append(key)
            if not chosen_tools:
                sys.stdout.write(f"  {_YELLOW}⚠{_RESET}  Select at least one tool.\n\n")
                sys.stdout.flush()
                continue

            # Show what will be installed
            sys.stdout.write(f"\n  {_BOLD}What gets installed:{_RESET}\n")
            for tool in chosen_tools:
                display = _TOOL_DISPLAY.get(tool, tool)
                tag = (
                    f"  {_DIM}(already installed — will update){_RESET}"
                    if tool in already_installed
                    else ""
                )
                sys.stdout.write(f"\n  {_CYAN}{display}{_RESET}{tag}\n")
                if tool == "claude":
                    sys.stdout.write(f"    {_DIM}MCP → ~/.claude.json{_RESET}\n")
                    sys.stdout.write(f"    {_DIM}agents → ~/.claude/agents/{_RESET}\n")
                    sys.stdout.write(f"    {_DIM}skills → ~/.claude/skills/syntagma/{_RESET}\n")
                    sys.stdout.write(f"    {_DIM}Hooks  → ~/.claude/settings.json{_RESET}\n")
                    sys.stdout.write(f"    {_DIM}Mandates → CLAUDE.md{_RESET}\n")
                    for name in _AGENT_DISPLAY.values():
                        sys.stdout.write(f"    {_GREEN}+{_RESET} {_DIM}{name}{_RESET}\n")
                elif tool == "cursor":
                    sys.stdout.write(f"    {_DIM}MCP → ~/.cursor/mcp.json{_RESET}\n")
                    sys.stdout.write(f"    {_DIM}agents → ~/.cursor/agents/{_RESET}\n")
                    sys.stdout.write(f"    {_DIM}skills → ~/.cursor/skills/syntagma/{_RESET}\n")
                    sys.stdout.write(f"    {_DIM}rules  → ~/.cursor/rules/syntagma.mdc{_RESET}\n")
                    sys.stdout.write(f"    {_DIM}Hooks  → ~/.cursor/hooks.json{_RESET}\n")
                    sys.stdout.write(f"    {_DIM}Rules  → .cursorrules{_RESET}\n")
                    for name in _AGENT_DISPLAY.values():
                        sys.stdout.write(f"    {_GREEN}+{_RESET} {_DIM}{name}{_RESET}\n")
                elif tool == "gemini":
                    sys.stdout.write(f"    {_DIM}MCP → ~/.gemini/mcp.json{_RESET}\n")
                    sys.stdout.write(f"    {_DIM}agents → ~/.gemini/agents/{_RESET}\n")
                    sys.stdout.write(f"    {_DIM}skills → ~/.gemini/skills/syntagma/{_RESET}\n")
                    sys.stdout.write(f"    {_DIM}Hooks  → ~/.gemini/settings.json{_RESET}\n")
                    sys.stdout.write(f"    {_DIM}Local  → GEMINI.md{_RESET}\n")
                    for name in _AGENT_DISPLAY.values():
                        sys.stdout.write(f"    {_GREEN}+{_RESET} {_DIM}{name}{_RESET}\n")
                elif tool == "codex":
                    sys.stdout.write(f"    {_DIM}Hooks  → ~/.codex/hooks.json{_RESET}\n")
                    sys.stdout.write(
                        f"    {_DIM}Config → .codex/config.toml (feature flag){_RESET}\n"
                    )
                    sys.stdout.write(f"    {_DIM}AGENTS.md — Syntagma section{_RESET}\n")
                elif tool == "cline":
                    sys.stdout.write(f"    {_DIM}Global → ~/Documents/Cline/Hooks/{_RESET}\n")
                    sys.stdout.write(f"    {_DIM}Local  → .clinerules/hooks/{_RESET}\n")
                elif tool == "opencode":
                    sys.stdout.write(f"    {_DIM}MCP → ~/.config/opencode/opencode.json{_RESET}\n")
                    sys.stdout.write(f"    {_DIM}agents → ~/.config/opencode/agents/{_RESET}\n")
                    sys.stdout.write(
                        f"    {_DIM}skills → ~/.config/opencode/skills/syntagma/{_RESET}\n"
                    )
                    for name in _AGENT_DISPLAY.values():
                        sys.stdout.write(f"    {_GREEN}+{_RESET} {_DIM}{name}{_RESET}\n")
            sys.stdout.write("\n")
            sys.stdout.flush()
            step = 2

        elif step == 2:
            _header(step, TOTAL, "Environment  ·  Redis cache")
            _hint("Enter to confirm each field", "Esc back")
            if sys.stdin.isatty():
                skip = not _prompt_confirm("Configure Redis now? [Y/n] ")
            else:
                skip = True
            if skip:
                sys.stdout.write(f"  {_DIM}Skipped — defaults in config.yaml apply.{_RESET}\n\n")
                sys.stdout.flush()
                step = 3
            else:
                sys.stdout.write("\n")
                _step_redis(config_path, installer.dry_run)
                step = 3

        elif step == 3:
            _header(step, TOTAL, "Telemetry")
            _hint("Esc back")
            _step_telemetry(config_path, installer.dry_run)
            step = 4  # exit loop

    # Confirm
    sys.stdout.write(f"\n{'─' * 48}\n\n")
    sys.stdout.flush()
    if not _prompt_confirm("Install now? [Y/n] "):
        sys.stdout.write(f"\n  {_DIM}Aborted.{_RESET}\n")
        sys.stdout.flush()
        return 1

    sys.stdout.write("\n")
    sys.stdout.flush()
    return _do_install(installer, chosen_tools, _AGENT_FILES)


# ---------------------------------------------------------------------------
# Data seeding — GitHub Release download
# ---------------------------------------------------------------------------

_GITHUB_REPO = "epicsagas/Syntagma"
_DATA_ASSET_PREFIX = "syntagma-data-"
_DATA_SEED_EXCLUDE = {"syntagma.db"}


def _get_package_version() -> str:
    try:
        from importlib.metadata import version

        return version("syntagma")
    except Exception:
        return "latest"


def _resolve_release_asset_url(version: str) -> str:
    """
    Return the download URL for syntagma-data-<version>.tar.gz.
    Falls back to the latest release if version is not found.
    """
    import urllib.error

    api_base = f"https://api.github.com/repos/{_GITHUB_REPO}/releases"

    def _fetch_json(url: str) -> Any:
        req = urllib.request.Request(url, headers={"Accept": "application/vnd.github+json"})
        with urllib.request.urlopen(req, timeout=15) as resp:
            return json.loads(resp.read().decode())

    # Try exact version tag first, then fall back to latest
    for endpoint in (f"{api_base}/tags/v{version}", f"{api_base}/latest"):
        try:
            release = _fetch_json(endpoint)
            for asset in release.get("assets", []):
                if asset["name"].startswith(_DATA_ASSET_PREFIX) and asset["name"].endswith(
                    ".tar.gz"
                ):
                    return str(asset["browser_download_url"])
        except Exception:
            continue

    raise RuntimeError(
        f"No data asset found for v{version}.\nCheck: https://github.com/{_GITHUB_REPO}/releases"
    )


def _download_with_progress(url: str, dest: Path) -> None:
    """Download url → dest, printing a simple progress indicator."""

    def _reporthook(block_num: int, block_size: int, total_size: int) -> None:
        if total_size <= 0:
            return
        downloaded = min(block_num * block_size, total_size)
        pct = downloaded * 100 // total_size
        bar = "#" * (pct // 5)
        print(f"\r  [{bar:<20}] {pct:3d}%", end="", flush=True)

    urllib.request.urlretrieve(url, dest, reporthook=_reporthook)
    print()  # newline after progress bar


def _extract_data_archive(archive: Path, tmp_dir: Path) -> tuple[Path, Path]:
    """Extract tar.gz and return (meta_dir, raw_dir) inside the extracted tree."""
    with tarfile.open(archive, "r:gz") as tf:
        tf.extractall(tmp_dir, filter="data")  # prevent path-traversal attacks

    meta_dir = tmp_dir / "meta"
    raw_dir = tmp_dir / "raw"
    return meta_dir, raw_dir


def _find_bundled_db(extract_root: Path) -> Path | None:
    """Return bundled DB path from extracted data archive if present."""
    candidates = (
        extract_root / "meta" / "syntagma.db",
        extract_root / "db" / "syntagma.db",
        extract_root / "syntagma.db",
    )
    for candidate in candidates:
        if candidate.exists():
            return candidate
    return None


def _copy_tree(src_dir: Path, dst_dir: Path, flat: bool, prefix: str, dry_run: bool) -> int:
    """Copy files from src_dir → dst_dir. flat=True means dst is a flat directory."""
    seeded = 0
    glob = src_dir.iterdir() if flat else src_dir.rglob("*")

    for src_file in sorted(glob):
        if not src_file.is_file():
            continue
        if src_file.name in _DATA_SEED_EXCLUDE:
            continue
        if src_file.name.startswith("._"):
            continue

        rel = src_file.relative_to(src_dir)
        dst_file = dst_dir / (src_file.name if flat else rel)

        if not dst_file.exists() or src_file.stat().st_mtime > dst_file.stat().st_mtime:
            label = src_file.name if flat else str(rel)
            print(f"  {prefix}{'data' if flat else 'raw'}: {label}")
            if not dry_run:
                dst_file.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(src_file, dst_file)
            seeded += 1

    return seeded


_CONFIG_YAML_TEMPLATE = """\
# Syntagma configuration
# Values here are overridden by environment variables (SYNTAGMA_*).

redis:
  host: localhost
  port: 6379
  db: 0
  ttl: 3600       # cache TTL in seconds
  enabled: true

api:
  host: "0.0.0.0"
  port: 8000

mcp:
  host: localhost
  port: 43175

embedding:
  provider: local  # "local" | "openai"
"""


def _init_config(home: Path, dry_run: bool = False) -> None:
    config_path = home / "config.yaml"
    if config_path.exists():
        print(f"  config: already exists — {config_path}")
        return
    prefix = "[dry-run] " if dry_run else ""
    print(f"  {prefix}config: creating {config_path}")
    if not dry_run:
        config_path.parent.mkdir(parents=True, exist_ok=True)
        config_path.write_text(_CONFIG_YAML_TEMPLATE, encoding="utf-8")


def _init_data(dry_run: bool = False, offline: str | None = None, local: bool = False) -> None:
    """
    Seed ~/.syntagma/ with knowledge data.

    Source priority:
      1. --offline <path>  — local tar.gz file
      2. --local           — source tree meta/ + raw/ (dev/build, skips GitHub)
      3. GitHub Release    — downloads syntagma-data-<version>.tar.gz
      4. Package fallback  — meta/ and raw/ inside the source tree (dev installs)
    """
    from syntagma import config as _config

    prefix = "[dry-run] " if dry_run else ""

    _init_config(_config.SYNTAGMA_HOME, dry_run=dry_run)

    if not dry_run:
        for d in (
            _config.DATA_DIR,
            _config.RAW_DIR,
            _config.DB_DIR,
            _config.LOG_DIR,
            _config.CACHE_DIR,
        ):
            d.mkdir(parents=True, exist_ok=True)

    seeded = 0

    # --- Try archive source (offline / local / GitHub Release) ---
    archive_path: Path | None = None

    if local:
        print("  Using source tree (--local)")
    elif offline:
        archive_path = Path(offline).expanduser().resolve()
        if not archive_path.exists():
            print(f"  Error: file not found: {archive_path}", file=sys.stderr)
            return
        print(f"  Using local archive: {archive_path}")
    else:
        version = _get_package_version()
        cached = _config.CACHE_DIR / f"{_DATA_ASSET_PREFIX}{version}.tar.gz"

        if cached.exists():
            print(f"  Using cached data: {cached.name}")
            archive_path = cached
        else:
            try:
                print(f"  Fetching knowledge data v{version} from GitHub Releases...")
                url = _resolve_release_asset_url(version)
                print(f"  Downloading: {url}")
                if not dry_run:
                    _config.CACHE_DIR.mkdir(parents=True, exist_ok=True)
                    _download_with_progress(url, cached)
                    archive_path = cached
                else:
                    print(f"  {prefix}Would download to: {cached}")
            except Exception as exc:
                print(f"  Warning: GitHub download failed ({exc})")
                print("  Falling back to local package data...")

    db_src: Path | None = None
    archive_mode = bool(archive_path)
    archive_had_db = False

    if archive_path and not dry_run:
        with tempfile.TemporaryDirectory(prefix="syntagma-seed-") as tmp:
            tmp_path = Path(tmp)
            meta_dir, raw_dir = _extract_data_archive(archive_path, tmp_path)

            if meta_dir.exists():
                seeded += _copy_tree(
                    meta_dir, _config.DATA_DIR, flat=True, prefix=prefix, dry_run=False
                )
            db_src = _find_bundled_db(tmp_path)
            archive_had_db = db_src is not None
            if db_src:
                dst_db = _config.DB_PATH
                if not dst_db.exists() or db_src.stat().st_mtime > dst_db.stat().st_mtime:
                    dst_db = _config.DB_PATH
                    dst_db.parent.mkdir(parents=True, exist_ok=True)
                    shutil.copy2(db_src, dst_db)
                    print(f"  {prefix}db: syntagma.db → {dst_db}")
                    seeded += 1
            if raw_dir.exists():
                seeded += _copy_tree(
                    raw_dir, _config.RAW_DIR, flat=False, prefix=prefix, dry_run=False
                )
    else:
        # Dev / source-tree fallback
        repo_root = Path(__file__).parent.parent.parent.parent
        src_meta = repo_root / "meta"
        src_raw = repo_root / "raw"

        if src_meta.exists():
            seeded += _copy_tree(
                src_meta, _config.DATA_DIR, flat=True, prefix=prefix, dry_run=dry_run
            )
            db_in_src = src_meta / "syntagma.db"
            if db_in_src.exists() and not dry_run:
                dst_db = _config.DB_PATH
                dst_db.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(db_in_src, dst_db)
                print(f"  {prefix}db: syntagma.db → {dst_db}")
                seeded += 1
                db_src = db_in_src
        if src_raw.exists():
            seeded += _copy_tree(
                src_raw, _config.RAW_DIR, flat=False, prefix=prefix, dry_run=dry_run
            )

    # Seed the vector database if bundled
    if db_src and not dry_run and not archive_mode:
        dst_db = _config.DB_PATH
        if not dst_db.exists() or db_src.stat().st_mtime > dst_db.stat().st_mtime:
            dst_db.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(db_src, dst_db)
            print(f"  {prefix}db: syntagma.db → {dst_db}")
            seeded += 1

    if seeded == 0:
        print("  data: already up to date")

    if archive_mode and not archive_had_db and not _config.DB_PATH.exists():
        print("  Warning: archive did not include syntagma.db")

    if not dry_run and not _config.DB_PATH.exists():
        print(f"\n  Run `syntagma build` to generate the vector database.\n  → {_config.DB_PATH}")


def _do_install(installer: SyntagmaInstaller, tools: list[str], agent_files: list[str]) -> int:
    """Execute installation for the given tools and agent files."""
    project_dir = Path.cwd()

    for tool in tools:
        display = _TOOL_DISPLAY.get(tool, tool)
        print(f"\nInstalling for {display}...")
        try:
            installer.install_tool(tool, agent_files=agent_files, project_dir=project_dir)
        except PermissionError as exc:
            print(f"  Permission denied: {exc}", file=sys.stderr)
            print("  Try running with appropriate permissions.", file=sys.stderr)
        except OSError as exc:
            print(f"  Error: {exc}", file=sys.stderr)

    print("\nDone.")
    return 0


# ---------------------------------------------------------------------------
# main entry point
# ---------------------------------------------------------------------------


def _resolve_package_registry_dir() -> Path:
    """Return the registry/ directory bundled with the installed package."""
    # Preferred: registry/ co-located inside the syntagma package (works when installed)
    bundled = Path(__file__).parent.parent / "registry"
    if bundled.exists():
        return bundled
    # Fallback: running directly from the source repo root
    repo_root = Path(__file__).parent.parent.parent.parent
    return repo_root / "registry"


def _resolve_tools(raw: list[str]) -> tuple[list[str], list[str]]:
    """
    Validate and expand the tool list from positional args.
    Returns (resolved_tools, errors).
    """
    non_all = [t for t in SUPPORTED_TOOLS if t != "all"]
    resolved: list[str] = []
    errors: list[str] = []

    for name in raw:
        canonical = _TOOL_ALIASES.get(name.lower(), name.lower())
        if canonical == "all":
            return list(non_all), []
        if canonical not in non_all:
            errors.append(name)
        elif canonical not in resolved:
            resolved.append(canonical)

    return resolved, errors


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)

    registry_dir = _resolve_package_registry_dir()
    skills_dir = registry_dir / "skills"

    installer = SyntagmaInstaller(
        registry_dir=registry_dir,
        skills_dir=skills_dir,
        dry_run=args.dry_run,
    )

    if args.dry_run:
        print("[dry-run mode] No files will be written.\n")

    # Always initialise ~/.syntagma data dirs first
    print("Initialising ~/.syntagma data directory...")
    _init_data(dry_run=args.dry_run, offline=args.offline, local=args.local)

    # --local with no tools = data-only seed, skip tool installation
    if args.local and not args.tools:
        return 0

    # Non-interactive: positional tools provided
    if args.tools:
        tools, errors = _resolve_tools(args.tools)
        if errors:
            print(
                f"Unknown tool(s): {', '.join(errors)}\nChoose from: {', '.join(SUPPORTED_TOOLS)}",
                file=sys.stderr,
            )
            return 2
        return _do_install(installer, tools, _AGENT_FILES)

    # Interactive
    return _run_interactive(installer)
