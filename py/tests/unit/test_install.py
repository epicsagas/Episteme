"""
Tests for syntagma install command (src/syntagma/cli/install.py)
"""

import json
import sys
from pathlib import Path
from unittest.mock import MagicMock, patch, mock_open

import pytest

# Ensure src is importable
sys.path.insert(0, str(Path(__file__).parent.parent.parent / "src"))

from syntagma.cli.install import MANIFEST_FILENAME


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _make_installer():
    """Import and return SyntagmaInstaller (lazy to avoid import errors before impl)."""
    from syntagma.cli.install import SyntagmaInstaller

    return SyntagmaInstaller


# ---------------------------------------------------------------------------
# 1. Module-level sanity: install module is importable
# ---------------------------------------------------------------------------


class TestInstallModuleImport:
    def test_module_importable(self):
        from syntagma.cli import install  # noqa: F401

    def test_main_function_exists(self):
        from syntagma.cli.install import main

        assert callable(main)

    def test_installer_class_exists(self):
        from syntagma.cli.install import SyntagmaInstaller

        assert SyntagmaInstaller is not None


# ---------------------------------------------------------------------------
# 2. CLI argument parsing
# ---------------------------------------------------------------------------


class TestArgumentParsing:
    def test_help_exits_zero(self):
        from syntagma.cli.install import build_parser

        parser = build_parser()
        with pytest.raises(SystemExit) as exc:
            parser.parse_args(["--help"])
        assert exc.value.code == 0

    def test_positional_single_tool_parsed(self):
        from syntagma.cli.install import build_parser

        parser = build_parser()
        args = parser.parse_args(["claude"])
        assert args.tools == ["claude"]

    def test_positional_multiple_tools_parsed(self):
        from syntagma.cli.install import build_parser

        parser = build_parser()
        args = parser.parse_args(["cursor", "codex"])
        assert args.tools == ["cursor", "codex"]

    def test_dry_run_flag_parsed(self):
        from syntagma.cli.install import build_parser

        parser = build_parser()
        args = parser.parse_args(["--dry-run"])
        assert args.dry_run is True

    def test_default_tools_is_empty(self):
        from syntagma.cli.install import build_parser

        parser = build_parser()
        args = parser.parse_args([])
        assert args.tools == []

    def test_valid_tool_choices(self):
        from syntagma.cli.install import SUPPORTED_TOOLS

        expected = {"claude", "cursor", "codex", "gemini", "cline", "opencode", "all"}
        assert expected == set(SUPPORTED_TOOLS)


# ---------------------------------------------------------------------------
# 3. SyntagmaInstaller — MCP JSON merge logic
# ---------------------------------------------------------------------------


class TestMcpJsonMerge:
    def test_merge_into_empty_file(self, tmp_path):
        Installer = _make_installer()
        inst = Installer(
            registry_dir=tmp_path / "agents", skills_dir=tmp_path / "skills", dry_run=False
        )

        target = tmp_path / "mcp.json"
        inst._merge_mcp_json(target, "syntagma", {"command": "syntagma-mcp", "args": []})

        data = json.loads(target.read_text())
        assert data["mcpServers"]["syntagma"]["command"] == "syntagma-mcp"

    def test_merge_preserves_existing_servers(self, tmp_path):
        Installer = _make_installer()
        inst = Installer(
            registry_dir=tmp_path / "agents", skills_dir=tmp_path / "skills", dry_run=False
        )

        target = tmp_path / "mcp.json"
        target.write_text(
            json.dumps({"mcpServers": {"other-tool": {"command": "other-cmd", "args": []}}})
        )

        inst._merge_mcp_json(target, "syntagma", {"command": "syntagma-mcp", "args": []})

        data = json.loads(target.read_text())
        assert "other-tool" in data["mcpServers"]
        assert "syntagma" in data["mcpServers"]

    def test_merge_already_installed_is_idempotent(self, tmp_path):
        Installer = _make_installer()
        inst = Installer(
            registry_dir=tmp_path / "agents", skills_dir=tmp_path / "skills", dry_run=False
        )

        target = tmp_path / "mcp.json"
        existing = {"mcpServers": {"syntagma": {"command": "syntagma-mcp", "args": []}}}
        target.write_text(json.dumps(existing))

        result = inst._merge_mcp_json(target, "syntagma", {"command": "syntagma-mcp", "args": []})
        assert result == "already_installed"


# ---------------------------------------------------------------------------
# 4. SyntagmaInstaller — agent/skill file copy
# ---------------------------------------------------------------------------


