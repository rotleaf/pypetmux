SHELL := /usr/bin/env bash

PACKAGE := pypetmux
PYTHON ?= python3
PIP ?= $(PYTHON) -m pip
MATURIN ?= maturin

DIST_DIR := dist
SITE_DIR := site

.PHONY: help stubgen build install rebuild docs-serve docs-build clean uninstall deps docs-deps

help:
	@echo "Targets:"
	@echo "  make stubgen     - generate stubs into python/"
	@echo "  make build       - generate stubs, build wheel, install it"
	@echo "  make install     - install newest wheel from dist/"
	@echo "  make rebuild     - clean, then build"
	@echo "  make docs-serve  - build/install package, then run mkdocs serve"
	@echo "  make docs-build  - build/install package, then build docs"
	@echo "  make deps        - install build dependencies"
	@echo "  make docs-deps   - install docs dependencies"
	@echo "  make uninstall   - uninstall package"
	@echo "  make clean       - remove build artifacts"

deps:
	$(PIP) install --upgrade pip
	$(PIP) install maturin

docs-deps:
	$(PIP) install --upgrade pip
	$(PIP) install -r requirements-docs.txt
	$(PIP) install maturin

stubgen:
	cargo run --bin stub-gen

build: stubgen
	$(MATURIN) build --release --out $(DIST_DIR)
	$(MAKE) install

install:
	@WHEEL="$$(ls -t $(DIST_DIR)/*.whl 2>/dev/null | head -n 1)"; \
	if [ -z "$$WHEEL" ]; then \
		echo "No wheel found in $(DIST_DIR). Run 'make build' first."; \
		exit 1; \
	fi; \
	echo "Installing $$WHEEL"; \
	$(PIP) install --force-reinstall "$$WHEEL"

rebuild: clean build

docs-serve: 
	NO_MKDOCS_2_WARNING=1 mkdocs serve

docs-build: docs-deps build
	NO_MKDOCS_2_WARNING=1 mkdocs build --site-dir $(SITE_DIR)

uninstall:
	-$(PIP) uninstall -y $(PACKAGE)

clean:
	rm -rf build $(DIST_DIR) $(SITE_DIR) .mypy_cache .pytest_cache .ruff_cache
	find . -type d -name __pycache__ -prune -exec rm -rf {} +
	find . -type f \( -name '*.so' -o -name '*.pyd' -o -name '*.pyc' \) -delete