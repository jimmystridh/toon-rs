//! TOON Specification Conformance Tests
//!
//! Runs tests from the official spec/tests/fixtures directory.
//! Set TOON_CONFORMANCE=1 to run these tests.

#![cfg(feature = "json")]
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct FixtureFile {
    #[allow(dead_code)]
    version: String,
    #[allow(dead_code)]
    category: String,
    #[allow(dead_code)]
    description: String,
    tests: Vec<TestCase>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TestCase {
    name: String,
    input: serde_json::Value,
    expected: serde_json::Value,
    #[serde(default)]
    should_error: bool,
    #[serde(default)]
    options: TestOptions,
    #[allow(dead_code)]
    #[serde(default)]
    spec_section: Option<String>,
    #[allow(dead_code)]
    #[serde(default)]
    note: Option<String>,
    #[allow(dead_code)]
    #[serde(default)]
    min_spec_version: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TestOptions {
    #[serde(default)]
    delimiter: Option<String>,
    #[allow(dead_code)]
    #[serde(default)]
    indent: Option<usize>,
    #[serde(default)]
    strict: Option<bool>,
    #[serde(default)]
    key_folding: Option<String>,
    #[allow(dead_code)]
    #[serde(default)]
    flatten_depth: Option<usize>,
    #[serde(default)]
    expand_paths: Option<String>,
}

fn fixtures_root() -> Option<PathBuf> {
    let mut dir = std::env::current_dir().ok()?;
    loop {
        let p = dir.join("spec/tests/fixtures");
        if p.exists() {
            return Some(p);
        }
        if !dir.pop() {
            break;
        }
    }
    None
}

fn should_run() -> bool {
    std::env::var("TOON_CONFORMANCE").ok().as_deref() == Some("1")
}

fn make_encode_options(opts: &TestOptions) -> toon::Options {
    let mut o = toon::Options::default();
    if let Some(ref d) = opts.delimiter {
        o.delimiter = match d.as_str() {
            "\t" => toon::Delimiter::Tab,
            "|" => toon::Delimiter::Pipe,
            _ => toon::Delimiter::Comma,
        };
    }
    if let Some(ref kf) = opts.key_folding {
        o.key_folding = match kf.as_str() {
            "safe" => toon::KeyFolding::Safe,
            _ => toon::KeyFolding::Off,
        };
    }
    if let Some(fd) = opts.flatten_depth {
        o.flatten_depth = Some(fd);
    }
    if let Some(indent) = opts.indent {
        o.indent = indent;
    }
    o
}

fn make_decode_options(opts: &TestOptions) -> toon::Options {
    let mut o = toon::Options::default();
    if let Some(strict) = opts.strict {
        o.strict = strict;
    }
    if let Some(indent) = opts.indent {
        o.indent = indent;
    }
    if let Some(ref ep) = opts.expand_paths {
        o.expand_paths = match ep.as_str() {
            "safe" => toon::ExpandPaths::Safe,
            _ => toon::ExpandPaths::Off,
        };
    }
    o
}

#[test]
fn decode_fixtures() -> Result<(), Box<dyn std::error::Error>> {
    if !should_run() {
        eprintln!("TOON_CONFORMANCE not set; skipping conformance decode tests");
        return Ok(());
    }
    let Some(root) = fixtures_root() else {
        eprintln!("spec fixtures not found; skipping");
        return Ok(());
    };
    let dir = root.join("decode");
    if !dir.exists() {
        eprintln!("no decode fixtures; skipping");
        return Ok(());
    }

    let mut total = 0;
    let mut passed = 0;
    let mut skipped = 0;
    let mut failed_tests: Vec<String> = Vec::new();

    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }

        let content = fs::read_to_string(&path)?;
        let fixture: FixtureFile = serde_json::from_str(&content)?;

        eprintln!(
            "  Testing decode/{}",
            path.file_name().unwrap().to_string_lossy()
        );

        for test in &fixture.tests {
            total += 1;

            // Skip key folding tests until implemented for decoding
            if test.options.key_folding.is_some()
                && test.options.key_folding.as_deref() != Some("off")
            {
                skipped += 1;
                continue;
            }

            let toon_input = match &test.input {
                serde_json::Value::String(s) => s.clone(),
                _ => {
                    eprintln!("    SKIP {}: decode input must be string", test.name);
                    skipped += 1;
                    continue;
                }
            };

            let opts = make_decode_options(&test.options);
            let result: Result<serde_json::Value, _> = toon::decode_from_str(&toon_input, &opts);

            if test.should_error {
                if result.is_err() {
                    passed += 1;
                } else {
                    eprintln!(
                        "    FAIL {}: expected error but got {:?}",
                        test.name,
                        result.unwrap()
                    );
                    failed_tests.push(format!(
                        "decode/{}: {}",
                        path.file_name().unwrap().to_string_lossy(),
                        test.name
                    ));
                }
            } else {
                match result {
                    Ok(got) => {
                        if got == test.expected {
                            passed += 1;
                        } else {
                            eprintln!("    FAIL {}", test.name);
                            eprintln!("      input: {:?}", toon_input);
                            eprintln!("      expected: {:?}", test.expected);
                            eprintln!("      got: {:?}", got);
                            failed_tests.push(format!(
                                "decode/{}: {}",
                                path.file_name().unwrap().to_string_lossy(),
                                test.name
                            ));
                        }
                    }
                    Err(e) => {
                        eprintln!("    FAIL {}: unexpected error: {}", test.name, e);
                        failed_tests.push(format!(
                            "decode/{}: {}",
                            path.file_name().unwrap().to_string_lossy(),
                            test.name
                        ));
                    }
                }
            }
        }
    }

    eprintln!("✓ decode: {}/{} passed, {} skipped", passed, total, skipped);
    if !failed_tests.is_empty() {
        eprintln!("Failed tests:");
        for t in &failed_tests {
            eprintln!("  - {}", t);
        }
    }
    assert!(failed_tests.is_empty(), "Some decode tests failed");
    Ok(())
}

