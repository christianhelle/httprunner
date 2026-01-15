# Building and Testing the Tauri GUI

## Quick Start

### On a System with Dependencies Installed

If you have already installed the required dependencies (see main README.md), you can build directly:

```bash
cd src/gui-tauri/src-tauri
cargo build
```

For release build:

```bash
cargo build --release
```

The binary will be available at:
- Debug: `target/debug/httprunner-gui-tauri`
- Release: `target/release/httprunner-gui-tauri`

### First Time Setup

1. Install system dependencies using the setup script:
   ```bash
   cd src/gui-tauri
   ./setup.sh
   ```

2. Build the application:
   ```bash
   cd src-tauri
   cargo build --release
   ```

3. Run the application:
   ```bash
   ./target/release/httprunner-gui-tauri
   ```

## Development Mode

For development with hot-reload (requires Tauri CLI):

```bash
# Install Tauri CLI (one time)
cargo install tauri-cli

# Run in development mode
cd src/gui-tauri
cargo tauri dev
```

This will:
- Watch for backend changes and recompile
- Watch for frontend changes and reload the UI
- Provide a better development experience

## Testing

To test the GUI functionality:

1. Launch the application
2. Click "📁 Open Directory" and select a directory containing .http files
3. Click on a file in the left sidebar to view its contents
4. Click on individual requests in the "Request Details" panel to run them
5. Or click "▶ Run All Requests" to execute all requests in the file
6. View results in the "Results" panel below

## Troubleshooting

### Linux Build Issues

If you encounter errors about missing libraries:

```bash
# Ubuntu/Debian
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

# Fedora
sudo dnf install webkit2gtk4.0-devel \
    openssl-devel \
    curl \
    wget \
    gtk3-devel \
    libappindicator-gtk3-devel \
    librsvg2-devel

# Arch
sudo pacman -S webkit2gtk \
    base-devel \
    curl \
    wget \
    openssl \
    gtk3 \
    libappindicator-gtk3 \
    librsvg
```

### macOS Build Issues

If you get Xcode errors:
```bash
xcode-select --install
```

### Windows Build Issues

Make sure you have:
- Visual Studio 2019 or later with C++ build tools
- Windows SDK

## Frontend Development

The frontend is simple HTML/CSS/JS in the `dist/` directory. You can:

1. Edit `dist/index.html` for structure
2. Edit `dist/styles.css` for styling  
3. Edit `dist/app.js` for functionality

The frontend communicates with the Rust backend via Tauri's IPC mechanism using the `invoke` function.

## Backend Development

The backend is in `src-tauri/src/main.rs`. Each `#[tauri::command]` function is callable from the frontend.

To add a new command:

1. Add the function in `main.rs`:
   ```rust
   #[tauri::command]
   fn my_command(param: String) -> Result<String, String> {
       Ok(format!("Received: {}", param))
   }
   ```

2. Register it in the `invoke_handler!` macro:
   ```rust
   .invoke_handler(tauri::generate_handler![
       my_command,
       // ... other commands
   ])
   ```

3. Call it from the frontend:
   ```javascript
   const result = await invoke('my_command', { param: 'hello' });
   ```

## Comparison with egui GUI

| Aspect | egui GUI | Tauri GUI |
|--------|----------|-----------|
| **Technology** | Native Rust UI (egui) | Web UI (HTML/CSS/JS) |
| **Bundle Size** | ~10-15 MB | ~15-20 MB |
| **Memory Usage** | Lower (~50-100 MB) | Moderate (~100-150 MB) |
| **Startup Time** | Faster | Slightly slower |
| **Customization** | Limited to egui widgets | Full web stack freedom |
| **Development** | Rust only | Separate frontend/backend |
| **Hot Reload** | Limited | Excellent |
| **Dependencies** | Fewer system deps | Requires WebKit/GTK on Linux |
| **Look & Feel** | egui style | Modern web UI |

## Next Steps

- Try building with different frontend frameworks (React, Vue, Svelte)
- Add syntax highlighting using Monaco Editor or CodeMirror
- Implement dark mode
- Add request history feature
- Export results functionality
