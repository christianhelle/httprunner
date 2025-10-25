# Dev Container for HTTP File Runner

This directory contains the development container configuration for the HTTP File Runner project.

## What's Included

- **Rust toolchain**: The programming language and toolchain (stable channel)
- **PowerShell Core**: For build scripts and cross-platform automation
- **VS Code Rust Extensions**: `rust-analyzer` for syntax highlighting, IntelliSense, and debugging
- **Universal Dev Container**: Pre-configured Linux environment with common development tools

## Usage

### GitHub Codespaces

1. Navigate to the [repository](https://github.com/christianhelle/httprunner)
2. Click the green "Code" button
3. Select "Codespaces" tab
4. Click "Create codespace on main"
5. Wait for the container to build and start
6. Start coding! 🚀

### VS Code Dev Containers

1. Install the [Dev Containers extension](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-containers)
2. Clone this repository locally
3. Open the repository in VS Code
4. When prompted, click "Reopen in Container" or use Command Palette: "Dev Containers: Reopen in Container"
5. Wait for the container to build and start

## Building and Testing

Once inside the dev container, you can:

```bash
# Build the project
cargo build

# Run tests
cargo test

# Run the application
cargo run -- --help

# Test with example files
cargo run -- examples/simple.http
```

## Features

The dev container provides:

- Rust analyzer for code completion and analysis
- PowerShell Core for build scripts and automation
- Syntax highlighting for `.rs` files
- Integrated terminal with Rust toolchain in PATH
- Git support for version control
- All dependencies pre-installed and configured

## Troubleshooting

If you encounter issues:

1. Ensure you have the latest version of VS Code
2. Make sure the Dev Containers extension is installed and updated
3. Try rebuilding the container: Command Palette → "Dev Containers: Rebuild Container"
4. Check the dev container logs for any error messages

## Legacy Zig Implementation

The Zig implementation has been moved to a separate repository: [christianhelle/httprunner-zig](https://github.com/christianhelle/httprunner-zig).