#[test]
fn encode_fixtures() -> Result<(), Box<dyn std::error::Error>> {
    if !should_run() {
        eprintln!("TOON_CONFORMANCE not set; skipping conformance encode tests");
        return Ok(());
    }
    let Some(root) = fixtures_root() else {
        eprintln!("spec fixtures not found; skipping");
        return Ok(());
    };
    let dir = root.join("encode");
    if !dir.exists() {
        eprintln!("no encode fixtures; skipping");
        return Ok(());
    }

    let mut total = 0;
    let mut passed = 0;
    let mut skipped = 0;
    let mut failed_tests: Vec<String> = Vec::new();

    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }

        let content = fs::read_to_string(&path)?;
        let fixture: FixtureFile = serde_json::from_str(&content)?;

        eprintln!(
            "  Testing encode/{}",
            path.file_name().unwrap().to_string_lossy()
        );

        for test in &fixture.tests {
            total += 1;

            let expected_toon = match &test.expected {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Null if test.should_error => String::new(),
                _ => {
                    eprintln!("    SKIP {}: encode expected must be string", test.name);
                    skipped += 1;
                    continue;
                }
            };

            let opts = make_encode_options(&test.options);
            let result = toon::encode_to_string(&test.input, &opts);

            if test.should_error {
                if result.is_err() {
                    passed += 1;
                } else {
                    eprintln!(
                        "    FAIL {}: expected error but got {:?}",
                        test.name,
                        result.unwrap()
                    );
                    failed_tests.push(format!(
                        "encode/{}: {}",
                        path.file_name().unwrap().to_string_lossy(),
                        test.name
                    ));
                }
            } else {
                match result {
                    Ok(got) => {
                        // Normalize newlines for comparison
                        let norm = |s: &str| s.replace("\r\n", "\n");
                        if norm(&got) == norm(&expected_toon) {
                            passed += 1;
                        } else {
                            eprintln!("    FAIL {}", test.name);
                            eprintln!("      input: {:?}", test.input);
                            eprintln!("      expected: {:?}", expected_toon);
                            eprintln!("      got: {:?}", got);
                            failed_tests.push(format!(
                                "encode/{}: {}",
                                path.file_name().unwrap().to_string_lossy(),
                                test.name
                            ));
                        }
                    }
                    Err(e) => {
                        eprintln!("    FAIL {}: unexpected error: {}", test.name, e);
                        failed_tests.push(format!(
                            "encode/{}: {}",
                            path.file_name().unwrap().to_string_lossy(),
                            test.name
                        ));
                    }
                }
            }
        }
    }

    eprintln!("✓ encode: {}/{} passed, {} skipped", passed, total, skipped);
    if !failed_tests.is_empty() {
        eprintln!("Failed tests:");
        for t in &failed_tests {
            eprintln!("  - {}", t);
        }
    }
    assert!(failed_tests.is_empty(), "Some encode tests failed");
    Ok(())
}
