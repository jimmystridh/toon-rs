# Changelog

## Unreleased
- Decode performance: zero-copy scanner, borrowed splitter, numeric fast paths
- Direct serde::Deserializer over the scanner (feature `de_direct`)
- Optional perf: `perf_memchr`, `perf_smallvec`, `perf_lexical`
- Benchmarks with Criterion and baseline compare instructions
- CLI `toon-cli` with JSON ↔ TOON
- Strict validation (indentation, tabular checks)
- Spec conformance tests (enable with `TOON_CONFORMANCE=1`)
- Optional `chrono` feature for DateTime
