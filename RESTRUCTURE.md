# Repository Restructuring Summary

## Overview

This document summarizes the restructuring of the HTTP File Runner repository to support both Zig and Rust implementations, with Rust as the default.

## Changes Made

### 1. Directory Structure

**Before:**
```
httprunner/
├── src/              # Zig source code
├── rust/             # Rust implementation
├── build.zig         # Zig build configuration
├── examples/         # Example .http files
└── README.md         # Zig-focused documentation
```

**After:**
```
httprunner/
├── src/              # Rust source code (default)
├── zig/              # Zig implementation (deprecated)
│   ├── src/          # Zig source code
│   ├── build.zig     # Zig build configuration
│   └── README.md     # Deprecation notice
├── examples/         # Example .http files (shared)
├── Cargo.toml        # Rust configuration
└── README.md         # Rust-focused documentation
```

### 2. Files Moved to `zig/` Directory

The following Zig-specific files were moved to the `zig/` subdirectory:

- `build.zig` - Zig build configuration
- `src/` - Zig source code
- `dev-setup.ps1` - Zig development setup script
- `run.ps1` - PowerShell wrapper for Zig binary
- `Dockerfile` - Zig-specific Docker configuration
- `snapcraft.yaml` - Zig-specific Snap packaging
- `Makefile` - Zig build helpers

### 3. Files Moved to Root

The following Rust files were moved from `rust/` to the repository root:

- `src/` - Rust source code
- `Cargo.toml` - Rust package configuration
- `Cargo.lock` - Rust dependency lock file
- `build.rs` - Rust build script
- `test.http` - Test HTTP file
- `COMPLETION_REPORT.md` - Rust port completion report
- `PORT_SUMMARY.md` - Rust port summary
- `QUICKSTART.md` - Quickstart guide

### 4. Documentation Updates

#### Root README.md
- Updated badge from Zig to Rust version
- Added deprecation notice for Zig implementation
- Added "Implementations" section explaining both versions
- Changed all build instructions to use `cargo` instead of `zig build`
- Updated all binary paths from `zig-out/bin/httprunner` to `target/release/httprunner`
- Removed Zig-specific PowerShell wrapper (`run.ps1`) references
- Updated code structure section to reflect Rust modules
- Updated development section with Rust tooling (cargo, rust-analyzer, etc.)
- Added "Legacy Zig Implementation" section pointing to `zig/` directory

#### zig/README.md (New)
- Created comprehensive deprecation notice
- Explained why the project moved to Rust
- Provided migration guide
- Included basic build instructions for legacy users
- Set expectations for removal timeline

### 5. Configuration Updates

#### .gitignore
- Added Rust-specific ignore patterns at the top (`/target/`, `Cargo.lock`, `*.rs.bk`)
- Moved Zig patterns to "legacy" section
- Added paths for Zig subdirectory artifacts (`zig/zig-out/`, `zig/zig-cache/`, etc.)
- Reorganized for clarity with Rust as primary

### 6. Shared Resources

The following directories remain shared between both implementations:

- `examples/` - HTTP test files used by both implementations
- `docs/` - Documentation website
- `.github/` - CI/CD workflows (may need updating)
- `.devcontainer/` - Development container configuration
- `install.ps1` / `install.sh` - Installation scripts

## Implementation Status

### Rust Implementation (Active)
- ✅ Located in repository root
- ✅ Full feature parity with Zig version
- ✅ Actively maintained and developed
- ✅ Default for new users and installations
- ✅ Successfully builds and runs

### Zig Implementation (Deprecated)
- ✅ Located in `zig/` subdirectory
- ✅ Marked as deprecated in documentation
- ✅ Includes deprecation notice in README
- ⚠️ No longer actively developed
- 📅 Planned for removal in future version

## Migration Guide for Users

### For New Users
- Follow the main README.md instructions
- Use Rust implementation by default
- No action needed regarding Zig

### For Existing Zig Users
1. Install Rust from https://rustup.rs/
2. Clone/pull the latest repository
3. Build with `cargo build --release`
4. Binary will be at `target/release/httprunner`
5. All features are available in the Rust version

### For Contributors
- New contributions should target the Rust implementation (root `src/`)
- Zig implementation is frozen - no new features
- Update workflows to prioritize Rust builds

## Testing Performed

✅ Rust build succeeds: `cargo build`
✅ Rust binary runs: `./target/debug/httprunner --version`
✅ Rust binary executes HTTP requests: `./target/debug/httprunner examples/simple.http`
✅ Directory structure is clean and organized
✅ Documentation updated and consistent

## Next Steps

### Immediate
- ✅ Repository restructured
- ✅ Documentation updated
- ✅ Deprecation notices added

### Future
- Update CI/CD workflows to prioritize Rust builds
- Update release process to build Rust binaries
- Update Docker Hub images to use Rust implementation
- Set timeline for Zig implementation removal
- Update development container to focus on Rust

## Benefits of This Restructure

1. **Technical**: Rust implementation resolves Zig's HTTP/HTTPS configuration limitations
2. **Clarity**: Clear distinction between active and deprecated implementations
3. **Default**: Rust is obviously the default (in root directory)
4. **Preservation**: Zig code preserved for reference and transition period
5. **Migration Path**: Clear path for users to migrate from Zig to Rust
6. **Maintainability**: Easier to maintain with clear focus on Rust
7. **Community**: Better aligned with Rust ecosystem and community

## Technical Context

The restructuring was driven by fundamental limitations in Zig's HTTP implementation:

- **Primary Issue**: Zig's `std.http` client cannot bypass certificate validation for HTTPS calls
- **Impact**: Unable to test against development environments with self-signed certificates
- **Failed Alternative**: libcurl integration was too complex for cross-platform maintenance
- **Rust Solution**: The `reqwest` crate provides flexible, cross-platform HTTP/HTTPS support

## Rollback Plan

If needed, changes can be easily rolled back:
1. Move `zig/*` contents back to root
2. Move root `src/` and Rust files to `rust/` subdirectory  
3. Revert README.md and .gitignore changes
4. All git history is preserved

## Questions and Support

For questions about this restructuring:
- Open an issue: https://github.com/christianhelle/httprunner/issues
- Discussions: https://github.com/christianhelle/httprunner/discussions
- Author: Christian Helle (https://christianhelle.com)
