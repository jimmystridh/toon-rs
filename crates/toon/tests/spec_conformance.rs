use std::fs;
use std::path::PathBuf;

fn fixtures_root() -> Option<PathBuf> {
    let mut dir = std::env::current_dir().ok()?;
    loop {
        let p = dir.join("spec/tests/fixtures");
        if p.exists() { return Some(p); }
        if !dir.pop() { break; }
    }
    None
}

fn should_run() -> bool {
    std::env::var("TOON_CONFORMANCE").ok().as_deref() == Some("1")
}

#[test]
fn decode_fixtures() -> Result<(), Box<dyn std::error::Error>> {
    if !should_run() { eprintln!("TOON_CONFORMANCE not set; skipping conformance decode tests"); return Ok(()); }
    let Some(root) = fixtures_root() else { eprintln!("spec fixtures not found; skipping"); return Ok(()); };
    let dir = root.join("decode");
    if !dir.exists() { eprintln!("no decode fixtures; skipping"); return Ok(()); }

    let mut cnt = 0;
    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("toon") { continue; }
        let expected_json = path.with_extension("json");
        if !expected_json.exists() { continue; }
        let toon = fs::read_to_string(&path)?;
        let mut opts = toon::Options::default();
        opts.strict = true;
        let got: serde_json::Value = toon::decode_from_str(&toon, &opts)?;
        let exp: serde_json::Value = serde_json::from_str(&fs::read_to_string(&expected_json)?)?;
        assert_eq!(got, exp, "Mismatch for {:?}", path.file_name().unwrap());
        cnt += 1;
    }
    eprintln!("✓ ran {} decode fixtures", cnt);
    Ok(())
}

#[test]
fn encode_fixtures() -> Result<(), Box<dyn std::error::Error>> {
    if !should_run() { eprintln!("TOON_CONFORMANCE not set; skipping conformance encode tests"); return Ok(()); }
    let Some(root) = fixtures_root() else { eprintln!("spec fixtures not found; skipping"); return Ok(()); };
    let dir = root.join("encode");
    if !dir.exists() { eprintln!("no encode fixtures; skipping"); return Ok(()); }

    let mut cnt = 0;
    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") { continue; }
        let expected_toon = path.with_extension("toon");
        if !expected_toon.exists() { continue; }
        let json: serde_json::Value = serde_json::from_str(&fs::read_to_string(&path)?)?;
        let out = toon::encode_to_string(&json, &toon::Options::default())?;
        let expected = fs::read_to_string(&expected_toon)?;
        // Compare normalized newlines and trim trailing whitespace lines
        let norm = |s: &str| s.replace("\r\n", "\n");
        assert_eq!(norm(&out), norm(&expected), "Mismatch for {:?}", path.file_name().unwrap());
        cnt += 1;
    }
    eprintln!("✓ ran {} encode fixtures", cnt);
    Ok(())
}
