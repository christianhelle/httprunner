#!/bin/bash

# Setup script for Tauri development dependencies

set -e

echo "🔧 Setting up Tauri development environment..."

# Detect OS
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    echo "📦 Installing Linux dependencies..."
    
    # Check if running Debian/Ubuntu
    if command -v apt-get &> /dev/null; then
        sudo apt-get update
        sudo apt-get install -y \
            libwebkit2gtk-4.0-dev \
            build-essential \
            curl \
            wget \
            libssl-dev \
            libgtk-3-dev \
            libayatana-appindicator3-dev \
            librsvg2-dev
        echo "✅ Linux dependencies installed"
    else
        echo "⚠️  Unsupported Linux distribution. Please install dependencies manually."
        echo "See: https://tauri.app/v1/guides/getting-started/prerequisites"
        exit 1
    fi
    
elif [[ "$OSTYPE" == "darwin"* ]]; then
    echo "📦 Installing macOS dependencies..."
    
    # Check if Xcode Command Line Tools are installed
    if ! xcode-select -p &> /dev/null; then
        echo "Installing Xcode Command Line Tools..."
        xcode-select --install
        echo "✅ Xcode Command Line Tools installation started"
        echo "Please complete the installation and run this script again"
        exit 0
    else
        echo "✅ Xcode Command Line Tools already installed"
    fi
    
elif [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" || "$OSTYPE" == "win32" ]]; then
    echo "✅ Windows detected - no additional dependencies needed"
    echo "Make sure you have Visual Studio Build Tools installed"
    
else
    echo "⚠️  Unknown operating system: $OSTYPE"
    echo "Please refer to Tauri documentation for prerequisites"
    exit 1
fi

echo ""
echo "✨ Setup complete! You can now build the Tauri GUI:"
echo ""
echo "  cd src/gui-tauri/src-tauri"
echo "  cargo build --release"
echo ""
echo "Or for development mode:"
echo "  cd src/gui-tauri"
echo "  cargo tauri dev"
echo ""
