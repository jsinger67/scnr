use std::fs;

use criterion::{criterion_group, criterion_main, Criterion};
use scnr::{Match, ScannerBuilder, ScannerMode};

const SCANNER_INPUT: &str = include_str!("./input_1.txt");

fn scanner_benchmark(c: &mut Criterion) {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/parol.modes");
    let file = fs::File::open(path).unwrap();
    let scanner_modes: Vec<ScannerMode> = serde_json::from_reader(file).unwrap();
    // Create a scanner from the scanner builder
    let scanner = ScannerBuilder::new()
        .add_scanner_modes(&scanner_modes)
        .build()
        .unwrap();

    c.bench_function("scanner_benchmark", |b| {
        b.iter(|| {
            // Create a matches iterator
            let find_iter = scanner.find_iter(SCANNER_INPUT).unwrap();
            // Collect all matches
            let _matches: Vec<Match> = find_iter.collect();
        });
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(50);
    targets = scanner_benchmark
}

criterion_main!(benches);
