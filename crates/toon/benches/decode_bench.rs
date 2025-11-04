use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion, Throughput};
use std::{fs, path::PathBuf};

fn fixtures_decode() -> Vec<(String, String)> {
    let mut out = Vec::new();
    if let Some(dir) = find_fixtures("decode") {
        if let Ok(rd) = fs::read_dir(&dir) {
            for e in rd.flatten() {
                let p = e.path();
                if p.extension().and_then(|s| s.to_str()) == Some("toon") {
                    if let Ok(s) = fs::read_to_string(&p) {
                        out.push((p.file_name().unwrap().to_string_lossy().to_string(), s));
                    }
                }
            }
        }
    }
    if out.is_empty() {
        out.push(("small".into(), "a: 1\nb:\n  - true\n  - \"x\"\n".to_string()));
        out.push(("tabular_1k".into(), make_tabular_toons(1000)));
    }
    out
}

fn find_fixtures(kind: &str) -> Option<PathBuf> {
    let mut dir = std::env::current_dir().ok()?;
    loop {
        let p = dir.join("spec/tests/fixtures").join(kind);
        if p.exists() { return Some(p); }
        if !dir.pop() { break; }
    }
    None
}

fn make_tabular_toons(rows: usize) -> String {
    let mut s = String::new();
    s.push_str("rows:\n  @, a, b\n");
    for i in 0..rows { s.push_str(&format!("  - {}, {}\n", i, i + 1)); }
    s
}

pub fn decode_benchmarks(c: &mut Criterion) {
    let cases = fixtures_decode();
    let mut group = c.benchmark_group("decode_toon_to_json");
    for (name, toon) in cases {
        group.throughput(Throughput::Bytes(toon.len() as u64));
        group.bench_function(format!("non_strict::{name}"), |b| {
            b.iter_batched(
                || toon.clone(),
                |s| {
                    let v: serde_json::Value = toon::decode_from_str(&s, &toon::Options::default()).unwrap();
                    black_box(v)
                },
                BatchSize::SmallInput,
            )
        });
        group.bench_function(format!("strict::{name}"), |b| {
            b.iter_batched(
                || toon.clone(),
                |s| {
                    let mut opts = toon::Options::default();
                    opts.strict = true;
                    let v: serde_json::Value = toon::decode_from_str(&s, &opts).unwrap();
                    black_box(v)
                },
                BatchSize::SmallInput,
            )
        });
    }
    group.finish();
}

criterion_group!(benches, decode_benchmarks);
criterion_main!(benches);
