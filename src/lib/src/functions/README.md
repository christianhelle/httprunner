# Functions Module

This module handles the substitution of dynamic function calls in HTTP request files, including random data generation and encoding utilities.

## Structure

- `mod.rs` - Module entry point and public API
- `substitution.rs` - Function substitution trait and orchestration
- `generator.rs` - Built-in function implementations
- `tests.rs` - Test suite
- `generator_tests.rs` - Generator-specific tests

## Usage

```rust
use crate::functions::substitute_functions;

let url = substitute_functions("https://api.example.com/users/{{guid()}}")?;
let body = substitute_functions(r#"{"id": "{{guid()}}", "name": "{{string()}}"}"#)?;
```

## Supported Functions

### GUID Generation
Generates a random UUID v4 in simple format (no hyphens):
```
{{guid()}}
```

Example output: `a1b2c3d4e5f67890a1b2c3d4e5f67890`

### Random String
Generates a random 20-character alphanumeric string:
```
{{string()}}
```

Example output: `aB3dE5fG7hI9jK1lM3nO`

### Random Number
Generates a random integer between 0 and 100:
```
{{number()}}
```

Example output: `42`

### Base64 Encoding
Encodes a string value to Base64:
```
{{base64_encode('value to encode')}}
```

Example: `{{base64_encode('username:password')}}` → `dXNlcm5hbWU6cGFzc3dvcmQ=`

## Implementation Details

Functions are case-insensitive and processed through the `FunctionSubstitutor` trait, which provides:
- Pattern matching via regex
- Value generation or transformation
- String replacement logic

All functions are applied sequentially to the input text, allowing for multiple function calls in a single string.
