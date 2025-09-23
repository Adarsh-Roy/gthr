#!/bin/bash

# Manual build script for release assets
set -e

VERSION="v0.1.0"
echo "Building gthr $VERSION for multiple targets..."

# Create build directory
mkdir -p build-$VERSION
cd build-$VERSION

# Build for macOS ARM64 (if on macOS)
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "Building for macOS ARM64..."
    cargo build --release --target aarch64-apple-darwin
    cp ../target/aarch64-apple-darwin/release/gthr gthr-aarch64-apple-darwin
    tar -czf gthr-aarch64-apple-darwin.tar.gz gthr-aarch64-apple-darwin

    echo "Building for macOS x86_64..."
    cargo build --release --target x86_64-apple-darwin
    cp ../target/x86_64-apple-darwin/release/gthr gthr-x86_64-apple-darwin
    tar -czf gthr-x86_64-apple-darwin.tar.gz gthr-x86_64-apple-darwin

    echo "✅ Built macOS binaries:"
    ls -la *.tar.gz

    echo ""
    echo "📁 Upload these files to the GitHub release manually:"
    echo "   - gthr-aarch64-apple-darwin.tar.gz"
    echo "   - gthr-x86_64-apple-darwin.tar.gz"
    echo ""
    echo "🌐 Go to: https://github.com/Adarsh-Roy/gthr/releases/tag/$VERSION"
    echo "📎 Drag and drop the .tar.gz files to attach them"
fi
