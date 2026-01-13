# HTTP Runner GUI

> **⚠️ Experimental**: This GUI application is currently in an experimental phase. Features and interface may change as development continues. 

A native cross-platform graphical user interface for HTTP Runner built with Rust and egui.

## Features

- 🎨 **Native UI** - Pure Rust, no web technologies
- 🌐 **Cross-platform** - Works on Windows, macOS, and Linux
- 📁 **File Tree View** - Browse and select .http files with folder navigation
- 📋 **Request Inspector** - View request details including method, URL, headers, and body
- ▶️ **Run Requests** - Execute individual requests or entire files
- 🌍 **Environment Support** - Select environments for variable substitution
- 📊 **Live Results** - See execution results in real-time
- 🚀 **Fast & Responsive** - Thread-based async execution

## Building

### Prerequisites

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

## Running

```bash
# Run from source
cargo run --bin httprunner-gui --features gui

# Or run the compiled binary
./target/release/httprunner-gui
```

## Usage

1. **Open Directory**: Use `File -> Open Directory` to select a folder containing .http files
2. **Browse Files**: Click on files in the left panel to view their contents
3. **View Requests**: Expand requests in the center panel to see details
4. **Select Environment** (optional): Choose an environment from the dropdown in the top menu
5. **Run Requests**: 
   - Click "▶ Run All Requests" to execute all requests in the file
   - Click "▶ Run this request" on any individual request to execute it
6. **View Results**: See execution results in the bottom panel

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
┌───────────────────────────────────────────────────────────┐
│ File | Environment: [None ▾]                              │
├──────────┬────────────────────────────────────────────────┤
│          │                                                │
│ Dir      │  Request Details                               │
│  File    │                                                │
│  File    │  [Request 1] -> Run this request               │
│          │  [Request 2] -> Run this request               │
│          │                                                │
│          │  Run All Requests                              │
│          │                                                │
├──────────┼────────────────────────────────────────────────┤
│          │  Results                                       │
│          │                                                │
│          │  SUCCESS                                       │
│          │  GET https://...                               │
│          │  Status: 200                                   │
│          │  Duration: 123 ms                              │
│          │                                                │
│          │  Response:                                     │
│          │  { "data": "..." }                             │
└──────────┴────────────────────────────────────────────────┘
│ Working Directory: /path/to/files                         │
└───────────────────────────────────────────────────────────┘
```

## Keyboard Shortcuts

- **Ctrl+Q** / **Cmd+Q**: Quit application (menu)
- **Ctrl+O** / **Cmd+O**: Open directory (menu)

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
