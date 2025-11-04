use assert_cmd::prelude::*;
use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;

#[test]
fn help_works() -> Result<(), Box<dyn std::error::Error>> {
    Command::new(assert_cmd::cargo::cargo_bin!("toon-cli"))
        .arg("--help")
        .assert()
        .success();
    Ok(())
}

#[test]
fn encode_outputs_toon_like_syntax() -> Result<(), Box<dyn std::error::Error>> {
    let input = "{\n  \"a\": 1,\n  \"b\": [true, \"x\"]\n}\n";
    let mut tmp = NamedTempFile::new()?;
    write!(tmp, "{}", input)?;

    let output = Command::new(assert_cmd::cargo::cargo_bin!("toon-cli"))
        .arg(tmp.path())
        .output()?;
    assert!(output.status.success());
    let out = String::from_utf8(output.stdout)?;
    // Expect TOON-ish output with key lines and list markers
    assert!(out.contains("a: 1"));
    assert!(out.contains("b:"));
    assert!(out.contains("- true"));
    assert!(out.contains("- x") || out.contains("- \"x\""));
    Ok(())
}

#[test]
fn decode_toon_to_json() -> Result<(), Box<dyn std::error::Error>> {
    let input = "a: 2\n";
    let mut tmp = NamedTempFile::new()?;
    write!(tmp, "{}", input)?;

    let output = Command::new(assert_cmd::cargo::cargo_bin!("toon-cli"))
        .arg("--decode")
        .arg(tmp.path())
        .output()?;
    assert!(output.status.success());
    let out = String::from_utf8(output.stdout)?;
    let v_out: serde_json::Value = serde_json::from_str(&out)?;
    assert_eq!(v_out, serde_json::json!({"a": 2}));
    Ok(())
}
