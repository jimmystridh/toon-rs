#!/bin/bash
set -e

echo "Building TOON WebAssembly module..."

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "Error: wasm-pack is not installed"
    echo "Install it with: cargo install wasm-pack"
    exit 1
fi

# Build the WASM module
cd "$(dirname "$0")/../.."
wasm-pack build crates/toon-wasm --target web --out-dir ../../examples/web/pkg

echo "Build complete! The WASM module is in examples/web/pkg/"
echo ""
echo "To run the example:"
echo "  cd examples/web"
echo "  python3 -m http.server 8000"
echo ""
echo "Then open http://localhost:8000 in your browser"
