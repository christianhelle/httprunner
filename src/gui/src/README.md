# HTTP Runner GUI

> **⚠️ Experimental**: This GUI application is currently in an experimental phase. Features and interface may change as development continues. 

A native cross-platform graphical user interface for HTTP Runner built with Rust and FLTK.

## Features

- 🎨 **Native UI** - Pure Rust with FLTK (Fast Light Toolkit)
- 🌐 **Cross-platform** - Works on Windows, macOS, and Linux
- 📁 **File Tree View** - Browse and select .http files with folder navigation
- 📋 **Request Inspector** - View request details including method, URL, headers, and body
- ▶️ **Run Requests** - Execute individual requests or entire files
- 🌍 **Environment Support** - Select environments for variable substitution
- 📊 **Live Results** - See execution results in real-time
- 🚀 **Fast & Responsive** - Thread-based async execution
- 💾 **State Persistence** - Remembers window size, last file, and results
- ⌨️ **Keyboard Shortcuts** - Ctrl+O, Ctrl+Q, F5 for common operations

## Building

### Prerequisites

- Rust 1.92 or later
- Development libraries for your platform:
  - **Linux**: X11/Wayland libraries, fontconfig, cairo, pango
  - **macOS**: Xcode command-line tools
  - **Windows**: Windows SDK (usually included with Visual Studio)

### Ubuntu/Debian

```bash
# Install dependencies
sudo apt-get install libx11-dev libxext-dev libxft-dev \
    libxinerama-dev libxcursor-dev libxrender-dev \
    libxfixes-dev libpango1.0-dev libcairo2-dev \
    libfontconfig1-dev

# Build the GUI
cargo build -p httprunner-gui --release
```

### macOS

```bash
# No additional dependencies needed
cargo build -p httprunner-gui --release
```

### Windows

```bash
# No additional dependencies needed (PowerShell or CMD)
cargo build -p httprunner-gui --release
```

## Running

```bash
# Run from source
cargo run -p httprunner-gui

# Or run the compiled binary
./target/release/httprunner-gui
```

## Usage

1. **Open Directory**: Use `File -> Open Directory` or press **Ctrl+O** to select a folder containing .http files
2. **Browse Files**: Click on files in the left panel to view their contents
3. **View Requests**: See request details displayed in the center panel
4. **Select Environment** (optional): Choose an environment from the dropdown in the top menu
5. **Run Requests**: Click "Run All Requests" button or press **F5** to execute all requests in the file
6. **View Results**: See execution results in the bottom panel

## Architecture

The GUI is structured into modular components:

- **`main.rs`** - Application entry point and FLTK window setup
- **`app_fltk.rs`** - Main application state, UI layout, and message handling
- **`file_tree.rs`** - File discovery and management
- **`request_view.rs`** - Request data access
- **`results_view.rs`** - Execution results and async runner
- **`request_editor.rs`** - Request editing logic
- **`state.rs`** - Application state persistence

The GUI shares the core logic with the CLI through the `httprunner-lib` library, ensuring consistent behavior.

## UI Layout

```
┌───────────────────────────────────────────────────────────┐
│ File | Environment: [None ▾]                              │
├──────────┬────────────────────────────────────────────────┤
│          │                                                │
│ Files    │  Request Details                               │
│  file1   │                                                │
│  file2   │  GET https://...                               │
│          │  Headers: ...                                  │
│          │                                                │
│          │  [Run All Requests]                            │
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
│ Working Directory: /path/to/files | Selected: file.http   │
└───────────────────────────────────────────────────────────┘
```

## Keyboard Shortcuts

- **F5**: Run all requests in the selected file
- **Ctrl+O** / **Cmd+O**: Open directory
- **Ctrl+Q** / **Cmd+Q**: Quit application

## Technical Details

- **Framework**: FLTK (Fast Light Toolkit) - retained mode GUI
- **Window System**: FLTK native windowing
- **File Dialogs**: native-dialog (native file picker)
- **Threading**: Standard library threads for async execution
- **Theme**: Greybird theme for modern appearance
- **Rendering**: Default backend (glow for OpenGL, wgpu optional)

## Troubleshooting

### Linux: Application won't start

If you see errors about missing display:
```
Error: DISPLAY is not set
```

Ensure you're running in a graphical environment with X11 or Wayland.

### Linux: Missing libraries

If you get linking errors about missing libraries, install the development libraries:
```bash
sudo apt-get install libx11-dev libxext-dev libxft-dev \
    libxinerama-dev libxcursor-dev libxrender-dev \
    libxfixes-dev libpango1.0-dev libcairo2-dev \
    libfontconfig1-dev
```

### macOS: "Cannot be opened because the developer cannot be verified"

```bash
# Remove quarantine attribute
xattr -d com.apple.quarantine ./target/release/httprunner-gui
```

### Windows: DPI Scaling

FLTK automatically handles DPI scaling on Windows. If issues persist, try adjusting your Windows display scaling settings.

## Development

To run in development mode with debugging output:

```bash
RUST_LOG=debug cargo run -p httprunner-gui
```

## License

MIT License - same as the parent HTTP Runner project
