#!/usr/bin/env python3
"""
Test script for the 9 new code smell detectors
"""

from syntagma.parsers.base import LanguageParser, CodeMetrics, SmellType


class TestParser(LanguageParser):
    """Concrete test implementation of LanguageParser"""

    def parse_file(self, file_path: str):
        pass

    def parse_code(self, code: str, file_name: str = "temp"):
        pass

    def get_supported_extensions(self):
        return [".test"]


def test_duplicate_code():
    """Test SMELL-03: Duplicate Code"""
    parser = TestParser()
    metrics = CodeMetrics(
        loc=20,
        cyclomatic_complexity=5,
        nesting_depth=2,
        parameter_count=3,
        local_variables=2,
        return_statements=1,
        ast_hash="abc123",
    )

    # Test with duplicate hash
    all_hashes = {"abc123": ["file1.py:10", "file2.py:20", "file3.py:30"]}

    result = parser.detect_duplicate_code(metrics, "file1.py:10", "test_func", all_hashes)
    assert result is not None, "Should detect duplicate code"
    assert result.smell_id == "SMELL-13"
    assert result.confidence >= 0.7
    print(f"✓ Duplicate Code: {result.confidence:.2f} - {result.reasons}")


def test_god_object():
    """Test SMELL-21: God Object"""
    parser = TestParser()
    metrics = CodeMetrics(
        loc=600,
        cyclomatic_complexity=60,
        nesting_depth=3,
        parameter_count=3,
        local_variables=5,
        return_statements=2,
        method_count=35,
        field_count=25,
    )

    result = parser.detect_god_object(metrics, "file.py:1", "GodClass")
    assert result is not None, "Should detect God Object"
    assert result.smell_id == "SMELL-21"
    assert result.confidence >= 0.6
    print(f"✓ God Object: {result.confidence:.2f} - {result.reasons}")


def test_switch_statements():
    """Test SMELL-06: Switch Statements"""
    parser = TestParser()
    metrics = CodeMetrics(
        loc=50,
        cyclomatic_complexity=12,
        nesting_depth=2,
        parameter_count=2,
        local_variables=3,
        return_statements=1,
        branch_count=8,
    )

    result = parser.detect_switch_statements(metrics, "file.py:10", "process_type")
    assert result is not None, "Should detect excessive branching"
    assert result.smell_id == "SMELL-06"
    assert result.confidence >= 0.6
    print(f"✓ Switch Statements: {result.confidence:.2f} - {result.reasons}")


def test_primitive_obsession():
    """Test SMELL-03: Primitive Obsession"""
    parser = TestParser()
    metrics = CodeMetrics(
        loc=30,
        cyclomatic_complexity=5,
        nesting_depth=2,
        parameter_count=6,
        local_variables=2,
        return_statements=1,
        primitive_params=5,
    )

    result = parser.detect_primitive_obsession(metrics, "file.py:15", "create_user")
    assert result is not None, "Should detect primitive obsession"
    assert result.smell_id == "SMELL-03"
    assert result.confidence >= 0.55
    print(f"✓ Primitive Obsession: {result.confidence:.2f} - {result.reasons}")


def test_shotgun_surgery():
    """Test SMELL-09: Shotgun Surgery"""
    parser = TestParser()
    metrics = CodeMetrics(
        loc=20,
        cyclomatic_complexity=3,
        nesting_depth=1,
        parameter_count=2,
        local_variables=1,
        return_statements=1,
    )

    result = parser.detect_shotgun_surgery(metrics, "file.py:5", "shared_util", dependency_count=12)
    assert result is not None, "Should detect shotgun surgery"
    assert result.smell_id == "SMELL-09"
    assert result.confidence >= 0.65
    print(f"✓ Shotgun Surgery: {result.confidence:.2f} - {result.reasons}")


