//! Benchmark comparing manual parser vs chumsky parser

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use mtg_forge_rs::puzzle::parser_chumsky;
use mtg_forge_rs::puzzle::PuzzleFile;
use std::fs;
use std::path::PathBuf;

fn load_sample_puzzles() -> Vec<(String, String)> {
    let puzzle_dir = PathBuf::from("forge-java/forge-gui/res/puzzle");

    if !puzzle_dir.exists() {
        eprintln!("Warning: puzzle directory not found, using synthetic samples");
        return vec![(
            "synthetic.pzl".to_string(),
            r#"
[metadata]
Name=Test Puzzle
Goal=Win
Turns=1
Difficulty=Easy

[state]
turn=1
activeplayer=p0
activephase=MAIN1
p0life=20
p0hand=Lightning Bolt;Grizzly Bears
p0battlefield=Mountain;Mountain;Mountain
p1life=10
p1battlefield=Forest|Tapped;Plains
"#
            .to_string(),
        )];
    }

    // Load a representative sample of puzzles
    let mut puzzles = Vec::new();

    for entry in fs::read_dir(&puzzle_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("pzl") {
            if let Ok(contents) = fs::read_to_string(&path) {
                // Skip the two files with multi-line values that chumsky can't handle
                let filename = path.file_name().unwrap().to_string_lossy();
                if filename.contains("tutorial02") || filename.contains("tutorial03") {
                    continue;
                }

                puzzles.push((filename.to_string(), contents));

                // Sample first 20 puzzles for benchmark
                if puzzles.len() >= 20 {
                    break;
                }
            }
        }
    }

    puzzles
}

fn bench_manual_parser(c: &mut Criterion) {
    let samples = load_sample_puzzles();

    let mut group = c.benchmark_group("manual_parser");

    for (name, contents) in &samples {
        let size = contents.len();
        group.throughput(Throughput::Bytes(size as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            &contents,
            |b, contents| {
                b.iter(|| PuzzleFile::parse(black_box(contents)).unwrap());
            },
        );
    }

    group.finish();
}

fn bench_chumsky_parser(c: &mut Criterion) {
    let samples = load_sample_puzzles();

    let mut group = c.benchmark_group("chumsky_parser");

    for (name, contents) in &samples {
        let size = contents.len();
        group.throughput(Throughput::Bytes(size as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            &contents,
            |b, contents| {
                b.iter(|| parser_chumsky::parse_puzzle(black_box(contents)).unwrap());
            },
        );
    }

    group.finish();
}

fn bench_comparison(c: &mut Criterion) {
    let samples = load_sample_puzzles();

    // Take one medium-sized sample for direct comparison
    if let Some((name, contents)) = samples.get(10) {
        let mut group = c.benchmark_group("parser_comparison");
        let size = contents.len();
        group.throughput(Throughput::Bytes(size as u64));

        group.bench_function("manual", |b| {
            b.iter(|| PuzzleFile::parse(black_box(contents)).unwrap());
        });

        group.bench_function("chumsky", |b| {
            b.iter(|| parser_chumsky::parse_puzzle(black_box(contents)).unwrap());
        });

        group.finish();

        println!("\nBenchmarking file: {} ({} bytes)", name, size);
    }
}

criterion_group!(
    benches,
    bench_manual_parser,
    bench_chumsky_parser,
    bench_comparison
);
criterion_main!(benches);
