# HTTP File Runner - GitHub Copilot Instructions

Always reference these instructions first and fallback to search or bash commands only when you encounter unexpected information that does not match the info here.

## Project Overview

This is a Rust project. The original Zig implementation has been moved to a separate repository: [christianhelle/httprunner-zig](https://github.com/christianhelle/httprunner-zig).

**Why Rust?** The primary reason for migration was Zig's HTTP limitations - the standard library cannot configure insecure HTTPS calls (bypassing certificate validation), which is necessary for development environments with self-signed certificates. libcurl integration proved too complex for cross-platform maintenance.

## Working Effectively

### Prerequisites and Setup
- Install Rust 1.70 or later from https://rustup.rs/
- Ensure git is available for version generation
- CRITICAL: This project requires internet access for HTTP testing - many validation scenarios will fail in offline environments

### Bootstrap and Build Process
- `cargo build` -- NEVER CANCEL: Build takes 15-30 seconds typically. Set timeout to 2+ minutes.
- `cargo test` -- NEVER CANCEL: Unit tests take 5-15 seconds. Set timeout to 2+ minutes.
- `cargo build --release` -- NEVER CANCEL: Release build takes 30-60 seconds. Set timeout to 3+ minutes.
- `cargo fmt --check` -- Code formatting check, takes 1-2 seconds
- `cargo fmt` -- Auto-format code, takes 1-2 seconds
- `cargo clippy` -- Linter check, takes 10-20 seconds

### Development Commands
- Debug build: `cargo build` (creates `target/debug/httprunner` on Unix, `target/debug/httprunner.exe` on Windows)
- Run with examples: `cargo run -- examples/simple.http`
- Run with verbose mode: `cargo run -- examples/simple.http --verbose`
- Run discovery mode: `cargo run -- --discover`
- Show help: `cargo run -- --help`
- Show version: `cargo run -- --version`
- Direct execution: `./target/debug/httprunner examples/simple.http`

## Critical Build and Test Information

### Build Timing and Timeouts
- Initial builds download and compile dependencies (15-30 seconds)
- Subsequent builds are incremental (5-10 seconds)
- Release builds take longer due to optimization (30-60 seconds)
- Set timeouts of 2-3+ minutes for build operations
- Build artifacts go to `target/debug/` or `target/release/` directory
- Clean builds with: `cargo clean`

### Validation Scenarios
Always test these complete scenarios after making changes:

1. **Basic Build Validation**:
   ```bash
   cargo build
   ./target/debug/httprunner --help
   ./target/debug/httprunner --version
   ```

2. **HTTP Request Testing** (requires internet):
   ```bash
   cargo run -- examples/simple.http
   cargo run -- examples/simple.http --verbose
   cargo run -- examples/basic.http
   ```

3. **Feature Testing**:
   ```bash
   cargo run -- examples/variables.http
   cargo run -- examples/request-variables.http
   cargo run -- examples/asserts.http
   cargo run -- --discover
   ```

4. **Cross-platform Testing** (Windows):
   ```powershell
   cargo build
   .\target\debug\httprunner.exe examples\simple.http
   ```

## Repository Structure and Key Files

### Core Application Files
```
src/
├── main.rs            # Application entry point
├── cli.rs             # Command-line parsing with clap
├── parser.rs          # HTTP file parsing
├── runner.rs          # HTTP execution engine with reqwest
├── processor.rs       # Request processing
├── types.rs           # Data structures
├── colors.rs          # Terminal colors with colored crate
├── discovery.rs       # File discovery with walkdir
├── assertions.rs      # Response validation
├── request_variables.rs  # Request chaining
├── environment.rs     # Environment variables
├── log.rs            # Logging functionality
└── upgrade.rs        # Self-update feature
```

### Legacy Zig Implementation (Moved)
The Zig implementation has been moved to a separate repository: [christianhelle/httprunner-zig](https://github.com/christianhelle/httprunner-zig). It was moved due to HTTP/HTTPS configuration limitations.

### Example Files for Testing
```
examples/
├── simple.http           # 4 basic requests - quick test
├── basic.http           # GET requests to various APIs
├── apis.http            # Public API tests
├── comprehensive.http   # Full feature demonstration
├── variables.http       # Variable substitution
├── request-variables.http # Request chaining
├── asserts.http         # Response assertions
├── status-codes.http    # HTTP status testing
├── environment-variables.http # Environment file usage
└── http-client.env.json # Environment configuration
```

## Testing and Validation

### Unit Testing
- Run all tests: `cargo test`
- Run tests with output: `cargo test -- --nocapture`
- Run specific test: `cargo test test_name`
- Tests are embedded in source files using Rust's built-in test system
- No external test dependencies required

### Integration Testing
- Use example files to test actual HTTP functionality
- Test with `--verbose` flag to see detailed request/response information
- Test `--discover` mode to validate file discovery
- Test environment variables with `--env` flag

### CI/CD Validation
Always run these before committing:
```bash
cargo fmt --check       # Format validation
cargo clippy            # Linter
cargo build             # Debug build
cargo test              # Unit tests
cargo build --release   # Release build
```

## Common Development Tasks

### Adding New Features
1. Modify appropriate source files in `src/`
2. Add/update tests in the same files (Rust convention)
3. Update documentation if needed
4. Test with example files
5. Run formatting and build validation: `cargo fmt && cargo clippy && cargo test`

### HTTP File Format
The application parses `.http` files with this structure:
```http
# Comments start with #
@variable=value

METHOD URL
Header: value

{optional body}

EXPECTED_RESPONSE_STATUS 200
EXPECTED_RESPONSE_BODY "expected text"
EXPECTED_RESPONSE_HEADERS "Header: value"
```

### Environment Variables
- Environment files: `http-client.env.json`
- Usage: `httprunner file.http --env environment_name`
- Variables use `{{variable_name}}` syntax

### Request Variables
- Name requests with `# @name request_name`
- Reference with `{{request_name.response.body.$.property}}`
- Enables request chaining and data extraction

## Platform-Specific Notes

### Windows Development
- Build creates `target\debug\httprunner.exe` or `target\release\httprunner.exe`
- PowerShell encoding: `[Console]::OutputEncoding = [System.Text.Encoding]::UTF8`
- Use `cargo run -- examples\simple.http` for testing

### Linux/macOS Development
- Build creates `target/debug/httprunner` or `target/release/httprunner`
- Direct execution: `./target/release/httprunner examples/simple.http`
- Standard terminal UTF-8 support

## Troubleshooting

### Build Issues
- Ensure Rust 1.70+ is installed from https://rustup.rs/
- Check git is available for version generation
- Clean build: `cargo clean`
- Network issues may affect dependency downloads on first build
- Use `cargo build --verbose` for detailed build information

### Runtime Issues
- HTTP requests require internet connectivity
- Use `--verbose` flag for detailed debugging
- Check example files for correct syntax
- Validate `.http` file format
- For HTTPS issues with self-signed certificates, the Rust implementation can be configured to accept them

### Performance Notes
- Release builds: `cargo build --release`
- Debug builds include symbols for debugging
- Binary size: ~5-10MB for release builds
- Memory usage: minimal, suitable for CI/CD environments
- Performance is optimized for I/O-bound HTTP operations

## Version Management
- Version info auto-generated from git tags at build time via `build.rs`
- Build script runs before compilation and sets environment variables
- Includes git tag, commit hash, and build timestamp
- Use `httprunner --version` to display version information

## Legacy Zig Implementation

The Zig implementation has been moved to a separate repository: [christianhelle/httprunner-zig](https://github.com/christianhelle/httprunner-zig). This repository now only contains the Rust implementation, which is actively maintained and recommended for all use cases.