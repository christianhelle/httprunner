# Types Module

This module defines the core data structures used throughout HTTP File Runner.

## Structure

- `mod.rs` - Module entry point and public API
- `assertion.rs` - Assertion-related types
- `condition.rs` - Condition-related types
- `context.rs` - Execution context and result aggregation types
- `request.rs` - HTTP request types
- `request_variable.rs` - Request variable reference types
- `result.rs` - HTTP execution result types
- `variable.rs` - Variable definition types

## Core Types

### HttpRequest
Represents an HTTP request with all its components:
- Method, URL, headers, body
- Timeouts (request and connection)
- Assertions, conditions, dependencies
- Request name and variables

### HttpResult
Represents the result of an HTTP request execution:
- Status code, headers, body
- Duration
- Error information (if any)

### Assertion & AssertionResult
Types for defining and evaluating response assertions:
- `Assertion`: Expected values for status, headers, body
- `AssertionResult`: Actual values and pass/fail status

### Condition & ConditionType
Types for conditional request execution:
- `Condition`: Condition definition with request name and type
- `ConditionType`: Status checks or JSON body checks

### RequestVariable
Represents references to data from previous requests:
- Source: request or response
- Target: body or headers
- Path: JSON path or header name

### RequestContext
Stores execution context for each named request:
- Request and response data
- Available for reference by subsequent requests

### ProcessorResults & HttpFileResults
Result aggregation types:
- Per-file results
- Overall statistics (success, failed, skipped counts)
- Success rate calculation

### Variable
Simple key-value pair for environment and inline variables.

## Usage

These types are used throughout the application and provide a consistent data model for HTTP request processing, validation, and result reporting.
