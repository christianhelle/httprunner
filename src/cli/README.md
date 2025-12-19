# CLI Module

This module handles command-line interface parsing, validation, and banner display for HTTP File Runner.

## Structure

- `mod.rs` - Module entry point and public API
- `args.rs` - CLI argument parsing using clap
- `banner.rs` - Donation banner display logic
- `tests.rs` - Test suite

## Usage

```rust
use crate::cli::Cli;
use clap::Parser;

let cli = Cli::parse();
println!("Files: {:?}", cli.files);
```

## CLI Arguments

### Positional Arguments
- `files`: One or more HTTP file paths to execute

### Optional Arguments
- `--verbose`, `-v`: Enable verbose output
- `--log <FILE>`: Write verbose output to file
- `--env <NAME>`: Environment name (e.g., dev, staging, prod)
- `--insecure`: Allow insecure HTTPS connections (for self-signed certificates)
- `--pretty-json`: Pretty print JSON responses
- `--no-color`: Disable colored output
- `--discovery`: Discovery mode - scan for .http files
- `--generate-report`: Generate Markdown report

### Version and Help
- `--version`: Display version information
- `--help`: Display help information

## Donation Banner

The donation banner is displayed periodically to support the project. It includes:
- GitHub Sponsors link
- Instructions to disable the banner
- Rate limiting (shown once per day maximum)
