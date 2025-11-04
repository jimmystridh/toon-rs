<div align="center">

# 🎨 toon-rs

**Human-readable data serialization for the modern age**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg?style=flat-square)](LICENSE)

[Website](https://toonformat.dev) • [Examples](#examples) • [Contributing](#contributing)

</div>

---

## What is TOON?

**TOON** (Token-Oriented Object Notation) is a compact, human-readable serialization format designed for passing structured data to Large Language Models with significantly reduced token usage. Created by [Johann Schopplich](https://github.com/johannschopplich), TOON combines the best aspects of JSON, YAML, and CSV:

- **Token-efficient**: Typically 30-60% fewer tokens than JSON
- **Human-readable**: Clean indentation-based syntax without excessive punctuation
- **Machine-friendly**: Unambiguous parsing with strict mode validation
- **Tabular arrays**: Automatic CSV-like rendering for uniform object arrays
- **Git-friendly**: Line-oriented format that plays well with version control diffs

This repository provides a **complete Rust implementation** following the [official TOON v1.3 specification](https://github.com/toon-format/spec) with zero-copy parsing, streaming serialization via serde, and a full-featured CLI.

## Why TOON?

```rust
// JSON: verbose, hard to read large datasets
{"users": [{"id": 1, "name": "Alice", "role": "admin"}, {"id": 2, "name": "Bob", "role": "user"}]}

// TOON: clean, structured, tabular when appropriate
users:
  @| id name  role
  |  1  Alice admin
  |  2  Bob   user
```

TOON automatically detects when arrays contain uniform objects and renders them as inline tables, dramatically improving readability for datasets while maintaining full structural fidelity.

## Features

- Zero-copy scanner and parser (borrowed slices) for fast decode
- Direct serde::Deserializer over the scanner (feature `de_direct`)
- Smart tabular arrays — CSV-like rows under a header for uniform object arrays
- Strict mode — Optional validation for production-grade data integrity
- Streaming serialization — Memory-efficient encoding of large datasets
- Full serde integration — Serialize/deserialize any Rust type with `#[derive]`
- DateTime support — Native `chrono` integration (optional feature)
- Powerful CLI — Standalone tool for JSON ↔ TOON conversion
- Spec conformant — Comprehensive test suite against official fixtures
- Configurable delimiters — Use comma (default), tab, or pipe
- No-std support — Works in embedded environments (with `alloc`)

Feature flags:
- `de_direct` — enable direct Deserializer (bypasses intermediate JSON Value)
- `perf_memchr` — faster scanning/splitting via memchr
- `perf_smallvec` — reduce small allocations in hot paths
- `perf_lexical` — faster numeric parsing via lexical-core
- `chrono` — DateTime support

## Installation

### Library

Add to your `Cargo.toml`:

```toml
[dependencies]
toon = "0.1"

# Optional: DateTime serialization
toon = { version = "0.1", features = ["chrono"] }
```

### CLI

```bash
cargo install toon-cli

# Or run from source
cargo install --path crates/toon-cli
```

## Examples

### Basic Usage

```rust
use toon::Options;
use serde_json::json;

// Encode JSON value to TOON
let data = json!({
    "name": "toon-rs",
    "version": "0.1.0",
    "features": ["fast", "safe", "expressive"]
});

let opts = Options::default();
let toon_str = toon::encode_to_string(&data, &opts)?;
println!("{}", toon_str);

// Decode back to JSON
let value: serde_json::Value = toon::decode_from_str(&toon_str, &opts)?;
assert_eq!(value, data);
```

### Type-Safe Serialization

```rust
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Config {
    database: Database,
    servers: Vec<Server>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Database {
    host: String,
    port: u16,
    ssl: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Server {
    name: String,
    region: String,
    capacity: u32,
}

let config = Config {
    database: Database {
        host: "db.example.com".into(),
        port: 5432,
        ssl: true,
    },
    servers: vec![
        Server { name: "web-1".into(), region: "us-east".into(), capacity: 1000 },
        Server { name: "web-2".into(), region: "eu-west".into(), capacity: 1500 },
    ],
};

// Serialize with streaming (memory efficient)
let opts = Options::default();
let toon_output = toon::ser::to_string_streaming(&config, &opts)?;

// Deserialize back
let parsed: Config = toon::de::from_str(&toon_output, &opts)?;
assert_eq!(config, parsed);
```

Output:
```
database:
  host: db.example.com
  port: 5432
  ssl: true
servers:
  @| name  region  capacity
  |  web-1 us-east 1000
  |  web-2 eu-west 1500
```

### CLI Usage

```bash
# Convert JSON to TOON
toon-cli data.json > data.toon

# Convert TOON to JSON
toon-cli --decode data.toon > data.json

# Use pipes
echo '{"hello": "world"}' | toon-cli | toon-cli --decode

# Custom delimiter for tables
toon-cli --delimiter ',' data.json

# Strict mode validation
toon-cli --strict --decode data.toon
```

## Performance

Criterion benchmarks (decode) with optional perf features and direct deserializer produce large gains on typical datasets. To run:

```bash
# Save a baseline
cargo bench --bench decode_bench -- --sample-size 200 --measurement-time 10 --warm-up-time 5 --save-baseline before

# Compare after enabling direct + perf features
cargo bench --bench decode_bench \
  --features "de_direct perf_memchr perf_smallvec" \
  -- --sample-size 200 --measurement-time 10 --warm-up-time 5 --baseline before
```

Highlights (representative):
- small documents: ~1.5–2.0x faster decode
- 1k-row tabular arrays: ~2–3x faster decode

Notes:
- `perf_lexical` can further improve numeric-heavy workloads.
- Results vary by CPU and dataset.

## Development

```bash
# Clone the repository
git clone https://github.com/toon-format/toon
cd toon

# Initialize spec conformance fixtures
git submodule update --init --recursive

# Build everything
cargo build --workspace

# Run tests
cargo test --workspace

# Run conformance tests
TOON_CONFORMANCE=1 cargo test -p toon --tests

# Run CLI
cargo run -p toon-cli -- --help

# Format and lint
cargo fmt --all
cargo clippy --workspace -- -D warnings
```

### no_std / alloc builds

```bash
# Build (alloc-only)
cargo check -p toon --no-default-features --features "alloc,serde"

# Run alloc-only tests (JSON interop disabled)
cargo test -p toon --no-default-features --features "alloc,serde"
```

### Conformance: finer-grained runs

```bash
# Only the conformance test file
TOON_CONFORMANCE=1 cargo test -p toon --test spec_conformance

# Filter to one group
TOON_CONFORMANCE=1 cargo test -p toon --test spec_conformance decode_fixtures
TOON_CONFORMANCE=1 cargo test -p toon --test spec_conformance encode_fixtures
```

Note: Tests are skipped if TOON_CONFORMANCE is unset or fixtures are missing under spec/tests/fixtures. Decode conformance runs with strict validation; encode comparison normalizes newlines.

## Project Structure

```
toon-rs/
├── crates/
│   ├── toon/          # Core library
│   │   ├── encode/    # TOON encoder
│   │   ├── decode/    # TOON parser
│   │   ├── ser/       # serde::Serializer
│   │   └── de/        # serde::Deserializer
│   └── toon-cli/      # Command-line tool
├── spec/              # Official test fixtures (submodule)
└── README.md
```

## Specification

This implementation follows the [official TOON v1.3 specification](https://github.com/toon-format/spec/blob/main/SPEC.md) with extensive conformance testing. Key features:

- **Line-oriented**: Each logical token on its own line (or tabular row)
- **Indentation-based**: Two-space indentation indicates nesting
- **Minimal quoting**: Strings only quoted when necessary (delimiters, special chars)
- **Tabular arrays**: Objects with identical primitive keys rendered as CSV-like tables
- **Strict validation**: Optional mode catches malformed data

For the complete format specification and conformance tests, see:
- [TOON Specification v1.3](https://github.com/toon-format/spec/blob/main/SPEC.md)
- [Conformance Test Suite](https://github.com/toon-format/spec/tree/main/tests)
- [Official TypeScript Implementation](https://github.com/toon-format/toon)

## Contributing

We welcome contributions! Here's how to get started:

1. **Fork** the repository
2. **Create** a feature branch (`git checkout -b feature/amazing-feature`)
3. **Make** your changes with tests
4. **Ensure** tests pass (`cargo test --workspace`)
5. **Run** conformance tests (`TOON_CONFORMANCE=1 cargo test -p toon --tests`)
6. **Format** your code (`cargo fmt --all`)
7. **Lint** with clippy (`cargo clippy --workspace -- -D warnings`)
8. **Commit** with conventional format: `feat(encoder): add support for X`
9. **Push** to your fork
10. **Open** a Pull Request

### Areas for Contribution

- Performance optimizations and benchmarks
- Additional CLI features and utilities
- Documentation improvements and examples
- Property-based testing with proptest
- Fuzzing harness for parser robustness
- Language bindings (Python, JavaScript, etc.)

## Roadmap

- [x] Core encoding/decoding
- [x] Full serde integration
- [x] Streaming serialization
- [x] Tabular array detection
- [x] Strict mode validation
- [x] CLI tool
- [x] Spec conformance tests
- [x] Performance benchmarks
- [ ] Property-based tests
- [x] No-std support (alloc-only)
- [ ] WASM bindings
- [ ] Language server protocol (LSP) for editors

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- **TOON format** created by [Johann Schopplich](https://github.com/johannschopplich)
- **Official specification** maintained at [toon-format/spec](https://github.com/toon-format/spec)
- **Reference implementation** in TypeScript: [toon-format/toon](https://github.com/toon-format/toon)
- Repository structure inspired by [serde](https://serde.rs)'s excellent design
- Built with Rust's powerful serialization ecosystem

## Related Projects

- [Official TOON TypeScript/JavaScript](https://github.com/toon-format/toon) — Reference implementation with npm packages
- [TOON Specification](https://github.com/toon-format/spec) — Format specification v1.3 and conformance tests
- [Other TOON implementations](https://github.com/toon-format/toon#other-implementations) — Community ports to Python, Go, .NET, Swift, and more

---

<div align="center">

**[⬆ Back to Top](#-toon-rs)**

</div>
