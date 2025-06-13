#!/bin/bash

# Build and install snapcraft if not already installed
if ! command -v snapcraft &> /dev/null; then
    echo "Installing snapcraft..."
    sudo snap install snapcraft --classic
fi

# Clean any previous builds
snapcraft clean

# Build the snap
echo "Building snap package..."
snapcraft

# List the generated snap file
echo "Snap package created:"
ls -la *.snap

echo ""
echo "To install locally for testing:"
echo "sudo snap install --dangerous --devmode ./httprunner_*.snap"
echo ""
echo "To upload to the Snap Store:"
echo "snapcraft upload ./httprunner_*.snap"
