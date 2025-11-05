# Changelog

## Unreleased
- TODO

## v0.2.0 (unreleased)
- feat(core): align encoder with TOON v1.4 canonical number formatting
- feat(decode): treat tokens with forbidden leading zeros as strings
- feat(encode): normalize non-finite floats (NaN/±Infinity) to `null`
- chore(spec): update conformance fixtures to v1.4.0
- docs: document type normalization policy and update spec references to v1.4

- Decode performance: zero-copy scanner, borrowed splitter, numeric fast paths
- Decode performance: zero-copy scanner, borrowed splitter, numeric fast paths
- Direct serde::Deserializer over the scanner (feature `de_direct`)
- Optional perf: `perf_memchr`, `perf_smallvec`, `perf_lexical`
- Benchmarks with Criterion and baseline compare instructions
- CLI `toon-cli` with JSON ↔ TOON
- Strict validation (indentation, tabular checks)
- Spec conformance tests (enable with `TOON_CONFORMANCE=1`)
- Optional `chrono` feature for DateTime
