.PHONY: check test lint typecheck fix build dist-data dist clean help

VERSION := $(shell python3 -c "import tomllib; print(tomllib.load(open('pyproject.toml','rb'))['project']['version'])")

check:
	@bash -c '\
	  tmp=$$(mktemp -d); \
	  mypy src/ > "$$tmp/mypy.log" 2>&1 & pid_mypy=$$!; \
	  ruff check src/ > "$$tmp/ruff.log" 2>&1 & pid_ruff=$$!; \
	  python -m pytest tests/ -q > "$$tmp/pytest.log" 2>&1 & pid_pytest=$$!; \
	  fail=0; \
	  for job in mypy ruff pytest; do \
	    eval "wait \$$pid_$$job"; code=$$?; \
	    echo ""; \
	    echo "# === $$job $$([ $$code -eq 0 ] && echo ✅ || echo ❌) ==="; \
	    cat "$$tmp/$$job.log"; \
	    [ $$code -ne 0 ] && fail=1; \
	  done; \
	  rm -rf "$$tmp"; \
	  exit $$fail'

test:
	python -m pytest tests/ -v

lint:
	ruff check src/

typecheck:
	mypy src/

fix:
	ruff check --fix src/
	ruff format src/

# ── Distribution ──────────────────────────────────────────────────────────────

dist-data:
	@echo "Copying syntagma.db into meta/..."
	@cp ~/.syntagma/db/syntagma.db meta/syntagma.db
	@echo "Packaging syntagma-data-$(VERSION).tar.gz..."
	@tar -czf dist/syntagma-data-$(VERSION).tar.gz \
	  --exclude='meta/__pycache__' \
	  meta/ raw/
	@echo "Created: dist/syntagma-data-$(VERSION).tar.gz"
	@rm -f meta/syntagma.db

build:
	@mkdir -p dist
	syntagma install --local
	syntagma build
	uv build
	$(MAKE) dist-data

clean:
	rm -rf dist/ build/ src/*.egg-info src/syntagma/_keys.py

help:
	@echo "Usage: make <target>"
	@echo ""
	@echo "Development:"
	@echo "  check       Run mypy + ruff + pytest in parallel"
	@echo "  test        Run pytest"
	@echo "  lint        Run ruff"
	@echo "  typecheck   Run mypy"
	@echo "  fix         Auto-fix ruff + format"
	@echo ""
	@echo "Distribution:"
	@echo "  build       Build wheel + sdist + data archive"
	@echo "  dist-data   Package syntagma-data-$(VERSION).tar.gz only"
	@echo "  clean       Remove dist/, build/, egg-info"
