use std::{fs, sync::LazyLock, time::Duration};

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use scnr::{Scanner, ScannerBuilder, ScannerMode};

const PAR_SCANNER_INPUT: &str = include_str!("./input_1.par");
const VERYL_SCANNER_INPUT: &str = include_str!("./veryl_input.veryl");

static PAR_SCANNER_MODES: LazyLock<Vec<ScannerMode>> = LazyLock::new(|| {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/parol.json");
    let file = fs::File::open(path).unwrap();
    serde_json::from_reader(file).unwrap()
});

static VERLY_SCANNER_MODES: LazyLock<Vec<ScannerMode>> = LazyLock::new(|| {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "./benches/veryl_modes.json");
    let file = fs::File::open(path).unwrap();
    serde_json::from_reader(file).unwrap()
});

static PAR_SCANNER: LazyLock<Scanner> = LazyLock::new(|| {
    ScannerBuilder::new()
        .add_scanner_modes(&PAR_SCANNER_MODES)
        .build()
        .unwrap()
});

static VERYL_SCANNER: LazyLock<Scanner> = LazyLock::new(|| {
    ScannerBuilder::new()
        .add_scanner_modes(&VERLY_SCANNER_MODES)
        .build()
        .unwrap()
});

fn build_parol_scanner(c: &mut Criterion) {
    c.bench_function("build_par_scanner", |b| {
        b.iter(|| {
            black_box(
                ScannerBuilder::new()
                    .add_scanner_modes(&PAR_SCANNER_MODES)
                    .build()
                    .unwrap(),
            );
        });
    });
}

fn build_veryl_scanner(c: &mut Criterion) {
    c.bench_function("build_veryl_scanner", |b| {
        b.iter(|| {
            black_box(
                ScannerBuilder::new()
                    .add_scanner_modes(&PAR_SCANNER_MODES)
                    .build()
                    .unwrap(),
            );
        });
    });
}

fn run_parol_scanner(c: &mut Criterion) {
    let mut group = c.benchmark_group("parol_scanner_benchmark");
    group.throughput(Throughput::Bytes(PAR_SCANNER_INPUT.len() as u64));
    group.bench_function("throughput", |b| {
        b.iter(|| {
            // Create a matches iterator
            let find_iter = PAR_SCANNER.find_iter(PAR_SCANNER_INPUT);
            // Collect all matches
            for t in find_iter {
                black_box(t);
            }
        });
    });
}

fn run_veryl_scanner(c: &mut Criterion) {
    let mut group = c.benchmark_group("veryl_scanner_benchmark");
    group.throughput(Throughput::Bytes(VERYL_SCANNER_INPUT.len() as u64));
    group.bench_function("throughput", |b| {
        b.iter(|| {
            // Create a matches iterator
            let find_iter = VERYL_SCANNER.find_iter(VERYL_SCANNER_INPUT);
            // Collect all matches
            for t in find_iter {
                black_box(t);
            }
        });
    });
}

criterion_group! {
    name = benchesscanner;
    config = Criterion::default().measurement_time(Duration::from_secs(15)).sample_size(50);
    targets = run_parol_scanner, run_veryl_scanner
}

criterion_group! {
    name = benchesbuilder;
    config = Criterion::default();
    targets = build_parol_scanner, build_veryl_scanner
}

criterion_main!(benchesscanner, benchesbuilder);
