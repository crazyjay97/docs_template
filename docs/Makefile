.PHONY: help build build-en build-zh clean dist deploy

# Default target
help:
	@echo "Documentation Build System"
	@echo ""
	@echo "Usage:"
	@echo "  make build     - Build documentation for both languages"
	@echo "  make build-en  - Build English documentation only"
	@echo "  make build-zh  - Build Chinese documentation only"
	@echo "  make dist      - Build documentation and prepare for deployment (creates dist/)"
	@echo "  make clean     - Clean build artifacts"
	@echo "  make help      - Show this help message"

# Build documentation for both languages
build: build-en build-zh
	@echo ""
	@echo "✓ Build completed for all languages"

# Build English documentation
build-en:
	@echo "Building English documentation..."
	@cd en && $(MAKE) html
	@echo "✓ English build completed"

# Build Chinese documentation
build-zh:
	@echo "Building Chinese documentation..."
	@cd zh_CN && $(MAKE) html
	@echo "✓ Chinese build completed"

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	@rm -rf _build/
	@rm -rf dist/
	@find . -name "_build" -type d -exec rm -rf {} + 2>/dev/null || true
	@cd en && $(MAKE) clean 2>/dev/null || true
	@cd zh_CN && $(MAKE) clean 2>/dev/null || true
	@echo "✓ Clean completed"

# Build and prepare for deployment
dist: build
	@echo "Building documentation and preparing for deployment..."
	@./build_dist.sh

# Alias for dist
deploy: dist
