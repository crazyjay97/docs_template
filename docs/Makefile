.PHONY: help build clean dist deploy

# Default target
help:
	@echo "Documentation Build System"
	@echo ""
	@echo "Usage:"
	@echo "  make build     - Build documentation for both languages"
	@echo "  make dist      - Build documentation and prepare for deployment (creates dist/)"
	@echo "  make clean     - Clean build artifacts"
	@echo "  make help      - Show this help message"

# Build documentation for both languages
build:
	@echo "Building documentation for all languages..."
	@source venv/bin/activate && build-docs

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	@rm -rf _build/
	@rm -rf dist/
	@find . -name "_build" -type d -exec rm -rf {} + 2>/dev/null || true

# Build and prepare for deployment
dist:
	@echo "Building documentation and preparing for deployment..."
	@./build_dist.sh

# Alias for dist
deploy: dist