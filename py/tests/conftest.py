"""
Pytest configuration and shared fixtures for Syntagma tests
"""

import sys
from pathlib import Path
import pytest

# Add src to path for imports
project_root = Path(__file__).parent.parent
sys.path.insert(0, str(project_root / "src"))


@pytest.fixture
def project_root():
    """Return the project root directory"""
    return Path(__file__).parent.parent


@pytest.fixture
def test_data_dir(project_root):
    """Return the test data directory"""
    return project_root / "tests" / "data"


@pytest.fixture
def meta_dir(project_root):
    """Return the meta directory"""
    return project_root / "meta"


@pytest.fixture
def raw_dir(project_root):
    """Return the raw data directory"""
    return project_root / "raw"
