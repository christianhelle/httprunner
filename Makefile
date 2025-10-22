.PHONY: all build test clean install run help fmt check release

# Default target
all: build

# Build the project in debug mode
build:
	@echo "Building httprunner (debug)..."
	@zig build

# Build the project in release mode
release:
	@echo "Building httprunner (release)..."
	@zig build -Doptimize=ReleaseFast

# Run tests
test:
	@echo "Running tests..."
	@zig build test

# Format code
fmt:
	@echo "Formatting code..."
	@zig fmt .

# Check code formatting
check:
	@echo "Checking code format..."
	@zig fmt --check .

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	@rm -rf zig-out/ zig-cache/ .zig-cache/

# Install to system (requires root)
install: release
	@echo "Installing httprunner to /usr/local/bin..."
	@install -m 755 zig-out/bin/httprunner /usr/local/bin/

# Run with simple example
run: build
	@./zig-out/bin/httprunner examples/simple.http

# Run with verbose mode
run-verbose: build
	@./zig-out/bin/httprunner examples/simple.http --verbose

# Run discovery mode
discover: build
	@./zig-out/bin/httprunner --discover

# Show version
version: build
	@./zig-out/bin/httprunner --version

# Show help
help:
	@echo "HTTP File Runner - Makefile targets:"
	@echo ""
	@echo "  make build        - Build in debug mode (default)"
	@echo "  make release      - Build in release mode"
	@echo "  make test         - Run unit tests"
	@echo "  make fmt          - Format source code"
	@echo "  make check        - Check code formatting"
	@echo "  make clean        - Remove build artifacts"
	@echo "  make install      - Install to /usr/local/bin (requires root)"
	@echo "  make run          - Build and run with simple example"
	@echo "  make run-verbose  - Build and run with verbose output"
	@echo "  make discover     - Run file discovery mode"
	@echo "  make version      - Show version information"
	@echo "  make help         - Show this help message"
	@echo ""
	@echo "Examples:"
	@echo "  make && ./zig-out/bin/httprunner examples/basic.http"
	@echo "  make release && ./zig-out/bin/httprunner --help"
