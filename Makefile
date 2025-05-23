# file: Makefile
# description: Professional production-ready Makefile for rust-tld library

# Project Configuration
PROJECT_NAME := rust-tld
PACKAGE_NAME := rust_tld
VERSION := $(shell grep '^version' Cargo.toml | head -n1 | cut -d'"' -f2)
RUST_VERSION := $(shell rustc --version | cut -d' ' -f2)

# Build Configuration
CARGO := cargo
RUSTFLAGS := 
TARGET_DIR := target
RELEASE_DIR := $(TARGET_DIR)/release
DEBUG_DIR := $(TARGET_DIR)/debug

# Documentation Configuration
DOCS_DIR := $(TARGET_DIR)/doc
DOCS_PORT := 8000

# Testing Configuration
TEST_TIMEOUT := 300
COVERAGE_DIR := $(TARGET_DIR)/coverage

# Linting Configuration
CLIPPY_ARGS := -- -W clippy::all -W clippy::pedantic -W clippy::nursery

# Colors for output
RED := \033[0;31m
GREEN := \033[0;32m
YELLOW := \033[0;33m
BLUE := \033[0;34m
MAGENTA := \033[0;35m
CYAN := \033[0;36m
WHITE := \033[0;37m
RESET := \033[0m

# Default target
.DEFAULT_GOAL := help

##@ General Commands

.PHONY: help
help: ## Display this help message
	@echo "$(CYAN)$(PROJECT_NAME) v$(VERSION)$(RESET)"
	@echo "$(YELLOW)Rust $(RUST_VERSION)$(RESET)"
	@echo ""
	@awk 'BEGIN {FS = ":.*##"; printf "Usage:\n  make $(CYAN)<target>$(RESET)\n"} /^[a-zA-Z_0-9-]+:.*?##/ { printf "  $(CYAN)%-20s$(RESET) %s\n", $$1, $$2 } /^##@/ { printf "\n$(MAGENTA)%s$(RESET)\n", substr($$0, 5) } ' $(MAKEFILE_LIST)

.PHONY: info
info: ## Show project information
	@echo "$(CYAN)Project Information$(RESET)"
	@echo "  Name:           $(PROJECT_NAME)"
	@echo "  Version:        $(VERSION)"
	@echo "  Rust Version:   $(RUST_VERSION)"
	@echo "  Target Dir:     $(TARGET_DIR)"
	@echo "  Package Name:   $(PACKAGE_NAME)"

##@ Development

.PHONY: dev
dev: format lint test ## Run full development workflow (format, lint, test)

.PHONY: build
build: ## Build the project in debug mode
	@echo "$(GREEN)Building $(PROJECT_NAME) (debug)...$(RESET)"
	$(CARGO) build

.PHONY: build-release
build-release: ## Build the project in release mode
	@echo "$(GREEN)Building $(PROJECT_NAME) (release)...$(RESET)"
	$(CARGO) build --release

.PHONY: run
run: ## Run the example with default parameters
	@echo "$(GREEN)Running basic example...$(RESET)"
	$(CARGO) run --example basic_usage

.PHONY: run-verbose
run-verbose: ## Run the example with verbose output
	@echo "$(GREEN)Running basic example (verbose)...$(RESET)"
	$(CARGO) run --example basic_usage -- --verbose

.PHONY: run-with-args
run-with-args: ## Run example with custom URLs (Usage: make run-with-args URLS="url1 url2")
	@echo "$(GREEN)Running basic example with custom URLs...$(RESET)"
	$(CARGO) run --example basic_usage -- $(URLS)

##@ Testing

.PHONY: test
test: ## Run all tests
	@echo "$(GREEN)Running tests...$(RESET)"
	$(CARGO) test

.PHONY: test-verbose
test-verbose: ## Run tests with verbose output
	@echo "$(GREEN)Running tests (verbose)...$(RESET)"
	$(CARGO) test -- --nocapture

.PHONY: test-release
test-release: ## Run tests in release mode
	@echo "$(GREEN)Running tests (release)...$(RESET)"
	$(CARGO) test --release

.PHONY: test-doc
test-doc: ## Test documentation examples
	@echo "$(GREEN)Testing documentation examples...$(RESET)"
	$(CARGO) test --doc

.PHONY: test-integration
test-integration: ## Run integration tests only
	@echo "$(GREEN)Running integration tests...$(RESET)"
	$(CARGO) test --test '*'

.PHONY: test-unit
test-unit: ## Run unit tests only
	@echo "$(GREEN)Running unit tests...$(RESET)"
	$(CARGO) test --lib

.PHONY: test-examples
test-examples: ## Test all examples
	@echo "$(GREEN)Testing examples...$(RESET)"
	$(CARGO) test --examples

.PHONY: test-all
test-all: test test-doc test-examples ## Run all types of tests

##@ Code Quality

.PHONY: format
format: ## Format code using rustfmt
	@echo "$(GREEN)Formatting code...$(RESET)"
	$(CARGO) fmt

.PHONY: format-check
format-check: ## Check if code is formatted correctly
	@echo "$(GREEN)Checking code formatting...$(RESET)"
	$(CARGO) fmt -- --check

.PHONY: lint
lint: ## Run clippy linter
	@echo "$(GREEN)Running clippy linter...$(RESET)"
	$(CARGO) clippy --all-targets --all-features $(CLIPPY_ARGS)

.PHONY: lint-fix
lint-fix: ## Run clippy with automatic fixes
	@echo "$(GREEN)Running clippy with fixes...$(RESET)"
	$(CARGO) clippy --all-targets --all-features --fix $(CLIPPY_ARGS)

.PHONY: audit
audit: ## Run security audit
	@echo "$(GREEN)Running security audit...$(RESET)"
	$(CARGO) audit

.PHONY: outdated
outdated: ## Check for outdated dependencies
	@echo "$(GREEN)Checking for outdated dependencies...$(RESET)"
	$(CARGO) outdated

.PHONY: bloat
bloat: ## Analyze binary size and dependencies
	@echo "$(GREEN)Analyzing binary size...$(RESET)"
	$(CARGO) bloat --release --crates

##@ Documentation

.PHONY: doc
doc: ## Generate documentation
	@echo "$(GREEN)Generating documentation...$(RESET)"
	$(CARGO) doc --no-deps

.PHONY: doc-open
doc-open: doc ## Generate and open documentation
	@echo "$(GREEN)Opening documentation...$(RESET)"
	$(CARGO) doc --no-deps --open

.PHONY: doc-all
doc-all: ## Generate documentation with dependencies
	@echo "$(GREEN)Generating documentation with dependencies...$(RESET)"
	$(CARGO) doc

.PHONY: doc-serve
doc-serve: doc ## Serve documentation locally
	@echo "$(GREEN)Serving documentation at http://localhost:$(DOCS_PORT)$(RESET)"
	@cd $(DOCS_DIR) && python3 -m http.server $(DOCS_PORT)

##@ Benchmarks and Performance

.PHONY: bench
bench: ## Run benchmarks
	@echo "$(GREEN)Running benchmarks...$(RESET)"
	$(CARGO) bench

.PHONY: bench-baseline
bench-baseline: ## Run benchmarks and save as baseline
	@echo "$(GREEN)Running benchmarks (baseline)...$(RESET)"
	$(CARGO) bench -- --save-baseline baseline

.PHONY: profile
profile: ## Profile the application
	@echo "$(GREEN)Profiling application...$(RESET)"
	$(CARGO) build --release
	perf record --call-graph=dwarf $(RELEASE_DIR)/$(PROJECT_NAME)
	perf report

##@ Release and Distribution

.PHONY: check
check: ## Run cargo check
	@echo "$(GREEN)Running cargo check...$(RESET)"
	$(CARGO) check --all-targets --all-features

.PHONY: check-all
check-all: format-check lint audit check test-all ## Run all checks (CI pipeline)

.PHONY: release-dry
release-dry: ## Dry run of release process
	@echo "$(GREEN)Dry run release process...$(RESET)"
	$(CARGO) publish --dry-run

.PHONY: release
release: check-all ## Build and prepare for release
	@echo "$(GREEN)Preparing release v$(VERSION)...$(RESET)"
	@echo "$(YELLOW)Warning: This will tag and potentially publish!$(RESET)"
	@read -p "Continue? [y/N] " -n 1 -r; \
	if [[ $$REPLY =~ ^[Yy]$$ ]]; then \
		git tag -a v$(VERSION) -m "Release v$(VERSION)"; \
		echo "$(GREEN)Tagged v$(VERSION)$(RESET)"; \
		echo "$(YELLOW)Run 'make publish' to publish to crates.io$(RESET)"; \
	else \
		echo "$(RED)Release cancelled$(RESET)"; \
	fi

.PHONY: publish
publish: ## Publish to crates.io
	@echo "$(GREEN)Publishing to crates.io...$(RESET)"
	$(CARGO) publish

##@ Installation and Setup

.PHONY: install
install: build-release ## Install the binary locally
	@echo "$(GREEN)Installing $(PROJECT_NAME)...$(RESET)"
	$(CARGO) install --path .

.PHONY: uninstall
uninstall: ## Uninstall the binary
	@echo "$(GREEN)Uninstalling $(PROJECT_NAME)...$(RESET)"
	$(CARGO) uninstall $(PROJECT_NAME)

.PHONY: setup
setup: ## Setup development environment
	@echo "$(GREEN)Setting up development environment...$(RESET)"
	rustup component add rustfmt clippy
	$(CARGO) install cargo-audit cargo-outdated cargo-bloat
	@echo "$(GREEN)Development environment ready!$(RESET)"

##@ Maintenance

.PHONY: clean
clean: ## Clean build artifacts
	@echo "$(GREEN)Cleaning build artifacts...$(RESET)"
	$(CARGO) clean

.PHONY: clean-all
clean-all: clean ## Clean all artifacts including docs
	@echo "$(GREEN)Cleaning all artifacts...$(RESET)"
	rm -rf $(TARGET_DIR)
	rm -rf Cargo.lock

.PHONY: update
update: ## Update dependencies
	@echo "$(GREEN)Updating dependencies...$(RESET)"
	$(CARGO) update

.PHONY: deps
deps: ## Show dependency tree
	@echo "$(GREEN)Dependency tree:$(RESET)"
	$(CARGO) tree

.PHONY: deps-graph
deps-graph: ## Generate dependency graph (requires graphviz)
	@echo "$(GREEN)Generating dependency graph...$(RESET)"
	$(CARGO) deps --all-deps | dot -Tpng > deps.png
	@echo "$(GREEN)Dependency graph saved as deps.png$(RESET)"

##@ Docker (Optional)

.PHONY: docker-build
docker-build: ## Build Docker image
	@echo "$(GREEN)Building Docker image...$(RESET)"
	docker build -t $(PROJECT_NAME):$(VERSION) .
	docker tag $(PROJECT_NAME):$(VERSION) $(PROJECT_NAME):latest

.PHONY: docker-run
docker-run: ## Run Docker container
	@echo "$(GREEN)Running Docker container...$(RESET)"
	docker run --rm -it $(PROJECT_NAME):latest

.PHONY: docker-clean
docker-clean: ## Clean Docker images
	@echo "$(GREEN)Cleaning Docker images...$(RESET)"
	docker rmi $(PROJECT_NAME):$(VERSION) $(PROJECT_NAME):latest || true

##@ Git Workflow

.PHONY: git-status
git-status: ## Show git status with colors
	@echo "$(CYAN)Git Status:$(RESET)"
	@git status --short --branch

.PHONY: git-log
git-log: ## Show formatted git log
	@git log --oneline --graph --decorate --all -10

.PHONY: pre-commit
pre-commit: format lint test ## Run pre-commit checks
	@echo "$(GREEN)Pre-commit checks passed!$(RESET)"

##@ Utility

.PHONY: lines
lines: ## Count lines of code
	@echo "$(CYAN)Lines of Code:$(RESET)"
	@find src -name "*.rs" | xargs wc -l | tail -1
	@echo "$(CYAN)Lines of Tests:$(RESET)"
	@find . -name "*.rs" -path "*/tests/*" | xargs wc -l | tail -1 || echo "0 total"

.PHONY: size
size: build-release ## Show binary size
	@echo "$(CYAN)Binary Size:$(RESET)"
	@ls -lh $(RELEASE_DIR)/examples/basic_usage 2>/dev/null || echo "No binary found"
	@du -sh $(TARGET_DIR) || echo "No target directory"

.PHONY: todo
todo: ## Find TODO and FIXME comments
	@echo "$(CYAN)TODO and FIXME items:$(RESET)"
	@grep -rn "TODO\|FIXME\|XXX\|HACK" src/ || echo "None found"

.PHONY: watch
watch: ## Watch for changes and run tests
	@echo "$(GREEN)Watching for changes...$(RESET)"
	@command -v cargo-watch >/dev/null 2>&1 || { echo "Installing cargo-watch..."; $(CARGO) install cargo-watch; }
	$(CARGO) watch -x test

##@ Advanced

.PHONY: miri
miri: ## Run tests with Miri for undefined behavior detection
	@echo "$(GREEN)Running Miri tests...$(RESET)"
	$(CARGO) +nightly miri test

.PHONY: expand
expand: ## Expand macros to see generated code
	@echo "$(GREEN)Expanding macros...$(RESET)"
	$(CARGO) expand

.PHONY: asm
asm: ## Show assembly output for release build
	@echo "$(GREEN)Generating assembly...$(RESET)"
	$(CARGO) asm --rust --release

# Include local makefile extensions if they exist
-include Makefile.local

# Declare all targets as phony to avoid conflicts with files
.PHONY: all