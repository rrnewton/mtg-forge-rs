# MTG Forge Rust - Development Makefile
#
# Quick reference for common development tasks
.PHONY: help build test validate clean run check fmt clippy doc examples bench profile heapprofile count setup-claude claude-github claude-beads happy

# Default target - show available commands
help:
	@echo "MTG Forge Rust - Available Commands:"
	@echo ""
	@echo "  make build       - Build the project (cargo build)"
	@echo "  make test        - Run unit tests (cargo test)"
	@echo "  make validate    - Full pre-commit validation (tests + examples + lint)"
	@echo "  make examples    - Run all examples"
	@echo "  make bench       - Run performance benchmarks (cargo bench)"
	@echo "  make profile     - Profile game execution with flamegraph (CPU time)"
	@echo "  make heapprofile - Profile allocations with heaptrack"
	@echo "  make clean       - Clean build artifacts (cargo clean)"
	@echo "  make run         - Run the main binary (cargo run)"
	@echo "  make check       - Fast compilation check (cargo check)"
	@echo "  make fmt         - Format code (cargo fmt)"
	@echo "  make clippy      - Run linter (cargo clippy)"
	@echo "  make doc         - Generate documentation (cargo doc)"
	@echo ""

# Build the project
build:
	@echo "=== Building project ==="
	cargo build

# Build release version
build-release:
	@echo "=== Building release ==="
	cargo build --release

# Run unit tests
test:
	@echo "=== Running unit tests ==="
	cargo nextest run

# Fast compilation check (no codegen)
check:
	@echo "=== Running cargo check ==="
	cargo check

# Format code
fmt:
	@echo "=== Formatting code ==="
	cargo fmt --all

# Check formatting without modifying files
fmt-check:
	@echo "=== Checking code formatting ==="
	cargo fmt --all -- --check

# Run clippy linter
clippy:
	@echo "=== Running clippy ==="
	cargo clippy --all-targets --all-features -- -D warnings

count:
	@echo "=== Counting lines of code ==="
	cargo install cloc 2>/dev/null || true
	cloc src; cloc scripts; cloc tests

# Run all examples
examples:
	@echo "=== Running examples ==="
	@echo ""
	@./scripts/run_examples.sh

# Comprehensive pre-commit validation with caching
# Runs all tests, examples, and checks
# Caches results based on commit hash to avoid redundant validation
# Use: make validate ARGS=--force to skip cache
# Use: make validate ARGS=--sequential to run sequentially (fail on first error)
# Use: make validate ARGS="--force --sequential" to combine options
# See scripts/validate.sh for implementation details
validate:
	@./scripts/validate.sh $(ARGS)

# Internal target that actually runs validation
# This is called by scripts/validate.sh
# Runs validation steps in parallel using make -j
validate-impl:
	@echo "=== Starting parallel validation ==="
	@echo ""
	@$(MAKE) -j4 validate-parallel-steps
	@echo ""
	@echo "=== All validation steps completed ==="
	@echo ""

# Sequential validation - runs steps one at a time, fails on first error
# This is called by scripts/validate.sh when --sequential flag is used
validate-impl-sequential:
	@echo "=== Starting sequential validation ==="
	@echo ""
	@$(MAKE) validate-fmt-check-step
	@echo ""
	@$(MAKE) validate-clippy-step
	@echo ""
	@$(MAKE) validate-test-step
	@echo ""
	@$(MAKE) validate-examples-step
	@echo ""
	@echo "=== All validation steps completed ==="
	@echo ""

# Parallel validation steps - these will run concurrently when invoked with -j
.PHONY: validate-parallel-steps validate-impl-sequential validate-fmt-check-step validate-clippy-step validate-test-step validate-examples-step
validate-parallel-steps: validate-fmt-check-step validate-clippy-step validate-test-step validate-examples-step

validate-fmt-check-step:
	@$(MAKE) fmt-check
	@echo "✓ fmt-check completed"

validate-clippy-step:
	@$(MAKE) clippy
	@echo "✓ clippy completed"

validate-test-step:
	@$(MAKE) test
	@echo "✓ test completed"

validate-examples-step:
	@$(MAKE) examples
	@echo "✓ examples completed"

