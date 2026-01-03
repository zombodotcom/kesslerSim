# Makefile for faster development workflow
# Usage: make check, make test, make run, make watch

.PHONY: check test run watch clean help

# Quick check (faster than build)
check:
	cargo check

# Run tests
test:
	cargo test --lib

# Run the app
run:
	cargo run --release

# Watch for changes and check
watch:
	cargo watch -x check

# Clean build artifacts
clean:
	cargo clean

# Help
help:
	@echo "Available commands:"
	@echo "  make check  - Quick compile check (fastest)"
	@echo "  make test   - Run tests"
	@echo "  make run    - Run the app"
	@echo "  make watch  - Watch for changes and check"
	@echo "  make clean  - Clean build artifacts"

