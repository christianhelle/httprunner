# Discovery Module

This module handles automatic discovery of HTTP request files (.http) in the current directory and subdirectories.

## Structure

- `mod.rs` - Module entry point and public API
- `scanner.rs` - File system scanning and .http file discovery
- `tests.rs` - Test suite

## Usage

```rust
use crate::discovery::run_discovery_mode;

run_discovery_mode(
    true,              // verbose
    Some("output.log"), // log file
    Some("dev"),       // environment
    false,             // insecure
    true               // pretty_json
)?;
```

## Features

### File Discovery
- Recursively scans the current working directory
- Finds all files with `.http` extension
- Displays list of discovered files
- Prompts user for confirmation before execution

### Interactive Mode
- Shows count of discovered files
- Lists all file paths
- Asks for user confirmation (y/n)
- Proceeds with execution on confirmation
- Exits gracefully on cancellation

## Discovery Process

1. Scan current directory and subdirectories
2. Filter for `.http` files
3. Display discovered files to user
4. Prompt for execution confirmation
5. If confirmed, execute all discovered files
6. Report results

## Integration

Discovery mode is activated via the `--discovery` CLI flag and serves as an alternative to explicitly specifying file paths.