class TestAgentSkillCopy:
    def test_install_agent_copies_file(self, tmp_path):
        Installer = _make_installer()

        agents_src = tmp_path / "src_agents"
        agents_src.mkdir()
        agent_file = agents_src / "syntagma-advisor.md"
        agent_file.write_text("---\nname: syntagma-advisor\n---\n")

        agents_dst = tmp_path / "dst_agents"
        inst = Installer(registry_dir=agents_src, skills_dir=tmp_path / "skills", dry_run=False)

        inst._copy_agents(agents_src, agents_dst, agent_files=["syntagma-advisor.md"])

        assert (agents_dst / "syntagma-advisor.md").exists()

    def test_dry_run_does_not_create_files(self, tmp_path):
        Installer = _make_installer()

        agents_src = tmp_path / "src_agents"
        agents_src.mkdir()
        agent_file = agents_src / "syntagma-advisor.md"
        agent_file.write_text("---\nname: syntagma-advisor\n---\n")

        agents_dst = tmp_path / "dst_agents"
        inst = Installer(registry_dir=agents_src, skills_dir=tmp_path / "skills", dry_run=True)

        inst._copy_agents(agents_src, agents_dst, agent_files=["syntagma-advisor.md"])

        assert not agents_dst.exists() or not any(agents_dst.rglob("*.md"))

    def test_remove_stale_agents_protects_non_syntagma_files(self, tmp_path):
        Installer = _make_installer()

        src_base = tmp_path / "src_agents"
        src_base.mkdir()
        (src_base / "syntagma-advisor.md").write_text("---\nname: syntagma-advisor\n---\n")

        dst_base = tmp_path / "dst_agents"
        dst_base.mkdir()
        (dst_base / "syntagma-advisor.md").write_text("---\nname: syntagma-advisor\n---\n")
        third_party = dst_base / "third-party-agent.md"
        third_party.write_text("# third-party\n")

        inst = Installer(registry_dir=src_base, skills_dir=tmp_path / "skills", dry_run=False)
        inst._remove_stale_agents(src_base, dst_base, agent_files=[])

        assert not (dst_base / "syntagma-advisor.md").exists()
        assert third_party.exists()

    def test_remove_stale_agents_uses_manifest_to_remove_deleted_source(self, tmp_path):
        Installer = _make_installer()

        src_base = tmp_path / "src_agents"
        src_base.mkdir()
        # syntagma-old.md was deleted from source — not present in src_base

        dst_base = tmp_path / "dst_agents"
        dst_base.mkdir()
        stale = dst_base / "syntagma-old.md"
        stale.write_text("# old\n")
        # manifest records the previously installed stem
        (dst_base / MANIFEST_FILENAME).write_text("syntagma-old\n", encoding="utf-8")

        inst = Installer(registry_dir=src_base, skills_dir=tmp_path / "skills", dry_run=False)
        inst._remove_stale_agents(src_base, dst_base, agent_files=[])

        assert not stale.exists()

    def test_remove_stale_agents_uses_manifest_when_src_base_missing(self, tmp_path):
        Installer = _make_installer()

        src_base = tmp_path / "nonexistent_src"
        # src_base intentionally NOT created

        dst_base = tmp_path / "dst_agents"
        dst_base.mkdir()
        agent_file = dst_base / "syntagma-advisor.md"
        agent_file.write_text("---\nname: syntagma-advisor\n---\n")
        (dst_base / MANIFEST_FILENAME).write_text("syntagma-advisor\n", encoding="utf-8")

        inst = Installer(registry_dir=src_base, skills_dir=tmp_path / "skills", dry_run=False)
        inst._remove_stale_agents(src_base, dst_base, agent_files=[])

        assert not agent_file.exists()

    def test_remove_stale_agents_skips_symlinks(self, tmp_path):
        Installer = _make_installer()

        src_base = tmp_path / "src_agents"
        src_base.mkdir()

        dst_base = tmp_path / "dst_agents"
        dst_base.mkdir()
        # Create an actual file elsewhere that the symlink points to
        real_file = tmp_path / "real-syntagma-advisor.md"
        real_file.write_text("---\nname: syntagma-advisor\n---\n")
        symlink = dst_base / "syntagma-advisor.md"
        symlink.symlink_to(real_file)
        (dst_base / MANIFEST_FILENAME).write_text("syntagma-advisor\n", encoding="utf-8")

        inst = Installer(registry_dir=src_base, skills_dir=tmp_path / "skills", dry_run=False)
        inst._remove_stale_agents(src_base, dst_base, agent_files=[])

        assert symlink.exists()

    def test_copy_agents_writes_manifest(self, tmp_path):
        Installer = _make_installer()

        src_base = tmp_path / "src_agents"
        src_base.mkdir()
        (src_base / "syntagma-advisor.md").write_text("---\nname: syntagma-advisor\n---\n")
        (src_base / "syntagma-researcher.md").write_text("---\nname: syntagma-researcher\n---\n")

        dst_base = tmp_path / "dst_agents"
        inst = Installer(registry_dir=src_base, skills_dir=tmp_path / "skills", dry_run=False)
        inst._copy_agents(
            src_base,
            dst_base,
            agent_files=["syntagma-advisor.md", "syntagma-researcher.md"],
        )

        manifest = dst_base / MANIFEST_FILENAME
        assert manifest.exists()
        stems = {
            line.strip()
            for line in manifest.read_text(encoding="utf-8").splitlines()
            if line.strip()
        }
        assert stems == {"syntagma-advisor", "syntagma-researcher"}


