# Quick Reference: Zig to Rust Migration

This document provides a quick reference for developers and users transitioning from the Zig implementation to the Rust implementation.

## Build Commands

| Task | Zig (Legacy) | Rust (Current) |
|------|--------------|----------------|
| Debug build | `zig build` | `cargo build` |
| Release build | `zig build -Doptimize=ReleaseFast` | `cargo build --release` |
| Run tests | `zig build test` | `cargo test` |
| Format code | `zig fmt .` | `cargo fmt` |
| Lint code | N/A | `cargo clippy` |
| Clean build | `rm -rf zig-out zig-cache` | `cargo clean` |

## Binary Location

| Implementation | Binary Path (Debug) | Binary Path (Release) |
|----------------|---------------------|----------------------|
| Zig | `zig-out/bin/httprunner` | `zig-out/bin/httprunner` |
| Rust | `target/debug/httprunner` | `target/release/httprunner` |

## File Locations

| Item | Zig (Legacy) | Rust (Current) |
|------|--------------|----------------|
| Source code | `src/*.zig` | `src/*.rs` |
| Build config | `build.zig` | `Cargo.toml` |
| Entry point | `src/main.zig` | `src/main.rs` |
| All Zig files | `zig/` directory | N/A |

## Usage Examples

### Running the binary

**Zig (Legacy):**
```bash
cd zig
./zig-out/bin/httprunner ../examples/simple.http --verbose
```

**Rust (Current):**
```bash
./target/release/httprunner examples/simple.http --verbose
```

### Development mode

**Zig (Legacy):**
```bash
cd zig
zig build
# Test with examples
./zig-out/bin/httprunner ../examples/simple.http
```

**Rust (Current):**
```bash
cargo build
# Or run directly
cargo run -- examples/simple.http
```

## Feature Parity

All features from the Zig implementation are available in Rust:

| Feature | Zig | Rust | Notes |
|---------|-----|------|-------|
| HTTP methods (GET, POST, etc.) | ✅ | ✅ | Full parity |
| Variable substitution | ✅ | ✅ | Full parity |
| Request variables | ✅ | ✅ | Full parity |
| Response assertions | ✅ | ✅ | Full parity |
| Environment files | ✅ | ✅ | Full parity |
| File discovery | ✅ | ✅ | Full parity |
| Verbose mode | ✅ | ✅ | Full parity |
| Logging to file | ✅ | ✅ | Full parity |
| Version info | ✅ | ✅ | Full parity |
| Self-upgrade | ✅ | ✅ | Full parity |
| Colored output | ✅ | ✅ | Full parity |

## Key Differences

### Dependencies

**Zig:**
- Zero external dependencies
- Uses standard library only
- Smaller binary size

**Rust:**
- Uses crates from crates.io
- Modern HTTP client (reqwest)
- Better ecosystem support
- Slightly larger binary

### Performance

Both implementations have similar performance characteristics:
- Network I/O is the bottleneck (not language)
- Both compile to native code
- Both are suitable for CI/CD pipelines

### Development

**Zig:**
- Simpler build system
- Fewer abstractions
- Less tooling

**Rust:**
- Rich ecosystem (cargo, rust-analyzer, clippy)
- Better IDE support
- More community resources
- Easier to maintain

## Installation

### For End Users

No change needed! Install scripts automatically download the correct binary:

```bash
# Linux/macOS
curl -fsSL https://christianhelle.com/httprunner/install | bash

# Windows
irm https://christianhelle.com/httprunner/install.ps1 | iex
```

### For Developers

**Zig (Legacy):**
```bash
# Install Zig 0.15.1+
cd zig
zig build
```

**Rust (Current):**
```bash
# Install Rust from https://rustup.rs/
cargo build --release
```

## Why the Switch?

1. **Ecosystem**: Rust has a richer ecosystem with more libraries
2. **Maintenance**: Easier to maintain with better tooling
3. **Community**: Larger community, more contributors
4. **Features**: Faster to implement new features
5. **Stability**: More stable language and tooling

## Migration Timeline

- ✅ **Now**: Rust is default, Zig is deprecated
- 📅 **Next**: CI/CD updates to prioritize Rust
- 📅 **Future**: Zig implementation will be removed

## Getting Help

- **Issues**: https://github.com/christianhelle/httprunner/issues
- **Discussions**: https://github.com/christianhelle/httprunner/discussions
- **Documentation**: See main README.md
- **Zig Legacy Docs**: See zig/README.md

## Common Migration Questions

### Q: Do I need to change my .http files?
**A:** No! The `.http` file format is identical between implementations.

### Q: Will my environment files still work?
**A:** Yes! `http-client.env.json` format is unchanged.

### Q: Can I still use the Zig version?
**A:** Yes, it's in the `zig/` directory, but it's no longer maintained.

### Q: What if I prefer Zig?
**A:** The Zig version will remain available for now, but we recommend migrating to Rust for continued updates and support.

### Q: Will CI/CD workflows need updates?
**A:** Yes, workflows should be updated to build the Rust implementation.

### Q: Are release binaries changing?
**A:** Yes, future releases will be built from the Rust implementation.

## Additional Resources

- Main README: [README.md](README.md)
- Restructuring details: [RESTRUCTURE.md](RESTRUCTURE.md)
- Rust quickstart: [QUICKSTART.md](QUICKSTART.md)
- Zig deprecation notice: [zig/README.md](zig/README.md)
