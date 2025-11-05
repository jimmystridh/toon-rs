# Fuzzing Findings

This document tracks bugs found by fuzzing.

## Bug #1: Empty Collections Roundtrip Failure

**Status**: FIXED (partial - empty arrays only)
**Fuzzer**: `fuzz_structured`
**Date**: 2025-11-05
**Fixed**: 2025-11-05
**Severity**: Medium - High

### Description

Encoding empty JSON collections (`[]` and `{}`) to TOON produces an empty string, but decoding an empty string produces `null` instead of the original collection. This breaks the fundamental roundtrip property.

### Reproduction

```bash
# Empty array
echo '[]' | cargo run -p toon-cli --quiet
# Output: (empty string)

# Empty object
echo '{}' | cargo run -p toon-cli --quiet
# Output: (empty string)

# Decoding empty string
echo -n '' | cargo run -p toon-cli --quiet -- --decode
# Output: null
```

**Roundtrip failure:**
```bash
# [] → "" → null (should be [])
# {} → "" → null (should be {})
```

### Expected Behavior

Either:
1. Empty collections should encode to a distinguishable TOON representation:
   - `[]` → some representation that decodes back to `[]`
   - `{}` → some representation that decodes back to `{}`
2. Or the decoder should interpret empty input differently based on context

### Minimized Test Case

The fuzzer minimized the crashing input to a single byte `0x6a` ('j'), which generates an empty array through the structured fuzzer's arbitrary generation.

**Artifacts:**
```
fuzz/artifacts/fuzz_structured/crash-23b54dec02fdd05a55e66e047d0fc93c8d58afd4 (original)
fuzz/artifacts/fuzz_structured/minimized-from-23b54dec02fdd05a55e66e047d0fc93c8d58afd4 (minimized)
```

**Minimization command:**
```bash
cd fuzz
./fuzz.sh tmin fuzz_structured artifacts/fuzz_structured/crash-23b54dec02fdd05a55e66e047d0fc93c8d58afd4
```

### Root Cause

Empty collections have no elements to encode, resulting in zero lines of TOON output. The decoder interprets empty input as `null`, creating an asymmetry in the encode/decode cycle.

### Impact

- Breaks the encode→decode identity property
- Data loss when roundtripping empty collections
- May cause issues in applications relying on type preservation

### Fix

**Empty Arrays**: Fixed by implementing the `[0]:` syntax per TOON spec.
- Encoder: Root-level empty arrays now encode to `[0]:`
- Decoder: Added special case to parse `[0]:` as empty array
- Files changed:
  - `crates/toon/src/ser/stream.rs` (lines 336-340)
  - `crates/toon/src/decode/parser.rs` (lines 308-318)

**Empty Objects**: Limitation remains
- Root-level empty objects `{}` still encode to empty string
- Empty string still decodes to `null`
- This is a design limitation of TOON: root-level documents are implicitly objects (key-value pairs), and an empty object is indistinguishable from an empty document
- Workaround: Wrap empty objects in a parent object or use a marker field

**Verification**:
```bash
echo '[]' | cargo run -p toon-cli
# Output: [0]:

echo '[0]:' | cargo run -p toon-cli -- --decode
# Output: []
```

## Bug #2: Float Precision Loss (Whole Numbers)

**Status**: FIXED
**Fuzzer**: `fuzz_structured`
**Date**: 2025-11-05
**Fixed**: 2025-11-05
**Severity**: Medium

### Description

Floats with zero decimal parts (like `0.0`, `1.0`, `2.0`) were being encoded without the decimal point, losing type information and causing roundtrip failures.

### Reproduction (Before Fix)

```bash
echo '{"x":0.0}' | cargo run -p toon-cli
# Output: x: 0  (should be x: 0.0)

echo '0.0' | cargo run -p toon-cli | cargo run -p toon-cli -- --decode
# Roundtrip fails: 0.0 → 0 → 0
```

### Root Cause

Rust's `f64::to_string()` method outputs `"0"` for `0.0` and `"1"` for `1.0`, stripping the decimal point for whole number floats. This makes floats indistinguishable from integers during encoding.

### Fix

Added `format_f64()` helper function that ensures floats always have a decimal point:
- `0.0` → `"0.0"`
- `1.0` → `"1.0"`
- `1.5` → `"1.5"` (unchanged)
- `1e10` → `"10000000000.0"` (scientific notation expanded)

**Files changed:**
- `crates/toon/src/encode/primitives.rs` - Added `format_f64()` function
- `crates/toon/src/value.rs` - Updated `Number::to_string()` for F64 variant
- `crates/toon/src/ser/stream.rs` - Updated all `f64.to_string()` calls to use `format_f64()`

