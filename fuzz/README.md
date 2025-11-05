# TOON Fuzzing Harnesses

This directory contains fuzzing targets for testing the robustness of the TOON parser and serializer.

## Prerequisites

Fuzzing requires **nightly Rust** with **rustup** (not Homebrew Rust) due to sanitizer support.

### If you have Homebrew Rust installed

You need to switch to rustup-managed Rust:

```bash
# Uninstall Homebrew Rust (optional but recommended)
brew uninstall rust

# Install rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Restart your shell or run:
source ~/.cargo/env

# Install nightly toolchain
rustup install nightly

# Install cargo-fuzz
cargo install cargo-fuzz
```

### Setting up fuzzing

```bash
# Option 1: Run with explicit nightly toolchain
cargo +nightly fuzz run <target_name>

# Option 2: Set nightly as default for fuzz directory
cd fuzz
rustup override set nightly
cargo fuzz run <target_name>
```

### Quick verification (without nightly)

To verify the fuzz targets compile without setting up nightly:

```bash
cd fuzz
./smoke_test.sh
```

This builds all targets but doesn't run them with sanitizers.

### Working around Homebrew Rust conflicts

If you have both Homebrew Rust and rustup installed, use the wrapper script:

```bash
cd fuzz
./fuzz.sh <subcommand> <target_name> [args...]
```

Examples:
```bash
# Run fuzzing
./fuzz.sh run fuzz_structured -- -max_total_time=60

# Minimize test case
./fuzz.sh tmin fuzz_structured artifacts/fuzz_structured/crash-<hash>

# Get coverage
./fuzz.sh coverage fuzz_structured
```

## Available Fuzz Targets

### Parser Targets

- **fuzz_decode_default** - Basic parser fuzzing with default options
  ```bash
  cargo +nightly fuzz run fuzz_decode_default
  ```

- **fuzz_decode_strict** - Parser fuzzing with strict mode enabled
  ```bash
  cargo +nightly fuzz run fuzz_decode_strict
  ```

### Roundtrip Targets

- **fuzz_roundtrip_json** - JSON → TOON → JSON roundtrip verification

  Takes arbitrary JSON as input, encodes to TOON, decodes back, and verifies the result matches the original. This catches encoding/decoding asymmetries.

  ```bash
  cargo +nightly fuzz run fuzz_roundtrip_json
  ```

- **fuzz_roundtrip_toon** - TOON → JSON → TOON roundtrip verification

  Takes arbitrary TOON strings, decodes to JSON, re-encodes to TOON, and verifies consistency.

  ```bash
  cargo +nightly fuzz run fuzz_roundtrip_toon
  ```

### Encoder Targets

- **fuzz_encode** - Encoder-only fuzzing with multiple delimiter options

  Tests the encoding path with different delimiter configurations to find encoder crashes or panics.

  ```bash
  cargo +nightly fuzz run fuzz_encode
  ```

### Differential Targets

- **fuzz_differential** - Differential fuzzing between default and strict modes

  Compares decoder behavior with different options. Ensures strict mode never accepts less than default mode, and when both succeed, they produce identical results.

  ```bash
  cargo +nightly fuzz run fuzz_differential
  ```

### Structured Input Targets

- **fuzz_structured** - Structure-aware fuzzing using `arbitrary`

  Uses the `arbitrary` crate to generate well-formed JSON structures, providing better coverage of valid input space compared to byte-level fuzzing.

  ```bash
  cargo +nightly fuzz run fuzz_structured
  ```

## Running Fuzzing

### Quick test (1 minute)

```bash
cargo +nightly fuzz run fuzz_roundtrip_json -- -max_total_time=60
```

### With specific number of iterations

```bash
cargo +nightly fuzz run fuzz_decode_default -- -runs=10000
```

### Parallel fuzzing

```bash
cargo +nightly fuzz run fuzz_roundtrip_json -- -workers=8
```

### Using a corpus

```bash
# Use test fixtures as initial corpus
cargo +nightly fuzz run fuzz_decode_default corpus/decode_default
```

## Interpreting Results

When a crash is found, cargo-fuzz will:
1. Save the crashing input to `fuzz/artifacts/<target>/`
2. Display the crash and backtrace
3. Provide a path to the input file

To reproduce a crash:

```bash
# With wrapper (Homebrew Rust users)
./fuzz.sh run fuzz_decode_default artifacts/fuzz_decode_default/crash-<hash>

# Or with nightly directly
cargo +nightly fuzz run fuzz_decode_default artifacts/fuzz_decode_default/crash-<hash>
```

To minimize a crash:

```bash
# With wrapper (Homebrew Rust users)
./fuzz.sh tmin fuzz_decode_default artifacts/fuzz_decode_default/crash-<hash>

# Or with nightly directly
cargo +nightly fuzz tmin fuzz_decode_default artifacts/fuzz_decode_default/crash-<hash>
```

## Continuous Fuzzing

For longer fuzzing runs, consider using tmux or screen:

```bash
# Run multiple targets in parallel
tmux new-session -d -s fuzz1 'cargo fuzz run fuzz_decode_default'
tmux new-session -d -s fuzz2 'cargo fuzz run fuzz_roundtrip_json'
tmux new-session -d -s fuzz3 'cargo fuzz run fuzz_structured'
```

## Coverage

To see code coverage from fuzzing:

```bash
cargo fuzz coverage fuzz_roundtrip_json
```

Then generate a coverage report using llvm-cov tools.

## Tips

- **fuzz_structured** is best for finding logical bugs in well-formed inputs
- **fuzz_decode_default** is best for finding parser crashes on malformed inputs
- **fuzz_roundtrip_json** is best for finding encode/decode mismatches
- **fuzz_differential** is best for finding strict mode validation bugs

Run multiple targets to maximize coverage!
