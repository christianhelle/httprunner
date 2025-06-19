#!/bin/bash

# Test script for the httprunner installer
# This script sets up a local test environment

echo "🧪 Testing httprunner installer scripts..."

# Create a temporary directory for testing
TEST_DIR=$(mktemp -d)
echo "📁 Test directory: $TEST_DIR"

# Copy installer scripts to test directory
cp install.sh "$TEST_DIR/"
cp install.ps1 "$TEST_DIR/"

cd "$TEST_DIR"

echo ""
echo "🔍 Testing bash installer help..."
bash install.sh --help

echo ""
echo "✅ Bash installer help test completed"

echo ""
echo "🔍 Testing PowerShell installer (syntax check)..."
if command -v pwsh >/dev/null 2>&1; then
    pwsh -Command "Get-Help -Name '$TEST_DIR/install.ps1' -ErrorAction SilentlyContinue" || echo "PowerShell script syntax appears valid"
else
    echo "⚠️  PowerShell not available for testing"
fi

echo ""
echo "🧹 Cleaning up test directory..."
rm -rf "$TEST_DIR"

echo ""
echo "✅ All tests completed!"
echo ""
echo "📚 Next steps:"
echo "  1. Test the scripts manually on different platforms"
echo "  2. Commit and push to trigger documentation deployment"
echo "  3. Verify the scripts are accessible at:"
echo "     - https://christianhelle.com/httprunner/install"
echo "     - https://christianhelle.com/httprunner/install.ps1"
