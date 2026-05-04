"""
Tests for fixes in src/syntagma/cli/install.py:
  Fix 1 — _transform_agent() state machine: indented non-"- " lines must not corrupt output
  Fix 2 — Cline detection: directory-only existence must not cause false positive
  Fix 3 — _merge_json_array: upserts legacy flat hook schema to nested schema
"""

import sys
from pathlib import Path
from unittest.mock import patch

import pytest

sys.path.insert(0, str(Path(__file__).parent.parent.parent / "src"))


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _installer_instance():
    from syntagma.cli.install import SyntagmaInstaller
    from pathlib import Path

    return SyntagmaInstaller(registry_dir=Path("/tmp"), skills_dir=Path("/tmp"), dry_run=True)


def _detect_installed():
    from syntagma.cli.install import _detect_installed_tools

    return _detect_installed_tools


# ---------------------------------------------------------------------------
# Fix 1 — _transform_agent() indented continuation lines
# ---------------------------------------------------------------------------


class TestTransformAgentIndentedLines:
    """An indented line that does not start with '- ' must not be appended to base_lines."""

    # Minimal agent content with a realistic frontmatter that includes
    # a tools: block containing a plain "- item" entry, plus an indented
    # continuation-style line (no leading "- ").
    _INPUT = (
        "---\n"
        "name: test-agent\n"
        "description: a test agent\n"
        "tools:\n"
        "  - search_knowledge\n"
        "  - get_entity\n"
        "  indented_but_not_list_item: value\n"  # <-- the problematic line
        "---\n"
        "# Test Agent\n"
        "Body text.\n"
    )

    def test_indented_non_list_line_does_not_appear_in_output_fm(self):
        """The indented non-'- ' line must NOT appear in the rebuilt frontmatter."""
        inst = _installer_instance()
        result = inst._transform_agent(self._INPUT, "gemini")

        # Extract frontmatter from result
        assert result.startswith("---\n"), "Result must start with ---"
        fm_end = result.index("\n---\n", 4)
        fm = result[4:fm_end]

        assert "indented_but_not_list_item" not in fm, (
            "Indented non-list line leaked into rebuilt frontmatter"
        )

    def test_indented_non_list_line_body_preserved(self):
        """The body content must remain intact."""
        inst = _installer_instance()
        result = inst._transform_agent(self._INPUT, "gemini")
        assert "# Test Agent" in result
        assert "Body text." in result

    def test_valid_frontmatter_keys_preserved(self):
        """name and description must survive the transformation."""
        inst = _installer_instance()
        result = inst._transform_agent(self._INPUT, "gemini")
        fm_end = result.index("\n---\n", 4)
        fm = result[4:fm_end]
        assert "name: test-agent" in fm
        assert "description: a test agent" in fm

    def test_tools_list_items_are_removed(self):
        """Original '- search_knowledge' entries must not appear in rebuilt frontmatter."""
        inst = _installer_instance()
        result = inst._transform_agent(self._INPUT, "gemini")
        fm_end = result.index("\n---\n", 4)
        fm = result[4:fm_end]
        assert "search_knowledge" not in fm
        assert "get_entity" not in fm


# ---------------------------------------------------------------------------
# Fix 2 — Cline detection false positive
# ---------------------------------------------------------------------------


class TestClineDetection:
    """detect_installed_tools must check for the hook *file*, not just the directory."""

    def test_directory_exists_without_hook_file_is_not_detected(self, tmp_path):
        """Directory present but no hook file → cline must NOT be in result."""
        hooks_dir = tmp_path / "Documents" / "Cline" / "Hooks"
        hooks_dir.mkdir(parents=True)

        detect = _detect_installed()
        with (
            patch("pathlib.Path.home", return_value=tmp_path),
            patch("pathlib.Path.cwd", return_value=tmp_path),
        ):
            result = detect()

        assert "cline" not in result, (
            "cline must not be detected when hook directory exists but hook file is absent"
        )

    def test_hook_file_present_is_detected(self, tmp_path):
        """Hook file present → cline must be in result."""
        hooks_dir = tmp_path / "Documents" / "Cline" / "Hooks"
        hooks_dir.mkdir(parents=True)
        # The hook filenames created by install_cline_hooks are: TaskStart, PreToolUse, TaskComplete
        (hooks_dir / "TaskStart").write_text("#!/bin/bash\nsyntagma-hook ground\n")

        detect = _detect_installed()
        with (
            patch("pathlib.Path.home", return_value=tmp_path),
            patch("pathlib.Path.cwd", return_value=tmp_path),
        ):
            result = detect()

        assert "cline" in result, (
            "cline must be detected when a Syntagma hook file exists in the Hooks directory"
        )

    def test_no_directory_no_detection(self, tmp_path):
        """Neither directory nor file → cline must not be in result."""
        detect = _detect_installed()
        with (
            patch("pathlib.Path.home", return_value=tmp_path),
            patch("pathlib.Path.cwd", return_value=tmp_path),
        ):
            result = detect()

        assert "cline" not in result


