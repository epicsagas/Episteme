# Contributing to Syntagma

Thank you for your interest in contributing to Syntagma! This document provides guidelines for contributing to the project.

---

## 🚀 Quick Start for Contributors

1. **Fork the repository**
   ```bash
   git clone https://github.com/YOUR_USERNAME/Syntagma.git
   cd Syntagma
   ```

2. **Set up development environment**
   ```bash
   # Install uv (recommended)
   curl -LsSf https://astral.sh/uv/install.sh | sh
   
   # Install dependencies
   uv sync
   source .venv/bin/activate
   ```

3. **Create a feature branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

4. **Make your changes and test**
   ```bash
   # Run tests
   python -m pytest tests/
   
   # Test parsers
   python -c "from syntagma.parsers import get_parser; parser = get_parser('python'); print('✅ Parser works')"
   ```

5. **Commit with conventional commits**
   ```bash
   git add .
   git commit -m "feat(scope): add your feature description"
   ```

6. **Push and create pull request**
   ```bash
   git push origin feature/your-feature-name
   # Then create PR on GitHub
   ```

---

## 📋 Contribution Areas

### 1. Add New Language Parser

**Location:** `src/syntagma/parsers/`

**Steps:**
1. Create new parser file (e.g., `scala_parser.py`)
2. Extend `LanguageParser` base class
3. Implement required methods:
   - `parse_file(file_path)` - Parse a file
   - `parse_code(code, file_name)` - Parse code string
   - `get_supported_extensions()` - Return file extensions
4. Add parser to `__init__.py` factory
5. Add test cases in `tests/unit/`

**Example:**
```python
from syntagma.parsers.base import LanguageParser, SmellDetection, CodeMetrics

class ScalaParser(LanguageParser):
    def get_supported_extensions(self):
        return ['.scala']
    
    def parse_file(self, file_path):
        # Implementation
        pass
    
    def parse_code(self, code, file_name="temp.scala"):
        # Implementation
        pass
```

### 2. Add New Code Smell Detector

**Location:** `src/syntagma/parsers/base.py`

**Steps:**
1. Add smell type to `SmellType` enum
2. Add detection method to `LanguageParser` base class
3. Define thresholds and confidence scoring
4. Update `meta/code_smells.json` with smell definition

**Example:**
```python
def detect_circular_dependency(self, metrics, location, name):
    """Detect circular dependency smell"""
    reasons = []
    confidence = 0.0
    
    # Detection logic
    if metrics.dependency_cycles > 0:
        reasons.append(f"Found {metrics.dependency_cycles} circular dependencies")
        confidence = 0.8
    
    if confidence >= 0.5:
        return SmellDetection(
            smell_id="SMELL-XX",
            smell_name="Circular Dependency",
            confidence=confidence,
            location=location,
            function_name=name,
            metrics=metrics,
            reasons=reasons
        )
    return None
```

### 3. Improve Documentation

**Locations:**
- `README.md` - User-facing documentation
- `DEVELOPMENT.md` - Development roadmap
- `docs/api.md` - API reference
- `docs/*.md` - Specific guides

**What to improve:**
- Fix typos and grammar
- Add code examples
- Clarify unclear sections
- Add diagrams
- Update outdated information

### 4. Add Test Cases

**Location:** `tests/`

**Types of tests needed:**
- Unit tests for parsers
- Integration tests for API
- Performance benchmarks
- Edge case coverage

### 5. Enhance Knowledge Base

**Location:** `raw/`

**What to add:**
- New design patterns
- Additional refactoring techniques
- Software engineering laws
- Real-world examples

---

## 🔍 Code Review Guidelines

### What We Look For

✅ **Code Quality**
- Follows existing code style
- Includes docstrings
- Uses type hints where appropriate
- No hardcoded values

✅ **Testing**
- New features include tests
- Tests pass locally
- Edge cases covered
- Test names are descriptive

✅ **Documentation**
- README updated if needed
- API changes documented
- Code comments for complex logic
- Examples provided

✅ **Performance**
- No performance regressions
- Efficient algorithms used
- Large files handled properly
- Memory usage considered

---

## 📝 Commit Message Guidelines

We use [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types:**
- `feat` - New feature
- `fix` - Bug fix
- `docs` - Documentation changes
- `style` - Code style changes (formatting)
- `refactor` - Code refactoring
- `perf` - Performance improvements
- `test` - Test additions/changes
- `chore` - Build/tooling changes

**Examples:**
```bash
feat(parsers): add scala language support
fix(api): correct cache invalidation logic
docs(readme): update installation instructions
perf(gpu): optimize batch size selection
```

---

## 🐛 Bug Reports

When reporting bugs, please include:

1. **Description** - Clear description of the issue
2. **Steps to Reproduce** - Minimal steps to reproduce
3. **Expected Behavior** - What should happen
4. **Actual Behavior** - What actually happens
5. **Environment** - Python version, OS, etc.
6. **Code Sample** - Minimal code that triggers the bug

**Template:**
```markdown
## Bug Description
Brief description of the bug

## Steps to Reproduce
1. Step 1
2. Step 2
3. Step 3

## Expected Behavior
What should happen

## Actual Behavior
What actually happens

## Environment
- Python version: 3.11.5
- OS: macOS 14.2
- Syntagma version: 0.0.5

## Code Sample
```python
# Minimal code to reproduce
```
```

---

## 💡 Feature Requests

When requesting features, please include:

1. **Use Case** - Why is this feature needed?
2. **Proposed Solution** - How should it work?
3. **Alternatives** - What alternatives did you consider?
4. **Additional Context** - Screenshots, examples, etc.

---

## 🧪 Testing

### Run All Tests
```bash
# All tests
python -m pytest tests/ -v

# Unit tests only
python -m pytest tests/unit/ -v

# Integration tests
python -m pytest tests/integration/ -v
```

### Run Specific Tests
```bash
# Test specific module
python -m pytest tests/unit/test_parsers.py -v

# Test with coverage
python -m pytest tests/ --cov=syntagma --cov-report=html
```

---

## 📦 Dependencies

### Adding New Dependencies

1. **Update pyproject.toml**
   ```toml
   dependencies = [
       "existing-package>=1.0.0",
       "new-package>=2.0.0",  # Add here
   ]
   ```

2. **Install and test**
   ```bash
   uv sync
   # Test that everything still works
   ```

3. **Document why** - Explain in PR why the dependency is needed

---

## 🔐 Security

### Reporting Security Issues

**DO NOT** open public issues for security vulnerabilities.

Instead, email: epicsagas@research.org

Include:
- Description of vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

---

## 📄 License

By contributing to Syntagma, you agree that your contributions will be licensed under the Apache-2 License.

---

## 🙏 Thank You!

Your contributions help make Syntagma better for everyone. We appreciate your time and effort!

For more details, see:
- [Development Guide](DEVELOPMENT.md) - Detailed development roadmap
- [README](README.md) - Project overview
- [API Documentation](docs/api.md) - API reference

---

*Last Updated: 2026-04-29*
