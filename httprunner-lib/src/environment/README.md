# Environment Module

This module handles loading and parsing of environment-specific variable files for HTTP File Runner.

## Structure

- `mod.rs` - Module entry point and public API
- `loader.rs` - Environment file discovery and parsing
- `tests.rs` - Test suite

## Usage

```rust
use crate::environment::load_environment_file;

let variables = load_environment_file(Some("dev"), true)?;
for var in variables {
    println!("{} = {}", var.name, var.value);
}
```

## Environment File Format

Environment files use the naming pattern: `http-client.env.{name}.json`

Example: `http-client.env.dev.json`
```json
{
  "API_URL": "https://dev.api.example.com",
  "API_KEY": "dev-key-12345",
  "TIMEOUT": "5000"
}
```

## File Discovery

The module searches for environment files in:
1. Current working directory
2. Parent directories (up to 5 levels)

This allows environment files to be placed at the project root while running the tool from subdirectories.

## Variable Loading

- Reads JSON file with environment-specific variables
- Converts to internal `Variable` type
- Available for use in request templates via `{{VARIABLE_NAME}}` syntax
- Supports string values

## Error Handling

- Gracefully handles missing environment files (returns empty vector)
- Reports parsing errors for invalid JSON
- Validates file format and structure