# Generate documentation
doc:
	@echo "=== Generating documentation ==="
	cargo doc --no-deps --open

# Clean build artifacts
clean:
	@echo "=== Cleaning build artifacts ==="
	cargo clean

# Run the main binary
run:
	@echo "=== Running main binary ==="
	cargo run

# Run with release optimizations
run-release:
	@echo "=== Running release binary ==="
	cargo run --release

# Install development dependencies
setup:
	@echo "=== Installing development tools ==="
	rustup component add rustfmt clippy

# Show project info
info:
	@echo "Project: MTG Forge Rust"
	@echo "Rust version: $$(rustc --version)"
	@echo "Cargo version: $$(cargo --version)"
	@cargo tree --depth 1

# Run performance benchmarks
bench:
	@echo "=== Running benchmarks ==="
	cargo bench --bench game_benchmark

# Profile game execution with flamegraph (CPU time profiling)
# Requires cargo-flamegraph: cargo install flamegraph
profile:
	@echo "=== Profiling game execution with flamegraph (CPU time) ==="
	@echo "This will run 1000 games (seed 42) and generate a flamegraph"
	@echo "Output will be saved to experiment_results/flamegraph.svg"
	@echo ""
	@mkdir -p experiment_results
	@if ! command -v cargo-flamegraph >/dev/null 2>&1; then \
		echo "Error: cargo-flamegraph not found"; \
		echo "Install with: cargo install flamegraph"; \
		exit 1; \
	fi
	cargo flamegraph --bin mtg --output experiment_results/flamegraph.svg -- profile --games 1000 --seed 42
	@echo ""
	@echo "Flamegraph saved to: experiment_results/flamegraph.svg"
	@echo "Open with: firefox experiment_results/flamegraph.svg (or your browser of choice)"

profile2: build-release
	@mkdir -p experiment_results
	cd experiment_results && perf record -g --call-graph dwarf -- ../target/release/mtg profile --games 5000 --seed 42
	cd experiment_results && perf report

# Profile allocations with heaptrack
# Requires cargo-heaptrack: cargo install cargo-heaptrack
# Also requires heaptrack: apt-get install heaptrack (or equivalent)
heapprofile:
	@echo "=== Profiling allocations with heaptrack ==="
	@echo "This will run 100 games (seed 42) and generate allocation profile"
	@echo "Output will be saved to experiment_results/heaptrack.profile.*.zst"
	@echo ""
	@mkdir -p experiment_results
	@if ! command -v cargo-heaptrack >/dev/null 2>&1; then \
		echo "Error: cargo-heaptrack not found"; \
		echo "Install with: cargo install cargo-heaptrack"; \
		echo ""; \
		echo "Also requires heaptrack system package:"; \
		echo "  Ubuntu/Debian: sudo apt-get install heaptrack"; \
		echo "  Fedora: sudo dnf install heaptrack"; \
		echo "  Arch: sudo pacman -S heaptrack"; \
		exit 1; \
	fi
	HEAPTRACK_OUTPUT=experiment_results cargo heaptrack --bin mtg --release -- profile --games 100 --seed 42
	@# Move heaptrack files to experiment_results if they were created in root
	@if ls heaptrack.profile.* 2>/dev/null; then \
		mv heaptrack.profile.* experiment_results/ 2>/dev/null || true; \
	fi
	@echo ""
	@echo "=== Profiling complete! Now analyzing results ==="
	@echo ""
	./scripts/analyze_heapprofile.sh
	@echo ""
	@echo "Analysis complete! Check output above for top allocation sites."

# TODO: need a way to pass this into happy on the CLI.
# It can take an MCP action to change it, but that's lame and wasteful.
.session_title.txt:
	echo mtg-container-`hostname` > $@

happy: .session_title.txt
	TZ=America/New_York happy claude --dangerously-skip-permissions "Tell happy to change the title to "`cat .session_title.txt`

setup-claude: claude-github claude-beads

claude-github:
	claude mcp add --transport http github https://api.githubcopilot.com/mcp -H "Authorization: Bearer $GITHUB_PERSONAL_ACCESS_TOKEN"

claude-beads:
	claude plugin marketplace add steveyegge/beads
	claude plugin install beads
