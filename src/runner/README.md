# Runner Module

This module handles the execution of individual HTTP requests using the reqwest library.

## Structure

- `mod.rs` - Module entry point and public API
- `executor.rs` - HTTP request execution with timeout support

## Usage

```rust
use crate::runner::execute_http_request;

let result = execute_http_request(&request, insecure, &mut log).await?;
println!("Status: {}", result.status);
println!("Body: {}", result.body);
```

## Features

### HTTP Request Execution
- Supports all standard HTTP methods (GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS)
- Custom headers
- Request body (text or binary)
- Query parameters in URL

### Timeout Support
- Request timeout (overall request duration)
- Connection timeout (time to establish connection)
- Configurable via `@timeout` and `@connection-timeout` directives
- Default timeouts if not specified

### HTTPS Configuration
- Secure HTTPS by default
- Optional insecure mode via `--insecure` flag
- Allows self-signed certificates in development environments
- Certificate validation bypass when needed

### Response Handling
- Captures HTTP status code
- Reads response headers
- Reads response body as string
- Measures request duration
- Error handling for network failures

## Request Timeouts

```rust
// Example request with timeouts
HttpRequest {
    method: "GET",
    url: "https://api.example.com/data",
    timeout: Some(5000),  // 5 second request timeout
    connection_timeout: Some(3000),  // 3 second connection timeout
    // ...
}
```

## Error Handling

The module captures and returns errors for:
- Connection failures
- Timeout exceeded
- Invalid URLs
- Network errors
- TLS/SSL errors

## Async Execution

All requests are executed asynchronously using Tokio runtime for better performance and concurrent request handling.