# ---------------------------------------------------------------------------
# 5. Claude Code installer
# ---------------------------------------------------------------------------


class TestClaudeCodeInstaller:
    def test_install_claude_code_adds_mcp_to_claude_json(self, tmp_path):
        Installer = _make_installer()

        registry_dir = tmp_path / "registry"
        agents_src = registry_dir / "agents"
        agents_src.mkdir(parents=True)
        (agents_src / "syntagma-advisor.md").write_text("---\nname: syntagma-advisor\n---\n")

        skills_src = tmp_path / "src_skills"
        skills_src.mkdir()

        claude_home = tmp_path / "claude_home"
        claude_json = tmp_path / ".claude.json"
        inst = Installer(registry_dir=registry_dir, skills_dir=skills_src, dry_run=False)

        inst.install_claude_code(
            agent_files=["syntagma-advisor.md"],
            claude_home=claude_home,
            claude_json=claude_json,
        )

        assert claude_json.exists()
        data = json.loads(claude_json.read_text())
        assert "syntagma" in data["mcpServers"]

    def test_install_claude_code_preserves_existing_claude_json(self, tmp_path):
        Installer = _make_installer()

        registry_dir = tmp_path / "registry"
        agents_src = registry_dir / "agents"
        agents_src.mkdir(parents=True)
        skills_src = tmp_path / "src_skills"
        skills_src.mkdir()

        claude_home = tmp_path / "claude_home"
        claude_json = tmp_path / ".claude.json"
        claude_json.write_text(
            json.dumps({"numStartups": 5, "mcpServers": {"other": {"command": "other"}}})
        )

        inst = Installer(registry_dir=registry_dir, skills_dir=skills_src, dry_run=False)
        inst.install_claude_code(agent_files=[], claude_home=claude_home, claude_json=claude_json)

        data = json.loads(claude_json.read_text())
        assert data["numStartups"] == 5
        assert "other" in data["mcpServers"]
        assert "syntagma" in data["mcpServers"]

    def test_install_claude_code_creates_agents_dir(self, tmp_path):
        Installer = _make_installer()

        registry_dir = tmp_path / "registry"
        agents_src = registry_dir / "agents"
        agents_src.mkdir(parents=True)
        (agents_src / "syntagma-advisor.md").write_text("---\nname: syntagma-advisor\n---\n")

        skills_src = tmp_path / "src_skills"
        skills_src.mkdir()

        claude_home = tmp_path / "claude_home"
        claude_json = tmp_path / ".claude.json"
        inst = Installer(registry_dir=registry_dir, skills_dir=skills_src, dry_run=False)
        inst.install_claude_code(
            agent_files=["syntagma-advisor.md"], claude_home=claude_home, claude_json=claude_json
        )

        assert (claude_home / "agents").is_dir()
        assert (claude_home / "agents" / "syntagma-advisor.md").exists()


# ---------------------------------------------------------------------------
# 6. Cursor installer
# ---------------------------------------------------------------------------


class TestCursorInstaller:
    def test_install_cursor_creates_cursor_mcp_json(self, tmp_path, monkeypatch):
        monkeypatch.setattr(Path, "home", staticmethod(lambda: tmp_path))

        Installer = _make_installer()
        inst = Installer(registry_dir=tmp_path / "a", skills_dir=tmp_path / "s", dry_run=False)

        inst.install_cursor()

        mcp_path = tmp_path / ".cursor" / "mcp.json"
        assert mcp_path.exists()
        data = json.loads(mcp_path.read_text())
        assert "syntagma" in data["mcpServers"]
        assert data["mcpServers"]["syntagma"]["command"] == "syntagma-mcp"


