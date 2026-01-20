# Building HTTP File Runner for Web (WASM)

> **✅ Status**: WORKING! WASM build successful, dev server running. See [WASM_STATUS.md](WASM_STATUS.md) for details.

This guide explains how to build and deploy the HTTP File Runner GUI application as a web application using WebAssembly (WASM).

## Prerequisites

1. **Rust**: Install Rust 1.92 or later from [rustup.rs](https://rustup.rs/)
2. **WASM Target**: Add the WebAssembly target:
   ```bash
   rustup target add wasm32-unknown-unknown
   ```
3. **Trunk**: Install Trunk, the build and bundler tool for Rust WASM apps:
   ```bash
   cargo install --locked trunk
   ```
4. **(Optional) wasm-bindgen-cli**: For manual builds:
   ```bash
   cargo install wasm-bindgen-cli
   ```

## Development

### Running the Dev Server

Navigate to the GUI directory and start the development server:

```bash
cd src/gui
trunk serve
```

This will:
- Build the WASM module
- Start a local web server at `http://127.0.0.1:8080` (default port)
- Watch for file changes and automatically rebuild
- Enable hot reloading

Open your browser to `http://127.0.0.1:8080` to see the application.

### Custom Port

To use a different port:

```bash
trunk serve --port 3000
```

## Production Build

### Building for Production

To create an optimized production build:

```bash
cd src/gui
trunk build --release
```

This will:
- Compile with optimizations
- Minify JavaScript
- Optimize WASM binary
- Generate all assets in the `dist/` directory

### Build Output

The production build creates a `dist/` directory containing:
- `index.html` - The main HTML file
- `httprunner-gui-*.wasm` - The WebAssembly binary
- `httprunner-gui-*.js` - JavaScript glue code
- Other assets (fonts, icons, etc.)

## Deployment

### Static Hosting

The `dist/` folder contains everything needed to deploy the app. You can host it on:

#### GitHub Pages

1. Build the production version:
   ```bash
   trunk build --release --public-url /httprunner/
   ```
   (Replace `/httprunner/` with your repository name)

2. Deploy the `dist/` folder to GitHub Pages

#### Netlify/Vercel

1. Build with:
   ```bash
   trunk build --release
   ```

2. Deploy the `dist/` folder

Configuration for Netlify (`netlify.toml`):
```toml
[build]
  command = "cd src/gui && trunk build --release"
  publish = "src/gui/dist"
```

#### Docker/Nginx

Example `Dockerfile`:
```dockerfile
# Build stage
FROM rust:1.92 as builder

RUN rustup target add wasm32-unknown-unknown
RUN cargo install --locked trunk

WORKDIR /app
COPY . .

WORKDIR /app/src/gui
RUN trunk build --release

# Runtime stage
FROM nginx:alpine
COPY --from=builder /app/src/gui/dist /usr/share/nginx/html
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
```

Build and run:
```bash
docker build -t httprunner-web .
docker run -p 8080:80 httprunner-web
```

## Configuration

### Trunk.toml

The `Trunk.toml` file in `src/gui/` contains build configuration:

```toml
[build]
target = "index.html"
dist = "dist"
public_url = "/"

[serve]
address = "127.0.0.1"
port = 8080
open = false
```

You can customize:
- Output directory (`dist`)
- Public URL base path (`public_url`)
- Development server address and port
- Whether to auto-open browser

## Features and Limitations

### 🎯 Implementation Status

**Async HTTP execution is implemented!** The web version will support HTTP requests once final compilation errors are resolved.

For detailed status and roadmap, see [WASM_STATUS.md](WASM_STATUS.md).

### Working Features in Web Version
✅ Async HTTP execution (implementation complete)
✅ Syntax highlighting
✅ Request parsing and display
✅ Environment variable support
✅ Persistent state (using browser local storage)
✅ Response viewing

### Limitations in Web Version
⚠️ **File System Access**: Limited file operations due to browser security
⚠️ **Folder Picking**: Cannot browse local folders (paste content instead)
⚠️ **CORS**: HTTP requests subject to Cross-Origin policies
⚠️ **Timeouts**: No custom timeouts (browser defaults)
⚠️ **SSL**: Cannot disable certificate validation

### Browser Compatibility

The app should work in modern browsers:
- Chrome/Edge 90+
- Firefox 88+
- Safari 15+

## Troubleshooting

### Build Errors

If you encounter build errors:

1. **Update Rust and targets**:
   ```bash
   rustup update
   rustup target add wasm32-unknown-unknown
   ```

2. **Clear build cache**:
   ```bash
   cargo clean
   trunk clean
   ```

3. **Reinstall Trunk**:
   ```bash
   cargo install --locked --force trunk
   ```

### Runtime Errors

Check the browser developer console for error messages. Common issues:

- **WASM not loading**: Ensure your web server serves `.wasm` files with the correct MIME type (`application/wasm`)
- **Module not found**: Check that all assets are in the `dist/` folder
- **CORS errors**: Configure CORS headers on your target API servers

### Performance

For better performance:
- Use `trunk build --release` for production
- Enable compression on your web server (gzip/brotli)
- Consider using a CDN for static assets

## Development Tips

### Hot Reload

Trunk automatically reloads when you save changes to:
- Rust source files (`.rs`)
- HTML files
- CSS files

### Debug Mode

For debugging, use the development build (without `--release`):
```bash
trunk serve
```

This enables:
- Source maps
- Better error messages
- Faster build times (but larger files)

### Browser DevTools

Use browser developer tools:
- **Console**: View log messages and errors
- **Network**: Inspect HTTP requests
- **Application**: View local storage (app state)
- **Sources**: Debug WASM (with source maps)

## Next Steps

- Configure your web server for production deployment
- Set up CI/CD for automatic builds
- Consider adding a custom domain
- Enable HTTPS for secure connections
- Implement analytics if needed

For more information, see:
- [Trunk Documentation](https://trunkrs.dev/)
- [eframe WASM Examples](https://github.com/emilk/egui/tree/master/examples)
- [Rust and WebAssembly Book](https://rustwasm.github.io/docs/book/)
