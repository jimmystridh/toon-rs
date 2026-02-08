use criterion::{BatchSize, Criterion, Throughput, black_box, criterion_group, criterion_main};
use rand::{Rng, SeedableRng, rngs::StdRng};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Row {
    a: u32,
    b: String,
    c: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Data {
    rows: Vec<Row>,
}

fn gen_data(n: usize) -> Data {
    let mut rng = StdRng::seed_from_u64(42);
    let mut rows = Vec::with_capacity(n);
    for i in 0..n as u32 {
        let s = (0..8)
            .map(|_| (b'a' + (rng.r#gen::<u8>() % 26)) as char)
            .collect::<String>();
        rows.push(Row {
            a: i,
            b: s,
            c: rng.gen_bool(0.5),
        });
    }
    Data { rows }
}

pub fn stream_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("stream_typed");
    for &n in &[100, 1_000, 10_000] {
        let data = gen_data(n);
        let json_sz = serde_json::to_vec(&data).unwrap().len() as u64;
        group.throughput(Throughput::Bytes(json_sz));
        group.bench_function(format!("to_string_streaming::{n}"), |b| {
            b.iter_batched(
                || data.clone(),
                |d| {
                    let out = toon_rs::ser::to_string_streaming(&d, &toon_rs::Options::default())
                        .unwrap();
                    black_box(out)
                },
                BatchSize::SmallInput,
            )
        });
        group.bench_function(format!("de_from_str::{n}"), |b| {
            let s = toon_rs::ser::to_string_streaming(&data, &toon_rs::Options::default()).unwrap();
            b.iter_batched(
                || s.clone(),
                |ss| {
                    let d: Data = toon_rs::de::from_str(&ss, &toon_rs::Options::default()).unwrap();
                    black_box(d)
                },
                BatchSize::SmallInput,
            )
        });
    }
    group.finish();
}

criterion_group!(benches, stream_benchmarks);
criterion_main!(benches);