**Tests added:**
- `crates/toon/tests/roundtrip.rs` - Added 3 tests for float precision

### Verification

```bash
echo '{"x":0.0, "y":1.0, "z":1.5}' | cargo run -p toon-cli
# Output: x: 0.0
#         y: 1.0
#         z: 1.5

# Roundtrip test
echo '0.0' | cargo run -p toon-cli | cargo run -p toon-cli -- --decode
# Output: 0.0 ✓
```

## Bug #3: Nested Empty Arrays

**Status**: FIXED
**Fuzzer**: `fuzz_structured`
**Date**: 2025-11-05
**Fixed**: 2025-11-05
**Severity**: Medium

### Description

Empty arrays nested inside other arrays were not roundtripping correctly. The encoder would write nothing for nested empty arrays, which the decoder would interpret as `null`.

### Reproduction

```bash
# Input: [[], null, null]
# TOON output:
-
- null
- null

# Decoded result: [null, null, null]  ❌ Should be: [[], null, null]
```

### Root Cause

The `[0]:` empty array syntax was only being written for root-level arrays (`indent == 0`). For nested empty arrays, the encoder wrote nothing, and the decoder returned `null` for missing content.

### Fix

Removed the `indent == 0` check in both encoder locations:
- `crates/toon/src/encode/encoders.rs` line 55: Changed condition from `if items.is_empty() && indent == 0` to `if items.is_empty()`
- `crates/toon/src/ser/stream.rs` line 337: Same change

Also updated the parser to recognize `[0]:` at any nesting level in `parse_node()`.

**Files changed:**
- `crates/toon/src/encode/encoders.rs` (line 55)
- `crates/toon/src/ser/stream.rs` (line 337)
- `crates/toon/src/decode/parser.rs` (added `[0]:` check to `parse_node`)

### Verification

```bash
echo '[[], null]' | cargo run -p toon-cli
# Output:
# -
#   [0]:
# - null

echo '[[], null]' | cargo run -p toon-cli | cargo run -p toon-cli -- --decode
# Output: [[], null] ✓
```

## Bug #4: Scientific Notation with Signs

**Status**: FIXED
**Fuzzer**: `fuzz_structured`
**Date**: 2025-11-05
**Fixed**: 2025-11-05
**Severity**: Medium

### Description

Numbers in scientific notation with signed exponents (e.g., `6e-323`, `1.5e+10`) were being decoded as strings instead of numbers.

### Reproduction

```bash
echo '[6e-323, null]' | cargo run -p toon-cli | cargo run -p toon-cli -- --decode
# Result: ["6e-323", null]  ❌ Should be: [6e-323, null]
```

### Root Cause

The `classify_numeric_hint` function in the parser checked for `e`/`E` exponent markers but didn't handle the optional `+`/`-` sign that can follow the exponent marker. When it encountered the `-` in `6e-323`, it returned `None` because `-` wasn't in the allowed character set for the simple loop.

### Fix

Updated `classify_numeric_hint` to use a state machine that tracks when we're in the exponent part and allows one optional sign character immediately after `e`/`E`.

**Files changed:**
- `crates/toon/src/decode/parser.rs` (lines 600-656): Enhanced `classify_numeric_hint` with exponent sign handling

### Verification

```bash
echo '[6e-323, 1.5e+10, 2.5e3]' | cargo run -p toon-cli | cargo run -p toon-cli -- --decode
# Output: [6e-323, 1.5e10, 2500.0] ✓
```

## Bug #5: Strict Mode Validation for + Prefix

**Status**: FIXED
**Found During**: Test suite execution after Bug #3 fix
**Date**: 2025-11-05
**Fixed**: 2025-11-05
**Severity**: Low

### Description

In strict mode, tabular cell values starting with `+` (like `+1`, `+123`) were not being flagged as requiring quotes, even though they are numeric-like and could be ambiguous.

### Reproduction

Test case `strict_unquoted_numeric_like_cell_errors` was failing after the empty object fix:

```toon
rows:
  @, s
  - +1
```

Expected: Error in strict mode
Actual: Successfully parsed as `{"rows": [{"s": 1}]}`

### Root Cause

The `cell_token_requires_quotes` function checked for leading `-` signs but not leading `+` signs.

### Fix

Added a check for `+` prefix in `cell_token_requires_quotes`:

```rust
// If it starts with '+', it's numeric-like and requires quotes for clarity
if t.starts_with('+') {
    return true;
}
```

**Files changed:**
- `crates/toon/src/decode/parser.rs` (lines 483-486)

### Verification

All tests pass, including `strict_unquoted_numeric_like_cell_errors`.

## Bug #6: Nested Empty Objects

