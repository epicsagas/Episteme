"""
Tests for file handle leak fix and launchd_stop warning in service.py.
"""
from __future__ import annotations

import subprocess
from pathlib import Path
from unittest.mock import MagicMock, call, patch

import pytest


# ---------------------------------------------------------------------------
# Test 1: file handles are closed after Popen in cmd_start()
# ---------------------------------------------------------------------------


def test_cmd_start_closes_file_handles(tmp_path):
    """File handles opened for stdout/stderr must be closed after Popen returns."""
    log_out = tmp_path / "mcp.out.log"
    log_err = tmp_path / "mcp.err.log"

    opened_files: list[MagicMock] = []

    real_open = open

    def tracking_open(path, mode="r", **kwargs):
        fh = real_open(path, mode, **kwargs)
        mock_fh = MagicMock(wraps=fh)
        # keep a reference so we can inspect close() calls
        opened_files.append(mock_fh)
        return mock_fh

    fake_proc = MagicMock()
    fake_proc.pid = 12345

    with (
        patch("syntagma.cli.service._find_pid_by_port", side_effect=[None, 12345]),
        patch("syntagma.cli.service._launchd_registered", return_value=False),
        patch("syntagma.cli.service._mcp_executable", return_value="/fake/syntagma-mcp"),
        patch("syntagma.cli.service._resolve_dyld_lib_path", return_value=""),
        patch("syntagma.cli.service._LOG_DIR", tmp_path),
        patch("syntagma.cli.service._wait_port_open", return_value=True),
        patch("syntagma.cli.service._write_pid"),
        patch("subprocess.Popen", return_value=fake_proc) as mock_popen,
        patch("builtins.open", side_effect=tracking_open),
    ):
        from syntagma.cli import service

        result = service.cmd_start()

    assert result == 0, "cmd_start should return 0 on success"
    assert len(opened_files) >= 2, "Expected at least 2 file handles to be opened"

    for fh in opened_files:
        assert fh.close.called, (
            f"File handle {fh} was not closed — file handle leak detected"
        )


# ---------------------------------------------------------------------------
# Test 2: warning is printed when _launchd_stop() returns False in cmd_stop()
# ---------------------------------------------------------------------------


def test_cmd_stop_warns_when_launchd_stop_fails(capsys):
    """When _launchd_stop() returns False, a warning must be printed to stderr."""
    with (
        patch("syntagma.cli.service._find_pid_by_port", return_value=99999),
        patch("syntagma.cli.service._launchd_registered", return_value=True),
        patch("syntagma.cli.service._launchd_stop", return_value=False),
        patch("syntagma.cli.service._is_running", return_value=False),
        patch("syntagma.cli.service._wait_port_free", return_value=True),
        patch("syntagma.cli.service._clear_pid"),
        patch("os.kill"),
    ):
        from syntagma.cli import service

        result = service.cmd_stop()

    captured = capsys.readouterr()
    assert "warn" in captured.err.lower() or "failed" in captured.err.lower(), (
        "Expected a warning message on stderr when _launchd_stop() returns False"
    )
