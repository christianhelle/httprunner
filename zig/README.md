# HTTP File Runner - Zig Implementation

> ⚠️ **DEPRECATED**: This Zig implementation is being phased out in favor of the Rust implementation.
> This code is kept temporarily for reference and will be removed in a future version.

## Migration Notice

The HTTP File Runner has been reimplemented in Rust for better maintainability, ecosystem support, and performance. The new Rust implementation is now the default and recommended version.

### Why Rust?

- **Better ecosystem**: Rich library support for HTTP, async operations, and CLI tools
- **Active development**: More active development and modern tooling
- **Performance**: Comparable or better performance with better memory safety
- **Maintainability**: Easier to maintain and extend with a larger community

## Zig Implementation (Legacy)

This directory contains the original Zig implementation of HTTP File Runner.

### Building (Legacy)

If you still need to build the Zig version:

```bash
cd zig
zig build
```

### Prerequisites

- Zig 0.15.1 or later from https://ziglang.org/download/
- Git for version generation

### Usage

```bash
cd zig
./zig-out/bin/httprunner ../examples/simple.http
```

## Migration Path

If you're using the Zig version, please migrate to the Rust implementation:

1. Install Rust from https://rustup.rs/
2. Build the Rust version from the repository root: `cargo build --release`
3. The binary will be at `target/release/httprunner` (or `httprunner.exe` on Windows)

All features from the Zig version are available in the Rust implementation, including:
- HTTP request execution
- Variable substitution
- Request chaining
- Response assertions
- Environment variables
- File discovery
- Verbose and logging modes

## Timeline

This Zig implementation is planned for removal in a future major version update. Please plan your migration accordingly.

## Questions?

For questions or issues, please open an issue on the main repository at https://github.com/christianhelle/httprunner
