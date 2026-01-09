# Report Module

This module handles the generation of test reports for HTTP File Runner in both Markdown and HTML formats.

## Structure

- `mod.rs` - Module entry point and public API
- `generator.rs` - Markdown report generation logic and content assembly
- `html_generator.rs` - HTML report generation logic and content assembly
- `formatter.rs` - Markdown formatting utilities (escaping special characters)
- `report_style.css` - Embedded CSS styles for HTML reports
- `writer.rs` - File writing operations with timestamped filenames
- `tests.rs` - Comprehensive test suite

## Usage

```rust
use crate::report::{generate_markdown, generate_html};
use crate::types::ProcessorResults;

let results = ProcessorResults { /* ... */ };

// Generate markdown report
let filename = generate_markdown(&results)?;
println!("Markdown report generated: {}", filename);

// Generate HTML report
let filename = generate_html(&results)?;
println!("HTML report generated: {}", filename);
```

## Report Formats

### Markdown Reports

The generated markdown report includes:

- **Overall Summary**: Total requests, pass/fail counts, success rate
- **Per-File Results**: Breakdown by HTTP file
- **Per-Request Details**:
  - Request information (method, URL, headers, body, timeouts, dependencies, conditions)
  - Response information (status, duration, headers, body, errors)
  - Assertion results (pass/fail status for each assertion)

### HTML Reports

The generated HTML report includes all the same information as markdown with additional features:

- **Responsive Design**: Works on mobile and desktop devices
- **Light/Dark Mode**: Automatic theme switching based on system preferences
- **Styled Components**: Color-coded statistics cards and status indicators
- **Modern UI**: Clean, professional styling with proper spacing and typography
- **Syntax Highlighting**: Code blocks with proper formatting

## Output Filename

Reports are saved with the format: 
- Markdown: `httprunner-report-YYYYMMDD-HHMMSS.md`
- HTML: `httprunner-report-YYYYMMDD-HHMMSS.html`

In test mode, a unique counter and process ID are appended to avoid conflicts.
