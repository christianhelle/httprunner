# HTTP File Runner

[![Build Linux](https://github.com/christianhelle/httprunner/actions/workflows/build-linux.yml/badge.svg)](https://github.com/christianhelle/httprunner/actions/workflows/build-linux.yml)
[![Build macOS](https://github.com/christianhelle/httprunner/actions/workflows/build-macos.yml/badge.svg)](https://github.com/christianhelle/httprunner/actions/workflows/build-macos.yml)
[![Build Windows](https://github.com/christianhelle/httprunner/actions/workflows/build-windows.yml/badge.svg)](https://github.com/christianhelle/httprunner/actions/workflows/build-windows.yml)

A simple command-line tool written in Zig that parses `.http` files and executes HTTP requests, providing colored output with emojis to indicate success or failure.

## Features

- 🚀 Parse and execute HTTP requests from `.http` files
- 📁 Support for multiple `.http` files in a single run
- 🔍 `--discover` mode to recursively find and run all `.http` files
- 📝 `--verbose` mode for detailed request and response information
- 📋 `--log` mode to save all output to a file for analysis and reporting
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

### Option 3: Use Docker

The httprunner is available as a Docker image on Docker Hub at `christianhelle/httprunner`.

```bash
# Pull the latest image
docker pull christianhelle/httprunner
```

## Usage

### If installed via Snap

```bash
# Run a single .http file
httprunner <http-file>

# Run a single .http file with verbose output
httprunner <http-file> --verbose

# Run a single .http file and save output to a log file
httprunner <http-file> --log

# Run a single .http file with verbose output and save to a custom log file
httprunner <http-file> --verbose --log results.txt

# Run multiple .http files
httprunner <http-file1> <http-file2> [...]

# Run multiple .http files and log output
httprunner <http-file1> <http-file2> [...] --log execution.log

# Discover and run all .http files recursively
httprunner --discover

# Discover and run all .http files with verbose output
httprunner --discover --verbose

# Discover and run all .http files and save output to log
httprunner --discover --log discovery.log

# Discover and run all .http files with verbose output and logging
httprunner --discover --verbose --log detailed_results.txt
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

# Run with verbose output
.\zig-out\bin\httprunner.exe <http-file> --verbose

# Run and save output to a log file
.\zig-out\bin\httprunner.exe <http-file> --log

# Run with verbose output and save to a custom log file
.\zig-out\bin\httprunner.exe <http-file> --verbose --log results.txt

# Run multiple files
.\zig-out\bin\httprunner.exe examples\simple.http examples\basic.http

# Run multiple files and log output
.\zig-out\bin\httprunner.exe examples\simple.http examples\basic.http --log execution.log

# Discover all .http files
.\zig-out\bin\httprunner.exe --discover

# Discover all .http files with verbose output
.\zig-out\bin\httprunner.exe --discover --verbose

# Discover all .http files and save output to log
.\zig-out\bin\httprunner.exe --discover --log discovery.log

# Discover all .http files with verbose output and logging
.\zig-out\bin\httprunner.exe --discover --verbose --log detailed_results.txt
```

### Command Line

```bash
# Run a single .http file
./zig-out/bin/httprunner <http-file>

# Run a single .http file with verbose output
./zig-out/bin/httprunner <http-file> --verbose

# Run a single .http file and save output to a log file
./zig-out/bin/httprunner <http-file> --log

# Run a single .http file with verbose output and save to a custom log file
./zig-out/bin/httprunner <http-file> --verbose --log results.txt

# Run multiple .http files
./zig-out/bin/httprunner <http-file1> <http-file2> [...]

# Run multiple .http files and log output
./zig-out/bin/httprunner <http-file1> <http-file2> [...] --log execution.log

# Discover and run all .http files recursively from current directory
./zig-out/bin/httprunner --discover

# Discover and run all .http files with verbose output
./zig-out/bin/httprunner --discover --verbose

# Discover and run all .http files and save output to log
./zig-out/bin/httprunner --discover --log discovery.log

# Discover and run all .http files with verbose output and logging
./zig-out/bin/httprunner --discover --verbose --log detailed_results.txt
```

### Examples

```bash
# Test basic functionality
./zig-out/bin/httprunner examples/simple.http

# Test basic functionality with verbose output
./zig-out/bin/httprunner examples/simple.http --verbose

# Test basic functionality and save output to log
./zig-out/bin/httprunner examples/simple.http --log

# Test basic functionality with verbose output and custom log file
./zig-out/bin/httprunner examples/simple.http --verbose --log simple_test.log

# Test various APIs
./zig-out/bin/httprunner examples/apis.http

# Test various APIs and log results
./zig-out/bin/httprunner examples/apis.http --log api_test.log

# Test different HTTP status codes
./zig-out/bin/httprunner examples/status-codes.http

# Test different HTTP status codes with verbose logging
./zig-out/bin/httprunner examples/status-codes.http --verbose --log status_test.log

# Test basic GET requests
./zig-out/bin/httprunner examples/basic.http

# Run multiple files at once
./zig-out/bin/httprunner examples/simple.http examples/quick.http

# Run multiple files with verbose output
./zig-out/bin/httprunner examples/simple.http examples/quick.http --verbose

# Run multiple files and log output
./zig-out/bin/httprunner examples/simple.http examples/quick.http --log multi_test.log

# Run multiple files with verbose output and logging
./zig-out/bin/httprunner examples/simple.http examples/quick.http --verbose --log detailed_multi_test.log

# Discover and run all .http files in the project
./zig-out/bin/httprunner --discover

# Discover and run all .http files with verbose output
./zig-out/bin/httprunner --discover --verbose

# Discover and run all .http files and save output to log
./zig-out/bin/httprunner --discover --log discovery.log

# Discover and run all .http files with verbose output and logging
./zig-out/bin/httprunner --discover --verbose --log full_discovery.log

# Run all files in a specific directory (using shell globbing)
./zig-out/bin/httprunner examples/*.http

# Run all files in a specific directory and log output
./zig-out/bin/httprunner examples/*.http --log examples_test.log
```

### If using Docker

```bash
# Run with a single .http file (mount current directory)
docker run -it --mount "type=bind,source=${PWD},target=/app,readonly" christianhelle/httprunner <http-file>

# Run with a single .http file with verbose output
docker run -it --mount "type=bind,source=${PWD},target=/app,readonly" christianhelle/httprunner <http-file> --verbose

# Run with a single .http file and save output to log
docker run -it --mount "type=bind,source=${PWD},target=/app,readonly" christianhelle/httprunner <http-file> --log

# Run with a single .http file with verbose output and custom log file
docker run -it --mount "type=bind,source=${PWD},target=/app,readonly" christianhelle/httprunner <http-file> --verbose --log results.txt

# Run multiple .http files
docker run -it --mount "type=bind,source=${PWD},target=/app,readonly" christianhelle/httprunner <http-file1> <http-file2>

# Run multiple .http files and log output
docker run -it --mount "type=bind,source=${PWD},target=/app,readonly" christianhelle/httprunner <http-file1> <http-file2> --log execution.log

# Discover and run all .http files in current directory
docker run -it --mount "type=bind,source=${PWD},target=/app,readonly" christianhelle/httprunner --discover

# Discover and run all .http files with verbose output
docker run -it --mount "type=bind,source=${PWD},target=/app,readonly" christianhelle/httprunner --discover --verbose

# Discover and run all .http files and save output to log
docker run -it --mount "type=bind,source=${PWD},target=/app,readonly" christianhelle/httprunner --discover --log discovery.log

# Discover and run all .http files with verbose output and logging
docker run -it --mount "type=bind,source=${PWD},target=/app,readonly" christianhelle/httprunner --discover --verbose --log full_discovery.log

# Alternative: Create an alias for easier usage
alias httprunner='docker run -it --mount "type=bind,source=${PWD},target=/app,readonly" christianhelle/httprunner'
httprunner --discover
httprunner examples/simple.http --verbose
httprunner examples/simple.http --log test.log
httprunner examples/simple.http --verbose --log detailed_test.log
httprunner examples/simple.http
```

**Note**: The Docker container mounts your current directory as `/app` in read-only mode to access your `.http` files. Make sure your `.http` files are in the current directory or subdirectories.

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

### Verbose Mode

The `--verbose` flag provides detailed information about HTTP requests and responses, including headers and response bodies. This is useful for debugging and detailed analysis of API interactions.

**What verbose mode shows:**

- 📤 **Request Details**: Method, URL, headers, and request body
- 📥 **Response Details**: Status code, duration, response headers, and response body
- ⏱️ **Timing Information**: Response times in milliseconds

### Logging Mode

The `--log` flag enables output logging to a file, which is essential for:

- **Automation & CI/CD**: Save test results for build reports and analysis
- **Debugging**: Preserve detailed output for later review
- **Documentation**: Generate test reports and API documentation
- **Monitoring**: Track API performance and reliability over time
- **Auditing**: Keep records of API testing activities

**How to use logging:**

- `--log` without filename: Saves to a file named 'log' in the current directory
- `--log filename.txt`: Saves to the specified filename
- Works with all other flags: `--verbose --log`, `--discover --log`, etc.
- Combines with verbose mode for detailed logged output

**Log file contents include:**

- All terminal output (colored text is preserved)
- HTTP request and response details (when using --verbose)
- Success/failure indicators with emojis
- Summary statistics
- Error messages and diagnostics
- Timestamps and execution duration

### Command Line Help

When running httprunner without any arguments, the following help text is displayed:

```text
Usage:
  httprunner <http-file> [http-file2] [...] [--verbose] [--log [filename]] 
  httprunner [--verbose] [--log [filename]] --discover

Arguments:
  <http-file>    One or more .http files to process
  --discover     Recursively discover and process all .http files from current directory
  --verbose      Show detailed HTTP request and response information
  --log [file]   Log output to a file (defaults to 'log' if no filename is specified)
```

### Practical Logging Examples

Here are common scenarios for using the `--log` functionality:

**Basic Logging:**

```bash
# Save output to default 'log' file
./zig-out/bin/httprunner examples/simple.http --log

# Save output to custom file
./zig-out/bin/httprunner examples/apis.http --log api_test_results.txt
```

**Verbose Logging for Debugging:**

```bash
# Detailed logging for debugging API issues
./zig-out/bin/httprunner examples/status-codes.http --verbose --log debug_session.log

# Log discovery results with full details
./zig-out/bin/httprunner --discover --verbose --log full_discovery.log
```

**CI/CD Integration:**

```bash
# Generate test reports for build systems
./zig-out/bin/httprunner --discover --log test_report_$(date +%Y%m%d_%H%M%S).log

# Daily API health checks
./zig-out/bin/httprunner examples/apis.http --verbose --log daily_health_check.log
```

**Performance Monitoring:**

```bash
# Track API performance over time
./zig-out/bin/httprunner examples/comprehensive.http --verbose --log performance_$(date +%Y%m%d).log

# Load testing documentation
./zig-out/bin/httprunner examples/*.http --log load_test_results.log
```

**Example Log File Output:**

When using `--log`, the log file will contain the exact same output as displayed in the terminal:

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

When combined with `--verbose`, the log file includes full request and response details, making it invaluable for debugging and documentation purposes.

### Verbose Mode Output

When using `--verbose`, you'll see detailed request and response information:

```text
🚀 HTTP File Runner - Processing file: examples/simple.http
==================================================
Found 4 HTTP request(s)

📤 Request Details:
Method: GET
URL: https://httpbin.org/status/200
------------------------------

✅ GET https://httpbin.org/status/200 - Status: 200 - 145ms

📥 Response Details:
Status: 200
Duration: 145ms
Headers:
  content-type: text/html; charset=utf-8
  content-length: 0
  server: gunicorn/19.9.0
  access-control-allow-origin: *
  access-control-allow-credentials: true
Body:

------------------------------

📤 Request Details:
Method: GET
URL: https://httpbin.org/status/404
------------------------------

❌ GET https://httpbin.org/status/404 - Status: 404 - 203ms

📥 Response Details:
Status: 404
Duration: 203ms
Headers:
  content-type: text/html; charset=utf-8
  content-length: 0
  server: gunicorn/19.9.0
  access-control-allow-origin: *
  access-control-allow-credentials: true
Body:

------------------------------

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
- Only basic authentication methods supported

## Future Enhancements

- [ ] Full custom headers support
- [ ] Authentication (Basic, Bearer tokens)
- [ ] Request timeouts configuration
- [ ] JSON response formatting
- [ ] Export results to different formats

## Code Structure

The codebase is organized into multiple modules for better maintainability:

```text
src/
├── main.zig       # Main application entry point and orchestration
├── cli.zig        # Command-line interface parsing and options handling
├── types.zig      # Data structures (HttpRequest, HttpResult, etc.)
├── colors.zig     # ANSI color constants for terminal output
├── parser.zig     # HTTP file parsing functionality
├── runner.zig     # HTTP request execution logic
├── processor.zig  # Request processing and output management
└── discovery.zig  # Recursive .http file discovery functionality
```

### Module Overview

- **`main.zig`**: Application entry point that orchestrates the overall workflow
- **`cli.zig`**: Handles command-line argument parsing and CLI options management
- **`types.zig`**: Defines the core data structures including `HttpRequest` and `HttpResult`
- **`colors.zig`**: Contains ANSI color codes for colored terminal output
- **`parser.zig`**: Handles parsing of `.http` files into structured requests
- **`runner.zig`**: Manages HTTP request execution and response handling
- **`processor.zig`**: Processes requests, manages logging, and handles output formatting
- **`discovery.zig`**: Implements recursive file system traversal to discover `.http` files

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