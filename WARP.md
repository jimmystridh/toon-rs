# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

Common commands
- Build all crates:
  - cargo build --workspace
- Test everything:
  - cargo test --workspace
- Run a single test (examples):
  - Library unit by name: cargo test -p toon strict_indent_increase_error
  - Specific integration file: cargo test -p toon --test spec_conformance
  - Specific tests within that file: cargo test -p toon --test spec_conformance decode_fixtures | encode_fixtures
- Lint/format:
  - cargo fmt --all
  - cargo clippy --workspace -D warnings
- Optional features:
  - Enable chrono for DateTime: cargo test -p toon --features chrono
- no_std/alloc builds:
  - Build (alloc-only): cargo check -p toon --no-default-features --features "alloc,serde"
  - Run alloc-only tests: cargo test -p toon --no-default-features --features "alloc,serde"
- CLI usage:
  - Help: cargo run -p toon-cli -- --help
  - Encode JSON → TOON: echo '{"a":1}' | cargo run -p toon-cli --
  - Decode TOON → JSON: cargo run -p toon-cli -- --decode path/to/data.toon
  - Pretty decode: cargo run -p toon-cli -- --decode --pretty path/to/data.toon
  - Delimiter override: cargo run -p toon-cli -- --delimiter tab …

Conformance tests
- Fixtures live under spec/tests/fixtures (optional; tests skip if missing).
- Initialize fixtures (if using submodules):
  - git submodule update --init --recursive
- Run all conformance tests (decode + encode fixtures):
  - TOON_CONFORMANCE=1 cargo test -p toon --tests
- Run only the conformance file:
  - TOON_CONFORMANCE=1 cargo test -p toon --test spec_conformance
- Filter to one group:
  - TOON_CONFORMANCE=1 cargo test -p toon --test spec_conformance decode_fixtures
  - TOON_CONFORMANCE=1 cargo test -p toon --test spec_conformance encode_fixtures
- Behavior:
  - Decode runs with strict validation (indentation/tabular rules). Encode comparison normalizes newlines. Tests are skipped if TOON_CONFORMANCE is unset or fixtures are absent.

Architecture and structure (big picture)
- Workspace
  - crates/toon: core library (encode/decode pipelines, Options, Error, serde integration, streaming serializer)
  - crates/toon-cli: CLI for JSON ↔ TOON
- Encoding (crates/toon/src/encode)
  - writer::LineWriter emits lines with indentation helpers (line, line_kv, line_list_item, line_key_only).
  - primitives.rs applies delimiter-aware quoting and escaping (quotes/backslashes/control chars, numeric/boolean/null lookalikes, leading/trailing spaces, list marker "- ", ':' or active delimiter).
  - encoders.rs converts serde_json::Value to TOON; detects “tabular arrays” (arrays of objects with identical key sets and only primitive values) and emits a header "@<delim> <keys>" followed by list rows; otherwise falls back to regular lists/objects.
  - normalize.rs is a placeholder for value normalization (e.g., NaN/Infinity, dates) before emission.
- Streaming serializer (crates/toon/src/ser/stream.rs)
  - Implements serde::Serializer that writes directly via LineWriter. Sequences are buffered as serde_json::Value to enable tabular detection; nested complex values reuse encode::encoders for consistency. Floats: finite as numbers; NaN/±Infinity as quoted strings.
- Decoding (crates/toon/src/decode)
  - scanner.rs tokenizes lines (Blank, Scalar, ListItem, KeyOnly, KeyValue) with quote-aware colon detection and treats '@…' header lines as scalar.
  - parser.rs builds a serde_json::Value, handling objects, lists, and tabular blocks: parses header '@<delim> …', splits cells quote-aware, assembles rows as arrays of objects, and enforces strict table rules when enabled.
  - validation.rs enforces indentation rules in strict mode: increases must be +2; decreases must be multiples of 2.
- Strict mode and options
  - Options { delimiter: Comma|Tab|Pipe, strict: bool } influence both encoding (active delimiter) and decoding (validation on). The CLI maps flags to Options.

Notes
- rust-toolchain is stable; workspace edition is 2024.
- Conformance fixtures and TOON_CONFORMANCE are optional to keep normal test runs fast.