def test_divergent_change():
    """Test SMELL-10: Divergent Change"""
    parser = TestParser()
    metrics = CodeMetrics(
        loc=250,
        cyclomatic_complexity=28,
        nesting_depth=3,
        parameter_count=3,
        local_variables=5,
        return_statements=2,
        method_count=18,
        field_count=12,
    )

    result = parser.detect_divergent_change(metrics, "file.py:1", "MultiPurposeClass")
    assert result is not None, "Should detect divergent change"
    assert result.smell_id == "SMELL-10"
    assert result.confidence >= 0.55
    print(f"✓ Divergent Change: {result.confidence:.2f} - {result.reasons}")


def test_speculative_generality():
    """Test SMELL-12: Speculative Generality"""
    parser = TestParser()
    metrics = CodeMetrics(
        loc=50,
        cyclomatic_complexity=5,
        nesting_depth=2,
        parameter_count=2,
        local_variables=2,
        return_statements=1,
        method_count=5,
    )

    # Abstract class with only one implementation
    result = parser.detect_speculative_generality(
        metrics, "file.py:1", "AbstractBase", subclass_count=1, usage_count=1
    )
    assert result is not None, "Should detect speculative generality"
    assert result.smell_id == "SMELL-12"
    assert result.confidence >= 0.6
    print(f"✓ Speculative Generality: {result.confidence:.2f} - {result.reasons}")


def test_message_chains():
    """Test SMELL-20: Message Chains"""
    parser = TestParser()
    metrics = CodeMetrics(
        loc=15,
        cyclomatic_complexity=3,
        nesting_depth=1,
        parameter_count=2,
        local_variables=3,
        return_statements=1,
        method_call_chains=6,
    )

    result = parser.detect_message_chains(metrics, "file.py:20", "get_data")
    assert result is not None, "Should detect message chains"
    assert result.smell_id == "SMELL-20"
    assert result.confidence >= 0.6
    print(f"✓ Message Chains: {result.confidence:.2f} - {result.reasons}")


def test_middle_man():
    """Test SMELL-14: Middle Man"""
    parser = TestParser()
    metrics = CodeMetrics(
        loc=40,
        cyclomatic_complexity=4,
        nesting_depth=1,
        parameter_count=2,
        local_variables=1,
        return_statements=1,
        method_count=10,
        delegation_methods=9,
    )

    result = parser.detect_middle_man(metrics, "file.py:1", "WrapperClass")
    assert result is not None, "Should detect middle man"
    assert result.smell_id == "SMELL-14"
    assert result.confidence >= 0.7
    print(f"✓ Middle Man: {result.confidence:.2f} - {result.reasons}")


def test_no_false_positives():
    """Test that clean code doesn't trigger detectors"""
    parser = TestParser()

    # Clean function metrics
    clean_metrics = CodeMetrics(
        loc=15,
        cyclomatic_complexity=3,
        nesting_depth=1,
        parameter_count=2,
        local_variables=2,
        return_statements=1,
        method_count=5,
        field_count=3,
        branch_count=2,
        primitive_params=1,
        method_call_chains=1,
        delegation_methods=0,
    )

    assert parser.detect_god_object(clean_metrics, "file.py:1", "clean") is None
    assert parser.detect_switch_statements(clean_metrics, "file.py:1", "clean") is None
    assert parser.detect_primitive_obsession(clean_metrics, "file.py:1", "clean") is None
    assert parser.detect_divergent_change(clean_metrics, "file.py:1", "clean") is None
    assert parser.detect_message_chains(clean_metrics, "file.py:1", "clean") is None
    assert parser.detect_middle_man(clean_metrics, "file.py:1", "clean") is None

    print("✓ No false positives for clean code")


if __name__ == "__main__":
    print("Testing 9 new code smell detectors...\n")

    test_duplicate_code()
    test_god_object()
    test_switch_statements()
    test_primitive_obsession()
    test_shotgun_surgery()
    test_divergent_change()
    test_speculative_generality()
    test_message_chains()
    test_middle_man()
    test_no_false_positives()

    print("\n✅ All tests passed!")
