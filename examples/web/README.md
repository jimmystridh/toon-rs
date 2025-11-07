# TOON WebAssembly Example

This is a web-based example demonstrating the TOON serialization format using WebAssembly.

**[Try the live demo on GitHub Pages →](https://jimmystridh.github.io/toon-rs/)**

## Features

- Convert JSON to TOON format
- Convert TOON to JSON format
- Interactive web interface with syntax highlighting
- Support for pipe (|) and comma (,) delimiters
- Strict mode validation
- Multiple example templates
- **Performance benchmark** comparing WASM vs JavaScript implementation

## Building

### Prerequisites

Install `wasm-pack`:

```bash
cargo install wasm-pack
```

### Build the WebAssembly module

From the repository root:

```bash
./examples/web/build.sh
```

Or manually:

```bash
wasm-pack build crates/toon-wasm --target web --out-dir ../../examples/web/pkg
```

## Automatic Deployment

The web demo is automatically deployed to GitHub Pages on every push to the `main` branch. The deployment workflow builds the WASM module and publishes it to https://jimmystridh.github.io/toon-rs/

## Running Locally

After building, serve the `examples/web` directory with any HTTP server:

```bash
# Using Python 3
cd examples/web
python3 -m http.server 8000

# Using Python 2
python -m SimpleHTTPServer 8000

# Using Node.js http-server
npx http-server -p 8000
```

Then open http://localhost:8000 in your browser.

## Pages

### Main Demo (`index.html`)

Interactive playground for converting between JSON and TOON formats.

### Benchmark (`benchmark.html`)

Performance comparison between the Rust/WASM implementation and the native JavaScript implementation from npm (`@toon-format/toon`).

**Features:**
- Comprehensive test cases (small, medium, tabular, large, nested)
- Multiple iterations for accurate timing
- Visual charts and detailed metrics
- Speedup calculations

**[Read the full benchmark documentation →](BENCHMARK.md)**

To access the benchmark:
```
http://localhost:8000/benchmark.html
```

## Usage

1. **JSON to TOON**: Enter JSON in the left panel and click "JSON → TOON"
2. **TOON to JSON**: Enter TOON in the right panel and click "TOON → JSON"
3. **Try Examples**: Click any example button to load sample data
4. **Options**:
   - **Pipe delimiter**: Use `|` instead of `,` for tabular arrays
   - **Strict mode**: Enable strict validation during encoding/decoding
   - **Pretty JSON**: Format JSON output with indentation

## What is TOON?

TOON (Token-Oriented Object Notation) is a human-readable, token-efficient alternative to JSON designed for LLM data exchange. It uses indentation-based structure and supports tabular arrays for efficient representation of structured data.

Example:

**JSON:**
```json
{
  "users": [
    {"id": 1, "name": "Alice", "age": 30},
    {"id": 2, "name": "Bob", "age": 25}
  ]
}
```

**TOON:**
```
users:
  @| id | name | age
  | 1 | Alice | 30
  | 2 | Bob | 25
```

## Learn More

- [TOON Specification](https://github.com/toon-format/spec)
- [TOON Rust Implementation](https://github.com/jimmystridh/toon-rs)
