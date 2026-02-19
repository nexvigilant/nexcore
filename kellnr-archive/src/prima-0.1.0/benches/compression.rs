// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Compression Benchmarks
//!
//! Measures lexical compression performance.
//!
//! ## Tier: T2-C (σ + μ + κ + N + →)
//!
//! Run with: `cargo bench -p prima`

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use prima::compress::Lexicon;

/// Sample Prima code for benchmarking
const SAMPLE_SMALL: &str = "fn add a N b N arrow N a plus b";
const SAMPLE_MEDIUM: &str = r#"
fn factorial n N arrow N
    if n less_than two
        one
    else
        n times factorial n minus one

fn fibonacci n N arrow N
    if n less_than two
        n
    else
        fibonacci n minus one plus fibonacci n minus two
"#;

const SAMPLE_LARGE: &str = r#"
fn quicksort arr sequence arrow sequence
    if length arr less_than two
        arr
    else
        let pivot equals head arr
        let rest equals tail arr
        let less equals filter rest fn x x less_than pivot
        let greater equals filter rest fn x x greater_than pivot
        concat quicksort less concat sequence pivot quicksort greater

fn merge left sequence right sequence arrow sequence
    if length left equals zero
        right
    else
        if length right equals zero
            left
        else
            if head left less_than head right
                concat sequence head left merge tail left right
            else
                concat sequence head right merge left tail right

fn map_reduce data fn mapper fn reducer initial
    fold map data mapper initial reducer
"#;

fn bench_lexicon_creation(c: &mut Criterion) {
    c.bench_function("lexicon_new", |b| b.iter(|| black_box(Lexicon::new())));
}

fn bench_compress(c: &mut Criterion) {
    let lex = Lexicon::new();

    let mut group = c.benchmark_group("compress_text");

    group.bench_with_input(
        BenchmarkId::new("small", "40 chars"),
        SAMPLE_SMALL,
        |b, input| b.iter(|| black_box(lex.compress_text(input))),
    );

    group.bench_with_input(
        BenchmarkId::new("medium", "~300 chars"),
        SAMPLE_MEDIUM,
        |b, input| b.iter(|| black_box(lex.compress_text(input))),
    );

    group.bench_with_input(
        BenchmarkId::new("large", "~900 chars"),
        SAMPLE_LARGE,
        |b, input| b.iter(|| black_box(lex.compress_text(input))),
    );

    group.finish();
}

fn bench_single_word(c: &mut Criterion) {
    let lex = Lexicon::new();

    let mut group = c.benchmark_group("compress_word");

    let words = ["sequence", "mapping", "function", "filter", "unknown_word"];

    for word in words {
        group.bench_with_input(BenchmarkId::from_parameter(word), word, |b, input| {
            b.iter(|| black_box(lex.compress(input)))
        });
    }

    group.finish();
}

fn bench_compression_ratio(c: &mut Criterion) {
    let lex = Lexicon::new();

    let mut group = c.benchmark_group("compression_ratio");

    group.bench_with_input(
        BenchmarkId::new("small", "ratio"),
        SAMPLE_SMALL,
        |b, input| b.iter(|| black_box(lex.compression_ratio(input))),
    );

    group.bench_with_input(
        BenchmarkId::new("large", "ratio"),
        SAMPLE_LARGE,
        |b, input| b.iter(|| black_box(lex.compression_ratio(input))),
    );

    group.finish();
}

fn bench_throughput(c: &mut Criterion) {
    let lex = Lexicon::new();

    // Generate 10KB of text
    let large_text = SAMPLE_LARGE.repeat(12); // ~10KB

    c.bench_function("throughput_10kb", |b| {
        b.iter(|| black_box(lex.compress_text(&large_text)))
    });
}

criterion_group!(
    benches,
    bench_lexicon_creation,
    bench_compress,
    bench_single_word,
    bench_compression_ratio,
    bench_throughput,
);

criterion_main!(benches);
