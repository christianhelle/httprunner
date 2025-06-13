# HTTP File Runner

[![Build](https://github.com/christianhelle/httprunner/actions/workflows/build.yml/badge.svg)](https://github.com/christianhelle/httprunner/actions/workflows/build.yml)

A simple command-line tool written in Zig that parses `.http` files and executes HTTP requests, providing colored output with emojis to indicate success or failure.

## Features

- 🚀 Parse and execute HTTP requests from `.http` files
- 📁 Support for multiple `.http` files in a single run
- 🔍 `--discover` mode to recursively find and run all `.http` files
- ✅ Color-coded output (green for success, red for failure)
- 📊 Summary statistics showing success/failure counts (per file and overall)
- 🌐 Support for various HTTP methods (GET, POST, PUT, DELETE, PATCH)
- 📝 Custom headers support (parsing implemented, execution pending)
- 🎯 Detailed error reporting with status codes
- 🛡️ Robust error handling for network issues

## Installation

### Option 1: Install from Snap Store (Recommended)

```bash
sudo snap install httprunner
```

### Option 2: Build from Source

Make sure you have Zig installed (version 0.14 or later).

```bash
zig build
```

## Usage

### If installed via Snap

```bash
# Run a single .http file
httprunner <http-file>

# Run multiple .http files
httprunner <http-file1> <http-file2> [...]

# Discover and run all .http files recursively
httprunner --discover
```

### If built from source

#### Windows PowerShell Users

For proper emoji display in PowerShell, you can either:

Option 1: Use the provided PowerShell script

```pwsh
.\run.ps1 <http-file>
```

Option 2: Set UTF-8 encoding manually

```pwsh
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
.\zig-out\bin\httprunner.exe <http-file>

# Run multiple files
.\zig-out\bin\httprunner.exe examples\simple.http examples\basic.http

# Discover all .http files
.\zig-out\bin\httprunner.exe --discover
```

### Command Line

```bash
# Run a single .http file
./zig-out/bin/httprunner <http-file>

# Run multiple .http files
./zig-out/bin/httprunner <http-file1> <http-file2> [...]

# Discover and run all .http files recursively from current directory
./zig-out/bin/httprunner --discover
```

### Examples

```bash
# Test basic functionality
./zig-out/bin/httprunner examples/simple.http

# Test various APIs
./zig-out/bin/httprunner examples/apis.http

# Test different HTTP status codes
./zig-out/bin/httprunner examples/status-codes.http

# Test basic GET requests
./zig-out/bin/httprunner examples/basic.http

# Run multiple files at once
./zig-out/bin/httprunner examples/simple.http examples/quick.http

# Discover and run all .http files in the project
./zig-out/bin/httprunner --discover

# Run all files in a specific directory (using shell globbing)
./zig-out/bin/httprunner examples/*.http
```

## .http File Format

The HTTP File Runner supports a simple format for defining HTTP requests:

```http
# Comments start with #

# Basic GET request
GET https://api.github.com/users/octocat

# Request with headers
GET https://httpbin.org/headers
User-Agent: HttpRunner/1.0
Accept: application/json

# POST request with body
POST https://httpbin.org/post
Content-Type: application/json

{
  "name": "test",
  "value": 123
}
```

### Supported Features

- **Methods**: GET, POST, PUT, DELETE, PATCH
- **Headers**: Key-value pairs separated by `:` (parsed but not yet applied to requests)
- **Body**: Content after headers (separated by empty line)
- **Comments**: Lines starting with `#`

## Example Files

The `examples/` directory contains several sample `.http` files:

- **`simple.http`** - Basic requests for quick testing (4 requests)
- **`basic.http`** - Various GET requests to different websites
- **`apis.http`** - Requests to public APIs (7 requests)
- **`status-codes.http`** - Tests different HTTP status codes (15 requests)

## Output

The tool provides colored output with emojis:

- ✅ **Green**: Successful requests (2xx status codes)
- ❌ **Red**: Failed requests (4xx, 5xx status codes, or connection errors)
- 🚀 **Blue**: Informational messages
- ⚠️ **Yellow**: Warnings

### Example Output

```text
🚀 HTTP File Runner - Processing file: examples/simple.http
==================================================
Found 4 HTTP request(s)

✅ GET https://httpbin.org/status/200 - Status: 200
❌ GET https://httpbin.org/status/404 - Status: 404
✅ GET https://api.github.com/zen - Status: 200
✅ GET https://jsonplaceholder.typicode.com/users/1 - Status: 200

==================================================
Summary: 3/4 requests succeeded
```

### Multiple Files Output

When running multiple files or using `--discover`, you'll see a summary for each file plus an overall summary:

```text
🔍 Discovering .http files recursively...
Found 7 .http file(s):
  📄 .\examples\apis.http
  📄 .\examples\basic.http
  📄 .\examples\simple.http
  📄 .\examples\quick.http

🚀 HTTP File Runner - Processing file: .\examples\simple.http
==================================================
Found 4 HTTP request(s)

✅ GET https://httpbin.org/status/200 - Status: 200
❌ GET https://httpbin.org/status/404 - Status: 404
✅ GET https://api.github.com/zen - Status: 200
✅ GET https://jsonplaceholder.typicode.com/users/1 - Status: 200

==================================================
File Summary: 3/4 requests succeeded

🚀 HTTP File Runner - Processing file: .\examples\quick.http
==================================================
Found 2 HTTP request(s)

✅ GET https://httpbin.org/status/200 - Status: 200
❌ GET https://httpbin.org/status/404 - Status: 404

==================================================
File Summary: 1/2 requests succeeded

🎯 Overall Summary:
Files processed: 2
Total requests: 4/6
```

### Status Code Examples

From `examples/status-codes.http`:

- **2xx Success**: Status 200, 201, 202 - shown in green ✅
- **3xx Redirects**: Status 301, 302 - automatically followed, shown as 200 ✅
- **4xx Client Errors**: Status 400, 401, 403, 404, 429 - shown in red ❌
- **5xx Server Errors**: Status 500, 502, 503 - shown in red ❌

## Error Handling

The tool handles various error conditions gracefully:

- **File not found**: Clear error message with red indicator
- **Invalid URLs**: Proper error reporting
- **Network issues**: Connection timeouts, unknown hosts, etc.
- **Invalid HTTP methods**: Validation and error reporting

## Current Limitations

- Custom headers are parsed but not yet applied to HTTP requests (planned for future version)
- Request bodies are parsed but not yet sent with requests
- Only basic authentication methods supported

## Future Enhancements

- [ ] Full custom headers support
- [ ] Request body transmission
- [ ] Authentication (Basic, Bearer tokens)
- [ ] Request timeouts configuration
- [ ] JSON response formatting
- [ ] Export results to different formats

## Code Structure

The codebase is organized into multiple modules for better maintainability:

```text
src/
├── main.zig       # Main application logic and CLI interface
├── types.zig      # Data structures (HttpRequest, HttpResult, etc.)
├── colors.zig     # ANSI color constants for terminal output
├── parser.zig     # HTTP file parsing functionality
└── runner.zig     # HTTP request execution logic
```

### Module Overview

- **`types.zig`**: Defines the core data structures including `HttpRequest` and `HttpResult`
- **`colors.zig`**: Contains ANSI color codes for colored terminal output
- **`parser.zig`**: Handles parsing of `.http` files into structured requests
- **`runner.zig`**: Manages HTTP request execution and response handling
- **`main.zig`**: Orchestrates the application flow and provides the CLI interface

This modular structure makes the code easier to understand, test, and extend.

## CI/CD Pipeline

This project uses GitHub Actions for continuous integration and deployment:

### Workflows

- **CI Pipeline** (`build.yml`): Runs on every push and pull request
  - Multi-platform builds (Linux, Windows, macOS)
  - Code formatting checks
  - Unit tests
  - Security scanning with Trivy

- **Release Pipeline** (`release.yml`): Triggered on version tags
  - Cross-platform binary builds
  - Automated GitHub releases
  - Container image publishing to GitHub Container Registry

- **Security Scanning** (`codeql.yml`): Weekly security analysis
  - CodeQL static analysis
  - Dependency vulnerability scanning

- **Dependency Updates** (`dependency-update.yml`): Automated dependency updates
  - Weekly Zig version checks
  - Automated pull requests for updates

### Release Process

1. Update version in relevant files
2. Create and push a git tag: `git tag v1.0.0 && git push origin v1.0.0`
3. GitHub Actions automatically creates a release with binaries
4. Container images are published to `ghcr.io/christianhelle/httprunner`

### Development Workflow

The project follows standard GitHub flow:

1. Fork the repository
2. Create a feature branch
3. Make changes and ensure tests pass
4. Submit a pull request
5. CI checks run automatically
6. Merge after review and approval

## License

This project is open source and available under the MIT License.