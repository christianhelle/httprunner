# HTTP File Runner (Docker Image)

Run portable HTTP/API test collections defined in `.http` files (same syntax as VS Code REST Client) directly via Docker. This README focuses on usage: request syntax, variables, chaining, assertions, conditional execution, environments, timeouts, and Docker-specific invocation.

## Pull & Run

```bash
docker pull christianhelle/httprunner:latest

# Basic run (mount current directory read-only at /app)
docker run -it --rm -v "${PWD}:/app:ro" christianhelle/httprunner examples/simple.http
```

### Helpful Alias
```bash
alias httprunner='docker run -it --rm -v "${PWD}:/app:ro" christianhelle/httprunner'
# Then use like a native binary:
httprunner examples/simple.http --verbose --log results.txt
```

### Multiple Files & Discovery
```bash
httprunner file1.http file2.http
httprunner --discover                    # Recursively run all .http files
httprunner --discover --verbose --log run.log
```

### Insecure HTTPS (self‑signed / local certs)
```bash
httprunner https-local.http --insecure
```
Use only in development/testing.

## .http File Syntax

Basic structure:
```http
# Comments start with # (// also supported for directives)
GET https://httpbin.org/headers
User-Agent: HttpRunner
Accept: application/json

### Separator (blank line before body if body exists)
POST https://httpbin.org/post
Content-Type: application/json

{
  "name": "example",
  "value": 42
}
```
Supported methods: GET, POST, PUT, DELETE, PATCH. Headers are key: value pairs. Body follows an empty line after headers.

## Variables
Define with `@Name=Value` and reference using `{{Name}}`.
```http
@host=api.github.com
@user=octocat
GET https://{{host}}/users/{{user}}
```
Variable composition (must be defined earlier):
```http
@hostname=localhost
@port=44320
@baseUrl=https://{{hostname}}:{{port}}
GET {{baseUrl}}/health
```
Order matters; later definitions can use earlier variables.

## Environment Files (http-client.env.json)
Place `http-client.env.json` alongside or in a parent directory of your `.http` file.
```json
{
  "dev": {
    "HostAddress": "https://localhost:44320",
    "ApiKey": "dev-key"
  },
  "prod": {
    "HostAddress": "https://api.example.com",
    "ApiKey": "prod-key"
  }
}
```
Use with:
```bash
httprunner myfile.http --env dev
```
Environment variables load first; in-file `@` variables override them.

## Request Naming & Chaining (Request Variables)
Name a request with `# @name <identifier>`. Later requests can extract data:
Pattern:
```
{{requestName.(request|response).(body|headers).(<JSONPath>|<header>|*)}}
```
Example authentication flow:
```http
# @name login
POST https://httpbin.org/post
Content-Type: application/json

{ "username": "admin", "token": "abc123", "role": "administrator" }

###
# @name get-data
GET https://httpbin.org/get
Authorization: Bearer {{login.response.body.$.json.token}}
X-User-Role: {{login.response.body.$.json.role}}
```
You can also reference original request body values:
```http
X-Original-User: {{login.request.body.$.username}}
```
JSONPath shortcuts: `$.prop`, `$.nested.prop`, `$.json.prop` (for httpbin style), `*` for full body.

## Conditional Execution
Skip or run requests based on earlier results.
- `@dependsOn <request>`: Only runs if the named request succeeded (HTTP 200).
- `@if <request>.response.status <code>`: Runs if status matches.
- `@if <request>.response.body.$.<path> <value>`: Runs if JSONPath equals value.
- `@if-not ...`: Negated condition.
Example:
```http
# @name check-user
GET https://api.example.com/user/123

###
# @name create-user
# @if check-user.response.status 404
POST https://api.example.com/user
Content-Type: application/json

{ "id": 123, "name": "New User" }
```
Multiple `@if` / `@if-not` lines = logical AND. Skipped requests appear in summary as "Skipped".

## Timeouts
Customize per-request:
- `@timeout <value>[ unit]` (read timeout)
- `@connection-timeout <value>[ unit]`
Units: ms, s (default), m.
Examples:
```http
# @timeout 120
// @connection-timeout 10
GET https://example.com/slow

# @timeout 1500 ms
GET https://example.com/fast

# @timeout 2 m
GET https://example.com/big-job
```

## Response Assertions
Add after the request block (order-independent):
- `EXPECTED_RESPONSE_STATUS <code>`
- `EXPECTED_RESPONSE_BODY "substring"` (may repeat)
- `EXPECTED_RESPONSE_HEADERS "Header: value-substring"` (may repeat)
Example:
```http
GET https://httpbin.org/status/200
EXPECTED_RESPONSE_STATUS 200

GET https://httpbin.org/json
EXPECTED_RESPONSE_STATUS 200
EXPECTED_RESPONSE_BODY "slideshow"
EXPECTED_RESPONSE_HEADERS "Content-Type: application/json"
```
Variables work inside assertion values:
```http
@base=https://httpbin.org
GET {{base}}/get
EXPECTED_RESPONSE_STATUS 200
EXPECTED_RESPONSE_BODY "{{base}}/get"
```
A request is marked failed if any assertion fails (even if status is 2xx). When assertions exist, response body & headers are captured even without `--verbose`.

