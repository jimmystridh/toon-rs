# TOON Benchmark: WASM vs JavaScript

This benchmark page compares the performance of the Rust/WebAssembly implementation against the native JavaScript implementation from the npm package `@toon-format/toon`.

## Features

- **Comprehensive test cases**: Small objects, medium objects, tabular data (100+ rows), large datasets (500+ items), and deeply nested structures
- **Accurate measurements**: Multiple iterations with precise timing using `performance.now()`
- **Visual results**: Interactive charts showing comparative performance
- **Detailed metrics**: Operation times, speedup factors, and input/output sizes
- **Real-time progress**: Progress bar and status updates during benchmark execution

## Prerequisites

1. **wasm-pack** - Install it if you haven't already:
   ```bash
   cargo install wasm-pack
   ```

## Building

From the repository root, run:

```bash
./examples/web/build.sh
```

Or manually:

```bash
wasm-pack build crates/toon-wasm --target web --out-dir ../../examples/web/pkg
```

This will build the WASM module and generate the necessary JavaScript bindings in `examples/web/pkg/`.

## Running the Benchmark

1. Build the WASM module (see above)

2. Start a local web server in the `examples/web` directory:

   ```bash
   cd examples/web

   # Using Python 3
   python3 -m http.server 8000

   # OR using Node.js
   npx http-server -p 8000
   ```

3. Open your browser and navigate to:
   - Main demo: http://localhost:8000/
   - Benchmark: http://localhost:8000/benchmark.html

4. Click "Run Benchmark" to start the performance comparison

## What It Tests

The benchmark evaluates both **encoding (JSON → TOON)** and **decoding (TOON → JSON)** operations across multiple test scenarios:

### Test Cases

1. **Small Object** (~50 bytes)
   - Simple flat object with basic types
   - Tests overhead and setup time

2. **Medium Object** (~200 bytes)
   - Nested objects with arrays
   - Tests nesting and mixed types

3. **Tabular Data** (100 rows)
   - Array of uniform objects
   - Tests TOON's tabular array optimization

4. **Large Object** (500 items)
   - Large array with nested metadata
   - Tests performance at scale

5. **Deeply Nested** (5+ levels)
   - Complex nested structure
   - Tests recursive processing

## Understanding the Results

### Metrics Displayed

- **Operation Time**: Average time per operation in milliseconds (ms) or microseconds (µs)
- **Speedup Factor**: How many times faster WASM is compared to JavaScript
  - Green badge: WASM is faster
  - Orange badge: JavaScript is faster (rare)
- **Input/Output Size**: Size of data being processed in bytes

### Expected Performance

The WASM implementation typically shows:
- **2-10x faster** for small to medium objects
- **5-20x faster** for tabular data (TOON's sweet spot)
- **10-50x faster** for large datasets

Performance advantages increase with:
- Larger data sizes
- More complex structures
- Tabular array encoding/decoding

## How the Benchmark Works

1. **Module Loading**
   - Loads the Rust/WASM module from `./pkg/toon_wasm.js`
   - Loads the JavaScript module from npm via CDN: `@toon-format/toon`

2. **Test Execution**
   - Runs each test case 100 times (configurable)
   - Measures average time using `performance.now()`
   - Tests both encoding and decoding operations

3. **Result Visualization**
   - Summary cards with speedup indicators
   - Bar chart comparing all operations
   - Detailed table with all measurements

## Customizing the Benchmark

To add your own test cases, edit `benchmark.html` and add entries to the `testData` object:

```javascript
const testData = {
  myTest: {
    name: 'My Test Case',
    data: { /* your JSON data */ }
  }
  // ... more tests
};
```

## Troubleshooting

### "WASM module not loaded" error
- Make sure you've built the WASM module first with `./examples/web/build.sh`
- Check that `examples/web/pkg/` directory exists and contains the built files

### "Error loading JS module" error
- This is expected if the npm package is not available
- The benchmark will continue with WASM-only results
- Check your internet connection as the JS module loads from CDN

### Performance seems slower than expected
- Run the benchmark multiple times to warm up the JIT compiler
- Close other tabs and applications to reduce system load
- Try different browsers (Chrome/Firefox/Safari have different WASM performance)

## Technical Details

### WASM Implementation
- Built from Rust source in `crates/toon-wasm/`
- Uses `wasm-bindgen` for JavaScript interop
- Uses `wee_alloc` for smaller binary size
- Zero-copy parsing where possible

### JavaScript Implementation
- Native TypeScript implementation
- Loaded from npm package `@toon-format/toon`
- Pure JavaScript with no native dependencies

### Benchmark Methodology
- Each operation runs 100 times
- Average time calculated from all runs
- `performance.now()` provides microsecond precision
- Results exclude module loading and setup time

## Learn More

- [TOON Specification](https://github.com/toon-format/spec)
- [TOON Rust Implementation](https://github.com/jimmystridh/toon-rs)
- [TOON JavaScript Implementation](https://github.com/toon-format/toon)
