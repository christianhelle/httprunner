# Upgrade Module

This module handles self-update functionality for HTTP File Runner, with platform-specific implementations for Windows, Linux, and macOS.

## Structure

- `mod.rs` - Module entry point with conditional compilation
- `windows.rs` - Windows-specific upgrade implementation
- `linux.rs` - Linux-specific upgrade implementation
- `macos.rs` - macOS-specific upgrade implementation
- `unsupported.rs` - Fallback for unsupported platforms

## Usage

```rust
use crate::upgrade::run_upgrade;

// Upgrade to latest version
run_upgrade()?;
```

## Platform-Specific Implementations

### Windows
- Downloads latest release from GitHub
- Uses PowerShell for installation
- Handles Windows-specific paths and permissions
- Updates binary in place

### Linux
- Downloads latest release from GitHub
- Uses shell scripts for installation
- Handles Unix permissions (chmod +x)
- Updates binary in place

### macOS
- Downloads latest release from GitHub
- Uses shell scripts for installation
- Handles macOS-specific permissions
- Updates binary in place

### Unsupported Platforms
- Returns error indicating upgrade is not supported
- Provides manual upgrade instructions

## Upgrade Process

1. Fetch latest release information from GitHub API
2. Download appropriate binary for platform
3. Verify download integrity
4. Replace current binary with new version
5. Clean up temporary files
6. Report success or failure

## Error Handling

- Network errors (GitHub API unreachable)
- Download failures
- Permission errors
- File system errors
- Version detection errors

## Safety

- Creates backups before replacing binaries
- Validates downloads before installation
- Handles interrupted upgrades gracefully
- Preserves existing installation on failure
