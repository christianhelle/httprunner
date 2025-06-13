# HTTP Runner

A simple command-line tool written in Zig that parses `.http` files and executes HTTP requests, providing colored output with emojis to indicate success or failure.

## Features

- 🚀 Parse and execute HTTP requests from `.http` files
- ✅ Color-coded output (green for success, red for failure)
- 📊 Summary statistics showing success/failure counts
- 🌐 Support for various HTTP methods (GET, POST, PUT, DELETE, PATCH)
- 📝 Custom headers support (parsing implemented, execution pending)
- 🎯 Detailed error reporting with status codes
- 🛡️ Robust error handling for network issues

## Building

Make sure you have Zig installed (version 0.14 or later).

```bash
zig build
```

## Usage

```bash
./zig-out/bin/httprunner <http-file>
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
```

## .http File Format

The HTTP runner supports a simple format for defining HTTP requests:

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
🚀 HTTP Runner - Processing file: examples/simple.http
==================================================
Found 4 HTTP request(s)

✅ GET https://httpbin.org/status/200 - Status: 200
❌ GET https://httpbin.org/status/404 - Status: 404
✅ GET https://api.github.com/zen - Status: 200
✅ GET https://jsonplaceholder.typicode.com/users/1 - Status: 200

==================================================
Summary: 3/4 requests succeeded
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

## License

This project is open source and available under the MIT License.