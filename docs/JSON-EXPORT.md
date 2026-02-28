# JSON Export Schema

The `--export-json` flag exports execution results as a single JSON file named `httprunner_results_<timestamp>.json`. This format is designed for integration with external tools such as Visual Studio, Visual Studio Code extensions, CI/CD pipelines, and other non-Rust applications.

## Usage

```bash
httprunner --export-json my-api.http
```

This produces a file like `httprunner_results_1735689600.json` in the current directory.

## Schema Overview

The root object is a **ProcessorResults** containing the overall success status and an array of per-file results.

```json
{
  "success": true,
  "files": [ ... ]
}
```

## Complete Schema Reference

### ProcessorResults (root)

| Field     | Type                  | Description                                               |
|-----------|-----------------------|-----------------------------------------------------------|
| `success` | `boolean`             | `true` if all files processed successfully, `false` otherwise |
| `files`   | `HttpFileResults[]`   | Array of results, one per `.http` file processed          |

### HttpFileResults

| Field              | Type                | Description                                     |
|--------------------|---------------------|-------------------------------------------------|
| `filename`         | `string`            | Path to the `.http` file that was processed     |
| `success_count`    | `integer`           | Number of requests that succeeded               |
| `failed_count`     | `integer`           | Number of requests that failed                  |
| `skipped_count`    | `integer`           | Number of requests that were skipped            |
| `result_contexts`  | `RequestContext[]`   | Detailed results for each request in the file   |

### RequestContext

| Field     | Type                  | Description                                             |
|-----------|-----------------------|---------------------------------------------------------|
| `name`    | `string`              | Name of the request (from `# @name`)                   |
| `request` | `HttpRequest`         | The request that was sent                               |
| `result`  | `HttpResult \| null`  | The response received, or `null` if the request was skipped |

### HttpRequest

| Field                | Type              | Description                                           |
|----------------------|-------------------|-------------------------------------------------------|
| `name`               | `string \| null`  | Request name (from `# @name`), or `null` if unnamed   |
| `method`             | `string`          | HTTP method (e.g., `"GET"`, `"POST"`, `"PUT"`)        |
| `url`                | `string`          | Target URL                                            |
| `headers`            | `Header[]`        | Request headers                                       |
| `body`               | `string \| null`  | Request body, or `null` if none                       |
| `assertions`         | `Assertion[]`     | Assertions defined for this request                   |
| `variables`          | `Variable[]`      | Variables defined in this request                     |
| `timeout`            | `integer \| null` | Read timeout in milliseconds, or `null` if not set    |
| `connection_timeout` | `integer \| null` | Connection timeout in milliseconds, or `null`         |
| `depends_on`         | `string \| null`  | Name of the request this depends on (`@dependsOn`)    |
| `conditions`         | `Condition[]`     | Conditions for execution (`@if` / `@if-not`)          |
| `pre_delay_ms`       | `integer \| null` | Delay before executing in milliseconds (`@pre-delay`) |
| `post_delay_ms`      | `integer \| null` | Delay after executing in milliseconds (`@post-delay`) |

### Header

| Field   | Type     | Description    |
|---------|----------|----------------|
| `name`  | `string` | Header name    |
| `value` | `string` | Header value   |

### HttpResult

| Field               | Type                          | Description                                        |
|---------------------|-------------------------------|----------------------------------------------------|
| `request_name`      | `string \| null`              | Name of the request that produced this result      |
| `status_code`       | `integer`                     | HTTP response status code (e.g., `200`, `404`)     |
| `success`           | `boolean`                     | Whether the request was successful                 |
| `error_message`     | `string \| null`              | Error message if the request failed                |
| `duration_ms`       | `integer`                     | Request duration in milliseconds                   |
| `response_headers`  | `object \| null`              | Response headers as key-value pairs, or `null`     |
| `response_body`     | `string \| null`              | Response body as a string, or `null`               |
| `assertion_results` | `AssertionResult[]`           | Results of assertion evaluations                   |

### Assertion

| Field            | Type     | Description                                        |
|------------------|----------|----------------------------------------------------|
| `assertion_type` | `string` | Type of assertion: `"Status"`, `"Body"`, or `"Headers"` |
| `expected_value` | `string` | The expected value for the assertion               |

