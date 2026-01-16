# HTTP File Runner - Tauri GUI

This directory contains the Tauri-based GUI implementation for HTTP File Runner.

## What is Tauri?

[Tauri](https://tauri.app/) is a framework for building desktop applications using web technologies (HTML, CSS, JavaScript) for the frontend and Rust for the backend. Unlike Electron, Tauri uses the system's native webview, resulting in smaller bundle sizes and lower memory footprint.

## Architecture

The Tauri GUI consists of two main parts:

### Backend (src-tauri/)
- Written in Rust
- Exposes commands that the frontend can call
- Handles file operations, HTTP request execution, environment management
- Uses the httprunner-lib core library for HTTP request processing

### Frontend (dist/)
- Simple HTML/CSS/JavaScript application
- No build step required - uses vanilla JS
- Communicates with the backend via Tauri's IPC mechanism

## Features

- ✅ Directory browsing and .http file discovery
- ✅ File content viewing and editing
- ✅ Request parsing and display
- ✅ Environment selection (via http-client.env.json)
- ✅ Single request execution
- ✅ Batch request execution
- ✅ Results display with formatted JSON
- ✅ State persistence

## Prerequisites

### Linux
```bash
sudo apt-get update
sudo apt-get install -y \
    libwebkit2gtk-4.0-dev \
    build-essential \
    curl \
    wget \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev
```

### macOS
```bash
xcode-select --install
```

### Windows
No additional dependencies required.

## Building

From the repository root:

```bash
# Install Tauri CLI (optional, can use cargo directly)
cargo install tauri-cli

# Build the Tauri app
cd src/gui-tauri/src-tauri
cargo build --release

# Or using Tauri CLI from the gui-tauri directory
cd src/gui-tauri
cargo tauri build
```

## Development

For development with hot-reload:

```bash
cd src/gui-tauri
cargo tauri dev
```

## Comparison with egui GUI

| Feature | egui GUI | Tauri GUI |
|---------|----------|-----------|
| UI Framework | egui (immediate mode GUI) | HTML/CSS/JS |
| Bundle Size | Smaller | Larger (but still smaller than Electron) |
| Memory Usage | Lower | Moderate (native webview) |
| Customization | Limited to egui widgets | Full web technologies |
| Cross-platform | Native rendering | Webview-based |
| Development Speed | Rust-only | Separate frontend/backend |
| Hot Reload | Limited | Excellent |

## Why Tauri?

The main goal of this port is to demonstrate different UI framework options in Rust:

1. **Easier UI Development**: Using HTML/CSS/JS is familiar to many developers
2. **Rich Ecosystem**: Access to all web technologies and libraries
3. **Modern Look**: Easy to create modern, polished UIs
4. **Framework Flexibility**: Can integrate React, Vue, Svelte, or vanilla JS

## Directory Structure

```
gui-tauri/
├── src-tauri/          # Rust backend
│   ├── Cargo.toml      # Dependencies
│   ├── build.rs        # Build script
│   ├── tauri.conf.json # Tauri configuration
│   ├── capabilities/   # Security permissions
│   └── src/
│       ├── main.rs     # Backend commands and app initialization
│       └── lib.rs      # Library exports
└── dist/               # Frontend (HTML/CSS/JS)
    ├── index.html      # Main application HTML
    ├── styles.css      # Styling
    └── app.js          # Application logic
```

## Backend Commands

The following commands are exposed to the frontend:

- `set_root_directory(path)` - Set working directory
- `get_root_directory()` - Get current working directory
- `list_http_files()` - List all .http files in directory
- `read_file_content(path)` - Read file content
- `write_file_content(path, content)` - Write file content
- `select_file(path)` - Mark file as selected
- `get_selected_file()` - Get currently selected file
- `parse_http_file(path)` - Parse .http file into requests
- `list_environments(path)` - Get available environments
- `set_environment(env)` - Set selected environment
- `get_environment()` - Get current environment
- `run_single_request(path, index, environment)` - Execute single request
- `run_all_requests(path, environment)` - Execute all requests

## Security

Tauri has a capability-based security model. The app only has permission to:
- Use core Tauri APIs
- Open file/folder dialogs
- Save file dialogs

All file operations are handled through backend commands with full control.

## Future Enhancements

Potential improvements for the Tauri GUI:

- [ ] Syntax highlighting for .http files (using Monaco Editor or CodeMirror)
- [ ] Request history
- [ ] Export results to file
- [ ] Dark mode support
- [ ] Custom themes
- [ ] Split view for multiple requests
- [ ] WebSocket support visualization
- [ ] GraphQL support
- [ ] Request collections/workspaces

## License

MIT License - Same as the parent project
