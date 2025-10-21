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
	@./run_examples.sh

# Comprehensive pre-commit validation
# Runs all tests, examples, and checks
validate: fmt-check clippy test examples
	@echo ""
	@echo "==================================="
	@echo "✓ All validation checks passed!"
	@echo "==================================="
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
	@echo "Output will be saved to flamegraph.svg"
	@echo ""
	@if ! command -v cargo-flamegraph >/dev/null 2>&1; then \
		echo "Error: cargo-flamegraph not found"; \
		echo "Install with: cargo install flamegraph"; \
		exit 1; \
	fi
	cargo flamegraph --bin profile --output flamegraph.svg
	@echo ""
	@echo "Flamegraph saved to: flamegraph.svg"
	@echo "Open with: firefox flamegraph.svg (or your browser of choice)"

# Profile allocations with heaptrack
# Requires cargo-heaptrack: cargo install cargo-heaptrack
# Also requires heaptrack: apt-get install heaptrack (or equivalent)
heapprofile:
	@echo "=== Profiling allocations with heaptrack ==="
	@echo "This will run 100 games (seed 42) and generate allocation profile"
	@echo "Output will be saved to heaptrack.profile.*.zst"
	@echo ""
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
	cargo heaptrack --bin profile --release -- 100
	@echo ""
	@echo "Heaptrack profile saved"
	@echo "To view: heaptrack_gui heaptrack.profile.*.zst"
	@echo "Or use CLI: heaptrack_print heaptrack.profile.*.zst"
