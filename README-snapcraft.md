# HTTP File Runner

A command-line tool that parses `.http` files and executes HTTP requests with colored output and detailed reporting.

## Features

* Parse and execute HTTP requests from `.http` files
* Support for multiple files and recursive discovery (`--discover`)
* Verbose mode (`--verbose`) for detailed request/response information
* Logging mode (`--log`) to save output for analysis
* Color-coded output (green for success, red for failure)
* Summary statistics with success/failure counts
* Support for GET, POST, PUT, DELETE, PATCH methods
* Custom headers and request bodies
* Variables and environment files support
* Response assertions for status codes, body content, and headers

## Usage

    # Run a single .http file
    httprunner myfile.http

    # Run with verbose output
    httprunner myfile.http --verbose

    # Run multiple files
    httprunner file1.http file2.http

    # Discover and run all .http files recursively
    httprunner --discover

    # Use environment variables
    httprunner myfile.http --env production

    # Show version information
    httprunner --version

## .http File Format

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

Define variables using `@VariableName=Value` syntax and reference them with `{{variable_name}}`:

    @hostname=localhost
    @port=8080
    
    GET https://{{hostname}}:{{port}}/api/users
    Authorization: Bearer {{token}}

Variables can reference other variables:

    @baseUrl=https://{{hostname}}:{{port}}
    GET {{baseUrl}}/api/search

## Environment Files

Create `http-client.env.json` for environment-specific variables:

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

Use with: `httprunner myfile.http --env dev`

## Response Assertions

Validate responses with assertion keywords:

    GET https://httpbin.org/status/200
    
    EXPECTED_RESPONSE_STATUS 200
    EXPECTED_RESPONSE_BODY "success"
    EXPECTED_RESPONSE_HEADERS "Content-Type: application/json"

## Output Features

* **Colored output**: Green for success, red for failure
* **Summary statistics**: Shows success/failure counts per file and overall  
* **Verbose mode**: Detailed request/response information with headers and timing
* **Logging**: Save output to files for CI/CD, debugging, and documentation
* **Error handling**: Clear messages for network issues, invalid URLs, and file errors

## License

MIT License. For more information: https://github.com/christianhelle/httprunner