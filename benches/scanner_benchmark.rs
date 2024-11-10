use std::{fs, sync::LazyLock, time::Duration};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use scnr::{Scanner, ScannerBuilder, ScannerMode};

const SCANNER_INPUT: &str = include_str!("./input_1.par");

static SCANNER_MODES: LazyLock<Vec<ScannerMode>> = LazyLock::new(|| {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/parol.json");
    let file = fs::File::open(path).unwrap();
    serde_json::from_reader(file).unwrap()
});

static SCANNER: LazyLock<Scanner> = LazyLock::new(|| {
    ScannerBuilder::new()
        .add_scanner_modes(&SCANNER_MODES)
        .build()
        .unwrap()
});

static NFA_SCANNER: LazyLock<Scanner> = LazyLock::new(|| {
    ScannerBuilder::new()
        .add_scanner_modes(&SCANNER_MODES)
        .use_nfa()
        .build()
        .unwrap()
});

fn builder_benchmark(c: &mut Criterion) {
    c.bench_function("builder_benchmark", |b| {
        b.iter(|| {
            black_box(
                ScannerBuilder::new()
                    .add_scanner_modes(&SCANNER_MODES)
                    .build()
                    .unwrap(),
            );
        });
    });
}

fn nfa_builder_benchmark(c: &mut Criterion) {
    c.bench_function("builder_benchmark", |b| {
        b.iter(|| {
            black_box(
                ScannerBuilder::new()
                    .add_scanner_modes(&SCANNER_MODES)
                    .use_nfa()
                    .build()
                    .unwrap(),
            );
        });
    });
}

fn scanner_benchmark(c: &mut Criterion) {
    c.bench_function("scanner_benchmark", |b| {
        b.iter(|| {
            // Create a matches iterator
            let find_iter = SCANNER.find_iter(SCANNER_INPUT);
            // Collect all matches
            for t in find_iter {
                black_box(t);
            }
        });
    });
}

fn nfa_scanner_benchmark(c: &mut Criterion) {
    c.bench_function("nfa_scanner_benchmark", |b| {
        b.iter(|| {
            // Create a matches iterator
            let find_iter = NFA_SCANNER.find_iter(SCANNER_INPUT);
            // Collect all matches
            for t in find_iter {
                black_box(t);
            }
        });
    });
}

criterion_group! {
    name = benchesscanner;
    config = Criterion::default().measurement_time(Duration::from_secs(10));
    targets = scanner_benchmark, nfa_scanner_benchmark
}

criterion_group! {
    name = benchesbuilder;
    config = Criterion::default();
    targets = builder_benchmark, nfa_builder_benchmark
}

criterion_main!(benchesscanner, benchesbuilder);
