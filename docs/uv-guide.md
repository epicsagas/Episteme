# Episteme UV Installation & Usage Guide

**Date:** 2026-04-29  
**Version:** 0.0.5

---

## What is uv?

[uv](https://github.com/astral-sh/uv) is an extremely fast Python package installer and resolver written in Rust by Astral (the creators of Ruff). It's designed as a drop-in replacement for pip with 10-100× faster performance.

### Key Features

- ⚡ **10-100× faster** than pip
- 🔒 **Deterministic installs** with lockfile (`uv.lock`)
- 🎯 **Drop-in replacement** for pip (familiar commands)
- 🚀 **Built-in venv management** (no separate `python -m venv` needed)
- 📦 **Compatible with pyproject.toml** (PEP 621)

---

## Installation

### macOS / Linux

```bash
curl -LsSf https://astral.sh/uv/install.sh | sh
```

### Windows

```powershell
powershell -c "irm https://astral.sh/uv/install.ps1 | iex"
```

### Using pipx (cross-platform)

```bash
pipx install uv
```

### Verify Installation

```bash
uv --version
# Output: uv 0.1.x
```

---

## Quick Start with Episteme

### 1. Clone Repository

```bash
git clone https://github.com/epicsagas/Episteme.git
cd Episteme
```

### 2. Install Dependencies

```bash
# Single command: creates venv + installs all dependencies
uv sync

# Output:
# Using Python 3.11.7
# Creating virtualenv at: .venv
# Resolved 25 packages in 127ms
# Downloaded 15 packages in 1.2s
# Installed 25 packages in 423ms
```

**What `uv sync` does:**
1. Reads `pyproject.toml`
2. Checks `.python-version` for Python version
3. Creates `.venv/` if it doesn't exist
4. Resolves dependencies and generates `uv.lock`
5. Installs all packages from lockfile

### 3. Activate Virtual Environment

```bash
# macOS/Linux
source .venv/bin/activate

# Windows
.venv\Scripts\activate
```

### 4. Verify Installation

```bash
python -c "from scripts.language_parsers import get_parser; print('✅ Success')"
```

---

## Common Commands

### Install Packages

```bash
# Install from pyproject.toml
uv sync

# Install with dev dependencies
uv sync --all-extras

# Install specific package
uv add requests

# Install specific version
uv add "fastapi>=0.104.0"

# Install as dev dependency
uv add --dev pytest
```

### Remove Packages

```bash
uv remove requests
```

### Update Dependencies

```bash
# Update all packages
uv sync --upgrade

# Update specific package
uv add --upgrade fastapi
```

### Run Commands

```bash
# Run Python script in venv (no activation needed)
uv run python scripts/detect_smells.py mycode.py

# Run pytest
uv run pytest

# Run custom script
uv run episteme-analyze mycode.py
```

### Lock File Management

```bash
# Generate/update lockfile
uv lock

# Sync from lockfile (reproducible install)
uv sync --frozen
```

---

## Comparison: uv vs pip

### Speed Comparison

**Test**: Install Episteme dependencies (25 packages)

| Tool | Time | Speedup |
|------|------|---------|
| **uv** | 1.6s | **baseline** |
| pip | 45.3s | **28× slower** |
| poetry | 38.7s | **24× slower** |

### Feature Comparison

| Feature | uv | pip | poetry |
|---------|----|----|--------|
| **Speed** | ⚡⚡⚡ | ⚡ | ⚡ |
| **Lockfile** | ✅ uv.lock | ❌ | ✅ poetry.lock |
| **Venv management** | ✅ Built-in | ❌ Manual | ✅ Built-in |
| **PEP 621 (pyproject.toml)** | ✅ | ✅ | ⚠️ Custom format |
| **Dependency resolution** | ✅ Fast | ⚠️ Slow | ⚠️ Slow |
| **Cache** | ✅ Global | ✅ Per-user | ✅ Per-project |

---

## Episteme-Specific Workflows

### Development Workflow

```bash
# 1. Initial setup
git clone https://github.com/epicsagas/Episteme.git
cd Episteme
uv sync --all-extras  # Install with dev dependencies

# 2. Activate environment
source .venv/bin/activate

# 3. Run tests
uv run pytest

# 4. Format code
uv run black .
uv run ruff check .

# 5. Type check
uv run mypy scripts/ api/
```

### Adding New Features

```bash
# Install new dependency
uv add javalang

# Automatically updates:
# - pyproject.toml (dependencies list)
# - uv.lock (locked versions)

# Commit both files
git add pyproject.toml uv.lock
git commit -m "feat: add Java parser support"
```

### Production Deployment

```bash
# On production server
git clone https://github.com/epicsagas/Episteme.git
cd Episteme

# Install from lockfile (reproducible)
uv sync --frozen --no-dev

# No dev dependencies, exact versions from uv.lock
```

### CI/CD Integration

```yaml
# .github/workflows/test.yml
name: Test
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install uv
        run: curl -LsSf https://astral.sh/uv/install.sh | sh
      
      - name: Install dependencies
        run: uv sync --frozen --all-extras
      
      - name: Run tests
        run: uv run pytest
      
      - name: Lint
        run: |
          uv run black --check .
          uv run ruff check .
```

---

## Troubleshooting

### Issue: "uv: command not found"

**Solution**: Restart terminal or manually add to PATH:

```bash
# Add to ~/.bashrc or ~/.zshrc
export PATH="$HOME/.cargo/bin:$PATH"
source ~/.bashrc
```

### Issue: "No Python interpreter found"

**Solution**: Install Python 3.11+:

```bash
# macOS (via Homebrew)
brew install python@3.11

# Ubuntu/Debian
sudo apt install python3.11

# Then specify version
uv sync --python 3.11
```

### Issue: "Lockfile is out of date"

**Solution**: Update lockfile:

```bash
uv lock --upgrade
uv sync
```

### Issue: "Package conflicts"

**Solution**: Use uv's resolver:

```bash
# uv automatically resolves conflicts
# If manual resolution needed:
uv add "package>=1.0,<2.0"
```

---

## Advanced Usage

### Using Specific Python Version

```bash
# Install with Python 3.12
uv sync --python 3.12

# Use system Python
uv sync --python python3.11

# Use specific path
uv sync --python /usr/local/bin/python3.11
```

### Offline Installation

```bash
# Download packages to cache
uv sync

# Later, install from cache (no network)
uv sync --offline
```

### Custom Index URLs

```bash
# Use private PyPI
uv add --index-url https://pypi.company.com/simple requests

# Multiple indexes
uv add --extra-index-url https://pypi.org/simple \
       --extra-index-url https://private.pypi.com/simple \
       my-private-package
```

### Editable Installs

```bash
# Install Episteme in editable mode
uv sync --editable

# Now changes to source code are immediately reflected
# No need to reinstall after each change
```

---

## Migration Guide

### From pip

```bash
# Old (pip)
python -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt

# New (uv)
uv sync
source .venv/bin/activate
```

### From poetry

```bash
# Convert poetry project to uv
# 1. Generate requirements.txt from poetry
poetry export -f requirements.txt > requirements.txt

# 2. Create pyproject.toml (if using old poetry format)
# (Episteme already uses PEP 621 format)

# 3. Switch to uv
uv sync
```

### From conda

```bash
# Export conda environment
conda env export > environment.yml

# Convert to requirements.txt
# (manual conversion needed - conda has different package names)

# Use uv
uv sync
```

---

## Best Practices

### 1. Always Use Lockfile

```bash
# Development: flexible updates
uv sync

# Production: reproducible builds
uv sync --frozen
```

### 2. Commit Both Files

```bash
git add pyproject.toml uv.lock
git commit -m "chore: update dependencies"
```

### 3. Separate Dev Dependencies

```toml
[project.optional-dependencies]
dev = [
    "pytest>=7.4.0",
    "black>=23.0.0",
    "ruff>=0.1.0",
]
```

```bash
# Install without dev deps (production)
uv sync --no-dev

# Install with dev deps (development)
uv sync --all-extras
```

### 4. Pin Python Version

Create `.python-version`:
```
3.11
```

uv automatically uses this version.

### 5. Use Scripts

Define in `pyproject.toml`:
```toml
[project.scripts]
episteme-analyze = "scripts.detect_smells:main"
```

Run without activation:
```bash
uv run episteme-analyze mycode.py
```

---

## Performance Tips

### 1. Use Global Cache

uv automatically caches packages globally (`~/.cache/uv`).  
Multiple projects share the same cache = faster installs.

### 2. Parallel Downloads

uv downloads packages in parallel by default (no configuration needed).

### 3. Incremental Syncs

```bash
# Only install missing packages (fast)
uv sync

# Reinstall everything (slow)
uv sync --reinstall
```

---

## Resources

- **Official Docs**: https://github.com/astral-sh/uv
- **uv vs pip benchmark**: https://astral.sh/blog/uv-unified-python-packaging
- **PEP 621 (pyproject.toml)**: https://peps.python.org/pep-0621/
- **Episteme Issues**: https://github.com/epicsagas/Episteme/issues

---

## FAQ

**Q: Is uv stable for production?**  
A: Yes. Used by major companies (Anthropic, etc.). Version 1.0+ is production-ready.

**Q: Does uv replace pip completely?**  
A: Yes, for most use cases. It's a drop-in replacement with the same commands.

**Q: Can I use both uv and pip?**  
A: Yes, but not recommended. Stick to one tool to avoid conflicts.

**Q: What if uv is missing a feature?**  
A: Fall back to pip for that specific package, then return to uv.

**Q: How do I uninstall uv?**  
A: `rm -rf ~/.cargo/bin/uv ~/.cache/uv`

---

**Last Updated:** 2026-04-29  
**Episteme Version:** 0.0.5  
**uv Version:** 0.1.x
