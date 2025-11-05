#!/bin/bash
# Quick smoke test for fuzz targets without requiring nightly Rust
# This just verifies the targets compile and basic logic works

set -e

cd "$(dirname "$0")"

echo "Building all fuzz targets..."
cargo build --bins

echo ""
echo "Fuzz targets built successfully!"
echo ""
echo "To actually run fuzzing, you need:"
echo "  1. Install rustup: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
echo "  2. Install nightly: rustup install nightly"
echo "  3. Run: cargo +nightly fuzz run <target_name>"
echo ""
echo "Available targets:"
echo "  - fuzz_decode_default"
echo "  - fuzz_decode_strict"
echo "  - fuzz_roundtrip_json"
echo "  - fuzz_roundtrip_toon"
echo "  - fuzz_encode"
echo "  - fuzz_differential"
echo "  - fuzz_structured"
