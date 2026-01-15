# HTTP Runner GUI

> **⚠️ Experimental**: This GUI application is currently in an experimental phase. Features and interface may change as development continues. 

A native cross-platform graphical user interface for HTTP Runner built with Rust and Slint.

## Features

- 🎨 **Native UI** - Pure Rust with declarative Slint UI language, no web technologies
- 🌐 **Cross-platform** - Works on Windows, macOS, and Linux
- 📁 **File Tree View** - Browse and select .http files
- 📋 **Request Inspector** - View request details including method, URL, headers, and body
- ▶️ **Run Requests** - Execute individual requests or entire files
- 🌍 **Environment Support** - Select environments for variable substitution
- 📊 **Live Results** - See execution results in real-time
- 🚀 **Fast & Responsive** - Native rendering with software or hardware acceleration

## Building

### Prerequisites

- Rust 1.92 or later
- Development libraries for your platform:
  - **Linux**: `libxcb`, `libxkbcommon`, `libwayland-client`, `libwayland-cursor`, `libfontconfig`
  - **macOS**: Xcode command-line tools
  - **Windows**: Windows SDK (usually included with Visual Studio)

### Ubuntu/Debian

```bash
# Install dependencies
sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev libfontconfig1-dev libwayland-dev

# Build the GUI
cargo build --package httprunner-gui --release
```

### macOS

```bash
# No additional dependencies needed
cargo build --package httprunner-gui --release
```

### Windows

```bash
# No additional dependencies needed (PowerShell or CMD)
cargo build --package httprunner-gui --release
```

## Running

```bash
# Run from source
cargo run --package httprunner-gui

# Or run the compiled binary
./target/release/httprunner-gui
```

## Usage

1. **Open Directory**: Use "Open Directory..." button to select a folder containing .http files
2. **Browse Files**: Click on files in the left panel to view their contents
3. **View Requests**: See all requests displayed in the center panel with their details
4. **Select Environment** (optional): Choose an environment from the dropdown in the top menu
5. **Run Requests**: 
   - Click "▶ Run All Requests" to execute all requests in the file
   - Click "▶ Run" on any individual request to execute it
6. **View Results**: See execution results in the bottom panel

## Architecture

The GUI is built with a clean separation between UI and business logic:

- **`ui/main.slint`** - Declarative UI definition using Slint markup language
- **`main.rs`** - Application entry point, UI callbacks, and state management
- **`state.rs`** - Application state persistence

The GUI shares the core HTTP execution logic with the CLI through the `httprunner-lib` library, ensuring consistent behavior.

## UI Layout

```
┌───────────────────────────────────────────────────────────┐
│ [Open Directory] [New .http File]  Environment: [None ▾] │
├──────────┬────────────────────────────────────────────────┤
│          │                                                │
│ HTTP     │  Request Details                               │
│ Files    │                                                │
│  📄 file │  [Request 1 - GET /api/users]                  │
│  📄 file │    Method: GET                                 │
│          │    URL: https://api.example.com/users          │
│          │    [▶ Run]                                     │
│          │                                                │
│          │  [Request 2 - POST /api/data]                  │
│          │    Method: POST                                │
│          │    URL: https://api.example.com/data           │
│          │    Headers: Content-Type: application/json     │
│          │    Body: {...}                                 │
│          │    [▶ Run]                                     │
│          │                                                │
│          │  [▶ Run All Requests] [💾 Save]                │
├──────────┼────────────────────────────────────────────────┤
│          │  Results                                       │
│          │                                                │
│          │  ┌─────────────────────────────────────────┐   │
│          │  │ GET https://api.example.com/users       │   │
│          │  │ Status: 200  Duration: 123ms            │   │
│          │  │ Response:                               │   │
│          │  │ { "users": [...] }                      │   │
│          │  └─────────────────────────────────────────┘   │
│          │                                                │
└──────────┴────────────────────────────────────────────────┘
│ Working Directory: /path/to/files  Selected: file.http    │
└───────────────────────────────────────────────────────────┘
```

## Technical Details

- **Framework**: Slint 1.14+ (declarative UI framework for Rust)
- **Rendering**: Software renderer via femtovg (hardware acceleration optional)
- **Window System**: Winit (cross-platform window creation)
- **File Dialogs**: rfd (native file picker)
- **State Management**: Rust with Slint property bindings and callbacks

### Why Slint?

The GUI was ported from egui to Slint to provide:
- **Better separation of concerns**: UI definition (`.slint` files) separate from business logic (Rust code)
- **Declarative UI**: Easier to maintain and understand UI layout
- **Flexible HTTPS configuration**: Full control over TLS settings including optional `--insecure` flag support
- **Embedded systems support**: Slint is designed to work well on resource-constrained devices

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
    libxkbcommon-dev libwayland-dev libfontconfig1-dev
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
RUST_LOG=debug cargo run --package httprunner-gui
```

### Modifying the UI

The UI is defined in `ui/main.slint`. Slint provides a declarative language for defining user interfaces. See the [Slint documentation](https://slint.dev/docs) for more information on the Slint language.

Changes to `.slint` files are automatically compiled during the build process via `build.rs`.

## Migration from egui

This GUI was originally built with egui and has been ported to Slint for better flexibility and maintainability. Key changes:

- UI definition moved from Rust code to declarative `.slint` files
- Simpler state management with Slint's property binding system
- More control over HTTPS behavior (supports `--insecure` flag)
- Better separation between UI and business logic

## License

MIT License - same as the parent HTTP Runner project
