# HTTP File Runner - Rust Port

This is a Rust port of the HTTP File Runner originally written in Zig.

## Features

- Execute HTTP requests from `.http` files
- Support for multiple HTTP methods (GET, POST, PUT, DELETE, PATCH)
- Variable substitution
- Request chaining with request variables
- Response assertions
- Environment variables support
- File discovery mode
- Verbose logging
- File logging with timestamps

## Building

### Prerequisites

- Rust 1.70 or later
- Git (for version generation)

### Build Commands

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Run with example
cargo run -- examples/simple.http

# Run with verbose mode
cargo run -- examples/simple.http --verbose

# Run discovery mode
cargo run -- --discover
```

## Usage

```bash
# Run a single .http file
httprunner file.http

# Run multiple .http files
httprunner file1.http file2.http

# Run with verbose output
httprunner file.http --verbose

# Run with logging
httprunner file.http --log mylog

# Run with environment
httprunner file.http --env production

# Discover and run all .http files
httprunner --discover

# Show version
httprunner --version

# Upgrade to latest version
httprunner --upgrade

# Show help
httprunner --help
```

## HTTP File Format

```http
# Comments start with #
@variable=value

GET https://api.example.com/endpoint
Header-Name: header-value

{optional body}

EXPECTED_RESPONSE_STATUS 200
EXPECTED_RESPONSE_BODY "expected text"
EXPECTED_RESPONSE_HEADERS "Header: value"
```

## Environment Variables

Create a `http-client.env.json` file:

```json
{
  "dev": {
    "baseUrl": "http://localhost:8080",
    "apiKey": "dev-key"
  },
  "production": {
    "baseUrl": "https://api.example.com",
    "apiKey": "prod-key"
  }
}
```

Use with: `httprunner file.http --env dev`

## Request Variables

Name requests and reference their responses:

```http
# @name login
POST https://api.example.com/login
Content-Type: application/json

{"username": "user", "password": "pass"}

###

# @name getUser
GET https://api.example.com/user
Authorization: Bearer {{login.response.body.$.token}}
```

## Differences from Zig Version

This Rust port aims to maintain feature parity with the original Zig implementation. Key differences:

1. Uses `reqwest` for HTTP requests instead of Zig's standard library
2. Uses `clap` for CLI argument parsing
3. Uses `colored` crate for terminal colors
4. Error handling uses `anyhow` for ergonomic error propagation
5. Some performance characteristics may differ due to Rust's runtime model

## Dependencies

- `reqwest` - HTTP client
- `tokio` - Async runtime
- `serde` / `serde_json` - JSON serialization
- `clap` - CLI argument parsing
- `colored` - Terminal colors
- `anyhow` - Error handling
- `walkdir` - Directory traversal
- `chrono` - Date/time handling (build only)

## License

MIT License - Same as the original Zig version

## Author

Christian Helle

## Links

- Original Zig version: https://github.com/christianhelle/httprunner
- Sponsor: https://github.com/sponsors/christianhelle
- Buy me a coffee: https://www.buymeacoffee.com/christianhelle
