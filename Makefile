# MTG Forge Rust - Development Makefile
#
# Quick reference for common development tasks

.PHONY: help build test validate clean run check fmt clippy doc examples bench profile heapprofile

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
	cargo test

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

# Run all examples
examples:
	@echo "=== Running examples ==="
	@echo ""
	@./scripts/run_examples.sh

# Comprehensive pre-commit validation with caching
# Runs all tests, examples, and checks
# Caches results based on commit hash to avoid redundant validation
validate:
	@echo "=== Starting validation with caching ==="
	@mkdir -p experiment_results
	@COMMIT_HASH=$$(git rev-parse HEAD 2>/dev/null || echo "unknown"); \
	CLEAN_STATUS=$$(git diff-index --quiet HEAD -- 2>/dev/null && echo "clean" || echo "dirty"); \
	if [ "$$CLEAN_STATUS" = "clean" ]; then \
		LOG_FILE="experiment_results/validate_$${COMMIT_HASH}.log"; \
	else \
		LOG_FILE="experiment_results/validate_$${COMMIT_HASH}_DIRTY.log"; \
	fi; \
	if [ "$$CLEAN_STATUS" = "clean" ] && [ -f "$$LOG_FILE" ]; then \
		echo ""; \
		echo "===================================";\
		echo "✓ Validation cache hit for commit $${COMMIT_HASH}"; \
		echo "✓ Validation already passed!"; \
		echo "===================================";\
		echo ""; \
		echo "Log file: $$LOG_FILE"; \
		echo ""; \
	else \
		echo "Running validation (cache miss or dirty working copy)..."; \
		echo "Commit: $${COMMIT_HASH} ($$CLEAN_STATUS)"; \
		echo "Log file: $$LOG_FILE"; \
		echo ""; \
		WIP_FILE="$${LOG_FILE}.wip"; \
		if $(MAKE) validate-impl 2>&1 | tee "$$WIP_FILE"; then \
			mv "$$WIP_FILE" "$$LOG_FILE"; \
			echo ""; \
			echo "===================================";\
			echo "✓ All validation checks passed!"; \
			echo "===================================";\
			echo ""; \
			echo "Results cached to: $$LOG_FILE"; \
			echo ""; \
		else \
			rm -f "$$WIP_FILE"; \
			echo ""; \
			echo "===================================";\
			echo "✗ Validation failed!"; \
			echo "===================================";\
			echo ""; \
			exit 1; \
		fi; \
	fi

# Internal target that actually runs validation
# This is called by the validate target above
validate-impl: fmt-check clippy test examples
	@echo ""
	@echo "=== All validation steps completed ==="
	@echo ""

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
	cargo flamegraph --bin profile --output experiment_results/flamegraph.svg
	@echo ""
	@echo "Flamegraph saved to: experiment_results/flamegraph.svg"
	@echo "Open with: firefox experiment_results/flamegraph.svg (or your browser of choice)"

profile2: build-release
	@mkdir -p experiment_results
	cd experiment_results && perf record -g --call-graph dwarf -- ../target/release/profile 5000
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
	HEAPTRACK_OUTPUT=experiment_results cargo heaptrack --bin profile --release -- 100
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
