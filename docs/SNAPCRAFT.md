# Snapcraft Distribution Guide

This guide explains how to build and distribute the HTTP Runner as a Snap package.

## Prerequisites

1. **Install Snapcraft**: 
   ```bash
   sudo snap install snapcraft --classic
   ```

2. **Create Snapcraft Account**: 
   - Go to https://snapcraft.io/
   - Create an account or log in
   - Register your app name: `snapcraft register httprunner`

## Building the Snap

### Method 1: Using the build script
```bash
./build-snap.sh
```

### Method 2: Manual build
```bash
snapcraft clean  # Clean previous builds
snapcraft        # Build the snap
```

## Testing Locally

Install the snap locally for testing:
```bash
sudo snap install --dangerous --devmode ./httprunner_*.snap
```

Test the installation:
```bash
# Test CLI tool
httprunner examples/simple.http

# Test GUI application
httprunner-gui
```

## Publishing to Snap Store

### One-time setup

1. **Login to Snapcraft**:
   ```bash
   snapcraft login
   ```

2. **Register the app name** (if not done already):
   ```bash
   snapcraft register httprunner
   ```

### Publishing

1. **Upload the snap**:
   ```bash
   snapcraft upload ./httprunner_*.snap
   ```

2. **Release to a channel**:
   ```bash
   snapcraft release httprunner <revision> stable
   ```

## Automated Publishing with GitHub Actions

The project includes a GitHub Actions workflow (`.github/workflows/snap.yml`) that automatically:

1. Builds the snap on every tag push
2. Uploads the snap as an artifact
3. Publishes to the Snap Store stable channel

### Setup GitHub Secrets

1. Generate Snapcraft credentials:
   ```bash
   snapcraft export-login credentials.txt
   ```

2. Add the contents of `credentials.txt` as a GitHub secret named `SNAPCRAFT_STORE_CREDENTIALS`

3. Create a git tag and push:
   ```bash
   git tag v1.0.0
   git push origin v1.0.0
   ```

## Snap Configuration

The snap is configured with:

- **Base**: core24 (Ubuntu 24.04 LTS)
- **Confinement**: strict (full security isolation)
- **Apps**: 
  - `httprunner`: CLI tool
  - `httprunner-gui`: GUI application
- **Plugs**: 
  - `network`: Required for HTTP requests
  - `home`: Access to user's home directory for .http files
  - GUI-specific plugs: `desktop`, `wayland`, `x11`, `opengl`, `pulseaudio`

## Files Created

- `snapcraft.yaml` - Primary snap configuration
- `snap/gui/httprunner-gui.desktop` - Desktop entry for GUI app
- `snap/gui/icon.png` - Application icon
- `build-snap.sh` - Build script (if exists)
- `.github/workflows/snap.yml` - GitHub Actions workflow
- `docs/SNAPCRAFT.md` - This documentation

## Troubleshooting

### Build Issues

1. **Missing GUI libraries**: The snap includes all necessary libraries for both CLI and GUI applications.

2. **Network issues during build**: Ensure the build environment has internet access for downloading dependencies.

### Installation Issues

1. **Permission denied**: Use `sudo` for snap installation
2. **Confinement issues**: Use `--devmode` for testing if strict confinement causes problems

### Publishing Issues

1. **Authentication**: Ensure you're logged in with `snapcraft login`
2. **Name conflicts**: The app name must be unique in the Snap Store
3. **Validation errors**: Check the snapcraft.yaml syntax

## Store Listing

When publishing, consider:

- **Description**: Clear description highlighting both CLI and GUI tools
- **Screenshots**: 
  - Terminal screenshots showing colored CLI output
  - GUI application screenshots showing the file browser and request inspector
- **Categories**: Choose appropriate categories (e.g., "Development", "Utilities")
- **Keywords**: HTTP, REST, API, testing, development tools, GUI, command-line

## Maintenance

- Update the version in snapcraft.yaml for new releases
- Monitor the Snap Store for user feedback and metrics
- Keep Rust toolchain and dependencies updated
- Test both CLI and GUI apps after updates
