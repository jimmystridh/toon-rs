#!/bin/bash
# Wrapper script to run cargo-fuzz with nightly toolchain
# This works around conflicts between Homebrew Rust and rustup

set -e

# Ensure rustup's cargo is used instead of Homebrew's
export PATH="$HOME/.rustup/toolchains/nightly-aarch64-apple-darwin/bin:$PATH"

# Run cargo fuzz with all arguments passed through
exec cargo fuzz "$@"
