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
httprunner examples/simple.http
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

- **Base**: core22 (Ubuntu 22.04 LTS)
- **Confinement**: strict (full security isolation)
- **Plugs**: 
  - `network`: Required for HTTP requests
  - `home`: Access to user's home directory for .http files

## Files Created

- `snap/snapcraft.yaml` - Primary snap configuration
- `.snapcraft.yaml` - Alternative configuration in root
- `build-snap.sh` - Build script
- `.github/workflows/snap.yml` - GitHub Actions workflow
- `docs/SNAPCRAFT.md` - This documentation

## Troubleshooting

### Build Issues

1. **Zig version conflicts**: The snap downloads a specific Zig version. Update the version in snapcraft.yaml if needed.

2. **Network issues during build**: Ensure the build environment has internet access for downloading Zig.

### Installation Issues

1. **Permission denied**: Use `sudo` for snap installation
2. **Confinement issues**: Use `--devmode` for testing if strict confinement causes problems

### Publishing Issues

1. **Authentication**: Ensure you're logged in with `snapcraft login`
2. **Name conflicts**: The app name must be unique in the Snap Store
3. **Validation errors**: Check the snapcraft.yaml syntax

## Store Listing

When publishing, consider:

- **Description**: Clear, concise description of functionality
- **Screenshots**: Terminal screenshots showing colored output
- **Categories**: Choose appropriate categories (e.g., "Development", "Utilities")
- **Keywords**: HTTP, REST, API, testing, development tools

## Maintenance

- Update the version in snapcraft.yaml for new releases
- Monitor the Snap Store for user feedback and metrics
- Keep Zig version updated in the build configuration
