# HTTP File Runner - GitHub Copilot Instructions

Always reference these instructions first and fallback to search or bash commands only when you encounter unexpected information that does not match the info here.

## Working Effectively

### Prerequisites and Setup
- Install Zig 0.15.1 or later from https://ziglang.org/download/
- Ensure git is available for version generation
- CRITICAL: This project requires internet access for HTTP testing - many validation scenarios will fail in offline environments

### Bootstrap and Build Process
- `zig build` -- NEVER CANCEL: Build takes 2-5 minutes typically. Set timeout to 10+ minutes.
- `zig build test` -- NEVER CANCEL: Unit tests take 1-2 minutes. Set timeout to 5+ minutes.
- `zig build -Doptimize=ReleaseFast` -- NEVER CANCEL: Release build takes 2-5 minutes. Set timeout to 10+ minutes.
- `zig fmt --check .` -- Code formatting check, takes 5-10 seconds
- `zig fmt .` -- Auto-format code, takes 5-10 seconds

### Development Commands
- Debug build: `zig build` (creates `zig-out/bin/httprunner` on Unix, `zig-out/bin/httprunner.exe` on Windows)
- Run with examples: `./zig-out/bin/httprunner examples/simple.http`
- Run with verbose mode: `./zig-out/bin/httprunner examples/simple.http --verbose`
- Run discovery mode: `./zig-out/bin/httprunner --discover`
- Show help: `./zig-out/bin/httprunner --help`
- Show version: `./zig-out/bin/httprunner --version`

## Critical Build and Test Information

### Build Timing and Timeouts
- **NEVER CANCEL BUILD COMMANDS** - Set timeouts of 10+ minutes for all build operations
- **NEVER CANCEL TEST COMMANDS** - Set timeouts of 5+ minutes for test operations
- Initial build generates `src/version_info.zig` from git information
- Build artifacts go to `zig-out/bin/` directory
- Clean builds with: `rm -rf zig-out/ zig-cache/`

### Validation Scenarios
Always test these complete scenarios after making changes:

1. **Basic Build Validation**:
   ```bash
   zig build
   ./zig-out/bin/httprunner --help
   ./zig-out/bin/httprunner --version
   ```

2. **HTTP Request Testing** (requires internet):
   ```bash
   ./zig-out/bin/httprunner examples/simple.http
   ./zig-out/bin/httprunner examples/simple.http --verbose
   ./zig-out/bin/httprunner examples/basic.http
   ```

3. **Feature Testing**:
   ```bash
   ./zig-out/bin/httprunner examples/variables.http
   ./zig-out/bin/httprunner examples/request-variables.http
   ./zig-out/bin/httprunner examples/asserts.http
   ./zig-out/bin/httprunner --discover
   ```

4. **Cross-platform Testing** (Windows):
   ```powershell
   zig build
   .\run.ps1 examples\simple.http
   ```

## Repository Structure and Key Files

### Core Application Files
```
src/
├── main.zig           # Application entry point
├── cli.zig            # Command-line parsing
├── parser.zig         # HTTP file parsing
├── runner.zig         # HTTP execution engine
├── processor.zig      # Request processing
├── types.zig          # Data structures
├── colors.zig         # Terminal colors
├── discovery.zig      # File discovery
├── assertions.zig     # Response validation
├── request_variables.zig  # Request chaining
├── environment.zig    # Environment variables
├── log.zig           # Logging functionality
└── upgrade.zig       # Self-update feature
```

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
- Run all tests: `zig build test`
- Tests are embedded in source files using Zig's built-in test system
- No external test dependencies required

### Integration Testing
- Use example files to test actual HTTP functionality
- Test with `--verbose` flag to see detailed request/response information
- Test `--discover` mode to validate file discovery
- Test environment variables with `--env` flag

### CI/CD Validation
Always run these before committing:
```bash
zig fmt --check .     # Format validation
zig build             # Debug build
zig build test        # Unit tests
zig build -Doptimize=ReleaseFast  # Release build
```

## Common Development Tasks

### Adding New Features
1. Modify appropriate source files in `src/`
2. Add/update tests in the same files
3. Update documentation if needed
4. Test with example files
5. Run formatting and build validation

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
- Use `.\run.ps1` for proper UTF-8 emoji support
- Build creates `zig-out\bin\httprunner.exe`
- PowerShell encoding: `[Console]::OutputEncoding = [System.Text.Encoding]::UTF8`

### Linux/macOS Development
- Build creates `zig-out/bin/httprunner`
- Direct execution: `./zig-out/bin/httprunner`
- Standard terminal UTF-8 support

## Troubleshooting

### Build Issues
- Ensure Zig 0.15.1+ is installed
- Check git is available for version generation
- Clean build: `rm -rf zig-out/ zig-cache/`
- Network issues may affect git commands in build.zig

### Runtime Issues
- HTTP requests require internet connectivity
- Use `--verbose` flag for detailed debugging
- Check example files for correct syntax
- Validate `.http` file format

### Performance Notes
- Release builds: `zig build -Doptimize=ReleaseFast`
- Debug builds include symbols for debugging
- Binary size: ~2MB for release builds
- Memory usage: minimal, suitable for CI/CD environments

## Version Management
- Version info auto-generated from git tags at build time
- Format: `src/version_info.zig` created during build
- Includes git tag, commit hash, and build timestamp
- Use `httprunner --version` to display version information

Remember: NEVER CANCEL long-running build or test commands. This is a Zig project with standard timing expectations for compilation and linking.