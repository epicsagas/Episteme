"""Unit tests for eval_runner pure functions."""

import json
import tempfile
from pathlib import Path

from eval_runner import (
    _ndcg_at_k,
    _reciprocal_rank,
    check_regression,
    compute_composite,
    dedup,
    parse_json_output,
)


# ---------------------------------------------------------------------------
# _reciprocal_rank
# ---------------------------------------------------------------------------


class TestReciprocalRank:
    def test_hit_at_rank_1(self) -> None:
        assert _reciprocal_rank(["A", "B", "C"], {"A"}, 5) == 1.0

    def test_hit_at_rank_3(self) -> None:
        assert _reciprocal_rank(["X", "Y", "A"], {"A"}, 5) == 1.0 / 3

    def test_no_hit(self) -> None:
        assert _reciprocal_rank(["X", "Y", "Z"], {"A"}, 5) == 0.0

    def test_beyond_k(self) -> None:
        """Relevant item at position > k should not be found."""
        ids = ["X", "Y", "Z", "A"]
        assert _reciprocal_rank(ids, {"A"}, 3) == 0.0

    def test_empty_ids(self) -> None:
        assert _reciprocal_rank([], {"A"}, 5) == 0.0

    def test_empty_relevant(self) -> None:
        assert _reciprocal_rank(["A", "B"], set(), 5) == 0.0


# ---------------------------------------------------------------------------
# _ndcg_at_k
# ---------------------------------------------------------------------------


class TestNdcgAtK:
    def test_perfect_ranking(self) -> None:
        """Single relevant doc at rank 1 should give NDCG=1.0."""
        result = _ndcg_at_k(["A", "B", "C"], {"A"}, 5)
        assert abs(result - 1.0) < 1e-6

    def test_no_relevant(self) -> None:
        assert _ndcg_at_k(["X", "Y"], {"A"}, 5) == 0.0

    def test_empty_ids(self) -> None:
        assert _ndcg_at_k([], {"A"}, 5) == 0.0

    def test_empty_relevant(self) -> None:
        assert _ndcg_at_k(["A", "B"], set(), 5) == 0.0

    def test_multiple_relevant(self) -> None:
        """Two relevant docs at ranks 1 and 2."""
        result = _ndcg_at_k(["A", "B", "X"], {"A", "B"}, 5)
        assert abs(result - 1.0) < 1e-6

    def test_suboptimal_order(self) -> None:
        """Relevant doc at rank 3 gives lower NDCG than rank 1."""
        low = _ndcg_at_k(["X", "Y", "A"], {"A"}, 5)
        high = _ndcg_at_k(["A", "X", "Y"], {"A"}, 5)
        assert low < high


# ---------------------------------------------------------------------------
# compute_composite
# ---------------------------------------------------------------------------


class TestComputeComposite:
    def test_all_perfect(self) -> None:
        sp = {"status": "ok", "metrics": {"hit@5": 1.0}}
        sn = {"status": "ok", "metrics": {"fp@5": 0.0}}
        smn = {"status": "ok", "metrics": {"specificity": 1.0}}
        result = compute_composite(sp, sn, smn)
        assert result["composite"] == 1.0
        assert result["recall"] == 1.0
        assert result["precision"] == 1.0
        assert result["specificity"] == 1.0

    def test_formula_weights(self) -> None:
        """0.4 * 0.5 + 0.3 * 0.8 + 0.3 * 0.6 = 0.2 + 0.24 + 0.18 = 0.62"""
        sp = {"status": "ok", "metrics": {"hit@5": 0.5}}
        sn = {"status": "ok", "metrics": {"fp@5": 0.2}}
        smn = {"status": "ok", "metrics": {"specificity": 0.6}}
        result = compute_composite(sp, sn, smn)
        assert abs(result["composite"] - 0.62) < 1e-6

    def test_skipped_suite_gives_zero(self) -> None:
        sp = {"status": "skipped"}
        sn = {"status": "ok", "metrics": {"fp@5": 0.5}}
        smn = {"status": "ok", "metrics": {"specificity": 0.8}}
        result = compute_composite(sp, sn, smn)
        assert result["recall"] == 0.0
        assert abs(result["precision"] - 0.5) < 1e-6
        assert abs(result["specificity"] - 0.8) < 1e-6


# ---------------------------------------------------------------------------
# check_regression
# ---------------------------------------------------------------------------


class TestCheckRegression:
    def test_no_previous_file(self) -> None:
        result = check_regression({"composite": 0.5}, Path("/nonexistent/file.json"))
        assert result["status"] == "no_previous"

    def test_no_regression(self) -> None:
        prev = {
            "composite_score": {"composite": 0.5, "recall": 0.6, "precision": 0.7, "specificity": 0.8},
            "suites": {"search_negative": {"status": "ok", "per_query": []}},
        }
        with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as f:
            f.write(json.dumps(prev))
            f.flush()
            result = check_regression(
                {"composite": 0.55, "recall": 0.65, "precision": 0.75, "specificity": 0.85},
                Path(f.name),
            )
        assert result["status"] == "PASS"
        assert result["delta"] > 0

    def test_composite_regression(self) -> None:
        prev = {
            "composite_score": {"composite": 0.5, "recall": 0.6, "precision": 0.7, "specificity": 0.8},
            "suites": {},
        }
        with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as f:
            f.write(json.dumps(prev))
            f.flush()
            result = check_regression(
                {"composite": 0.45, "recall": 0.6, "precision": 0.7, "specificity": 0.8},
                Path(f.name),
            )
        assert result["status"] == "FAIL"
        assert any("composite" in r for r in result["regressions"])

    def test_single_metric_regression(self) -> None:
        prev = {
            "composite_score": {"composite": 0.5, "recall": 0.8, "precision": 0.7, "specificity": 0.8},
            "suites": {},
        }
        with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as f:
            f.write(json.dumps(prev))
            f.flush()
            result = check_regression(
                {"composite": 0.5, "recall": 0.7, "precision": 0.7, "specificity": 0.8},
                Path(f.name),
            )
        assert result["status"] == "FAIL"
        assert any("recall" in r for r in result["regressions"])


# ---------------------------------------------------------------------------
# parse_json_output
# ---------------------------------------------------------------------------


class TestParseJsonOutput:
    def test_pure_json(self) -> None:
        data = {"smells": [{"smell_id": "SMELL-01"}]}
        assert parse_json_output(json.dumps(data)) == data

    def test_json_with_prefix(self) -> None:
        data = {"smells": []}
        output = "Analyzing file...\nSome log line\n" + json.dumps(data)
        assert parse_json_output(output) == data

    def test_json_with_suffix(self) -> None:
        data = {"smells": [{"smell_id": "SMELL-01"}]}
        output = json.dumps(data) + "\nDone."
        assert parse_json_output(output) == data

    def test_empty_output(self) -> None:
        assert parse_json_output("") is None

    def test_no_json(self) -> None:
        assert parse_json_output("just plain text\nno json here") is None

    def test_nested_json(self) -> None:
        data = {"outer": {"inner": [1, 2, 3]}}
        output = "prefix\n" + json.dumps(data)
        assert parse_json_output(output) == data


# ---------------------------------------------------------------------------
# dedup
# ---------------------------------------------------------------------------


class TestDedup:
    def test_preserves_order(self) -> None:
        assert dedup(["C", "A", "B", "A", "C"]) == ["C", "A", "B"]

    def test_empty(self) -> None:
        assert dedup([]) == []

    def test_no_dupes(self) -> None:
        assert dedup(["A", "B", "C"]) == ["A", "B", "C"]
