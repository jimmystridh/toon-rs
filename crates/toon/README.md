# toon

Rust library for TOON (Token-Oriented Object Notation). Provides:
- Encoder/decoder for TOON ↔ JSON values
- serde integration (typed de/ser)
- Streaming serializer that writes directly to TOON

## Features
- `serde` (default): serde integration
- `de_direct`: direct serde::Deserializer over the scanner (no intermediate JSON Value)
- `perf_memchr`, `perf_smallvec`, `perf_lexical`: optional micro-optimizations
- `chrono`: serialize `chrono::DateTime` as RFC3339 strings

## Quickstart

Enable performance features for fastest decode (optional):

```bash
# Library
cargo add toon --features "de_direct perf_memchr perf_smallvec"

# Or enable per build
RUSTFLAGS='' cargo test -p toon --features "de_direct perf_memchr perf_smallvec"
```

```rust
use serde_json::json;
let opts = toon::Options::default();
let s = toon::encode_to_string(&json!({"a": 1, "b": [true, "x"]}), &opts).unwrap();
let v: serde_json::Value = toon::decode_from_str(&s, &opts).unwrap();
assert_eq!(v, json!({"a":1, "b":[true, "x"]}));
```

Typed APIs:

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct User { id: u32, name: String }

let opts = toon::Options::default();
let user = User { id: 1, name: "Ada".into() };
let s = toon::ser::to_string_streaming(&user, &opts).unwrap();
let back: User = toon::de::from_str(&s, &opts).unwrap();
assert_eq!(user, back);
```

## Conformance
- Initialize fixtures: `git submodule update --init --recursive`
- Run: `TOON_CONFORMANCE=1 cargo test -p toon --tests`

## License
MIT
