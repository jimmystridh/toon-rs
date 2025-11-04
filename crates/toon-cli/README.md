# toon-cli

Command-line interface for converting between JSON and TOON.

## Usage

```sh
# Show help
toon-cli --help

# Encode JSON file to TOON
toon-cli path/to/input.json > out.toon

# Decode TOON file to JSON (pretty)
toon-cli --decode --pretty path/to/data.toon > out.json

# Read from stdin
cat input.json | toon-cli > out.toon
```

Options:
- `--decode`: TOON → JSON
- `--delimiter <comma|tab|pipe>`: set active delimiter for tabular arrays (default: comma)
- `--strict`: enable strict validation when decoding
- `--pretty`: pretty-print JSON on output when decoding
