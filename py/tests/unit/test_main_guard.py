"""Tests for the None-command guard in __main__.py.

Verifies that:
1. Running `syntagma` with no subcommand exits 0 and prints help.
2. Telemetry functions are NOT called when no subcommand is given.
"""

import os
import subprocess
import sys
from pathlib import Path
from unittest.mock import MagicMock, patch

# Absolute path to the src/ directory so subprocess tests can locate the package.
_SRC_DIR = str(Path(__file__).parent.parent.parent / "src")


def _run_main_with_args(argv):
    """Run main() with a patched sys.argv and capture SystemExit."""
    with patch("sys.argv", ["syntagma"] + argv):
        from syntagma.__main__ import main

        try:
            main()
        except SystemExit as exc:
            return exc.code
    return None


# ---------------------------------------------------------------------------
# Test 1: no subcommand → exits 0 and prints help
# ---------------------------------------------------------------------------


def test_no_subcommand_exits_zero(capsys):
    """syntagma with no args must exit 0 (not fall through to error paths)."""
    with patch("sys.argv", ["syntagma"]):
        from syntagma.__main__ import main

        try:
            main()
        except SystemExit as exc:
            exit_code = exc.code
        else:
            exit_code = None

    assert exit_code == 0, f"Expected exit code 0, got {exit_code!r}"


def test_no_subcommand_prints_help(capsys):
    """syntagma with no args must print help text to stdout."""
    with patch("sys.argv", ["syntagma"]):
        from syntagma.__main__ import main

        try:
            main()
        except SystemExit:
            pass

    captured = capsys.readouterr()
    # argparse prints help to stdout
    assert "syntagma" in captured.out.lower() or "usage" in captured.out.lower(), (
        f"Expected help text in stdout, got: {captured.out!r}"
    )


# ---------------------------------------------------------------------------
# Test 2: telemetry must NOT be called when no subcommand is given
# ---------------------------------------------------------------------------


def test_no_subcommand_does_not_call_telemetry():
    """ensure_consent_or_set_default and track_session_started must not fire
    when the user runs `syntagma` with no subcommand."""
    mock_ensure = MagicMock()
    mock_track_session = MagicMock()

    with patch("sys.argv", ["syntagma"]):
        with patch.multiple(
            "syntagma.telemetry",
            ensure_consent_or_set_default=mock_ensure,
            track_session_started=mock_track_session,
        ):
            from syntagma.__main__ import main

            try:
                main()
            except SystemExit:
                pass

    mock_ensure.assert_not_called()
    mock_track_session.assert_not_called()


# ---------------------------------------------------------------------------
# Test 3: subprocess smoke-test (integration, no import caching issues)
# ---------------------------------------------------------------------------


def _subprocess_env():
    """Build an env dict that makes the syntagma package importable."""
    env = os.environ.copy()
    existing = env.get("PYTHONPATH", "")
    env["PYTHONPATH"] = f"{_SRC_DIR}:{existing}" if existing else _SRC_DIR
    return env


def test_subprocess_no_subcommand_exit_zero():
    """Black-box: running `python -m syntagma` with no args exits 0."""
    result = subprocess.run(
        [sys.executable, "-m", "syntagma"],
        capture_output=True,
        text=True,
        env=_subprocess_env(),
    )
    assert result.returncode == 0, (
        f"Expected returncode 0, got {result.returncode}.\n"
        f"stdout: {result.stdout!r}\nstderr: {result.stderr!r}"
    )


def test_subprocess_no_subcommand_shows_help():
    """Black-box: running `python -m syntagma` with no args prints help."""
    result = subprocess.run(
        [sys.executable, "-m", "syntagma"],
        capture_output=True,
        text=True,
        env=_subprocess_env(),
    )
    combined = result.stdout + result.stderr
    assert "syntagma" in combined.lower() or "usage" in combined.lower(), (
        f"Expected help text, got: {combined!r}"
    )
