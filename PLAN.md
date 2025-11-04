# toon-rs: PLAN

Goal: Complete, spec-conformant Rust implementation of TOON with serde integration and a CLI, modeled after serde’s repository structure.

Status: Scaffolding complete (workspace, library, CLI, CI). Implementation pending.

## Milestones and tasks

- [ ] M1 Encoding core
- [x] encode/primitives.rs: delimiter-aware quoting + string escaping
- [x] encode/writer.rs: LineWriter for lines/indent/headers/rows
- [x] encode/encoders.rs: recursive encoder for serde_json::Value
  - [ ] encode/normalize.rs: policy for NaN/Infinity/dates, etc. (Rust mapping)
- [x] Unit tests: primitives, delimiter handling, tabular detection
  - Deliverable: encode JSON Value → TOON string for core shapes

- [ ] M2 Decoding core
- [x] decode/scanner.rs: tokenize lines to ParsedLine
- [x] decode/parser.rs: LineCursor parse structures, headers/rows (basic parser to Value)
- [x] decode/validation.rs: strict mode checks (indentation validation implemented)
- [x] Unit tests: scanner tokenization, strict indentation error case

- [ ] M3 Serde integration
  - [x] ser/: helpers `ser::to_string` / `ser::to_writer` (current impl converts to Value then encodes; streaming serializer TODO)
  - [x] de/: serde::Deserializer over parsed stream (via Value + IntoDeserializer)
  - [x] Public helpers: encode_to_string/encode_to_writer, decode_from_str/decode_from_reader, de::from_str
  - [x] Tests for user types (Serialize/Deserialize) and maps/structs

- [ ] M4 Quoting rules + tabular arrays parity
  - [ ] Finalize delimiter-aware quoting to match spec
- [x] Tabular detection: identical keysets, primitive-only values (basic encoder/decoder implemented with header '@<delim> <keys>')
  - [ ] More edge-case tests (control chars, list markers, numeric-looking strings)
  - [x] Basic tabular parsing/encoding added with strict checks for delimiter, header uniqueness, and row length

- [ ] M5 Spec conformance tests
  - [ ] Initialize `spec/` submodule and wire fixtures
  - [ ] decode: .toon → expected .json; encode: .json → expected .toon
  - [ ] CI phase that runs conformance if fixtures present

- [ ] M6 CLI parity
  - [ ] Flags: --decode, --delimiter, --strict, --pretty
  - [ ] Streaming stdin/stdout modes and file paths
  - [ ] CLI integration tests reflecting spec cases

- [ ] M7 Quality & docs
  - [ ] CI: fmt, clippy -D warnings, tests on stable + nightly
  - [ ] Crate docs + READMEs + examples
  - [ ] Property tests (lightweight set in CI; heavier behind feature)

- [ ] M8 Performance (post-conformance)
  - [ ] Criterion benches for encode/decode
  - [ ] Micro-optimizations after correctness

- [ ] M9 Optional: no_std path (alloc/std features)

## Implementation notes

- Delimiter-aware quoting: quote empty, leading/trailing spaces, contains delimiter/colon/quote/backslash/control chars, looks like bool/number/null, or starts with "- ".
- Tabular arrays: all elements objects, same key set (order irrelevant), all values primitives; header declares active delimiter.
- Strict mode: length mismatches, delimiter consistency, indentation, blank line rules.
- First pass operates on serde_json::Value; serde Serializer/Deserializer comes in M3.

## Commands

- Build all: `cargo build --workspace`
- Test all: `cargo test --workspace`
- Run CLI help: `cargo run -p toon-cli -- --help`
- Run CLI encode: `echo '{"a":1}' | cargo run -p toon-cli --`
- Run CLI decode: `cargo run -p toon-cli -- --decode path/to/data.toon`

## Progress log

- 2025-11-04: Initialized workspace, crates, CI; added PLAN.md; tests green with placeholder JSON transport.
- 2025-11-04: M1 partial complete — primitives, writer, encoder implemented; CLI encode now outputs TOON-like lines; tests updated and passing.
- 2025-11-04: Implemented basic decoder, strict indent validation, and tabular encoding/decoding with '@<delim>' header; tests passing end-to-end.
- 2025-11-04: Added serde Deserializer wrapper and typed decode helper; expanded quoting/unquoting tests; all tests passing.

## Next up

- Finish M1: refine encode/normalize.rs with NaN/Infinity/dates policy.
- Implement streaming serde::Serializer that buffers sequences to enable tabular detection but avoids full document materialization where possible.
  - Added custom ValueSerializer for NaN/Infinity normalization (strings).
  - Implemented streaming Serializer writing directly to LineWriter (buffers sequences only for tabular detection).
  - Chrono DateTime support behind feature `chrono` (added).
- Add delimiter consistency checks across all rows and header; provide informative strict errors (in progress: unquoted tokens validated; header precedence handled; trailing delimiter check done).
- Wire in spec fixtures and iterate until conformance passes (added guarded tests; enable with TOON_CONFORMANCE=1).
- Docs and release prep: README for toon-rs, crate docs with examples, CHANGELOG, and prepare Cargo.toml metadata for publishing.
