# Testing Guide for toon-wasm

This document provides comprehensive information about testing the WASM bindings.

## Unit Tests

Run unit tests with:

```bash
cargo test -p toon-wasm
```

### Test Coverage

Current unit tests cover:
- ✅ Basic conversion (JSON ↔ TOON)
- ✅ Input size validation (DoS protection)
- ✅ Empty strings and objects
- ✅ Null values
- ✅ Unicode handling (emoji, non-Latin scripts)
- ✅ Special characters (quotes, backslashes, control chars)
- ✅ Number edge cases (zero, negative, float, large integers)
- ✅ Boolean values
- ✅ Deeply nested structures
- ✅ Mixed-type arrays
- ✅ Delimiter options (pipe vs comma)
- ✅ Strict mode
- ✅ Pretty vs compact JSON formatting
- ✅ Large but valid inputs
- ✅ Whitespace handling
- ✅ Invalid input handling

## Browser Integration Tests

For browser-side testing, use `wasm-pack test`:

```bash
# Run in headless browser
wasm-pack test --headless --firefox crates/toon-wasm
wasm-pack test --headless --chrome crates/toon-wasm

# Run in browser with UI
wasm-pack test --firefox crates/toon-wasm
wasm-pack test --chrome crates/toon-wasm
```

### Writing Browser Tests

Create browser-specific tests with `#[wasm_bindgen_test]`:

```rust
#[cfg(test)]
mod browser_tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_in_browser() {
        let json = r#"{"test": "value"}"#;
        let result = json_to_toon(json, false, false);
        assert!(result.is_ok());
    }
}
```

### Setup for Browser Tests

1. Install `wasm-bindgen-test`:
   ```bash
   cargo install wasm-bindgen-cli
   ```

2. Add to `Cargo.toml`:
   ```toml
   [dev-dependencies]
   wasm-bindgen-test = "0.3"
   ```

3. Install browser drivers:
   ```bash
   # Firefox
   brew install geckodriver  # macOS

   # Chrome
   brew install chromedriver  # macOS
   ```

## Performance Benchmarks

### Manual Performance Testing

Test conversion performance in the browser console:

```javascript
// Test conversion speed without extra JSON.parse/stringify work
const largeValue = {
  users: Array.from({ length: 1000 }, (_, i) => ({
    id: i,
    name: `User ${i}`,
    email: `user${i}@example.com`,
    active: i % 2 === 0
  }))
};

console.time('value_to_toon');
const toon = value_to_toon(largeValue, true, false);
console.timeEnd('value_to_toon');

console.time('toon_to_value');
const objBack = toon_to_value(toon, false);
console.timeEnd('toon_to_value');

// JSON helpers remain available if you really need strings:
const toonFromJson = json_to_toon(JSON.stringify(largeValue), true, false);
const jsonBack = toon_to_json(toonFromJson, false, false);
```

### Allocator profiles

- Default builds prioritize runtime throughput and rely on the platform
  allocator.
- Enable the `size_opt` feature (for example,
  `wasm-pack build -- --features size_opt`) to restore `wee_alloc` when binary
  size matters more than raw speed.

`value_to_toon` walks the JavaScript object directly, so unsupported values
(functions, symbols, DOM nodes, etc.) will return an error before encoding. The
10 MB guard is enforced on the emitted TOON payload, which closely tracks the
input size for typical documents. `toon_to_value` feeds the TOON parser directly
into JS objects/arrays as it reads, removing the previous intermediate `Value`
allocation step.

### Criterion Benchmarks

For Rust-side benchmarks, create `benches/conversion.rs`:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use toon_wasm::*;

fn bench_json_to_toon(c: &mut Criterion) {
    let json = r#"{"name": "test", "value": 123, "nested": {"a": 1}}"#;

    c.bench_function("json_to_toon", |b| {
        b.iter(|| json_to_toon(black_box(json), false, false))
    });
}

criterion_group!(benches, bench_json_to_toon);
criterion_main!(benches);
```

Add to `Cargo.toml`:
```toml
[[bench]]
name = "conversion"
harness = false

[dev-dependencies]
criterion = "0.5"
```

Run with:
```bash
cargo bench -p toon-wasm
```

## Memory Testing

Test memory usage with browser DevTools:

1. Open DevTools → Memory tab
2. Take heap snapshot before conversion
3. Run conversion multiple times
4. Force garbage collection
5. Take another snapshot
6. Compare memory usage

### Expected Memory Profile

- Small inputs (<10KB): <100KB temporary allocation
- Medium inputs (~1MB): ~3-5x input size during processing
- Large inputs (near 10MB limit): ~20-30MB peak usage
- All memory should be freed after conversion completes

## Error Handling Tests

### Invalid UTF-8 Testing

WASM strings are always valid UTF-8 (enforced by JavaScript), so invalid UTF-8 from the browser is not possible. However, you can test encoding issues:

```javascript
// Test with problematic Unicode sequences
const weirdChars = '{"test": "\uD800\uDC00"}';  // Surrogate pair
const result = json_to_toon(weirdChars, false, false);
```

### Panic Testing

Test that panics are caught and logged properly:

```javascript
// This should trigger panic hook if something goes wrong internally
try {
  // Create pathological input
  const malicious = /* ... */;
  json_to_toon(malicious, false, false);
} catch (e) {
  console.log('Caught error:', e);
  // Check browser console for panic details
}
```

## CI/CD Integration

Add to `.github/workflows/test.yml`:

```yaml
- name: Test WASM
  run: |
    cargo test -p toon-wasm

- name: Browser Tests
  run: |
    wasm-pack test --headless --firefox crates/toon-wasm
    wasm-pack test --headless --chrome crates/toon-wasm
```

## Coverage Reports

Generate test coverage with `tarpaulin`:

```bash
cargo install cargo-tarpaulin
cargo tarpaulin -p toon-wasm --out Html
```

## Test Checklist

Before releasing:
- [ ] All unit tests pass
- [ ] Browser tests pass in Firefox and Chrome
- [ ] Memory leaks checked with DevTools
- [ ] Performance benchmarks show acceptable speeds
- [ ] Error messages are clear and helpful
- [ ] Large input handling tested (near 10MB limit)
- [ ] Unicode edge cases tested
- [ ] Panic hook provides useful debugging info

## Known Limitations

1. **Invalid UTF-8**: Cannot test from browser (JS always provides valid UTF-8)
2. **Memory exhaustion**: Hard to test without crashing the browser tab
3. **Concurrent access**: WASM is single-threaded, no race conditions possible
4. **Platform differences**: WASM is platform-agnostic, behavior is consistent
