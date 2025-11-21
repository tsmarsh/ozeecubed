#!/bin/bash

# Build script for OzeeCubed WASM

set -e

echo "🌊 Building OzeeCubed Web (WASM)..."

# Check if wasm-bindgen-cli is installed
if ! command -v wasm-bindgen &> /dev/null; then
    echo "❌ wasm-bindgen-cli is not installed"
    echo "📦 Install it with: cargo install wasm-bindgen-cli"
    exit 1
fi

# Check if wasm32-unknown-unknown target is installed
if ! rustup target list | grep -q "wasm32-unknown-unknown (installed)"; then
    echo "📦 Installing wasm32-unknown-unknown target..."
    rustup target add wasm32-unknown-unknown
fi

# Build the WASM binary (from project root)
echo "🔨 Compiling to WASM..."
cd ..
cargo build --target wasm32-unknown-unknown --release

# Generate JS bindings
echo "🔗 Generating JS bindings..."
cd wasm
wasm-bindgen ../target/wasm32-unknown-unknown/release/ozeecubed_wasm.wasm \
    --out-dir pkg \
    --target web \
    --no-typescript

echo "📋 Copying to docs/ozeecubed for GitHub Pages..."
mkdir -p ../docs/ozeecubed
cp -r pkg ../docs/ozeecubed/
cp index.html ../docs/ozeecubed/

echo "✅ Build complete!"
echo "📂 Output in ./pkg/"
echo "📂 GitHub Pages build in ../docs/ozeecubed/"
echo ""
echo "To test locally:"
echo "  cd wasm && python3 -m http.server 8888"
echo "  Open: http://localhost:8888"
echo ""
echo "GitHub Pages URL (after push):"
echo "  https://<username>.github.io/ozeecubed/ozeecubed/"
