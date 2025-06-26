# HTTP File Runner

A simple command-line tool written in Zig that parses `.http` files and executes HTTP requests, providing colored output with emojis to indicate success or failure.

## Features

* Parse and execute HTTP requests from `.http` files
* Support for multiple `.http` files in a single run
* `--discover` mode to recursively find and run all `.http` files
* `--verbose` mode for detailed request and response information
* `--log` mode to save all output to a file for analysis and reporting
* Color-coded output (green for success, red for failure)
* Summary statistics showing success/failure counts (per file and overall)
* Support for various HTTP methods (GET, POST, PUT, DELETE, PATCH)
* **Custom headers support** with full request header implementation
* Detailed error reporting with status codes
* Robust error handling for network issues
* **Response assertions** for status codes, body content, and headers
* **Variables support** with substitution in URLs, headers, and request bodies
* **Environment files** for different deployment environments
* **Build-time version generation** with automatic git integration

## Usage

### Basic Commands

    # Run a single .http file
    httprunner <http-file>

    # Run a single .http file with verbose output
    httprunner <http-file> --verbose

    # Run a single .http file and save output to a log file
    httprunner <http-file> --log

    # Run multiple .http files
    httprunner <http-file1> <http-file2> [...]

    # Discover and run all .http files recursively
    httprunner --discover

    # Discover and run all .http files with verbose output
    httprunner --discover --verbose

    # Run with environment variables
    httprunner <http-file> --env production

### Version Information

The application includes comprehensive version information:

    httprunner --version
    # or
    httprunner -v

This displays:

* Application version (semantic versioning)
* Git tag information
* Git commit hash
* Build timestamp

## .http File Format

The HTTP File Runner supports a simple format for defining HTTP requests:

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

## Variables

The HTTP File Runner supports variables to make your .http files more flexible and reusable. Variables are defined using the `@` syntax and can be referenced using double curly braces `{{variable_name}}`.

### Variable Definition

Variables are defined at the beginning of a line with the syntax `@VariableName=Value`:

    @hostname=localhost
    @port=8080
    @protocol=https

### Variable Usage

Variables can be referenced in URLs, headers, and request bodies using double curly braces:

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

### Variable Composition

Variables can be defined using values of other variables that were defined earlier in the file:

    @hostname=localhost
    @port=44320
    @host={{hostname}}:{{port}}
    @baseUrl=https://{{host}}
    
    GET {{baseUrl}}/api/search/tool

**Note:** Variables must be defined before they can be used. The order of definition matters.

## Environment Files

To give variables different values in different environments, create a file named `http-client.env.json`. This file should be located in the same directory as the `.http` file or in one of its parent directories.

### Environment File Format

The environment file is a JSON file that contains one or more named environments:

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

### Using Environment Variables

Variables from an environment file are referenced the same way as other variables:

    # This will use the HostAddress from the specified environment
    GET {{HostAddress}}/api/search/tool
    Authorization: Bearer {{ApiKey}}
    X-Environment: {{Environment}}

### Specifying Environment

Use the `--env` flag to specify which environment to use:

    # Use development environment
    httprunner myfile.http --env dev
    
    # Use production environment  
    httprunner myfile.http --env prod

## Response Assertions

The HTTP File Runner supports assertions to validate HTTP responses. You can assert on status codes, response body content, and response headers.

### Assertion Syntax

* **`EXPECTED_RESPONSE_STATUS`** - Assert on HTTP status code
* **`EXPECTED_RESPONSE_BODY`** - Assert that response body contains specific text
* **`EXPECTED_RESPONSE_HEADERS`** - Assert that response headers contain specific header-value pairs

### Assertion Examples

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

### Assertion Behavior

* **Status Code**: Exact match with expected HTTP status code
* **Response Body**: Checks if response body contains the expected text (substring match)
* **Response Headers**: Checks if the specified header exists and contains the expected value (substring match)
* **Assertion Results**: Detailed output shows which assertions passed/failed
* **Request Success**: A request is considered successful only if all assertions pass (in addition to 2xx status code)

## Output

The tool provides colored output with emojis:

* **Green**: Successful requests (2xx status codes)
* **Red**: Failed requests (4xx, 5xx status codes, or connection errors)
* **Blue**: Informational messages
* **Yellow**: Warnings

### Example Output

    HTTP File Runner - Processing file: examples/simple.http
    ==================================================
    Found 4 HTTP request(s)
    
    GET https://httpbin.org/status/200 - Status: 200
    GET https://httpbin.org/status/404 - Status: 404
    GET https://api.github.com/zen - Status: 200
    GET https://jsonplaceholder.typicode.com/users/1 - Status: 200
    
    ==================================================
    Summary: 3/4 requests succeeded

### Verbose Mode

The `--verbose` flag provides detailed information about HTTP requests and responses, including headers and response bodies. This is useful for debugging and detailed analysis of API interactions.

**What verbose mode shows:**

* **Request Details**: Method, URL, headers, and request body
* **Response Details**: Status code, duration, response headers, and response body
* **Timing Information**: Response times in milliseconds

### Logging Mode

The `--log` flag enables output logging to a file, which is essential for:

* **Automation & CI/CD**: Save test results for build reports and analysis
* **Debugging**: Preserve detailed output for later review
* **Documentation**: Generate test reports and API documentation
* **Monitoring**: Track API performance and reliability over time
* **Auditing**: Keep records of API testing activities

**How to use logging:**

* `--log` without filename: Saves to a file named 'log' in the current directory
* `--log filename.txt`: Saves to the specified filename
* Works with all other flags: `--verbose --log`, `--discover --log`, etc.
* Combines with verbose mode for detailed logged output

## Supported Features

* **Methods**: GET, POST, PUT, DELETE, PATCH
* **Headers**: Key-value pairs separated by `:` (fully supported and sent with requests)
* **Body**: Content after headers (separated by empty line)
* **Comments**: Lines starting with `#`

## Error Handling

The tool handles various error conditions gracefully:

* **File not found**: Clear error message with red indicator
* **Invalid URLs**: Proper error reporting
* **Network issues**: Connection timeouts, unknown hosts, etc.
* **Invalid HTTP methods**: Validation and error reporting

## Command Line Help

When running httprunner without any arguments, help text is displayed showing all available options and usage patterns.

## License

This project is open source and available under the MIT License.

For more information, visit: https://github.com/christianhelle/httprunner