**Status**: FIXED
**Fuzzer**: `fuzz_structured`
**Date**: 2025-11-05
**Fixed**: 2025-11-05
**Severity**: Medium - High

### Description

Empty objects nested inside arrays were not roundtripping correctly. Similar to Bug #3 but for objects instead of arrays.

### Reproduction

```bash
# Input: [{}, null]
# TOON output:
-
- null

# Decoded result: [null, null]  ❌ Should be: [{}, null]
```

### Root Cause

Unlike arrays which had the `[0]:` syntax (even if only at root level initially), empty objects had no representation at all - they just wrote nothing since there were no key-value pairs to iterate over.

### Fix

Implemented a symmetrical `{0}:` syntax for empty objects, mirroring the `[0]:` syntax for empty arrays:

**Encoder changes:**
- `crates/toon/src/encode/encoders.rs`: Added check for empty objects to write `{0}:`
- `crates/toon/src/ser/stream.rs`: Added `entry_count` field to `MapSer` struct, increment on serialize_value, check in end() to write `{0}:` if count is 0

**Decoder changes:**
- `crates/toon/src/decode/parser.rs`: Added `{0}:` recognition in both `parse_node()` and `parse_document()`

**Files changed:**
- `crates/toon/src/encode/encoders.rs` (lines 78-80)
- `crates/toon/src/ser/stream.rs` (lines 532, 260, 284, 557, 561-566)
- `crates/toon/src/decode/parser.rs` (lines 297-300, 330-333)

### Verification

```bash
echo '[{}, null]' | cargo run -p toon-cli
# Output:
# -
#   {0}:
# - null

echo '[{}, null]' | cargo run -p toon-cli | cargo run -p toon-cli -- --decode
# Output: [{}, null] ✓
```

## Bug #7: Lone Hyphen String

**Status**: FIXED
**Fuzzer**: `fuzz_structured`
**Date**: 2025-11-05
**Fixed**: 2025-11-05
**Severity**: Medium

### Description

A string containing only a hyphen `"-"` was being encoded without quotes as `-`, which is the TOON list item marker. This caused the decoder to interpret it as an array with one null element instead of a string.

### Reproduction

```bash
# Input: "-"
# TOON output: -
# Decoded result: [null]  ❌ Should be: "-"
```

### Root Cause

The `needs_quotes` function checked for strings starting with `"- "` (hyphen followed by space) but didn't handle the exact string `"-"` which is a reserved token in TOON syntax.

### Fix

Added explicit check for lone hyphen in `needs_quotes`:

```rust
// A lone hyphen is a list item marker and must be quoted
if s == "-" {
    return true;
}
```

**Files changed:**
- `crates/toon/src/encode/primitives.rs` (lines 37-40)

### Verification

```bash
echo '"-"' | cargo run -p toon-cli
# Output: "-"

echo '"-"' | cargo run -p toon-cli | cargo run -p toon-cli -- --decode
# Output: "-" ✓
```

## Bug #8: Empty String Keys in Tabular Arrays

**Status**: FIXED
**Fuzzer**: `fuzz_structured`
**Date**: 2025-11-05
**Fixed**: 2025-11-05
**Severity**: Low

### Description

When a tabular array has an empty string as a key, the encoder produces output that the decoder cannot parse correctly.

### Reproduction (Before Fix)

```bash
# Input: [{"": null}]
# TOON output:
@, ""
- null

# Decoded result: "@, \"\""  ❌ Should be: [{"": null}]
```

### Root Cause

The tabular header `@, ""` was being treated as a scalar string instead of being recognized as a tabular array header. The parser did not have logic to recognize root-level tabular arrays - it only handled tabular arrays that were children of object keys.

### Fix

Modified the `parse_scalar_line` function in `crates/toon/src/decode/parser.rs` to detect when a scalar line is actually a tabular array header (starts with `@` followed by a delimiter character) and parse it accordingly.

The fix:
1. Checks if a scalar line matches the tabular header pattern (`@` followed by delimiter and keys)
2. If so, parses the header and following list items as a tabular array
3. Otherwise, parses it as a regular scalar value

This enables root-level tabular arrays to be properly decoded, including those with empty string keys.

**Files changed:**
- `crates/toon/src/decode/parser.rs` (lines 276-410): Rewrote `parse_scalar_line` to handle tabular arrays

**Tests added:**
- `crates/toon/tests/roundtrip.rs`: Added 3 tests for tabular arrays with empty string keys

### Verification

```bash
echo '[{"": null}]' | cargo run -p toon-cli
# Output:
# @, ""
# - null

echo '[{"": null}]' | cargo run -p toon-cli | cargo run -p toon-cli -- --decode
# Output: [{"": null}] ✓
```
