# HTTP File Runner

A command-line tool that parses `.http` files and executes HTTP requests, providing colored output with emojis to indicate success or failure.

## Features

- 🚀 Parse and execute HTTP requests from `.http` files
- 📁 Support for multiple `.http` files in a single run
- 🔍 `--discover` mode to recursively find and run all `.http` files
- 📝 `--verbose` mode for detailed request and response information
- 📋 `--log` mode to save all output to a file for analysis and reporting
- ✅ Color-coded output (green for success, red for failure)
- 📊 Summary statistics showing success/failure counts (per file and overall)
- 🌐 Support for various HTTP methods (GET, POST, PUT, DELETE, PATCH)
- 📝 Custom headers support with full request header implementation
- 🎯 Detailed error reporting with status codes
- 🛡️ Robust error handling for network issues
- 🔒 **Insecure HTTPS support** with `--insecure` flag for development environments
- 🔍 Response assertions for status codes, body content, and headers
- 🔧 Variables support with substitution in URLs, headers, and request bodies
- 🔧 Request Variables for chaining requests and passing data between HTTP calls
- 📋 Semantic versioning with git tag and commit information
- 🔍 Build-time version generation with automatic git integration

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

## Usage

### Basic Commands

```bash
# Run a single .http file
httprunner <http-file>

# Run with verbose output
httprunner <http-file> --verbose

# Run with insecure HTTPS (accept invalid certificates)
httprunner <http-file> --insecure

# Run and save output to a log file
httprunner <http-file> --log

# Run with verbose output and save to a custom log file
httprunner <http-file> --verbose --log results.txt

# Run multiple .http files
httprunner <http-file1> <http-file2> [...]

# Discover and run all .http files recursively
httprunner --discover

# Discover with verbose output and logging
httprunner --discover --verbose --log discovery.log

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

For development and testing environments with self-signed certificates, use the `--insecure` flag to bypass certificate validation:

```bash
# Allow insecure HTTPS connections
httprunner myfile.http --insecure

# Combine with other flags
httprunner myfile.http --insecure --verbose
httprunner --discover --insecure --log results.txt
```

⚠️ **Security Warning**: The `--insecure` flag disables SSL/TLS certificate verification. Use only in development/testing environments. Never use in production.

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
- ⚠️ **Yellow**: Warnings

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
Summary: 3/4 requests succeeded
```

## Verbose Mode

The `--verbose` flag provides detailed information about HTTP requests and responses:

- 📤 **Request Details**: Method, URL, headers, and request body
- 📥 **Response Details**: Status code, duration, response headers, and response body
- ⏱️ **Timing Information**: Response times in milliseconds

## Logging Mode

The `--log` flag enables output logging to a file:

- `--log` without filename: Saves to a file named 'log'
- `--log filename.txt`: Saves to the specified filename
- Works with all other flags: `--verbose --log`, `--discover --log`, etc.

## Error Handling

The tool handles various error conditions gracefully:

- **File not found**: Clear error message with red indicator
- **Invalid URLs**: Proper error reporting
- **Network issues**: Connection timeouts, unknown hosts, etc.
- **Invalid HTTP methods**: Validation and error reporting

## Code Structure

```text
src/
├── main.rs              # Main application entry point
├── cli.rs               # Command-line interface parsing
├── types.rs             # Data structures
├── colors.rs            # Terminal color output
├── parser.rs            # HTTP file parsing
├── runner.rs            # HTTP request execution
├── processor.rs         # Request processing
├── discovery.rs         # Recursive file discovery
├── assertions.rs        # Response assertion validation
├── request_variables.rs # Request chaining
├── environment.rs       # Environment variable handling
├── log.rs               # Logging functionality
└── upgrade.rs           # Self-update feature
```

## License

This project is open source and available under the MIT License.

## Links

- [GitHub Repository](https://github.com/christianhelle/httprunner)
- [Documentation](https://github.com/christianhelle/httprunner)
- [Issue Tracker](https://github.com/christianhelle/httprunner/issues)

---

For more information and updates, visit [christianhelle.com](https://christianhelle.com)
