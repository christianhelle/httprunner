# Request Variables Module

This module handles parsing, extraction, and substitution of request variables that reference data from previous requests.

## Structure

- `mod.rs` - Module entry point and public API
- `parser.rs` - Parsing of request variable references
- `extractor.rs` - Extraction of values from request/response contexts
- `json.rs` - JSON property extraction with nested object and array support
- `substitution.rs` - Variable substitution in templates
- `tests.rs` - Test suite

## Usage

### Parsing Request Variables

```rust
use crate::request_variables::parse_request_variable;

let var = parse_request_variable("{{login.response.body.$.token}}")?;
```

### Extracting Values

```rust
use crate::request_variables::extract_request_variable_value;

let value = extract_request_variable_value(&var, &context)?;
```

### Substituting in Templates

```rust
use crate::request_variables::substitute_request_variables;

let url = substitute_request_variables(
    "https://api.example.com/users/{{getUser.response.body.$.id}}",
    &context
)?;
```

## Reference Syntax

Request variables use the format:
```
{{<request_name>.<source>.<target>.<path>}}
```

### Components

- **request_name**: Name of the previous request (from `@name` directive)
- **source**: `request` or `response`
- **target**: `body` or `headers`
- **path**: Property path or header name

### Examples

#### Request Body
```
{{createUser.request.body}}
```

#### Response Header
```
{{login.response.headers.Authorization}}
```

#### JSON Property
```
{{getUser.response.body.$.username}}
```

#### Nested JSON Property
```
{{getProfile.response.body.$.user.email}}
```

#### Array Indexing
```
{{listUsers.response.body.$.users[0].id}}
```

## JSON Path Support

The module supports:
- Simple properties: `$.username`
- Nested properties: `$.user.profile.name`
- Array indexing: `$.data[0]`
- Combined: `$.users[2].address.city`
