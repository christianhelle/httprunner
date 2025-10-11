# Insecure HTTPS Implementation Summary

## Overview

This document summarizes the implementation of insecure HTTPS support in the httprunner project. The feature allows users to bypass SSL certificate verification when testing against endpoints with self-signed or invalid certificates.

## Implementation Approach

Since the Zig standard library's `std.http.Client` does not provide built-in support for disabling SSL certificate verification, we switched to using **libcurl** as the HTTP client backend. libcurl provides robust control over SSL verification options.

## Changes Made

### 1. Build System (`build.zig`)

- Added `libcurl` as a system library dependency
- Linked against `libcurl` and `libc` for all builds
- Ensures curl library is available during compilation

### 2. Curl Wrapper (`src/curl.zig`)

Created a comprehensive Zig wrapper around libcurl C API:

- **Bindings**: Defined C bindings for curl functions and constants
- **Request Structure**: `CurlRequest` with support for insecure mode
- **Response Structure**: `CurlResponse` with headers and body
- **SSL Options**: 
  - `CURLOPT_SSL_VERIFYPEER = 0` - Disables peer certificate verification
  - `CURLOPT_SSL_VERIFYHOST = 0` - Disables hostname verification
- **Memory Management**: Proper allocation and cleanup
- **Callbacks**: Write and header callbacks for response handling

### 3. Type System (`src/types.zig`)

- Added `insecure: bool` field to `HttpRequest` structure
- Ensures insecure flag propagates through the request lifecycle

### 4. Parser (`src/parser.zig`)

- Initializes `insecure = false` for all parsed requests
- Recognizes `INSECURE` directive in `.http` files:
  - `INSECURE` (standalone)
  - `# INSECURE` (as comment)
  - `// INSECURE` (alternative comment style)
- Sets `request.insecure = true` when directive is found

### 5. CLI Interface (`src/cli.zig`)

Added `insecure` option to CLI:

- **Field**: Added `insecure: bool` to `CliOptions` struct
- **Flag Recognition**: Recognizes `--insecure` and `-k` flags
- **Help Text**: Updated usage information to document the flag
- **Initialization**: Properly initializes insecure flag in all code paths

### 6. HTTP Runner (`src/runner.zig`)

Complete rewrite to use libcurl:

- **Main Function**: `executeHttpRequest()` now delegates to `executeWithCurl()`
- **Curl Integration**: Converts Zig types to curl structures
- **Error Handling**: Maps curl errors to HttpResult errors
- **Response Processing**: Converts curl responses back to Zig types
- **Insecure Mode**: Passes `request.insecure` flag to curl request

### 7. Processor (`src/processor.zig`)

Enhanced to support global insecure flag:

- **Function Signature**: Added `insecure: bool` parameter to `processHttpFiles()`
- **Flag Application**: Applies CLI insecure flag to all requests when set
- **Cloning**: Updated `cloneHttpRequest()` to include insecure field
- **Priority**: CLI flag overrides per-request settings

### 8. Main Entry Point (`src/main.zig`)

- **Curl Initialization**: Calls `curl.init()` at startup
- **Curl Cleanup**: Calls `curl.deinit()` at shutdown
- **Flag Passing**: Passes `options.insecure` to processor

### 9. Documentation

#### README.md Updates:

- Added insecure HTTPS to features list
- Documented command-line usage with `--insecure` / `-k` flag
- Created comprehensive "Insecure HTTPS Support" section with:
  - Security warnings
  - Usage examples (CLI and file directive)
  - Implementation details
  - Use cases
- Updated build instructions to include libcurl dependency
- Updated help text example
- Added `insecure.http` to example files list

#### Example File (`examples/insecure.http`):

Created demonstration file with:
- Self-signed certificate endpoint
- Expired certificate endpoint
- Untrusted root certificate endpoint
- POST request with insecure mode
- Regular secure request for comparison

### 10. Docker Support (`Dockerfile`)

- Added `libcurl` to runtime dependencies
- Ensures curl library is available in container

## Usage

### Command-Line Flag (Global)

```bash
# Apply insecure mode to all requests
httprunner myfile.http --insecure
httprunner myfile.http -k

# Combined with other flags
httprunner --discover --insecure --verbose
```

### Per-Request Directive

```http
# Enable insecure for specific request
# INSECURE
GET https://self-signed.badssl.com/

###

# Alternative syntax
INSECURE
POST https://expired.badssl.com/api/test
Content-Type: application/json

{"test": "data"}
```

## Security Considerations

⚠️ **Important Security Notes:**

1. **Development Only**: Insecure mode should only be used in development/testing
2. **Never in Production**: Do not use with production endpoints or sensitive data
3. **Man-in-the-Middle Risk**: Disabling SSL verification opens the door to MITM attacks
4. **Data Exposure**: Credentials and data could be intercepted
5. **Audit Trail**: Consider logging when insecure mode is used

## Testing

The implementation supports testing against:

- Self-signed certificates (e.g., `self-signed.badssl.com`)
- Expired certificates (e.g., `expired.badssl.com`)
- Untrusted root CAs (e.g., `untrusted-root.badssl.com`)
- Hostname mismatches
- Internal development servers

## Dependencies

### Build-time:
- Zig 0.15.1+
- libcurl development headers/libraries

### Runtime:
- libcurl shared library

### Platform-specific Installation:

**Ubuntu/Debian:**
```bash
sudo apt-get install libcurl4-openssl-dev
```

**Fedora/RHEL:**
```bash
sudo dnf install libcurl-devel
```

**macOS:**
```bash
brew install curl
```

**Windows:**
- Usually included with Zig installations
- Can also use vcpkg or manually install curl

## Implementation Quality

### Strengths:
- ✅ End-to-end implementation from CLI to HTTP execution
- ✅ Comprehensive error handling
- ✅ Proper memory management with allocators
- ✅ Backward compatible (secure by default)
- ✅ Flexible (per-request or global settings)
- ✅ Well documented

### Considerations:
- libcurl adds external dependency
- Binary size increases slightly due to curl linking
- Platform-specific libcurl installation required for builds

## Future Enhancements

Potential improvements:
1. Support for specific TLS versions
2. Custom CA certificate bundles
3. Client certificate authentication
4. Proxy support with curl
5. HTTP/2 and HTTP/3 support via curl
6. Connection pooling and reuse
7. Request timeout configuration

## Conclusion

The implementation provides a robust, secure-by-default solution for insecure HTTPS connections. It uses industry-standard libcurl for SSL handling, provides both global and per-request control, and includes comprehensive documentation for users.
