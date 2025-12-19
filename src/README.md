# HTTP File Runner

A simple command-line tool written in Rust that parses `.http` files and executes HTTP requests, providing colored output with emojis to indicate success or failure.

## Features

- 🚀 Parse and execute HTTP requests from `.http` files
- 📁 Support for multiple `.http` files in a single run
- 🔍 `--discover` mode to recursively find and run all `.http` files
- 📝 `--verbose` mode for detailed request and response information
- 🎨 `--pretty-json` flag to format JSON payloads in verbose output for improved readability
- 📋 `--log` mode to save all output to a file for analysis and reporting
- 📊 `--report` flag to generate markdown summary reports for test results
- ✅ Color-coded output (green for success, red for failure, yellow for skipped)
- 📊 Summary statistics showing passed/failed/skipped counts (per file and overall)
- 🌐 Support for various HTTP methods (GET, POST, PUT, DELETE, PATCH)
- 📝 **Custom headers support** with full request header implementation
- 🎯 Detailed error reporting with status codes
- 🛡️ Robust error handling for network issues
- 🔒 **Insecure HTTPS support** with `--insecure` flag for development environments
- 🔍 **Response assertions** for status codes, body content, and headers
- 🔧 **Variables support** with substitution in URLs, headers, and request bodies
- 🔧 **Request Variables** for chaining requests and passing data between HTTP calls
- 🔀 **Conditional Execution** with `@dependsOn` and `@if` directives for request dependencies
- ⏱️ **Customizable timeouts** for connection and read operations with flexible time units
- 📋 **Semantic versioning** with git tag and commit information
- 🔍 **Build-time version generation** with automatic git integration

## Installation

```bash
# Install from crates.io
cargo install httprunner
```