### AssertionResult

| Field           | Type              | Description                                     |
|-----------------|-------------------|-------------------------------------------------|
| `assertion`     | `Assertion`       | The assertion that was evaluated                |
| `passed`        | `boolean`         | Whether the assertion passed                    |
| `actual_value`  | `string \| null`  | The actual value received, or `null`            |
| `error_message` | `string \| null`  | Error message if the assertion failed           |

### Variable

| Field   | Type     | Description    |
|---------|----------|----------------|
| `name`  | `string` | Variable name  |
| `value` | `string` | Variable value |

### Condition

| Field            | Type            | Description                                          |
|------------------|-----------------|------------------------------------------------------|
| `request_name`   | `string`        | Name of the request whose result is evaluated        |
| `condition_type` | `ConditionType` | The type of condition                                |
| `expected_value` | `string`        | The expected value for the condition                 |
| `negate`         | `boolean`       | `true` for `@if-not`, `false` for `@if`             |

### ConditionType

A tagged enum with the following variants:

- `"Status"` — Check the response status code
- `{ "BodyJsonPath": "<expression>" }` — Check a JSONPath expression in the response body

## Full Example

Given the following `.http` file:

```http
###
# @name get-users
GET https://api.example.com/users
Content-Type: application/json

###
# @name create-user
# @dependsOn get-users
POST https://api.example.com/users
Content-Type: application/json

{
  "name": "John Doe",
  "email": "john@example.com"
}
```

The JSON export would look like:

```json
{
  "success": true,
  "files": [
    {
      "filename": "api.http",
      "success_count": 2,
      "failed_count": 0,
      "skipped_count": 0,
      "result_contexts": [
        {
          "name": "get-users",
          "request": {
            "name": "get-users",
            "method": "GET",
            "url": "https://api.example.com/users",
            "headers": [
              {
                "name": "Content-Type",
                "value": "application/json"
              }
            ],
            "body": null,
            "assertions": [],
            "variables": [],
            "timeout": null,
            "connection_timeout": null,
            "depends_on": null,
            "conditions": [],
            "pre_delay_ms": null,
            "post_delay_ms": null
          },
          "result": {
            "request_name": "get-users",
            "status_code": 200,
            "success": true,
            "error_message": null,
            "duration_ms": 150,
            "response_headers": {
              "content-type": "application/json",
              "server": "nginx"
            },
            "response_body": "[{\"id\":1,\"name\":\"Alice\"}]",
            "assertion_results": []
          }
        },
        {
          "name": "create-user",
          "request": {
            "name": "create-user",
            "method": "POST",
            "url": "https://api.example.com/users",
            "headers": [
              {
                "name": "Content-Type",
                "value": "application/json"
              }
            ],
            "body": "{\n  \"name\": \"John Doe\",\n  \"email\": \"john@example.com\"\n}",
            "assertions": [],
            "variables": [],
            "timeout": null,
            "connection_timeout": null,
            "depends_on": "get-users",
            "conditions": [],
            "pre_delay_ms": null,
            "post_delay_ms": null
          },
          "result": {
            "request_name": "create-user",
            "status_code": 201,
            "success": true,
            "error_message": null,
            "duration_ms": 230,
            "response_headers": {
              "content-type": "application/json"
            },
            "response_body": "{\"id\":2,\"name\":\"John Doe\",\"email\":\"john@example.com\"}",
            "assertion_results": []
          }
        }
      ]
    }
  ]
}
```

## Integration Notes

- **File naming**: Output files follow the pattern `httprunner_results_<unix_timestamp>.json`
- **Skipped requests**: When a request is skipped (e.g., due to unmet conditions), its `result` field is `null`
- **Failed requests**: When a request fails, `result.success` is `false` and `result.error_message` contains details
- **Response body**: The response body is always a string. If the response was JSON, parse `result.response_body` separately
- **Assertions**: The `assertion_type` field uses serde's default serialization for Rust enums — simple variants are strings (e.g., `"Status"`), while variants with data are objects (e.g., `{ "BodyJsonPath": "$.id" }`)
- **Combining flags**: `--export-json` can be used alongside `--export` (raw request/response files) and `--report` (Markdown/HTML reports)
