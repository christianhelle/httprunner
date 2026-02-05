# Colors Module

This module provides ANSI color formatting helpers for terminal output.

## Structure

- `mod.rs` - Module entry point and public API
- `helpers.rs` - Color formatting functions using ANSI escape codes
- `tests.rs` - Test suite

## Usage

```rust
use crate::colors::{green, red, blue, yellow};

println!("{}", green("Success!"));
println!("{}", red("Error!"));
println!("{}", blue("Info"));
println!("{}", yellow("Warning"));
```

## Available Colors

- `green(text)` - Green text (typically for success messages)
- `red(text)` - Red text (typically for error messages)
- `blue(text)` - Blue text (typically for informational messages)
- `yellow(text)` - Yellow text (typically for warning messages)

## ANSI Codes

The module uses standard ANSI escape codes:
- Green: `\x1b[32m`
- Red: `\x1b[31m`
- Blue: `\x1b[34m`
- Yellow: `\x1b[33m`
- Reset: `\x1b[0m`

## Color Disabling

Colors can be disabled via the `--no-color` CLI flag, which is handled at the application level.