## Logging & Verbose Mode
```bash
httprunner examples/simple.http --log              # writes to ./log
httprunner examples/simple.http --log results.txt  # custom filename
httprunner examples/simple.http --verbose --log detailed.txt
```
`--verbose` adds request/response headers, body, and timing. `--no-banner` removes the donation banner (useful in CI).

## Running via Docker (Extended Examples)
```bash
# Single file
docker run -it --rm -v "${PWD}:/app:ro" christianhelle/httprunner tests/api.http

# Multiple files
docker run -it --rm -v "${PWD}:/app:ro" christianhelle/httprunner tests/a.http tests/b.http

# Discovery + logging
docker run -it --rm -v "${PWD}:/app:ro" christianhelle/httprunner --discover --log api-suite.log

# Using environment selection
docker run -it --rm -v "${PWD}:/app:ro" christianhelle/httprunner secure.http --env prod

# Insecure HTTPS for local dev
docker run -it --rm -v "${PWD}:/app:ro" christianhelle/httprunner local-ssl.http --insecure
```
Pass environment variables:
```bash
docker run -it --rm -e TOKEN=abc -v "${PWD}:/app:ro" christianhelle/httprunner auth.http
```

## GitHub Actions Snippet
```yaml
- name: API Tests
  run: |
    docker run --rm -v ${{ github.workspace }}:/app:ro \
      christianhelle/httprunner --discover --verbose --log artifacts/api-tests.log --no-banner
```

## Sample Chained & Conditional Scenario
```http
@base=https://httpbin.org

# @name authenticate
POST {{base}}/post
Content-Type: application/json

{ "username": "admin", "token": "abc123", "role": "admin" }
EXPECTED_RESPONSE_STATUS 200
EXPECTED_RESPONSE_BODY "admin"

###
# @name get-dashboard
# @dependsOn authenticate
# @if authenticate.response.status 200
# @if authenticate.response.body.$.json.role admin
GET {{base}}/get
Authorization: Bearer {{authenticate.response.body.$.json.token}}
EXPECTED_RESPONSE_STATUS 200

###
# @name audit
# @dependsOn get-dashboard
POST {{base}}/post
Content-Type: application/json

{
  "actor": "{{authenticate.request.body.$.username}}",
  "role": "{{authenticate.response.body.$.json.role}}",
  "contentType": "{{get-dashboard.response.headers.Content-Type}}"
}
EXPECTED_RESPONSE_STATUS 200
```

## Help Summary
(Shown when run without args)
```
HTTP File Runner - Execute HTTP requests from .http files

Usage: httprunner [OPTIONS] [FILE]...

Arguments:
  [FILE]...  One or more .http files to process

Options:
  -v, --verbose            Show detailed HTTP request and response information
      --log [<FILENAME>]   Log output to a file (defaults to 'log' if no filename is specified)
      --env <ENVIRONMENT>  Specify environment name to load variables from http-client.env.json
      --insecure           Allow insecure HTTPS connections (accept invalid certificates and hostnames)
      --discover           Recursively discover and process all .http files from current directory
      --upgrade            Update httprunner to the latest version
      --no-banner          Do not show the donation banner
  -h, --help               Print help
  -V, --version            Print version
```

## Output Indicators
- ✅ Success (2xx + assertions passed)
- ❌ Failure (status error or assertion failed)
- ⏭️ Skipped (conditions unmet)
Summary shows Passed / Failed / Skipped per file and overall.

## Practical Uses
- Smoke tests of public APIs
- Contract validation in CI pipelines
- Auth / token flow verification
- Chained integration scenarios
- Environment-specific regression suites

## Quick Reference Cheat Sheet
```
@var=value                      # Define variable
{{var}}                         # Use variable
# @name req                     # Name a request
{{req.response.body.$.prop}}    # JSONPath extraction
EXPECTED_RESPONSE_STATUS 200    # Status assertion
EXPECTED_RESPONSE_BODY "text"   # Body contains text
EXPECTED_RESPONSE_HEADERS "H: v"# Header contains substring
@timeout 30                     # Read timeout (seconds)
@connection-timeout 5           # Connect timeout (seconds)
@if login.response.status 200   # Conditional execution
@if-not check.response.status 404
@dependsOn previous-request     # Requires success (HTTP 200)
```

## Notes
- Requests can mix `@dependsOn`, `@if`, and `@if-not` (AND logic).
- Assertions trigger automatic capture of body & headers even without `--verbose`.
- Variables resolve in URLs, headers, bodies, and assertion values.
- Docker container has network view of host; ensure external services reachable.

Enjoy fast, declarative HTTP testing anywhere Docker runs. 🚀