For other installation options (including pre-built binaries for Linux, macOS, and Windows), see the [GitHub repository](https://github.com/christianhelle/httprunner).

## Usage

```bash
# Run a single .http file
httprunner <http-file>

# Run with verbose output
httprunner <http-file> --verbose

# Run with verbose output and pretty-printed JSON
httprunner <http-file> --verbose --pretty-json

# Run with insecure HTTPS (accept invalid certificates)
httprunner <http-file> --insecure

# Run and save output to a log file
httprunner <http-file> --log

# Run and generate a markdown summary report
httprunner <http-file> --report

# Run multiple .http files
httprunner <http-file1> <http-file2> [...]

# Discover and run all .http files recursively
httprunner --discover

# Show version information
httprunner --version

# Show help
httprunner --help
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

## Variables

Variables are defined using the `@` syntax and can be referenced using double curly braces `{{variable_name}}`.

### Variable Definition

```http
@hostname=localhost
@port=8080
@protocol=https
```

### Variable Usage

Variables can be referenced in URLs, headers, and request bodies:

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

Variables can be defined using values of other variables:

```http
@hostname=localhost
@port=44320
@host={{hostname}}:{{port}}
@baseUrl=https://{{host}}

GET {{baseUrl}}/api/search/tool
```

## Environment Files

Create a file named `http-client.env.json` to define environment-specific variables:

```json
{
  "dev": {
    "HostAddress": "https://localhost:44320",
    "ApiKey": "dev-api-key-123"
  },
  "prod": {
    "HostAddress": "https://contoso.com",
    "ApiKey": "prod-api-key-789"
  }
}
```

Use the `--env` flag to specify which environment to use:

```bash
httprunner myfile.http --env dev
```

## Insecure HTTPS

For development and testing environments with self-signed certificates, use the `--insecure` flag:

```bash
httprunner myfile.http --insecure
```

⚠️ **Security Warning**: Only use in development/testing environments. Never use in production.

## Request Variables

Request Variables allow you to chain HTTP requests by passing data from one request to another.

### Syntax

```text
{{<request_name>.(request|response).(body|headers).(*|JSONPath|XPath|<header_name>)}}
```

### Authentication Flow Example

```http
# @name authenticate
POST https://httpbin.org/post
Content-Type: application/json

{
  "username": "admin@example.com",
  "password": "secure123",
  "access_token": "jwt_token_here"
}

###

# @name get_data
GET https://httpbin.org/get
Authorization: Bearer {{authenticate.response.body.$.json.access_token}}
```

### Supported Extraction Patterns

**For JSON bodies:**

- `$.property_name` - Extract top-level properties
- `$.nested.property` - Extract nested properties
- `*` - Extract entire body

**For headers:**

- `header_name` - Extract specific header value (case-insensitive)

## Conditional Execution

Execute requests conditionally based on previous request results using `@dependsOn`, `@if`, and `@if-not` directives:

```http
# @name check-user
GET https://api.example.com/user/123

###
# Execute only if check-user returns 200
# @dependsOn check-user
PUT https://api.example.com/user/123

###
# Create if not found (404)
# @if check-user.response.status 404
POST https://api.example.com/user

###
# Update if user exists (NOT 404)
# @if-not check-user.response.status 404
PUT https://api.example.com/user/123
```

## Timeout Configuration

Customize request timeouts using comment directives:

```http
# Read timeout (default: 60 seconds)
# @timeout 600
GET https://example.com/api/long-running

# Connection timeout (default: 30 seconds)
// @connection-timeout 10
GET https://example.com/api

# Time units: ms (milliseconds), s (seconds), m (minutes)
# @timeout 2 m
# @timeout 5000 ms
GET https://example.com/api
```

## Response Assertions

Validate HTTP responses with assertions:

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
```

### Assertion Behavior

- ✅ **Status Code**: Exact match with expected HTTP status code
- ✅ **Response Body**: Checks if response body contains the expected text
- ✅ **Response Headers**: Checks if the specified header exists and contains the expected value
- ⚠️ **Request Success**: A request is considered successful only if all assertions pass

## Output

The tool provides colored output with emojis:

- ✅ **Green**: Successful requests (2xx status codes)
- ❌ **Red**: Failed requests (4xx, 5xx status codes, or connection errors)
- 🚀 **Blue**: Informational messages
- ⚠️ **Yellow**: Warnings and skipped requests

### Example Output

```text
🚀 HTTP File Runner - Processing file: examples/simple.http
==================================================
Found 4 HTTP request(s)

✅ GET https://httpbin.org/status/200 - Status: 200 - 145ms
❌ GET https://httpbin.org/status/404 - Status: 404 - 203ms
✅ GET https://api.github.com/zen - Status: 200 - 98ms
✅ GET https://jsonplaceholder.typicode.com/users/1 - Status: 200 - 112ms

==================================================
File Summary: 3 Passed, 1 Failed, 0 Skipped
```

## Verbose Mode

Use `--verbose` for detailed request and response information, including headers, body content, and timing. Add `--pretty-json` to format JSON payloads for improved readability.

## Report Generation

Generate markdown summary reports with `--report`:

```bash
# Generate timestamped markdown report
httprunner myfile.http --report

# Combine with other flags
httprunner myfile.http --verbose --report
httprunner --discover --report
```

The report includes overall statistics, per-file breakdowns, and detailed request results in an easy-to-read markdown format.

## Logging

Save output to a file with `--log`:

```bash
# Save to default 'log' file
httprunner myfile.http --log

# Save to custom file
httprunner myfile.http --log results.txt
```

## Documentation

For complete documentation, installation options, and more examples, visit the [GitHub repository](https://github.com/christianhelle/httprunner).

## License

MIT License - See [LICENSE](https://github.com/christianhelle/httprunner/blob/main/LICENSE) for details.

---

For more information, check out [christianhelle.com](https://christianhelle.com)

If you find this useful, feel free to [buy me a coffee ☕](https://www.buymeacoffee.com/christianhelle)
