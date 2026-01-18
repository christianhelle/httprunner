# httprunner-lib

A powerful Rust library for parsing and executing HTTP requests from `.http` files. This is the core library that powers the `httprunner` CLI tool.

[![Rust Version](https://img.shields.io/badge/rust-1.92-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Overview

`httprunner-lib` provides a complete solution for working with `.http` files - a simple text-based format for defining HTTP requests. It handles parsing, variable substitution, request execution, and response validation.

## Features

- 🚀 Parse and execute HTTP requests from `.http` files
- 🌐 Support for all standard HTTP methods (GET, POST, PUT, DELETE, PATCH, etc.)
- 📝 Custom headers and request bodies
- 🔧 **Variables** with substitution in URLs, headers, and bodies
- 🎲 **Built-in functions** for dynamic value generation (`guid()`, `string()`, `number()`, `base64_encode()`)
- 🔗 **Request variables** for chaining requests and passing data between calls
- 🔍 **Response assertions** for status codes, body content, and headers
- 🔀 **Conditional execution** with `@dependsOn` and `@if` directives
- ⏱️ **Customizable timeouts** for connection and read operations
- 🔒 **Insecure HTTPS support** for development environments
- 🌍 **Environment files** support for different deployment environments
- 📁 Recursive `.http` file discovery
- 🛡️ Robust error handling for network issues

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
httprunner-lib = "0.1.0"
```

## Quick Start

### Low-level API (Parse and Execute)

```rust
use httprunner_lib::{parser::parse_http_file, runner::execute_http_request};

fn main() -> anyhow::Result<()> {
    // Parse an .http file
    let requests = parse_http_file("example.http", None)?;
    
    // Execute the first request
    if let Some(request) = requests.first() {
        let result = execute_http_request(request, false, false)?;
        println!("Status: {}", result.status_code);
        println!("Body: {}", result.body);
    }
    
    Ok(())
}
```

### High-level API (Process Multiple Files)

```rust
use httprunner_lib::processor::process_http_files;

fn main() -> anyhow::Result<()> {
    // Process one or more .http files
    let files = vec!["example.http".to_string()];
    let results = process_http_files(
        &files,
        false,  // verbose
        None,   // log_filename
        None,   // environment
        false,  // insecure
        false,  // pretty_json
    )?;
    
    // Calculate totals from file results
    let total_success: u32 = results.files.iter().map(|f| f.success_count).sum();
    let total_failed: u32 = results.files.iter().map(|f| f.failed_count).sum();
    let total_skipped: u32 = results.files.iter().map(|f| f.skipped_count).sum();
    
    println!("Success: {}, Failed: {}, Skipped: {}", total_success, total_failed, total_skipped);
    
    Ok(())
}
```

## .http File Format

The library supports a simple, intuitive format for defining HTTP requests:

```http
# Basic GET request
GET https://api.github.com/users/octocat

###

# POST request with headers and body
POST https://httpbin.org/post
Content-Type: application/json

{
  "name": "test",
  "value": 123
}
```

## Variables

Variables make your `.http` files reusable across different environments:

```http
@hostname=localhost
@port=8080

GET https://{{hostname}}:{{port}}/api/users
Authorization: Bearer {{token}}
```

## Request Variables (Chaining)

Chain requests by extracting data from previous responses:

```http
# @name login
POST https://api.example.com/login
Content-Type: application/json

{
  "username": "admin",
  "password": "secret"
}

###

# Use the token from the login response
# @name get_data
GET https://api.example.com/data
Authorization: Bearer {{login.response.body.$.token}}
```

## Response Assertions

Validate responses with built-in assertions:

```http
GET https://api.example.com/users/1

EXPECTED_RESPONSE_STATUS 200
EXPECTED_RESPONSE_BODY "John Doe"
EXPECTED_RESPONSE_HEADERS "Content-Type: application/json"
```

## Conditional Execution

Execute requests conditionally based on previous results:

```http
# @name check-user
GET https://api.example.com/user/123

###

# Create only if user doesn't exist
# @if check-user.response.status 404
POST https://api.example.com/user
Content-Type: application/json

{
  "id": 123,
  "name": "New User"
}
```

## Built-in Functions

Generate dynamic values with built-in functions:

```http
POST https://api.example.com/users
Content-Type: application/json

{
  "id": "guid()",
  "sessionKey": "string()",
  "randomValue": "number()",
  "credentials": "base64_encode('username:password')"
}
```

## Environment Files

Support different environments with `http-client.env.json`:

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

Then reference variables in your `.http` files:

```http
GET {{HostAddress}}/api/data
Authorization: Bearer {{ApiKey}}
```

## Use Cases

- **API Testing**: Execute HTTP requests and validate responses
- **Integration Testing**: Chain requests and test complex workflows
- **API Documentation**: Use `.http` files as executable documentation
- **Development Tools**: Build CLI tools and testing frameworks
- **Automation**: Automate API interactions in scripts and pipelines

## Documentation

For comprehensive documentation, examples, and advanced features, see the [main repository](https://github.com/christianhelle/httprunner).

## License

This project is licensed under the MIT License - see the [LICENSE](https://github.com/christianhelle/httprunner/blob/main/LICENSE) file for details.

## Links

- **Repository**: <https://github.com/christianhelle/httprunner>
- **CLI Tool**: `cargo install httprunner`
- **Documentation**: <https://github.com/christianhelle/httprunner#readme>
