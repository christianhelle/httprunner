# HTTP Runner GUI

> **⚠️ Experimental**: This GUI application is currently in an experimental phase. Features and interface may change as development continues.

A native cross-platform graphical user interface for HTTP Runner built with Rust and **Iced**.

## Framework Change Notice

This GUI has been recently ported from **egui/eframe** to **Iced** framework for better cross-platform support and a more modern UI approach.

**Important Changes:**
- ✅ **Native Desktop Support** - Works on Windows, macOS, and Linux
- ⚠️ **WASM Support Temporarily Unavailable** - Iced 0.14 does not yet support WASM compilation
- 🎯 **Future-Ready** - When Iced adds WASM support, web deployment will be possible again

## Features

- 🎨 **Native UI** - Pure Rust, modern Iced framework
- 🌐 **Cross-platform** - Works on Windows, macOS, and Linux
- 📁 **File Tree View** - Browse and select .http files with folder navigation
- ✏️ **Text Editor** - Edit HTTP requests directly in the application
- 📋 **Request Inspector** - View request details including method, URL, headers, and body
- ▶️ **Run Requests** - Execute individual requests or entire files
- 🌍 **Environment Support** - Select environments for variable substitution
- 📊 **Live Results** - See execution results in real-time
- 🚀 **Fast & Responsive** - Leverages Iced's efficient rendering and async runtime

---

## Quick Start

### Desktop Application

```bash
# Run from source
cargo run --bin httprunner-gui

# Or run the compiled binary
./target/release/httprunner-gui
```

---

## Building

### Prerequisites

- Rust 1.92 or later
- Development libraries for your platform (Linux only):
  - **Ubuntu/Debian**: `libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libwayland-dev`
  - **macOS**: Xcode command-line tools
  - **Windows**: Windows SDK (usually included with Visual Studio)

### Native Application

#### Ubuntu/Debian

```bash
# Install dependencies
sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
    libxkbcommon-dev libssl-dev libfontconfig1-dev libwayland-dev

# Build the GUI
cargo build --bin httprunner-gui --release
```

#### macOS

```bash
cargo build --bin httprunner-gui --release
```

#### Windows

```bash
cargo build --bin httprunner-gui --release
```
```bash
cd src/gui
trunk build --release
# Output in dist/ folder
```

---

## Usage

### Desktop Application

1. **Open Directory**: Click "Open Folder" to select a folder containing .http files
2. **Browse Files**: Click on files in the left panel to view their contents
3. **View Requests**: Click "Toggle View" to switch between Text Editor and Request Details
4. **Select Environment** (optional): Environment is shown in the top toolbar
5. **Run Requests**: 
   - Click "▶ Run All" to execute all requests in the file
6. **View Results**: See execution results in the right panel
   - Click "Toggle View" to switch between Compact and Verbose modes

---

## Keyboard Shortcuts

Note: Keyboard shortcuts from the previous egui version are being reimplemented in Iced. Currently supported:
- Buttons and UI interactions work via mouse/touch

---

## Architecture

### Components

- **`main.rs`** - Native application entry point (Iced-based)
- **`app.rs`** - Main application state and UI layout (Iced Application)
- **`file_tree.rs`** - File browser component
- **`text_editor.rs`** - Text editor component using Iced's text_editor widget
- **`request_view.rs`** - Request details display
- **`results_view.rs`** - Execution results display
- **`environment_editor.rs`** - Environment variables viewer
- **`state.rs`** - Application state persistence

### Execution

**Native (Desktop):**
```rust
// Uses httprunner_core processor with async tasks
httprunner_core::processor::process_http_file_incremental(...)
```
```

---

## Development

### Running in Debug Mode

```bash
# Native
RUST_LOG=debug cargo run --bin httprunner-gui
```

### Building for Release

```bash
# Native
cargo build --bin httprunner-gui --release
```

### Code Organization

```text
src/gui/
├── src/
│   ├── main.rs              # Native entry point (Iced)
│   ├── app.rs               # Main application logic
│   ├── file_tree.rs         # File browser component
│   ├── text_editor.rs       # Text editor component
│   ├── request_view.rs      # Request display
│   ├── results_view.rs      # Results display
│   ├── environment_editor.rs # Environment viewer
│   └── state.rs             # Persistent state
└── Cargo.toml               # Dependencies
```

---

## Technical Stack

- **UI Framework**: [Iced](https://github.com/iced-rs/iced) - Cross-platform GUI library inspired by Elm
- **HTTP Client**: [httprunner-core](../core) - HTTP request processing and execution
- **File Dialogs**: [rfd](https://github.com/PolyMeilex/rfd) - Async native file picker
- **Async Runtime**: [Tokio](https://tokio.rs/) - Asynchronous runtime for Rust

---

## Contributing

Found a bug or have a suggestion? [Open an issue](https://github.com/christianhelle/httprunner/issues)!

## License

MIT License - same as the parent HTTP Runner project
