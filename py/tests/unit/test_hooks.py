"""
Tests for exit code behavior in syntagma.cli.hooks.
"""

import argparse
from pathlib import Path
from unittest.mock import MagicMock, patch

import pytest

from syntagma.cli.analyze import CodeMetrics, SmellDetection
from syntagma.cli.hooks import handle_audit, handle_ground, handle_sniff, main


def _make_detection(
    smell_id="CS-001",
    smell_name="God Class",
    location="f.py",
    function_name="<module>",
    confidence=0.9,
    reasons=None,
):
    metrics = CodeMetrics(
        loc=100,
        cyclomatic_complexity=10,
        nesting_depth=3,
        parameter_count=2,
        local_variables=5,
        return_statements=1,
    )
    return SmellDetection(
        smell_id=smell_id,
        smell_name=smell_name,
        confidence=confidence,
        location=location,
        function_name=function_name,
        metrics=metrics,
        reasons=reasons or ["test reason"],
    )


# ---------------------------------------------------------------------------
# handle_ground
# ---------------------------------------------------------------------------


def _make_ground_args(**kwargs):
    defaults = {"prompt": "test prompt", "limit": 3, "json": False}
    defaults.update(kwargs)
    return argparse.Namespace(**defaults)


def test_handle_ground_returns_0_on_success():
    mock_rag = MagicMock()
    mock_rag.search.return_value = []
    with patch("syntagma.cli.hooks.SyntagmaRAG", return_value=mock_rag):
        result = handle_ground(_make_ground_args())
    assert result == 0


def test_handle_ground_returns_1_on_missing_prompt(monkeypatch):
    # Simulate no prompt and TTY (stdin.isatty() == True)
    monkeypatch.setattr("sys.stdin", MagicMock(isatty=lambda: True, read=lambda: ""))
    result = handle_ground(_make_ground_args(prompt=None))
    assert result == 1


# ---------------------------------------------------------------------------
# handle_sniff
# ---------------------------------------------------------------------------


def _make_sniff_args(**kwargs):
    defaults = {
        "staged": False,
        "files": [],
        "min_confidence": 0.6,
        "json": False,
        "verbose": False,
    }
    defaults.update(kwargs)
    return argparse.Namespace(**defaults)


def test_handle_sniff_returns_0_when_clean(tmp_path):
    f = tmp_path / "clean.py"
    f.write_text("x = 1\n")
    with patch("syntagma.cli.hooks.analyze_path", return_value=[]):
        result = handle_sniff(_make_sniff_args(files=[str(f)]))
    assert result == 0


def test_handle_sniff_returns_1_when_smells_found(tmp_path):
    f = tmp_path / "smelly.py"
    f.write_text("x = 1\n")
    detection = _make_detection(location=str(f))
    with patch("syntagma.cli.hooks.analyze_path", return_value=[detection]):
        result = handle_sniff(_make_sniff_args(files=[str(f)]))
    assert result == 1


def test_handle_sniff_returns_1_on_no_files_no_staged():
    result = handle_sniff(_make_sniff_args(staged=False, files=[]))
    assert result == 1


def test_handle_sniff_returns_0_when_staged_but_nothing_staged():
    with patch("syntagma.cli.hooks.SyntagmaRAG", return_value=MagicMock()):
        with patch("syntagma.cli.hooks._get_staged_files", return_value=[]):
            result = handle_sniff(_make_sniff_args(staged=True, files=[]))
    assert result == 0


# ---------------------------------------------------------------------------
# handle_audit
# ---------------------------------------------------------------------------


def test_handle_audit_returns_0_when_clean():
    with patch("syntagma.cli.hooks.analyze_path", return_value=[]):
        result = handle_audit(argparse.Namespace(json=False))
    assert result == 0


def test_handle_audit_returns_1_when_issues_found():
    detection = _make_detection(
        smell_id="CS-002",
        smell_name="Long Method",
        location="some/file.py",
        function_name="do_something",
        confidence=0.8,
        reasons=["too long"],
    )
    with patch("syntagma.cli.hooks.analyze_path", return_value=[detection]):
        result = handle_audit(argparse.Namespace(json=False))
    assert result == 1


def test_handle_audit_returns_int():
    with patch("syntagma.cli.hooks.analyze_path", return_value=[]):
        result = handle_audit(argparse.Namespace(json=False))
    assert isinstance(result, int)


# ---------------------------------------------------------------------------
# main() propagates return value
# ---------------------------------------------------------------------------


def test_main_returns_int_from_handler():
    with patch("syntagma.cli.hooks.analyze_path", return_value=[]):
        result = main(["audit"])
    assert isinstance(result, int)
