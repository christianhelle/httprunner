# Report Module

This module handles the generation of Markdown test reports for HTTP File Runner.

## Structure

- `mod.rs` - Module entry point and public API
- `generator.rs` - Main report generation logic and content assembly
- `formatter.rs` - Markdown formatting utilities (escaping special characters)
- `writer.rs` - File writing operations with timestamped filenames
- `tests.rs` - Comprehensive test suite

## Usage

```rust
use crate::report::generate_markdown;
use crate::types::ProcessorResults;

let results = ProcessorResults { /* ... */ };
let filename = generate_markdown(&results)?;
println!("Report generated: {}", filename);
```

## Report Format

The generated report includes:

- **Overall Summary**: Total requests, pass/fail counts, success rate
- **Per-File Results**: Breakdown by HTTP file
- **Per-Request Details**:
  - Request information (method, URL, headers, body, timeouts, dependencies, conditions)
  - Response information (status, duration, headers, body, errors)
  - Assertion results (pass/fail status for each assertion)

## Output Filename

Reports are saved with the format: `httprunner-report-YYYYMMDD-HHMMSS.md`

In test mode, a unique counter and process ID are appended to avoid conflicts.
