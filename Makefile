# MTG Forge Rust - Development Makefile
#
# Quick reference for common development tasks

.PHONY: help build test validate clean run check fmt clippy doc examples

# Default target - show available commands
help:
	@echo "MTG Forge Rust - Available Commands:"
	@echo ""
	@echo "  make build      - Build the project (cargo build)"
	@echo "  make test       - Run unit tests (cargo test)"
	@echo "  make validate   - Full pre-commit validation (tests + examples + lint)"
	@echo "  make examples   - Run all examples"
	@echo "  make clean      - Clean build artifacts (cargo clean)"
	@echo "  make run        - Run the main binary (cargo run)"
	@echo "  make check      - Fast compilation check (cargo check)"
	@echo "  make fmt        - Format code (cargo fmt)"
	@echo "  make clippy     - Run linter (cargo clippy)"
	@echo "  make doc        - Generate documentation (cargo doc)"
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
	cargo fmt

# Check formatting without modifying files
fmt-check:
	@echo "=== Checking code formatting ==="
	cargo fmt -- --check

# Run clippy linter
clippy:
	@echo "=== Running clippy ==="
	cargo clippy -- -D warnings

# Run all examples
examples:
	@echo "=== Running examples ==="
	@echo ""
	@echo "--- Lightning Bolt Game Example ---"
	cargo run --example lightning_bolt_game

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