# ---------------------------------------------------------------------------
# Fix 3 — _merge_json_array upserts legacy flat schema to nested schema
# ---------------------------------------------------------------------------


class TestMergeJsonArrayUpsert:
    """_merge_json_array must replace a legacy flat hook entry with the nested schema."""

    def _inst(self):
        from syntagma.cli.install import SyntagmaInstaller

        return SyntagmaInstaller(registry_dir=Path("/tmp"), skills_dir=Path("/tmp"), dry_run=False)

    def test_flat_entry_is_replaced_with_nested(self, tmp_path):
        """A flat {matcher, type, command} entry must be replaced by {matcher, hooks:[...]}."""
        import json

        cfg = tmp_path / "settings.json"
        # Simulate existing flat (legacy) entry
        cfg.write_text(
            json.dumps(
                {
                    "hooks": {
                        "PreToolUse": [
                            {
                                "matcher": "Bash",
                                "type": "command",
                                "command": "syntagma-hook sniff --staged",
                                "timeout": 30,
                            }
                        ]
                    }
                }
            ),
            encoding="utf-8",
        )

        inst = self._inst()
        new_item = {
            "matcher": "Bash",
            "hooks": [
                {"type": "command", "command": "syntagma-hook sniff --staged", "timeout": 30}
            ],
        }
        inst._merge_json_array(cfg, ["hooks", "PreToolUse"], new_item)

        result = json.loads(cfg.read_text())
        entries = result["hooks"]["PreToolUse"]
        assert len(entries) == 1, "Must not duplicate — flat entry should be replaced"
        assert "hooks" in entries[0], "Replaced entry must use nested schema"
        assert "type" not in entries[0], "Replaced entry must not retain flat 'type' key"

    def test_already_nested_entry_is_not_duplicated(self, tmp_path):
        """An already-correct nested entry must not be duplicated."""
        import json

        cfg = tmp_path / "settings.json"
        existing = {
            "matcher": "Bash",
            "hooks": [
                {"type": "command", "command": "syntagma-hook sniff --staged", "timeout": 30}
            ],
        }
        cfg.write_text(json.dumps({"hooks": {"PreToolUse": [existing]}}), encoding="utf-8")

        inst = self._inst()
        inst._merge_json_array(cfg, ["hooks", "PreToolUse"], existing)

        result = json.loads(cfg.read_text())
        assert len(result["hooks"]["PreToolUse"]) == 1, (
            "Identical nested entry must not be duplicated"
        )

    def test_new_entry_is_appended(self, tmp_path):
        """A hook with a different command must be appended, not replace existing ones."""
        import json

        cfg = tmp_path / "settings.json"
        existing = {
            "matcher": "Bash",
            "hooks": [{"type": "command", "command": "other-hook", "timeout": 30}],
        }
        cfg.write_text(json.dumps({"hooks": {"PreToolUse": [existing]}}), encoding="utf-8")

        inst = self._inst()
        new_item = {
            "matcher": "Edit",
            "hooks": [
                {"type": "command", "command": "syntagma-hook sniff --staged", "timeout": 30}
            ],
        }
        inst._merge_json_array(cfg, ["hooks", "PreToolUse"], new_item)

        result = json.loads(cfg.read_text())
        assert len(result["hooks"]["PreToolUse"]) == 2, "Different command must be appended"

    def test_hook_command_helper_flat(self):
        """_hook_command extracts command from flat schema."""
        from syntagma.cli.install import SyntagmaInstaller

        flat = {"matcher": "Bash", "type": "command", "command": "syntagma-hook sniff --staged"}
        assert SyntagmaInstaller._hook_command(flat) == "syntagma-hook sniff --staged"

    def test_hook_command_helper_nested(self):
        """_hook_command extracts command from nested schema."""
        from syntagma.cli.install import SyntagmaInstaller

        nested = {
            "matcher": "Bash",
            "hooks": [{"type": "command", "command": "syntagma-hook sniff --staged"}],
        }
        assert SyntagmaInstaller._hook_command(nested) == "syntagma-hook sniff --staged"
