# HTTP File Runner

[![Build Linux](https://github.com/christianhelle/httprunner/actions/workflows/build-linux.yml/badge.svg)](https://github.com/christianhelle/httprunner/actions/workflows/build-linux.yml)
[![Build macOS](https://github.com/christianhelle/httprunner/actions/workflows/build-macos.yml/badge.svg)](https://github.com/christianhelle/httprunner/actions/workflows/build-macos.yml)
[![Build Windows](https://github.com/christianhelle/httprunner/actions/workflows/build-windows.yml/badge.svg)](https://github.com/christianhelle/httprunner/actions/workflows/build-windows.yml)
[![Rust Version](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A simple command-line tool written in Rust that parses `.http` files and executes HTTP requests, providing colored output with emojis to indicate success or failure.

> **Note**: This project was originally written in Zig. The Zig implementation has been moved to a separate repository: [christianhelle/httprunner-zig](https://github.com/christianhelle/httprunner-zig). This repository now contains only the Rust implementation, which is actively maintained and recommended for all use cases.

## Features

- 🚀 Parse and execute HTTP requests from `.http` files
- 📁 Support for multiple `.http` files in a single run
- 🔍 `--discover` mode to recursively find and run all `.http` files
- 📝 `--verbose` mode for detailed request and response information
- 📋 `--log` mode to save all output to a file for analysis and reporting
- ✅ Color-coded output (green for success, red for failure)
- 📊 Summary statistics showing success/failure counts (per file and overall)
- 🌐 Support for various HTTP methods (GET, POST, PUT, DELETE, PATCH)
- 📝 **Custom headers support** with full request header implementation
- 🎯 Detailed error reporting with status codes
- 🛡️ Robust error handling for network issues
- 🔒 **Insecure HTTPS support** with `--insecure` flag for development environments
- 🔍 **Response assertions** for status codes, body content, and headers
- 🔧 **Variables support** with substitution in URLs, headers, and request bodies
- 🔧 **Request Variables** for chaining requests and passing data between HTTP calls
- 📋 **Semantic versioning** with git tag and commit information
- 🔍 **Build-time version generation** with automatic git integration

## Version Information

The application includes comprehensive version information accessible via:

```bash
httprunner --version
# or
httprunner -v
```

This displays:

- Application version (semantic versioning)
- Git tag information
- Git commit hash
- Build timestamp

The version information is automatically generated at build time using git repository data.

## Installation

### Option 1: Quick Install Script (Recommended)

**Linux/macOS:**

```bash
curl -fsSL https://christianhelle.com/httprunner/install | bash
```

**Windows (PowerShell):**

```powershell
irm https://christianhelle.com/httprunner/install.ps1 | iex
```

The install scripts will:

- Automatically detect your platform and architecture
- Download the latest release from GitHub
- Install the binary to an appropriate location
- Add it to your PATH (if desired)

**Custom installation directory:**

```bash
# Linux/macOS
INSTALL_DIR=$HOME/.local/bin curl -fsSL https://christianhelle.com/httprunner/install | bash

# Windows
irm https://christianhelle.com/httprunner/install.ps1 | iex -InstallDir "C:\Tools"
```

### Option 2: Manual Download

Download the latest release for your platform from the [GitHub Releases page](https://github.com/christianhelle/httprunner/releases/latest):

- **Linux x86_64:** `httprunner-linux-x86_64.tar.gz`
- **macOS x86_64:** `httprunner-macos-x86_64.tar.gz`
- **macOS ARM64:** `httprunner-macos-aarch64.tar.gz`
- **Windows x86_64:** `httprunner-windows-x86_64.zip`

Extract the archive and add the binary to your PATH.

### Option 3: Install from Crates.io

If you have Rust tooling installed, you can install httprunner directly from Crates.io:

```bash
cargo install httprunner
```

This will download, compile, and install the latest version of httprunner. The binary will be installed to `~/.cargo/bin/` (or `%USERPROFILE%\.cargo\bin\` on Windows), which should already be in your PATH if you installed Rust via rustup.

### Option 4: Install from Snap Store

```bash
sudo snap install httprunner
```

### Option 5: Build from Source

Make sure you have Rust installed (version 1.70 or later).

```bash
git clone https://github.com/christianhelle/httprunner.git
cd httprunner
cargo build --release
```

The binary will be at `target/release/httprunner` (or `httprunner.exe` on Windows).

### Option 6: Use Docker

The httprunner is available as a Docker image on Docker Hub at `christianhelle/httprunner`.

```bash
# Pull the latest image
docker pull christianhelle/httprunner
```

## Upgrading

### Quick Upgrade

If you have httprunner already installed, you can easily upgrade to the latest version using the built-in upgrade command:

```bash
# Upgrade to the latest version
httprunner --upgrade
```

The upgrade command will:

- Automatically detect your platform (Windows, Linux, macOS)
- Download and run the appropriate install script
- Update httprunner to the latest version available
- Preserve your existing installation location

**What it runs under the hood:**

- **Linux/macOS:** `curl -fsSL https://christianhelle.com/httprunner/install | bash`
- **Windows:** `irm https://christianhelle.com/httprunner/install.ps1 | iex`

After upgrading, you may need to restart your terminal to use the updated version.

### Manual Upgrade

Alternatively, you can always re-run the installation scripts manually:

**Linux/macOS:**

```bash
curl -fsSL https://christianhelle.com/httprunner/install | bash
```

**Windows (PowerShell):**

```powershell
irm https://christianhelle.com/httprunner/install.ps1 | iex
```

## Usage

### If installed via Snap

```bash
# Run a single .http file
httprunner <http-file>

# Run a single .http file with verbose output
httprunner <http-file> --verbose

# Run a single .http file with insecure HTTPS (accept invalid certificates)
httprunner <http-file> --insecure

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

For proper emoji display in PowerShell, set UTF-8 encoding:

```pwsh
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
.\target\release\httprunner.exe <http-file>

# Run with verbose output
.\target\release\httprunner.exe <http-file> --verbose

# Run with insecure HTTPS (accept invalid certificates)
.\target\release\httprunner.exe <http-file> --insecure

# Run and save output to a log file
.\target\release\httprunner.exe <http-file> --log

# Run with verbose output and save to a custom log file
.\target\release\httprunner.exe <http-file> --verbose --log results.txt

# Run multiple files
.\target\release\httprunner.exe examples\simple.http examples\basic.http

# Run multiple files and log output
.\target\release\httprunner.exe examples\simple.http examples\basic.http --log execution.log

# Discover all .http files
.\target\release\httprunner.exe --discover

# Discover all .http files with verbose output
.\target\release\httprunner.exe --discover --verbose

# Discover all .http files and save output to log
.\target\release\httprunner.exe --discover --log discovery.log

# Discover all .http files with verbose output and logging
.\target\release\httprunner.exe --discover --verbose --log detailed_results.txt
```

### Command Line

```bash
# Run a single .http file
./target/release/httprunner <http-file>

# Run a single .http file with verbose output
./target/release/httprunner <http-file> --verbose

# Run a single .http file and save output to a log file
./target/release/httprunner <http-file> --log

# Run a single .http file with verbose output and save to a custom log file
./target/release/httprunner <http-file> --verbose --log results.txt

# Run multiple .http files
./target/release/httprunner <http-file1> <http-file2> [...]

# Run multiple .http files and log output
./target/release/httprunner <http-file1> <http-file2> [...] --log execution.log

# Discover and run all .http files recursively from current directory
./target/release/httprunner --discover

# Discover and run all .http files with verbose output
./target/release/httprunner --discover --verbose

# Discover and run all .http files and save output to log
./target/release/httprunner --discover --log discovery.log

# Discover and run all .http files with verbose output and logging
./target/release/httprunner --discover --verbose --log detailed_results.txt
```

### Examples

```bash
# Test basic functionality
./target/release/httprunner examples/simple.http

# Test basic functionality with verbose output
./target/release/httprunner examples/simple.http --verbose

# Test basic functionality and save output to log
./target/release/httprunner examples/simple.http --log

# Test basic functionality with verbose output and custom log file
./target/release/httprunner examples/simple.http --verbose --log simple_test.log

# Test various APIs
./target/release/httprunner examples/apis.http

# Test various APIs and log results
./target/release/httprunner examples/apis.http --log api_test.log

# Test different HTTP status codes
./target/release/httprunner examples/status-codes.http

# Test different HTTP status codes with verbose logging
./target/release/httprunner examples/status-codes.http --verbose --log status_test.log

# Test basic GET requests
./target/release/httprunner examples/basic.http

# Run multiple files at once
./target/release/httprunner examples/simple.http examples/quick.http

# Run multiple files with verbose output
./target/release/httprunner examples/simple.http examples/quick.http --verbose

# Run multiple files and log output
./target/release/httprunner examples/simple.http examples/quick.http --log multi_test.log

# Run multiple files with verbose output and logging
./target/release/httprunner examples/simple.http examples/quick.http --verbose --log detailed_multi_test.log

# Discover and run all .http files in the project
./target/release/httprunner --discover

# Discover and run all .http files with verbose output
./target/release/httprunner --discover --verbose

# Discover and run all .http files and save output to log
./target/release/httprunner --discover --log discovery.log

# Discover and run all .http files with verbose output and logging
./target/release/httprunner --discover --verbose --log full_discovery.log

# Run all files in a specific directory (using shell globbing)
./target/release/httprunner examples/*.http

# Run all files in a specific directory and log output
./target/release/httprunner examples/*.http --log examples_test.log
```

### If using Docker

```bash
# Run with a single .http file (mount current directory)
docker run -it --mount "type=bind,source=${PWD},target=/app,readonly" christianhelle/httprunner <http-file>

# Run with a single .http file with verbose output
docker run -it --mount "type=bind,source=${PWD},target=/app,readonly" christianhelle/httprunner <http-file> --verbose

# Run with insecure HTTPS (accept invalid certificates)
docker run -it --mount "type=bind,source=${PWD},target=/app,readonly" christianhelle/httprunner <http-file> --insecure

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

## Insecure HTTPS

By default, httprunner validates SSL/TLS certificates and hostnames for secure HTTPS connections. For development environments with self-signed certificates or testing scenarios, you can use the `--insecure` flag to bypass certificate validation.

### Using the --insecure Flag

```bash
# Accept self-signed certificates
httprunner https-endpoints.http --insecure

# Combine with other flags
httprunner https-endpoints.http --insecure --verbose
httprunner https-endpoints.http --insecure --log test.log
```

### What --insecure Does

When the `--insecure` flag is enabled:
- ✅ Accepts invalid SSL/TLS certificates
- ✅ Accepts invalid hostnames
- ✅ Allows connections to servers with self-signed certificates
- ⚠️ **Warning**: Only use in development/testing environments

### Example

```http
# This will fail without --insecure if the certificate is self-signed
GET https://localhost:44320/api/users
Authorization: Bearer {{token}}
```

Run with:
```bash
httprunner api-test.http --insecure
```

**Security Note**: The `--insecure` flag should **only** be used in development and testing environments. Never use it in production as it disables important security checks that protect against man-in-the-middle attacks.

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

## Variables

The HTTP File Runner supports variables to make your .http files more flexible and reusable. Variables are defined using the `@` syntax and can be referenced using double curly braces `{{variable_name}}`.

### Variable Definition

Variables are defined at the beginning of a line with the syntax `@VariableName=Value`:

```http
@hostname=localhost
@port=8080
@protocol=https
```

### Variable Usage

Variables can be referenced in URLs, headers, and request bodies using double curly braces:

```http
@hostname=localhost
@port=44320
GET https://{{hostname}}:{{port}}/

# Request with variable in headers
GET https://{{hostname}}:{{port}}/api/users
Authorization: Bearer {{token}}

# Request with variables in body
POST https://{{hostname}}:{{port}}/api/users
Content-Type: application/json

{
  "host": "{{hostname}}",
  "endpoint": "https://{{hostname}}:{{port}}/profile"
}
```

### Variable Composition

Variables can be defined using values of other variables that were defined earlier in the file:

```http
@hostname=localhost
@port=44320
@host={{hostname}}:{{port}}
@baseUrl=https://{{host}}

GET {{baseUrl}}/api/search/tool
```

**Note:** Variables must be defined before they can be used. The order of definition matters.

## Environment Files

To give variables different values in different environments, create a file named `http-client.env.json`. This file should be located in the same directory as the `.http` file or in one of its parent directories.

### Environment File Format

The environment file is a JSON file that contains one or more named environments. Here's an example:

```json
{
  "dev": {
    "HostAddress": "https://localhost:44320",
    "ApiKey": "dev-api-key-123",
    "Environment": "development"
  },
  "staging": {
    "HostAddress": "https://staging.contoso.com",
    "ApiKey": "staging-api-key-456",
    "Environment": "staging"
  },
  "prod": {
    "HostAddress": "https://contoso.com",
    "ApiKey": "prod-api-key-789", 
    "Environment": "production"
  }
}
```

### Using Environment Variables

Variables from an environment file are referenced the same way as other variables:

```http
# This will use the HostAddress from the specified environment
GET {{HostAddress}}/api/search/tool
Authorization: Bearer {{ApiKey}}
X-Environment: {{Environment}}
```

### Specifying Environment

Use the `--env` flag to specify which environment to use:

```bash
# Use development environment
httprunner myfile.http --env dev

# Use production environment  
httprunner myfile.http --env prod
```

### Variable Override Behavior

If a variable is defined in both the `.http` file and the environment file:

- **Environment variables** are loaded first
- **Variables in .http file** override environment variables with the same name
- This allows you to have environment defaults while still being able to override them per request file

## Request Variables

Request Variables allow you to chain HTTP requests by passing data from one request to another within the same `.http` file. This feature enables powerful workflows like authentication flows, data extraction, and response chaining.

### Request Variable Syntax

The syntax for request variables follows this pattern:

```text
{{<request_name>.(request|response).(body|headers).(*|JSONPath|XPath|<header_name>)}}
```

Where:

- `request_name`: The name of a previous request (defined with `# @name`)
- `request|response`: Whether to extract from the request or response
- `body|headers`: Whether to extract from body or headers  
- `*|JSONPath|XPath|header_name`: The extraction path

### Authentication Flow Example

```http
# @name authenticate
POST https://httpbin.org/post
Content-Type: application/json

{
  "username": "admin@example.com",
  "password": "secure123",
  "access_token": "jwt_token_here",
  "refresh_token": "refresh_jwt_here",
  "user_id": "admin_001",
  "role": "administrator"
}

###

# @name get_admin_data
GET https://httpbin.org/get
Authorization: Bearer {{authenticate.response.body.$.json.access_token}}
X-User-Role: {{authenticate.response.body.$.json.role}}
X-User-ID: {{authenticate.response.body.$.json.user_id}}

###

# @name create_audit_log
POST https://httpbin.org/post
Content-Type: application/json

{
  "action": "admin_data_access",
  "user_id": "{{authenticate.response.body.$.json.user_id}}",
  "original_request": {
    "username": "{{authenticate.request.body.$.username}}",
    "timestamp": "2025-07-01T21:16:46Z"
  },
  "response_content_type": "{{get_admin_data.response.headers.Content-Type}}"
}
```

### Supported Extraction Patterns

**For JSON bodies:**

- `$.property_name` - Extract top-level properties
- `$.nested.property` - Extract nested properties
- `$.json.property` - Extract from "json" field (like httpbin.org responses)
- `*` - Extract entire body

**For headers:**

- `header_name` - Extract specific header value (case-insensitive)

**For request data:**

- Same patterns as response, but extracts from the original request data

### Request Variable Benefits

- **Authentication Workflows**: Extract tokens from login responses
- **Data Chaining**: Pass IDs or data between sequential requests
- **Dynamic Headers**: Use response headers in subsequent requests
- **Request Auditing**: Reference original request data in follow-up calls
- **API Testing**: Create comprehensive test flows with dependent requests

**Note:** Request variables can only reference requests that appear earlier in the same `.http` file and have been named with `# @name`.

## Response Assertions

The HTTP File Runner supports assertions to validate HTTP responses. You can assert on status codes, response body content, and response headers.

### Assertion Syntax

- **`EXPECTED_RESPONSE_STATUS`** - Assert on HTTP status code
- **`EXPECTED_RESPONSE_BODY`** - Assert that response body contains specific text
- **`EXPECTED_RESPONSE_HEADERS`** - Assert that response headers contain specific header-value pairs

### Assertion Examples

```http
# Status code assertion
GET https://httpbin.org/status/200

EXPECTED_RESPONSE_STATUS 200

# Status code and response body assertion
GET https://httpbin.org/status/404

EXPECTED_RESPONSE_STATUS 404
EXPECTED_RESPONSE_BODY "Not Found"

# Response header assertion
GET https://httpbin.org/json

EXPECTED_RESPONSE_STATUS 200
EXPECTED_RESPONSE_HEADERS "Content-Type: application/json"

# Multiple assertions on the same request
GET https://httpbin.org/json

EXPECTED_RESPONSE_STATUS 200
EXPECTED_RESPONSE_BODY "slideshow"
EXPECTED_RESPONSE_HEADERS "Content-Type: application/json"
```

### Assertion Behavior

- ✅ **Status Code**: Exact match with expected HTTP status code
- ✅ **Response Body**: Checks if response body contains the expected text (substring match)
- ✅ **Response Headers**: Checks if the specified header exists and contains the expected value (substring match)
- 🔍 **Assertion Results**: Detailed output shows which assertions passed/failed
- ⚠️ **Request Success**: A request is considered successful only if all assertions pass (in addition to 2xx status code)

When assertions are present, the HTTP runner will:

1. Always capture response headers and body (even in non-verbose mode)
2. Evaluate all assertions against the response
3. Display detailed assertion results
4. Mark the request as failed if any assertion fails

### Supported Features

- **Methods**: GET, POST, PUT, DELETE, PATCH
- **Headers**: Key-value pairs separated by `:` (fully supported and sent with requests)
- **Body**: Content after headers (separated by empty line)
- **Comments**: Lines starting with `#`

## Example Files

The `examples/` directory contains several sample `.http` files:

- **`simple.http`** - Basic requests for quick testing (4 requests)
- **`basic.http`** - Various GET requests to different websites
- **`apis.http`** - Requests to public APIs (7 requests)
- **`status-codes.http`** - Tests different HTTP status codes (15 requests)
- **`request-variables.http`** - Demonstrates request chaining with variables (5 requests)
- **`variables.http`** - Shows variable usage and environment files
- **`comprehensive.http`** - Complete feature demonstration

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
HTTP File Runner v0.1.9
Usage:
  httprunner <http-file> [http-file2] [...] [--verbose] [--log [filename]] [--env <environment>] [--insecure]
  httprunner [--verbose] [--log [filename]] [--env <environment>] [--insecure] --discover
  httprunner --version | -v
  httprunner --upgrade
  httprunner --help | -h

Arguments:
  <http-file>    One or more .http files to process
  --discover     Recursively discover and process all .http files from current directory
  --verbose      Show detailed HTTP request and response information
  --log [file]   Log output to a file (defaults to 'log' if no filename is specified)
  --env <env>    Specify environment name to load variables from http-client.env.json
  --insecure     Allow insecure HTTPS connections (accept invalid certificates and hostnames)
  --version, -v  Show version information
  --upgrade      Update httprunner to the latest version
  --help, -h     Show this help message
```

### Practical Logging Examples

Here are common scenarios for using the `--log` functionality:

**Basic Logging:**

```bash
# Save output to default 'log' file
httprunner examples/simple.http --log

# Save output to custom file
httprunner examples/apis.http --log api_test_results.txt
```

**Verbose Logging for Debugging:**

```bash
# Detailed logging for debugging API issues
httprunner examples/status-codes.http --verbose --log debug_session.log

# Log discovery results with full details
httprunner --discover --verbose --log full_discovery.log
```

**CI/CD Integration:**

```bash
# Generate test reports for build systems
httprunner --discover --log test_report_$(date +%Y%m%d_%H%M%S).log

# Daily API health checks
httprunner examples/apis.http --verbose --log daily_health_check.log
```

**Performance Monitoring:**

```bash
# Track API performance over time
httprunner examples/comprehensive.http --verbose --log performance_$(date +%Y%m%d).log

# Load testing documentation
httprunner examples/*.http --log load_test_results.log
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

- Only basic authentication methods supported

## Future Enhancements

- [ ] Authentication (Basic, Bearer tokens)
- [ ] Request timeouts configuration
- [ ] JSON response formatting
- [ ] Export results to different formats

## Code Structure

The codebase is organized into multiple modules for better maintainability:

```text
src/
├── main.rs              # Main application entry point
├── cli.rs               # Command-line interface parsing
├── types.rs             # Data structures (HttpRequest, HttpResult, etc.)
├── colors.rs            # Terminal color output
├── parser.rs            # HTTP file parsing functionality
├── runner.rs            # HTTP request execution logic
├── processor.rs         # Request processing and output management
├── discovery.rs         # Recursive .http file discovery
├── assertions.rs        # Response assertion validation
├── request_variables.rs # Request chaining and variable extraction
├── environment.rs       # Environment variable handling
├── log.rs               # Logging functionality
└── upgrade.rs           # Self-update feature
```

### Module Overview

- **`main.rs`**: Application entry point that orchestrates the overall workflow
- **`cli.rs`**: Handles command-line argument parsing using `clap`
- **`types.rs`**: Defines the core data structures including `HttpRequest` and `HttpResult`
- **`colors.rs`**: Contains color output using the `colored` crate
- **`parser.rs`**: Handles parsing of `.http` files into structured requests
- **`runner.rs`**: Manages HTTP request execution using `reqwest`
- **`processor.rs`**: Processes requests, manages logging, and handles output formatting
- **`discovery.rs`**: Implements recursive file system traversal using `walkdir`
- **`assertions.rs`**: Validates response assertions (status, body, headers)
- **`request_variables.rs`**: Handles request chaining and JSONPath extraction
- **`environment.rs`**: Loads and processes environment files
- **`log.rs`**: Manages file logging with timestamps
- **`upgrade.rs`**: Implements self-update functionality

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

- **Dependency Updates**: Automated dependency updates via Renovate
  - Automated pull requests for Cargo dependency updates

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

## Installer Scripts

The project includes installer scripts for easy deployment:

- `install.sh` - Bash installer script for Linux and macOS
- `install.ps1` - PowerShell installer script for Windows
- Both scripts automatically detect platform/architecture and download the latest release
- Scripts are deployed to GitHub Pages and accessible at:
  - <https://christianhelle.com/httprunner/install>
  - <https://christianhelle.com/httprunner/install.ps1>

### Testing the Installer Scripts

```bash
# Test the installer scripts locally
./test-installer.sh
```

## Development

### Prerequisites

- Rust 1.70 or later (https://rustup.rs/)
- Git (for version generation)

### Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Run with example
cargo run -- examples/simple.http

# Run with verbose mode
cargo run -- examples/simple.http --verbose
```

### Dev Containers

For the easiest development experience, this repository includes a dev container configuration that provides a pre-configured environment with Rust and VS Code extensions.

**GitHub Codespaces:**
1. Open the repository on GitHub
2. Click the green "Code" button → "Codespaces" → "Create codespace on main"
3. Wait for the environment to set up automatically
4. Start coding! 🚀

**Local Development with VS Code:**
1. Install the [Dev Containers extension](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-containers)
2. Clone this repository: `git clone https://github.com/christianhelle/httprunner.git`
3. Open in VS Code: `code httprunner`
4. When prompted, click "Reopen in Container" or use Command Palette: "Dev Containers: Reopen in Container"

**What's included:**
- Rust toolchain (stable) pre-installed
- PowerShell Core for build scripts
- VS Code Rust extensions (rust-analyzer)
- All dependencies ready for development

### Manual Setup

For development, testing, and debugging without dev containers:

- Use `cargo build` for debug builds with symbols
- Use `cargo build --release` for optimized release builds
- Run tests with `cargo test`
- Format code with `cargo fmt`
- Lint code with `cargo clippy`

### Debugging Tips

- Use `println!()` or the `log` crate for logging
- Use VS Code's Rust debugging with rust-analyzer extension
- Use `cargo test -- --nocapture` to see test output
- Check `target/debug/` or `target/release/` for build artifacts

### Testing

Tests are located in the same files as the code (Rust convention) and can be run with:

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

## Legacy Zig Implementation

The original Zig implementation has been moved to a separate repository: [christianhelle/httprunner-zig](https://github.com/christianhelle/httprunner-zig). The Zig version is no longer maintained in this repository. Please use the Rust implementation for all projects.

## License

This project is open source and available under the MIT License

#

For tips and tricks on software development, check out [my blog](https://christianhelle.com)

If you find this useful and feel a bit generous then feel free to [buy me a coffee ☕](https://www.buymeacoffee.com/christianhelle)
