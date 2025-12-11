# Quick Start Guide - HTTP File Runner (Rust)

> **Note**: This is the Rust implementation. The original Zig implementation has been deprecated due to limitations in Zig's HTTP/HTTPS client that prevented configuring insecure HTTPS calls necessary for development environments with self-signed certificates.

## Installation

### Option 1: From Crates.io (Recommended for Rust users)

```bash
cargo install httprunner
```

The binary will be installed to `~/.cargo/bin/` (or `%USERPROFILE%\.cargo\bin\` on Windows).

### Option 2: From Source

```bash
git clone https://github.com/christianhelle/httprunner.git
cd httprunner
cargo build --release
# Binary will be at: target/release/httprunner
```

### Option 3: Install Locally

```bash
cargo install --path .
```

### Option 4: Quick Install Scripts

**Linux/macOS:**
```bash
curl -fsSL https://christianhelle.com/httprunner/install | bash
```

**Windows (PowerShell):**
```powershell
irm https://christianhelle.com/httprunner/install.ps1 | iex
```

## Basic Usage

### 1. Create a Simple HTTP File

Create `example.http`:

```http
# Simple GET request
GET https://httpbin.org/get

###

# POST request with JSON body
POST https://httpbin.org/post
Content-Type: application/json

{
  "name": "John Doe",
  "email": "john@example.com"
}
```

### 2. Run the HTTP File

```bash
httprunner example.http
```

Expected output:
```
🚀 HTTP File Runner - Processing file: example.http
==================================================
Found 2 HTTP request(s)

✅ GET https://httpbin.org/get - Status: 200 - 123ms
✅ POST https://httpbin.org/post - Status: 200 - 145ms

==================================================
File Summary: 2 Passed, 0 Failed, 0 Skipped
```

### 3. Use Variables

Create `with-variables.http`:

```http
@baseUrl=https://api.github.com
@username=octocat

GET {{baseUrl}}/users/{{username}}
```

Run it:
```bash
httprunner with-variables.http
```

### 4. Use Environment Files

Create `http-client.env.json`:

```json
{
  "dev": {
    "baseUrl": "http://localhost:3000",
    "apiKey": "dev-key"
  },
  "prod": {
    "baseUrl": "https://api.production.com",
    "apiKey": "prod-key"
  }
}
```

Create `api-test.http`:

```http
GET {{baseUrl}}/api/status
Authorization: Bearer {{apiKey}}
```

Run with environment:
```bash
httprunner api-test.http --env dev
```

### 5. Add Assertions

Create `with-assertions.http`:

```http
GET https://httpbin.org/status/200
EXPECTED_RESPONSE_STATUS 200

###

GET https://httpbin.org/json
EXPECTED_RESPONSE_BODY "slideshow"
EXPECTED_RESPONSE_HEADERS "Content-Type: application/json"
```

### 6. Chain Requests with Variables

Create `chained-requests.http`:

```http
# @name login
POST https://httpbin.org/post
Content-Type: application/json

{
  "username": "admin",
  "password": "secret"
}

###

# @name getUser
GET https://httpbin.org/get
Authorization: Bearer {{login.response.body.$.token}}
```

### 7. Verbose Mode

See detailed request and response information:

```bash
httprunner example.http --verbose
```

Output includes:
- Request headers
- Request body
- Response status
- Response headers
- Response body

### 8. Pretty-Print JSON

Format JSON payloads in verbose output for better readability:

```bash
# Enable pretty-printed JSON (requires --verbose)
httprunner example.http --verbose --pretty-json
```

This will automatically format any JSON content in request and response bodies:

```http
# Example: This POST request will show pretty-printed JSON in verbose mode
POST https://httpbin.org/post
Content-Type: application/json

{
  "user": {
    "name": "John Doe",
    "email": "john@example.com",
    "preferences": {
      "theme": "dark",
      "notifications": true
    }
  }
}
```

With `--verbose --pretty-json`, the JSON will be displayed with proper indentation and formatting, making nested structures much easier to read and debug.

### 9. File Logging

Log all output to a file:

```bash
# Default log filename with timestamp
httprunner example.http --log

# Custom log filename
httprunner example.http --log mytest
```

### 10. Discovery Mode

Automatically find and run all .http files:

```bash
httprunner --discover
```

This will:
- Recursively search for .http files
- Display all found files
- Execute them in sequence

### 11. Multiple Files

Process multiple files at once:

```bash
httprunner file1.http file2.http file3.http
```

## Common Patterns

### API Testing

```http
# Test authentication
POST {{baseUrl}}/auth/login
Content-Type: application/json

{"username": "test", "password": "test123"}

EXPECTED_RESPONSE_STATUS 200
EXPECTED_RESPONSE_BODY "token"

###

# Test protected endpoint
GET {{baseUrl}}/api/protected
Authorization: Bearer {{token}}

EXPECTED_RESPONSE_STATUS 200
```

### Different HTTP Methods

```http
# GET
GET https://httpbin.org/get

###

# POST
POST https://httpbin.org/post
Content-Type: application/json

{"key": "value"}

###

# PUT
PUT https://httpbin.org/put
Content-Type: application/json

{"key": "updated"}

###

# DELETE
DELETE https://httpbin.org/delete

###

# PATCH
PATCH https://httpbin.org/patch
Content-Type: application/json

{"key": "patched"}
```

### Testing Different Status Codes

```http
GET https://httpbin.org/status/200
EXPECTED_RESPONSE_STATUS 200

###

GET https://httpbin.org/status/404
EXPECTED_RESPONSE_STATUS 404

###

GET https://httpbin.org/status/500
EXPECTED_RESPONSE_STATUS 500
```

## Tips and Tricks

1. **Use Comments**: Start lines with `#` or `//` for comments
2. **Separate Requests**: Use `###` to separate multiple requests
3. **Variable Scope**: Variables defined at file level are available to all requests
4. **Request Names**: Use `# @name requestName` to name requests for chaining
5. **JSONPath**: Use `$.property.nested` for extracting JSON values
6. **Headers**: Multiple headers can be specified, one per line after the request line
7. **Pretty JSON**: Use `--verbose --pretty-json` to format JSON payloads for easier reading and debugging

## Troubleshooting

### Request Timeout
If requests timeout, they use a 30-second timeout by default. For longer requests, you may need to modify the timeout in the source code.

### SSL/TLS Errors
Ensure your system has up-to-date root certificates. The Rust version uses the system's certificate store.

### Variable Not Found
If `{{variable}}` is not substituted, check:
- Variable is defined before use
- Variable name matches exactly
- For request variables, ensure the referenced request has executed

### Connection Refused
This usually means:
- The server is not running (for localhost)
- The URL is incorrect
- Firewall is blocking the connection

## Next Steps

1. Explore the example files in `../examples/`
2. Read the full README.md for detailed documentation
3. Check PORT_SUMMARY.md for implementation details
4. Report issues at: https://github.com/christianhelle/httprunner/issues

## Support

If you find this tool useful:
- ⭐ Star the project on GitHub
- 💖 Sponsor: https://github.com/sponsors/christianhelle
- ☕ Buy me a coffee: https://www.buymeacoffee.com/christianhelle
