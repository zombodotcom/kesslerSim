#!/bin/bash
set -e

# Clean previous build artifacts
echo "Cleaning previous build artifacts..."
rm -rf pkg dist target/wasm32-unknown-unknown

# Install wasm-pack if not already installed
if ! command -v wasm-pack &> /dev/null; then
    echo "Installing wasm-pack..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

# Build the WASM package
echo "Building WASM package..."
wasm-pack build --target web --out-dir pkg

# Create dist directory
mkdir -p dist

# Copy HTML and assets
cp index.html dist/
cp -r assets dist/ 2>/dev/null || true

# Copy the built WASM files
cp -r pkg dist/

echo "Build complete! Output in dist/"

