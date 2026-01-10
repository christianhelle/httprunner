# Processor Executor Tests

## Overview

This module contains comprehensive tests for the processor executor using a test double (mock) pattern, following the principle of dependency injection commonly used in C#.

## Architecture

### Trait-Based Design (C# Interface Equivalent)

In Rust, **traits** serve the same purpose as **interfaces** in C#:

```rust
// Rust trait (equivalent to C# interface)
pub trait HttpExecutor {
    fn execute(&self, request: &HttpRequest, verbose: bool, insecure: bool) -> Result<HttpResult>;
}
```

C# equivalent:
```csharp
public interface IHttpExecutor 
{
    HttpResult Execute(HttpRequest request, bool verbose, bool insecure);
}
```

### Test Double Pattern

The `MockHttpExecutor` is a test double (similar to a C# mock using Moq or NSubstitute):

```rust
struct MockHttpExecutor {
    responses: Arc<Mutex<Vec<HttpResult>>>,
    call_count: Arc<Mutex<usize>>,
    executed_requests: Arc<Mutex<Vec<HttpRequest>>>,
}
```

C# equivalent using Moq:
```csharp
var mock = new Mock<IHttpExecutor>();
mock.Setup(x => x.Execute(It.IsAny<HttpRequest>(), It.IsAny<bool>(), It.IsAny<bool>()))
    .Returns(new HttpResult { ... });
```

### Key Differences from C#

1. **Arc<Mutex<T>>** - Rust's thread-safe shared ownership (similar to C#'s thread-safe collections)
2. **impl Trait** - Rust's way of implementing interfaces (similar to C#'s interface implementation)
3. **Result<T>** - Rust's error handling (similar to C#'s exceptions or Result pattern)

## Test Coverage

The test suite includes 19 comprehensive tests covering:

### Basic Functionality
- ✅ Basic request execution
- ✅ Multiple sequential requests
- ✅ Request tracking
- ✅ Default response handling

### HTTP Features
- ✅ All HTTP methods (GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS)
- ✅ Various HTTP status codes (2xx, 3xx, 4xx, 5xx)
- ✅ Request and response headers
- ✅ Request and response bodies

### Advanced Features
- ✅ Assertions (passed and failed)
- ✅ Timeout configuration
- ✅ Connection error simulation
- ✅ Request timeout simulation
- ✅ Insecure HTTPS flag handling
- ✅ Verbose mode handling

### Edge Cases
- ✅ Empty response bodies
- ✅ Named vs unnamed requests
- ✅ Multiple assertions per request

## Running the Tests

```bash
# Run all tests
cargo test

# Run only executor tests
cargo test processor::executor_tests

# Run with output
cargo test processor::executor_tests -- --nocapture

# Run a specific test
cargo test test_mock_executor_basic_request
```

## Example Usage

Here's how you would use the mock in a test (conceptual example):

```rust
#[test]
fn test_process_http_files_with_mock() {
    // Arrange
    let mock_responses = vec![
        create_http_result(Some("req1".to_string()), 200, true, Some("OK".to_string())),
    ];
    let mock = MockHttpExecutor::new(mock_responses);
    
    let request = create_simple_request("req1", "GET", "https://api.example.com");
    
    // Act
    let result = mock.execute(&request, false, false).unwrap();
    
    // Assert
    assert_eq!(result.status_code, 200);
    assert!(result.success);
    assert_eq!(mock.get_call_count(), 1);
}
```

## Benefits

1. **Testability** - Tests run without making actual HTTP requests
2. **Speed** - Tests execute instantly without network latency
3. **Reliability** - No external dependencies or network issues
4. **Isolation** - Each test is independent and deterministic
5. **Coverage** - Can test error scenarios that are hard to reproduce with real HTTP calls

## Future Enhancements

Potential areas for expansion:

- Test process_http_files() function with the mock executor
- Test dependency resolution (`@dependsOn`)
- Test conditional execution (`@if`, `@if-not`)
- Test variable substitution with mock responses
- Test assertion evaluation with various response types
- Test file parsing with mock HTTP execution
