# HTTP Runner - Meta Package

This is the main build package for the HTTP Runner project. It provides feature-based build control for the CLI and GUI applications.

## Building

### Default (Build Everything)
```bash
cargo build --release
```
This builds both the CLI (`httprunner`) and GUI (`httprunner-gui`) applications.

### CLI Only
```bash
cargo build --release --no-default-features --features=cli
```
This builds only the CLI application (`httprunner`).

### GUI Only
```bash
cargo build --release --no-default-features --features=gui
```
This builds only the GUI application (`httprunner-gui`).

## Project Structure

- `src/lib/` - Core library with all HTTP processing logic
- `src/cli/` - Command-line interface application
- `src/gui/` - Graphical user interface application
- `src/httprunner/` - Meta-package for feature-based build control (this package)

## Features

- `cli` - Enables the command-line interface binary
- `gui` - Enables the graphical user interface binary
- `default` - Enables both `cli` and `gui`
