# Repository Guidelines

## Project Structure & Module Organization
- Root `Cargo.toml` defines the workspace: core library in `crates/toon/src/{decode,encode,ser}` and CLI in `crates/toon-cli`.
- Integration tests live in `crates/toon/tests` and `crates/toon-cli/tests`; add end-to-end cases next to files such as `encode_basic.rs`.
- `spec/` vendors the official TOON fixtures; sync them when conformance behavior changes and include updates in the PR.
- `fuzz/` holds the cargo-fuzz setup and helper scripts; stash new seeds in `fuzz/corpus` and crashes in `fuzz/artifacts`.

## Build, Test, and Development Commands
- `cargo fmt --all` formats the workspace (CI runs `--check`).
- `cargo clippy --workspace -- -D warnings` must be clean before pushing.
- `cargo build --workspace` verifies both crates compile with default features.
- `cargo test --workspace` runs library and CLI test suites.
- `TOON_CONFORMANCE=1 cargo test -p toon --tests` exercises the specification fixtures (requires `spec/tests/fixtures`).
- `cargo run -p toon-cli -- encode examples/request.json` provides a quick CLI smoke test.
- `./fuzz/fuzz.sh run decode` kicks off the recommended fuzz target.

## Coding Style & Naming Conventions
- Rust 2024 edition with the stable toolchain (`rust-toolchain.toml`); stick to 4-space indentation and rustfmt defaults.
- Follow standard casing: `snake_case` for functions, `CamelCase` for types, `SCREAMING_SNAKE_CASE` for constants.
- Keep parsing helpers in `decode`, encoding logic in `encode`, and shared structures in `value.rs`; avoid cross-module glob imports.
- Treat Clippy warnings as errors and add concise comments only when behavior is non-obvious.

## Testing Guidelines
- Place integration tests in `crates/toon/tests` or CLI checks in `crates/toon-cli/tests`; reuse naming patterns such as `strict_tabular.rs`.
- Update spec fixtures when parser or encoder behavior shifts and rerun the `TOON_CONFORMANCE` suite before committing.
- Extend benches in `crates/toon/benches` for performance-sensitive changes and capture representative inputs.
- Use the fuzz harness for parser regressions, keeping minimized cases under `fuzz/corpus` so they run in CI smoke tests.

## Commit & Pull Request Guidelines
- Use conventional-style commits (`feat(encode): add strict table validation`) or match the focused subject lines already in history.
- Squash exploratory commits before opening a PR and push well-scoped branches (e.g., `feature/strict-mode-fixes`).
- PRs should describe motivation, list tests run, and link related issues; include CLI output or TOON samples when behavior changes.
- Confirm `fmt`, `clippy`, default tests, and conformance tests (when relevant) before requesting review.
