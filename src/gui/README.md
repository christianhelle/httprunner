# HTTP Runner GUI

> **⚠️ Experimental**: This GUI application is currently in an experimental phase. Features and interface may change as development continues.

A native cross-platform graphical user interface for HTTP Runner built with Rust and egui.

## 🌐 Try it Online

**Web App**: [HTTP Runner Web App](https://christianhelle.github.io/httprunner/app/)

The GUI is also available as a web application that runs in your browser - no installation required!

## Features

- 🎨 **Native UI** - Pure Rust, no web technologies
- 🌐 **Cross-platform** - Works on Windows, macOS, Linux, and **Web (WASM)**
- 📁 **File Tree View** - Browse and select .http files with folder navigation (native only)
- ✏️ **Text Editor** - Paste and edit HTTP requests directly (web version)
- 📋 **Request Inspector** - View request details including method, URL, headers, and body
- ▶️ **Run Requests** - Execute individual requests or entire files
- 🌍 **Environment Support** - Select environments for variable substitution
- 📊 **Live Results** - See execution results in real-time
- 🚀 **Fast & Responsive** - Desktop: thread-based async execution; Web: browser event-loop async

---

## Quick Start

### Web Version (Try Now!)

1. Go to [HTTP Runner Web App](https://christianhelle.github.io/httprunner/app/)
2. Paste your HTTP requests in the text editor
3. Click "Run All Requests"
4. View results!

**Note**: Web version is subject to CORS restrictions. For localhost/private APIs, use the desktop version.

### Desktop Version

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

### Web Application (WASM)

**Prerequisites:**
```bash
rustup target add wasm32-unknown-unknown
cargo install --locked trunk
```

**Development:**
```bash
cd src/gui
trunk serve
# Open http://127.0.0.1:8080/
```

**Production:**
```bash
cd src/gui
trunk build --release
# Output in dist/ folder
```

---

## Usage

### Desktop Application

1. **Open Directory**: `File -> Open Directory` or **Ctrl+O** to select a folder containing .http files
2. **Browse Files**: Click on files in the left panel to view their contents
3. **View Requests**: Expand requests in the center panel to see details
4. **Select Environment** (optional): Choose an environment from the dropdown or cycle with **Ctrl+E**
5. **Run Requests**: 
   - Click "▶ Run All Requests" to execute all requests in the file
   - Press **F5** to run all requests in the selected file
   - Click "▶ Run this request" on any individual request to execute it
6. **View Results**: See execution results in the right panel

### Web Application

1. **Paste HTTP Requests**: Use the text editor to paste or type HTTP requests
2. **Select Environment** (optional): Choose from the dropdown if you have environment variables
3. **Run Requests**: Click "▶ Run All Requests"
4. **View Results**: See responses in the bottom panel

Example request:
```http
GET https://httpbin.org/get
Accept: application/json
```

---

## Keyboard Shortcuts

- **F5**: Run all requests in the selected file
- **Ctrl+R** / **Cmd+R**: Run all requests
- **Ctrl+O** / **Cmd+O**: Open directory (native only)
- **Ctrl+Q** / **Cmd+Q**: Quit application
- **Ctrl+E** / **Cmd+E**: Cycle through environments
- **Ctrl+T** / **Cmd+T**: Toggle between Text Editor and Request Details view
- **Ctrl+B** / **Cmd+B**: Toggle file tree visibility
- **Ctrl+D** / **Cmd+D**: Toggle results view (Compact/Verbose)
- **Ctrl+S** / **Cmd+S**: Save file
- **Ctrl+Plus** / **Cmd+Plus**: Zoom in
- **Ctrl+Minus** / **Cmd+Minus**: Zoom out
- **Ctrl+0** / **Cmd+0**: Reset zoom

---

## Architecture

### Components

- **`main.rs`** - Native application entry point
- **`lib.rs`** - WASM entry point (web version)
- **`app.rs`** - Main application state and UI layout
- **`file_tree.rs`** - File browser with tree view (native only)
- **`text_editor.rs`** - Text editor for manual input (WASM-friendly)
- **`request_view.rs`** - Request details display
- **`results_view.rs`** - Execution results (sync for native)
- **`results_view_async.rs`** - Execution results (async for WASM)

### Dual Execution Modes

**Native (Desktop):**
```rust
// Uses synchronous reqwest with blocking I/O
pub fn execute_http_request(request: &HttpRequest) -> Result<HttpResult>
```

**WASM (Browser):**
```rust
// Uses async reqwest (required in browsers)
pub async fn execute_http_request_async(request: &HttpRequest) -> Result<HttpResult>
```

---

## Platform Comparison

| Feature | Native GUI | Web App |
|---------|------------|---------|
| HTTP Execution | ✅ Sync | ✅ Async |
| File System | ✅ Full | ❌ Limited (paste content) |
| Folder Browsing | ✅ Yes | ❌ No |
| Localhost APIs | ✅ Yes | ❌ CORS blocked |
| Custom Timeouts | ✅ Yes | ❌ Browser defaults |
| Self-signed Certs | ✅ Yes (with --insecure) | ❌ Browser enforces SSL |
| CORS | ✅ No restrictions | ⚠️ Server must allow |
| Installation | Required | None |
| Performance | ⚡ Native speed | 🌐 Near-native via WASM |

---

## Deployment (Web Version)

The web app is automatically deployed to GitHub Pages via `.github/workflows/docs.yml`:

1. Builds WASM with `trunk build --release --public-url /httprunner/app/`
2. Deploys to [HTTP Runner Web App](https://christianhelle.github.io/httprunner/app/)
3. Integrated with documentation site

### Manual Deployment

Deploy to any static hosting service:

```bash
cd src/gui
trunk build --release --public-url /
# Deploy dist/ folder to:
# - GitHub Pages
# - Netlify
# - Vercel
# - Any static host
```

---

## Troubleshooting

### Linux: Application won't start

```bash
# Ensure you're in a graphical environment
echo $DISPLAY  # Should output :0 or :1

# Install required libraries
sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
    libxkbcommon-dev libwayland-dev
```

### macOS: "Cannot be opened because the developer cannot be verified"

```bash
xattr -d com.apple.quarantine ./target/release/httprunner-gui
```

### Windows: DPI Scaling Issues

The application should automatically detect DPI settings. If text appears incorrect, adjust Windows display scaling.

### Web: CORS Errors

The browser blocks requests to APIs without CORS headers. Solutions:

1. **Use the desktop GUI** for localhost/private APIs
2. **Configure CORS** on your API server
3. **Test with public APIs** that support CORS (e.g., httpbin.org)

### Web: Can't Load Files

The web version doesn't have file system access. Instead:
- Paste HTTP request content directly into the text editor
- Content is stored in browser LocalStorage

---

## Development

### Running in Debug Mode

```bash
# Native
RUST_LOG=debug cargo run --bin httprunner-gui

# Web
cd src/gui
trunk serve
# Check browser console (F12) for logs
```

### Building for Release

```bash
# Native
cargo build --bin httprunner-gui --release

# Web
cd src/gui
trunk build --release
```

### Code Organization

```text
src/gui/
├── src/
│   ├── main.rs              # Native entry point
│   ├── lib.rs               # WASM entry point
│   ├── app.rs               # Main application logic
│   ├── file_tree.rs         # File browser (native only)
│   ├── text_editor.rs       # Text editor (WASM-friendly)
│   ├── request_view.rs      # Request display
│   ├── results_view.rs      # Results display (sync)
│   ├── results_view_async.rs # Results display (async)
│   └── state.rs             # Persistent state (FS/LocalStorage)
├── index.html               # Web app HTML template
├── Trunk.toml               # WASM build configuration
└── Cargo.toml               # Dependencies
```

---

## Technical Stack

- **UI Framework**: [egui](https://github.com/emilk/egui) - Immediate mode GUI
- **Window System**: [eframe](https://github.com/emilk/egui/tree/master/crates/eframe) - egui app framework
- **HTTP Client**: [reqwest](https://github.com/seanmonstar/reqwest) - Sync for native, async for WASM
- **WASM Build**: [Trunk](https://trunkrs.dev/) - WASM web application bundler
- **File Dialogs**: [rfd](https://github.com/PolyMeilex/rfd) - Native file picker (native only)
- **Time**: [web-time](https://github.com/daxpedda/web-time) - WASM-compatible timing

---

## Contributing

Found a bug or have a suggestion? [Open an issue](https://github.com/christianhelle/httprunner/issues)!

## License

MIT License - same as the parent HTTP Runner project
