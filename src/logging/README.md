# Log Module

This module handles verbose logging and output writing to files for HTTP File Runner.

## Structure

- `mod.rs` - Module entry point and public API
- `writer.rs` - Log writing implementation with buffering
- `tests.rs` - Test suite

## Usage

```rust
use crate::log::Log;

let mut log = Log::new(true, Some("output.log".to_string()))?;
log.write("Starting HTTP request...")?;
log.write("Response received")?;
```

## Features

### Conditional Logging
- Only writes when verbose mode is enabled
- Silent no-op when verbose is false
- Supports optional file output

### File Output
- Creates log file if path is provided
- Buffers writes for performance
- Automatically flushes on drop
- Appends to existing files

### Console Output
- Writes to stdout when verbose
- Timestamps not included (raw output)
- Immediate display

## Log Levels

The module provides basic logging without explicit levels. For different output types:
- Regular messages: Use `write()`
- Errors: Handle via Result types in caller
- Success/failure: Use colored output from colors module

## Buffer Management

- Uses `BufWriter` for efficient file I/O
- Automatically flushes on `Log` drop
- Manual flush available via `flush()` method

## Example

```rust
let mut log = Log::new(cli.verbose, cli.log.clone())?;

log.write(&format!("Processing file: {}", file_path))?;
log.write("Request completed successfully")?;
```
