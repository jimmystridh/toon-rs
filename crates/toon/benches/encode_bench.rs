use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion, Throughput};
use serde_json::Value;
use std::{fs, path::PathBuf};

fn fixtures_encode() -> Vec<(String, Value)> {
    let mut out = Vec::new();
    if let Some(dir) = find_fixtures("encode") {
        if let Ok(rd) = fs::read_dir(&dir) {
            for e in rd.flatten() {
                let p = e.path();
                if p.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Ok(s) = fs::read_to_string(&p) {
                        if let Ok(v) = serde_json::from_str::<Value>(&s) {
                            out.push((p.file_name().unwrap().to_string_lossy().to_string(), v));
                        }
                    }
                }
            }
        }
    }
    if out.is_empty() {
        // Fallback synthetic dataset
        out.push(("small_obj".into(), json_small()));
        out.push(("tabular_1k".into(), json_tabular(1000, 4)));
        out.push(("nested".into(), json_nested(4, 4)));
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

fn json_small() -> Value { serde_json::json!({"a":1,"b":[true,"x"]}) }

fn json_tabular(rows: usize, keys: usize) -> Value {
    let mut arr = Vec::with_capacity(rows);
    for i in 0..rows {
        let mut obj = serde_json::Map::with_capacity(keys);
        for k in 0..keys { obj.insert(format!("k{}", k), Value::from((i + k) as i64)); }
        arr.push(Value::Object(obj));
    }
    Value::Object(serde_json::Map::from_iter([(String::from("rows"), Value::Array(arr))]))
}

fn json_nested(depth: usize, breadth: usize) -> Value {
    fn rec(d: usize, b: usize) -> Value {
        if d == 0 { return Value::from(1); }
        let mut m = serde_json::Map::new();
        for i in 0..b { m.insert(format!("k{}", i), rec(d - 1, b)); }
        Value::Object(m)
    }
    rec(depth, breadth)
}

pub fn encode_benchmarks(c: &mut Criterion) {
    let cases = fixtures_encode();
    let mut group = c.benchmark_group("encode_json_to_toon");
    for (name, v) in cases {
        let s = serde_json::to_string(&v).unwrap();
        group.throughput(Throughput::Bytes(s.len() as u64));
        group.bench_function(format!("value_path::{name}"), |b| {
            b.iter_batched(
                || v.clone(),
                |vv| {
                    let out = toon::encode_to_string(&vv, &toon::Options::default()).unwrap();
                    black_box(out)
                },
                BatchSize::SmallInput,
            )
        });
        group.bench_function(format!("streaming::{name}"), |b| {
            b.iter_batched(
                || v.clone(),
                |vv| {
                    let out = toon::ser::to_string_streaming(&vv, &toon::Options::default()).unwrap();
                    black_box(out)
                },
                BatchSize::SmallInput,
            )
        });
    }
    group.finish();
}

criterion_group!(benches, encode_benchmarks);
criterion_main!(benches);
