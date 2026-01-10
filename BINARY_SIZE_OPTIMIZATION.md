# Binary Size Optimization Guide

This document describes the optimizations applied to minimize the binary size of `httprunner`.

## Results Summary

| Version | Binary Size | Reduction |
|---------|-------------|-----------|
| Original | 3.1 MB | - |
| Optimized | 2.7 MB | 400 KB (13%) |
| With UPX | 818 KB | 2.3 MB (74%) |

## Applied Optimizations

### 1. Dependency Removal

**Removed `uuid` crate** (saved ~50 KB)
- Replaced with simple random UUID v4 generation using `rand`
- Implementation in `src/functions/generator.rs`
- Generates RFC 4122 compliant UUID v4 strings without external dependency

**Removed `chrono` crate** (saved ~22 KB)
- Replaced with `std::time` for timestamp formatting
- Custom date/time formatting functions in:
  - `src/report/formatter.rs` - for report timestamps
  - `src/report/writer.rs` - for filename timestamps
- Pure Rust implementation with zero-copy UTC time conversion

### 2. Feature Optimization

All dependencies configured with minimal feature sets:

```toml
reqwest = { version = "0.12", features = ["blocking", "json", "native-tls"], default-features = false }
serde_json = { version = "1.0", default-features = false, features = ["std"] }
clap = { version = "4.5", features = ["derive", "std", "help", "usage", "error-context"], default-features = false }
anyhow = { version = "1.0", default-features = false, features = ["std"] }
base64 = { version = "0.22.1", default-features = false, features = ["std"] }
```

Key points:
- `default-features = false` prevents unused features from being compiled
- Only essential features are explicitly enabled
- Native TLS used instead of rustls (smaller binary size)

### 3. Cargo Profile Optimization

```toml
[profile.release]
opt-level = "z"       # Optimize for size
lto = "fat"           # Enable aggressive Link Time Optimization
codegen-units = 1     # Reduce parallel code generation for smaller binary
panic = "abort"       # Abort on panic instead of unwinding
strip = true          # Strip symbols from binary

[profile.release.build-override]
opt-level = "z"
codegen-units = 1
```

**Profile settings explained:**
- `opt-level = "z"` - Aggressive size optimization (slower runtime than "3", but smaller binary)
- `lto = "fat"` - Cross-crate optimizations, eliminates dead code
- `codegen-units = 1` - Single compilation unit for better optimization
- `panic = "abort"` - No unwinding support reduces binary size
- `strip = true` - Removes debug symbols

### 4. Optional: UPX Compression

For deployment scenarios where startup time is not critical, UPX compression can reduce binary size by ~70%:

```bash
# Install UPX (if not already installed)
# Ubuntu/Debian: apt-get install upx
# macOS: brew install upx

# Compress the binary
upx --best --lzma target/release/httprunner
```

**Pros:**
- Significant size reduction (2.7 MB → 818 KB)
- Works on most platforms

**Cons:**
- Slower startup time (decompression overhead)
- May trigger false positives in some antivirus software
- Not suitable for frequently-executed commands

## Size Analysis

Top contributors to binary size (from `cargo bloat --release --crates`):

| Crate | Size | Percentage | Notes |
|-------|------|------------|-------|
| std | 388 KB | 27% | Rust standard library (unavoidable) |
| regex_automata | 223 KB | 16% | Required for pattern matching |
| clap_builder | 117 KB | 8% | CLI argument parsing |
| aho_corasick | 117 KB | 8% | Regex dependency |
| reqwest | 124 KB | 9% | HTTP client (core functionality) |
| httprunner | 91 KB | 6% | Application code |

## Future Optimization Opportunities

1. **Alternative CLI Parser**: Consider replacing `clap` with a lighter alternative like `pico-args` or `lexopt` (could save ~100 KB)
2. **Regex Optimization**: Evaluate if all regex patterns can be simplified or replaced with string operations
3. **Selective Feature Compilation**: Add compile-time features to make some functionality optional

## Build Instructions

```bash
# Standard optimized build
cargo build --release

# The binary will be at: target/release/httprunner

# Optional: Compress with UPX
upx --best --lzma target/release/httprunner
```

## Testing

All optimizations maintain full compatibility with existing functionality:

```bash
cargo test
```

All 285 tests pass with optimized build.
