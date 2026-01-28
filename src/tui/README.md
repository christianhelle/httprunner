# HTTP Runner TUI

> **⚠️ Experimental**: This TUI application is currently in an experimental phase. Features and interface may change as development continues.

A Terminal User Interface (TUI) for HTTP Runner built with Rust and Ratatui.

🌐 **[Try the WASM app online](https://christianhelle.com/httprunner/app/)** - No installation required! (GUI version available in browser)

## Features

- 📁 **File Tree Navigation** - Browse and select .http files from a directory
- 📋 **Request Inspector** - View request details including method, URL, headers, and body
- ▶️ **Run Requests** - Execute individual requests or entire files
- 🌍 **Environment Support** - Select environments for variable substitution
- 📊 **Live Results** - See execution results in real-time
- ⌨️ **Keyboard-Driven** - Navigate efficiently using keyboard shortcuts

## Quick Start

```bash
# Run from source
cargo run --bin httprunner-tui

# Or run the compiled binary
./target/release/httprunner-tui
```

## Usage

The TUI is divided into three main panes:

1. **File Tree (Left)** - Browse and select .http files
2. **Request View (Center)** - View parsed requests from the selected file
3. **Results View (Right)** - See execution results

### Keyboard Shortcuts

- **Tab** - Switch between panes
- **↑/↓** or **k/j** - Navigate within current pane
- **Enter** - Run selected request (in Request View)
- **F5** or **Ctrl+R** - Run all requests in the selected file
- **Ctrl+E** - Cycle through available environments
- **Ctrl+Q** or **Ctrl+C** - Quit application
- **Page Up/Down** - Scroll results (in Results View)
- **Home/End** - Jump to first/last item

## Building

### Prerequisites

- Rust 1.92 or later

### Build Commands

```bash
# Debug build
cargo build --bin httprunner-tui

# Release build
cargo build --bin httprunner-tui --release
```

## Architecture

### Components

- **`main.rs`** - Application entry point and event loop
- **`app.rs`** - Main application state and event handling
- **`file_tree.rs`** - File browser with .http file discovery
- **`request_view.rs`** - Request details display
- **`results_view.rs`** - Execution results display
- **`state.rs`** - Persistent state management
- **`ui.rs`** - UI rendering with Ratatui

### State Persistence

The TUI saves the following state between sessions:
- Last opened directory
- Last selected file
- Last selected environment

State is stored in `~/.config/httprunner/tui_state.json` (or platform equivalent).

## Technical Stack

- **TUI Framework**: [Ratatui](https://github.com/ratatui/ratatui) - Terminal UI library
- **Terminal Backend**: [Crossterm](https://github.com/crossterm-rs/crossterm) - Cross-platform terminal manipulation
- **HTTP Execution**: Shared httprunner-lib for request parsing and execution

## Comparison with GUI

| Feature | TUI | GUI |
|---------|-----|-----|
| Resource Usage | ⚡ Minimal | 💻 Moderate |
| Remote Access | ✅ SSH-friendly | ❌ Requires X11/display |
| Mouse Support | ❌ Keyboard only | ✅ Full mouse support |
| Text Editing | ❌ Read-only | ✅ Edit requests |
| Performance | ⚡ Ultra-fast | ⚡ Fast |
| Accessibility | ⌨️ Keyboard-driven | 🖱️ Mouse-driven |

## Contributing

Found a bug or have a suggestion? [Open an issue](https://github.com/christianhelle/httprunner/issues)!

## License

MIT License - same as the parent HTTP Runner project
