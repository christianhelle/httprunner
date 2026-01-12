# HTTP File Runner - GitHub Copilot Instructions

Always reference these instructions first and fallback to search or bash commands only when you encounter unexpected information that does not match the info here.

## Project Overview

This is a Rust project. The original Zig implementation has been moved to a separate repository: [christianhelle/httprunner-zig](https://github.com/christianhelle/httprunner-zig).

**Why Rust?** The Rust implementation provides full control over HTTPS configuration, including optional insecure HTTPS support via the `--insecure` flag for development environments with self-signed certificates. The original Zig implementation had limitations in this area.

## Working Effectively

### Prerequisites and Setup
- Install Rust 1.92 or later from https://rustup.rs/
- Ensure git is available for version generation
- CRITICAL: This project requires internet access for HTTP testing - many validation scenarios will fail in offline environments

### Git Commit Strategy (MANDATORY FOR CODING AGENTS)
When implementing changes, ALWAYS commit your work to git in logical, atomic groups:

1. **NEVER Commit Directly to Main**: Before making any changes, check the current branch
   - If on `main` branch, create a new feature branch: `git checkout -b <descriptive-branch-name>`
   - Use descriptive branch names: `feature/timeout-support`, `fix/jsonpath-arrays`, `refactor/cli-parsing`
2. **Commit After Each Logical Unit**: Break work into small, focused commits
3. **Use Brief, Descriptive Messages**: Follow conventional commit format when appropriate
   - Examples: "Add timeout support to runner", "Fix JSONPath array parsing", "Update CLI help text"
4. **Commit Frequency**: Commit after completing each distinct change (e.g., after adding a feature, fixing a bug, updating documentation)
5. **Git Commands**:
   ```bash
   # Check current branch first
   git branch --show-current
   
   # If on main, create new branch
   git checkout -b feature/your-feature-name
   
   # Make commits
   git add <files>
   git commit -m "Brief description of change"
   ```
6. **Benefits**: Creates detailed history, enables easy rollback, makes code review easier, protects main branch

**IMPORTANT**: This applies to ALL coding agents working on this project. Never work directly on main. Commit early and often.

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
- Run with insecure HTTPS: `cargo run -- examples/simple.http --insecure`
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
   cargo run -- examples/simple.http --insecure
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

## Documentation Requirements (CRITICAL)

When implementing features or fixes that affect user-facing functionality, you MUST update ALL relevant documentation:

### Documentation Files to Update

1. **README.md** (Root Directory)
   - Primary project documentation
   - Contains comprehensive feature descriptions, installation options, usage examples
   - Target audience: General users, developers, contributors
   - Must be complete and detailed

2. **src/README.md** (Source Directory)
   - Used as the crates.io package description
   - Must be concise and focused on cargo users
   - Should NOT contain implementation details or source code structure
   - Focus on features, usage, and crates.io-specific installation
   - Keep in sync with README.md but more concise

3. **docs/** (Static Website)
   - GitHub Pages documentation site served at https://christianhelle.com/httprunner/
   - Key files to update:
     - `index.html` - Home page with features overview
     - `guide.html` - User guide with examples
     - `reference.html` - API reference and syntax documentation
     - `install.html` - Installation instructions
     - `cicd.html` - CI/CD integration guide
     - `docker.html` - Docker usage guide
   - Must reflect current features and functionality
   - Update when adding/changing features
   - Check for outdated content or examples
   - Ensure navigation and links are correct

### When to Update Documentation

**ALWAYS update documentation when:**
- Adding a new feature or command-line flag
- Changing existing behavior or syntax
- Adding/modifying .http file format support
- Changing installation procedures
- Updating version information or requirements
- Adding/removing dependencies that affect users
- Modifying environment variable behavior
- Changing timeout defaults or configuration
- Adding new assertion types or validation rules
- Modifying request chaining or variable syntax

### Documentation Update Checklist

When making feature changes or fixes:

- [ ] Update README.md with detailed explanation and examples
- [ ] Update src/README.md with concise description (remember: crates.io audience)
- [ ] Check docs/ folder for pages that need updates
- [ ] Add examples to demonstrate new features
- [ ] Update command-line help text if adding flags
- [ ] Update .http file format examples if syntax changes
- [ ] Verify all documentation examples still work
- [ ] Check that version requirements are current
- [ ] Ensure installation instructions are accurate

### Documentation Sync Guidelines

**README.md vs src/README.md:**
- README.md: Comprehensive, includes all installation methods, detailed examples, CI/CD info, development setup
- src/README.md: Focused on `cargo install` users, concise feature descriptions, essential usage only
- Both should cover the same features but with different levels of detail
- src/README.md should link to GitHub for complete documentation

**docs/ folder:**
- Keep static website content aligned with README.md
- Update feature pages when adding/changing functionality
- Ensure examples and screenshots are current
- Remove outdated content promptly

### Documentation Verification Steps

Before committing documentation changes:

1. **Syntax and Examples**
   - Verify all code examples have correct syntax
   - Test command-line examples actually work
   - Ensure .http file examples are valid
   - Check that all example files referenced exist

2. **Consistency Check**
   - Feature descriptions match across all docs
   - Version numbers are consistent
   - Installation commands are identical where appropriate
   - Command-line flags documentation matches `--help` output

3. **Completeness Check**
   - All new features are documented
   - Breaking changes are clearly noted
   - Migration guides are provided when needed
   - Examples demonstrate key functionality

4. **Link Validation**
   - Internal links work correctly
   - External links are valid
   - Example file paths are correct
   - GitHub links point to correct locations

## Common Development Tasks

### Adding New Features
1. Modify appropriate source files in `src/`
2. Add/update tests in the same files (Rust convention)
3. **Update ALL relevant documentation** (README.md, src/README.md, docs/)
4. Add example files to demonstrate the feature if applicable
5. Test with example files
6. Run formatting and build validation: `cargo fmt && cargo clippy && cargo test`
7. Verify documentation examples work correctly

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