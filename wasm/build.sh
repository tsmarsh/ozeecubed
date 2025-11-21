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

# Build the WASM binary
echo "🔨 Compiling to WASM..."
cargo build --target wasm32-unknown-unknown --release

# Generate JS bindings
echo "🔗 Generating JS bindings..."
wasm-bindgen target/wasm32-unknown-unknown/release/ozeecubed_wasm.wasm \
    --out-dir pkg \
    --target web \
    --no-typescript

echo "✅ Build complete!"
echo "📂 Output in ./pkg/"
echo ""
echo "To test locally:"
echo "  1. Install a local server: npm install -g http-server"
echo "  2. Run from wasm/ directory: http-server . -p 8080"
echo "  3. Open: http://localhost:8080"
