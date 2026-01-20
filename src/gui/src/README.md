# HTTP Runner GUI

> **⚠️ Experimental**: This GUI application is currently in an experimental phase. Features and interface may change as development continues. 

A native cross-platform graphical user interface for HTTP Runner built with Rust and egui.

> **🌐 Web Version Available**: The GUI can also run in the browser using WebAssembly! See [BUILD_WEB.md](BUILD_WEB.md) for instructions.

## Features

- 🎨 **Native UI** - Pure Rust, no web technologies
- 🌐 **Cross-platform** - Works on Windows, macOS, Linux, and **Web (WASM)**
- 📁 **File Tree View** - Browse and select .http files with folder navigation
- 📋 **Request Inspector** - View request details including method, URL, headers, and body
- ▶️ **Run Requests** - Execute individual requests or entire files
- 🌍 **Environment Support** - Select environments for variable substitution
- 📊 **Live Results** - See execution results in real-time
- 🚀 **Fast & Responsive** - Thread-based async execution

## Building

> **⚠️ Web Version Status**: The WASM/web version infrastructure is set up but HTTP execution is not yet functional due to blocking I/O limitations. See [WASM_STATUS.md](WASM_STATUS.md) for details.

### Native Application

#### Prerequisites

- Rust 1.92 or later
- Development libraries for your platform:
  - **Linux**: `libxcb`, `libxkbcommon`, `libwayland-client`, `libwayland-cursor`
  - **macOS**: Xcode command-line tools
  - **Windows**: Windows SDK (usually included with Visual Studio)

### Ubuntu/Debian

```bash
# Install dependencies
sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev libfontconfig1-dev libwayland-dev

# Build the GUI
cargo build --bin httprunner-gui --features gui --release
```

### macOS

```bash
# No additional dependencies needed
cargo build --bin httprunner-gui --features gui --release
```

### Windows

```bash
# No additional dependencies needed (PowerShell or CMD)
cargo build --bin httprunner-gui --features gui --release
```

### Web Application (WASM)

To build and run the web version:

```bash
# Install prerequisites
rustup target add wasm32-unknown-unknown
cargo install --locked trunk

# Development server
cd src/gui
trunk serve

# Production build
trunk build --release
```

For detailed instructions, deployment options, and troubleshooting, see [BUILD_WEB.md](BUILD_WEB.md).

## Running

```bash
# Run from source
cargo run --bin httprunner-gui --features gui

# Or run the compiled binary
./target/release/httprunner-gui
```

## Usage

1. **Open Directory**: Use `File -> Open Directory` or press **Ctrl+O** to select a folder containing .http files
2. **Browse Files**: Click on files in the left panel to view their contents
3. **View Requests**: Expand requests in the center panel to see details
4. **Select Environment** (optional): Choose an environment from the dropdown in the top menu or cycle through them with **Ctrl+E**
5. **Run Requests**: 
   - Click "▶ Run All Requests" to execute all requests in the file
   - Press **F5** to run all requests in the selected file
   - Click "▶ Run this request" on any individual request to execute it
6. **View Results**: See execution results in the right panel

## Architecture

The GUI is structured into modular components:

- **`main.rs`** - Application entry point and window setup
- **`app.rs`** - Main application state and UI layout
- **`file_tree.rs`** - File browser with tree view
- **`request_view.rs`** - Request details display
- **`results_view.rs`** - Execution results and async runner

The GUI shares the core logic with the CLI through the `httprunner` library, ensuring consistent behavior.

## UI Layout

```
┌───────────────────────────────────────────────────────────────┐
│ File | Environment: [None ▾]                                  │
├──────────┬────────────────────────────────┬────────────────────┤
│          │                                │                    │
│ Dir      │  Request Details               │  Results           │
│  File    │                                │                    │
│  File    │  [Request 1] -> Run this       │  SUCCESS           │
│          │  [Request 2] -> Run this       │  GET https://...   │
│          │                                │  Status: 200       │
│          │  Run All Requests              │  Duration: 123 ms  │
│          │                                │                    │
│          │                                │  Response:         │
│          │                                │  { "data": "..." } │
│          │                                │                    │
└──────────┴────────────────────────────────┴────────────────────┘
│ Working Directory: /path/to/files                             │
└───────────────────────────────────────────────────────────────┘
```

## Keyboard Shortcuts

- **F5**: Run all requests in the selected file
- **Ctrl+R** / **Cmd+R**: Run all requests in the selected file
- **Ctrl+O** / **Cmd+O**: Open directory
- **Ctrl+Q** / **Cmd+Q**: Quit application
- **Ctrl+E** / **Cmd+E**: Cycle through environments
- **Ctrl+T** / **Cmd+T**: Toggle between Text Editor and Request Details view
- **Ctrl+B** / **Cmd+B**: Toggle file tree visibility
- **Ctrl+D** / **Cmd+D**: Toggle results view (Compact/Verbose)
- **Ctrl+S** / **Cmd+S**: Save file
- **Ctrl+Plus** / **Cmd+Plus**: Zoom in (increase font size)
- **Ctrl+Minus** / **Cmd+Minus**: Zoom out (decrease font size)
- **Ctrl+0** / **Cmd+0**: Reset font size to default

## Technical Details

- **Framework**: egui (immediate mode GUI)
- **Window System**: eframe (egui app framework)
- **File Dialogs**: rfd (native file picker)
- **Threading**: Standard library threads for async execution
- **Rendering**: Default backend (glow for OpenGL, wgpu optional)

## Troubleshooting

### Linux: Application won't start

If you see errors about missing display:
```
Error: neither WAYLAND_DISPLAY nor WAYLAND_SOCKET nor DISPLAY is set
```

Ensure you're running in a graphical environment with X11 or Wayland.

### Linux: Missing libraries

If you get errors about missing `.so` files, install the development libraries:
```bash
sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
    libxkbcommon-dev libwayland-dev
```

### macOS: "Cannot be opened because the developer cannot be verified"

```bash
# Remove quarantine attribute
xattr -d com.apple.quarantine ./target/release/httprunner-gui
```

### Windows: DPI Scaling Issues

The application should automatically detect and use your system's DPI settings. If text appears too small or large, try adjusting your Windows display scaling settings.

## Development

To run in development mode with debugging output:

```bash
RUST_LOG=debug cargo run --bin httprunner-gui --features gui
```

## License

MIT License - same as the parent HTTP Runner project
