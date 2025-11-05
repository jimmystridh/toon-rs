# Fuzzing Findings

This document tracks bugs found by fuzzing.

## Bug #1: Empty Collections Roundtrip Failure

**Status**: Found
**Fuzzer**: `fuzz_structured`
**Date**: 2025-11-05
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