# ---------------------------------------------------------------------------
# 7. Codex installer
# ---------------------------------------------------------------------------


class TestCodexInstaller:
    def test_install_codex_creates_agents_md(self, tmp_path):
        Installer = _make_installer()
        inst = Installer(registry_dir=tmp_path / "a", skills_dir=tmp_path / "s", dry_run=False)

        project_dir = tmp_path / "project"
        project_dir.mkdir()

        inst.install_codex(project_dir=project_dir)

        agents_md = project_dir / "AGENTS.md"
        assert agents_md.exists()
        content = agents_md.read_text()
        assert "Syntagma" in content

    def test_install_codex_appends_if_agents_md_exists(self, tmp_path):
        Installer = _make_installer()
        inst = Installer(registry_dir=tmp_path / "a", skills_dir=tmp_path / "s", dry_run=False)

        project_dir = tmp_path / "project"
        project_dir.mkdir()
        agents_md = project_dir / "AGENTS.md"
        agents_md.write_text("# Existing content\n\nSome prior documentation.\n")

        inst.install_codex(project_dir=project_dir)

        content = agents_md.read_text()
        assert "Existing content" in content
        assert "Syntagma" in content

    def test_install_codex_does_not_duplicate_section(self, tmp_path):
        Installer = _make_installer()
        inst = Installer(registry_dir=tmp_path / "a", skills_dir=tmp_path / "s", dry_run=False)

        project_dir = tmp_path / "project"
        project_dir.mkdir()
        agents_md = project_dir / "AGENTS.md"
        agents_md.write_text("# AGENTS.md — Syntagma\nAlready installed.\n")

        inst.install_codex(project_dir=project_dir)

        content = agents_md.read_text()
        assert content.count("AGENTS.md — Syntagma") == 1


# ---------------------------------------------------------------------------
# 8. Gemini CLI installer
# ---------------------------------------------------------------------------


class TestGeminiInstaller:
    def test_install_gemini_creates_mcp_json(self, tmp_path):
        Installer = _make_installer()
        inst = Installer(registry_dir=tmp_path / "a", skills_dir=tmp_path / "s", dry_run=False)

        gemini_home = tmp_path / "gemini_home"
        inst.install_gemini(gemini_home=gemini_home)

        mcp_path = gemini_home / "mcp.json"
        assert mcp_path.exists()
        data = json.loads(mcp_path.read_text())
        assert "syntagma" in data["mcpServers"]


# ---------------------------------------------------------------------------
# 9. OpenCode installer
# ---------------------------------------------------------------------------


class TestOpenCodeInstaller:
    def test_install_opencode_creates_mcp_json(self, tmp_path):
        Installer = _make_installer()
        inst = Installer(registry_dir=tmp_path / "a", skills_dir=tmp_path / "s", dry_run=False)

        opencode_config = tmp_path / "opencode"

        inst.install_opencode(opencode_config=opencode_config)

        config_path = opencode_config / "opencode.json"
        assert config_path.exists()
        data = json.loads(config_path.read_text())
        assert "syntagma" in data["mcp"]
        assert data["mcp"]["syntagma"]["command"] == ["syntagma-mcp"]


# ---------------------------------------------------------------------------
# 10. __main__.py integration: install subcommand registered
# ---------------------------------------------------------------------------


class TestMainEntryPoint:
    def _env_with_src(self):
        import os

        env = os.environ.copy()
        src_dir = str(Path(__file__).parent.parent.parent / "src")
        existing = env.get("PYTHONPATH", "")
        env["PYTHONPATH"] = f"{src_dir}:{existing}" if existing else src_dir
        return env

    def test_install_subcommand_in_help(self):
        import subprocess

        result = subprocess.run(
            [sys.executable, "-m", "syntagma", "--help"],
            capture_output=True,
            text=True,
            cwd=str(Path(__file__).parent.parent.parent),
            env=self._env_with_src(),
        )
        assert "install" in result.stdout

    def test_install_help_exits_zero(self):
        import subprocess

        result = subprocess.run(
            [sys.executable, "-m", "syntagma", "install", "--help"],
            capture_output=True,
            text=True,
            cwd=str(Path(__file__).parent.parent.parent),
            env=self._env_with_src(),
        )
        assert result.returncode == 0
        assert "TOOL" in result.stdout
