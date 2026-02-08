# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust implementation of the TOON (Token-Oriented Object Notation) serialization format - a human-readable, token-efficient alternative to JSON designed for LLM data exchange. The codebase consists of:

- **crates/toon**: Core library with encoding/decoding and serde integration
- **crates/toon-cli**: Command-line tool for JSON ↔ TOON conversion

## Build & Test Commands

```bash
# Build everything
cargo build --workspace

# Run all tests (except conformance)
cargo test --workspace

# Run conformance tests (requires spec submodule)
git submodule update --init --recursive
TOON_CONFORMANCE=1 cargo test -p toon-rs --tests

# Run specific conformance test groups
TOON_CONFORMANCE=1 cargo test -p toon-rs --test spec_conformance decode_fixtures
TOON_CONFORMANCE=1 cargo test -p toon-rs --test spec_conformance encode_fixtures

# Run just one test file
cargo test -p toon-rs --test roundtrip

# Format and lint
cargo fmt --all
cargo clippy --workspace -- -D warnings

# Run CLI locally
cargo run -p toon-cli -- --help
cargo run -p toon-cli -- data.json
cargo run -p toon-cli -- --decode data.toon

# Fuzz (nightly; run from `fuzz/`)
cd fuzz
./fuzz.sh run fuzz_structured -- -max_total_time=600
```

## Architecture

### Core Pipeline

**Encoding (Rust → TOON string)**
1. `ser/` - Serde Serializer implementation captures Rust types
2. `ser/value_builder.rs` - Converts to serde_json::Value
3. `ser/stream.rs` - Streaming serialization path (preferred)
4. `encode/normalize.rs` - Normalizes JSON values (NaN/Infinity handling)
5. `encode/encoders.rs` - Main encoding logic with tabular array detection
6. `encode/primitives.rs` - String escaping, quoting, numeric formatting
7. `encode/writer.rs` - LineWriter accumulates output lines

**Decoding (TOON string → Rust)**
1. `decode/scanner.rs` - Tokenizes input into LogicalLine structs
2. `decode/validation.rs` - Strict mode validation (indentation, empty tables, etc.)
3. `decode/parser.rs` - Recursive descent parser producing serde_json::Value
4. `de/` - Serde Deserializer converts Value → target Rust type

### Key Concepts

**Tabular Arrays**: Arrays of uniform objects with identical primitive keys are automatically rendered as CSV-like tables with `key[N]{fields}:` header and delimiter-separated rows. Detection logic is in `encode/encoders.rs`.

**Strict Mode**: When `Options::strict` is true (default), validation catches malformed TOON (inconsistent indentation, unquoted cells, trailing delimiters, etc.). Essential for conformance.

**Delimiters**: Tables support `,` (default), `|`, or tab delimiters via `Options::delimiter`. Affects quoting logic in `encode/primitives.rs`.

**Zero-copy parsing**: The parser uses string slices (`&str`) from the original input where possible to minimize allocations.

## Feature Flags

- `std` (default): Standard library support
- `serde` (default): Full serde integration
- `chrono`: DateTime serialization support
- `alloc`: For no-std environments (not yet implemented)

## Testing Strategy

- `tests/*_*.rs`: Unit and integration tests organized by feature
- `tests/spec_conformance.rs`: Official TOON v3.0 spec conformance (decode runs with strict validation)
- `tests/roundtrip.rs`: Encode → decode identity tests
- Test files use descriptive names like `strict_unquoted_cells.rs`, `tabular_detection.rs`

## Common Patterns

**Adding a new primitive type**: Update `encode/primitives.rs` for encoding and `decode/parser.rs` for parsing.

**Modifying tabular detection**: See `is_tabularizable` and `encode_tabular_array` in `encode/encoders.rs`.

**Adding strict validation**: Add validation logic to `decode/validation.rs` and gate with `options.strict`.

## Specification Reference

Follows [TOON v3.0 spec](https://github.com/toon-format/spec/blob/main/SPEC.md). Key rules:
- Two-space indentation for nesting
- Keys followed by `:` on separate line from values (except scalars)
- Strings quoted only when needed (contain delimiters, special chars, numeric-like)
- Tabular arrays must have identical primitive keys across all objects
- Strict mode enforces spec compliance (default: true per spec)
- Key folding (encoding) and path expansion (decoding) supported in safe mode
- List-item objects with tabular first field use YAML-style encoding (§10)
