//! Criterion benchmarks for nexcore foundation operations.
//!
//! Benchmarks the core algorithms that back the documented performance claims:
//! - Levenshtein distance: short, medium, and long string pairs
//! - Fuzzy search: batch similarity matching at various scales
//! - SHA-256 hashing: various payload sizes (64B to 1MB)
//! - YAML parsing: documents of increasing complexity
//!
//! These establish Rust baseline numbers. The documented speedups (63x, 39x, 20x, 7x)
//! are relative to equivalent Python implementations (python-Levenshtein, hashlib, PyYAML).
//!
//! Run with: cargo bench --package nexcore-vigilance --bench foundation_benchmarks
//! For HTML reports: cargo bench --package nexcore-vigilance -- --plotting-backend gnuplot

use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use nexcore_vigilance::foundation::algorithms::crypto::{sha256_bytes, sha256_hash};
use nexcore_vigilance::foundation::algorithms::levenshtein::{
    fuzzy_search, levenshtein, levenshtein_distance,
};
use nexcore_vigilance::foundation::data::yaml::parse_yaml;

// ============================================================================
// Levenshtein Distance Benchmarks
// ============================================================================

fn bench_levenshtein_distance(c: &mut Criterion) {
    let mut group = c.benchmark_group("Levenshtein");

    // Short strings (5-7 chars) - the classic example
    group.bench_function("short_kitten_sitting", |b| {
        b.iter(|| levenshtein_distance(black_box("kitten"), black_box("sitting")));
    });

    // Medium strings (~30 chars) - typical drug/event names
    let medium_a = "acetaminophen hydrochloride";
    let medium_b = "acetaminophin hydrocloride";
    group.bench_function("medium_drug_names", |b| {
        b.iter(|| levenshtein_distance(black_box(medium_a), black_box(medium_b)));
    });

    // Longer strings (~80 chars) - adverse event descriptions
    let long_a = "Severe hepatotoxicity with elevated aminotransferases and conjugated bilirubin";
    let long_b = "Severe hepatoxicity with elevated aminotransferases and unconjugated bilirubin";
    group.bench_function("long_event_descriptions", |b| {
        b.iter(|| levenshtein_distance(black_box(long_a), black_box(long_b)));
    });

    // Very long strings (~200 chars) - full narrative text
    let very_long_a = "Patient presented with acute onset of generalized tonic-clonic seizures \
                        following administration of the study drug at the recommended therapeutic \
                        dose during the open-label extension phase of the clinical trial";
    let very_long_b = "Patient presented with acute onset of generalized tonic-clonic seizure \
                        following administrations of the study drugs at the recommended therapeutic \
                        doses during the open-label extension phases of the clinical trials";
    group.bench_function("very_long_narratives", |b| {
        b.iter(|| levenshtein_distance(black_box(very_long_a), black_box(very_long_b)));
    });

    // Identical strings (best case - still must iterate)
    group.bench_function("identical_medium", |b| {
        b.iter(|| levenshtein_distance(black_box(medium_a), black_box(medium_a)));
    });

    // Completely different strings (worst case)
    let diff_a = "abcdefghijklmnopqrstuvwxyz";
    let diff_b = "ZYXWVUTSRQPONMLKJIHGFEDCBA";
    group.bench_function("completely_different_26", |b| {
        b.iter(|| levenshtein_distance(black_box(diff_a), black_box(diff_b)));
    });

    group.finish();
}

fn bench_levenshtein_full(c: &mut Criterion) {
    let mut group = c.benchmark_group("Levenshtein_Full");

    // Full result (distance + similarity) vs raw distance
    let source = "acetaminophen hydrochloride";
    let target = "acetaminophin hydrocloride";

    group.bench_function("distance_only", |b| {
        b.iter(|| levenshtein_distance(black_box(source), black_box(target)));
    });

    group.bench_function("full_result", |b| {
        b.iter(|| levenshtein(black_box(source), black_box(target)));
    });

    group.finish();
}

// ============================================================================
// Fuzzy Search Benchmarks
// ============================================================================

fn bench_fuzzy_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("Fuzzy_Search");

    // Small candidate list (10 items) - typical autocomplete
    let small_candidates: Vec<String> = vec![
        "aspirin",
        "ibuprofen",
        "acetaminophen",
        "naproxen",
        "celecoxib",
        "diclofenac",
        "meloxicam",
        "piroxicam",
        "indomethacin",
        "ketorolac",
    ]
    .into_iter()
    .map(String::from)
    .collect();

    group.throughput(Throughput::Elements(10));
    group.bench_function("10_candidates", |b| {
        b.iter(|| fuzzy_search(black_box("acetaminofen"), black_box(&small_candidates), 5));
    });

    // Medium candidate list (100 items)
    let medium_candidates: Vec<String> = generate_drug_names(100);
    group.throughput(Throughput::Elements(100));
    group.bench_function("100_candidates", |b| {
        b.iter(|| fuzzy_search(black_box("acetaminofen"), black_box(&medium_candidates), 10));
    });

    // Large candidate list (1000 items)
    let large_candidates: Vec<String> = generate_drug_names(1000);
    group.throughput(Throughput::Elements(1000));
    group.bench_function("1000_candidates", |b| {
        b.iter(|| fuzzy_search(black_box("acetaminofen"), black_box(&large_candidates), 10));
    });

    // Very large candidate list (5000 items) - full drug dictionary
    let very_large_candidates: Vec<String> = generate_drug_names(5000);
    group.throughput(Throughput::Elements(5000));
    group.bench_function("5000_candidates", |b| {
        b.iter(|| {
            fuzzy_search(
                black_box("acetaminofen"),
                black_box(&very_large_candidates),
                10,
            )
        });
    });

    group.finish();
}

/// Generate synthetic drug-like names for benchmarking.
fn generate_drug_names(count: usize) -> Vec<String> {
    let prefixes = [
        "acet", "ibu", "nap", "cel", "dic", "mel", "pir", "ind", "ket", "ome", "lan", "eso", "pan",
        "rab", "des", "cet", "lor", "fex", "dip", "chl", "ator", "rosu", "sim", "pra", "flu",
        "par", "ser", "ven", "dul", "bup", "met", "gli", "sit", "sax", "emp", "can", "dar", "apa",
        "tic", "riv",
    ];
    let suffixes = [
        "aminophen",
        "profen",
        "roxen",
        "coxib",
        "fenac",
        "xicam",
        "prazole",
        "tidine",
        "zepam",
        "olol",
        "pril",
        "sartan",
        "statin",
        "vir",
        "mab",
        "nib",
        "zumab",
        "tinib",
        "ciclib",
        "rafenib",
    ];

    (0..count)
        .map(|i| {
            let prefix = prefixes[i % prefixes.len()];
            let suffix = suffixes[i % suffixes.len()];
            format!("{prefix}{suffix}")
        })
        .collect()
}

// ============================================================================
// SHA-256 Benchmarks
// ============================================================================

fn bench_sha256(c: &mut Criterion) {
    let mut group = c.benchmark_group("SHA256");

    // Various payload sizes
    let sizes: &[(u64, &str)] = &[
        (64, "64B"),
        (256, "256B"),
        (1024, "1KB"),
        (4096, "4KB"),
        (16384, "16KB"),
        (65536, "64KB"),
        (1_048_576, "1MB"),
    ];

    for &(size, label) in sizes {
        let payload: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
        let payload_str: String = (0..size.min(65536))
            .map(|i| (b'a' + (i % 26) as u8) as char)
            .collect();

        group.throughput(Throughput::Bytes(size));

        // Byte-based hashing (typical for file hashing)
        group.bench_with_input(BenchmarkId::new("bytes", label), &payload, |b, data| {
            b.iter(|| sha256_bytes(black_box(data)));
        });

        // String-based hashing (typical for content verification)
        if size <= 65536 {
            group.bench_with_input(
                BenchmarkId::new("string", label),
                &payload_str,
                |b, data| {
                    b.iter(|| sha256_hash(black_box(data)));
                },
            );
        }
    }

    // Benchmark typical use case: hashing a short identifier
    group.bench_function("short_identifier", |b| {
        b.iter(|| sha256_hash(black_box("adverse_event_12345")));
    });

    // Benchmark typical use case: hashing a JSON payload
    let json_payload = r#"{"drug":"aspirin","event":"headache","count":42,"source":"FAERS"}"#;
    group.bench_function("json_payload", |b| {
        b.iter(|| sha256_hash(black_box(json_payload)));
    });

    group.finish();
}

// ============================================================================
// YAML Parse Benchmarks
// ============================================================================

fn bench_yaml_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("YAML_Parse");

    // Minimal YAML (skill frontmatter)
    let minimal_yaml = r#"
name: test-skill
version: "1.0.0"
description: A simple test skill
"#;
    group.bench_function("minimal_3_keys", |b| {
        b.iter(|| parse_yaml(black_box(minimal_yaml)));
    });

    // Typical skill frontmatter
    let skill_yaml = r#"
name: pharmacovigilance-signal-detector
version: "2.1.0"
compliance-level: Diamond
description: Automated signal detection for adverse drug reactions
triggers:
  - pattern: signal detection
  - pattern: pv signal
  - pattern: disproportionality
tags:
  - pharmacovigilance
  - signal-detection
  - safety
  - bayesian
capabilities:
  - prr_calculation
  - ror_calculation
  - ic_bayesian
  - ebgm_shrinkage
dependencies:
  - nexcore-vigilance
  - statrs
"#;
    group.bench_function("skill_frontmatter_20_keys", |b| {
        b.iter(|| parse_yaml(black_box(skill_yaml)));
    });

    // Medium config (nested structure ~50 nodes)
    let medium_yaml = generate_nested_yaml(3, 4);
    group.bench_function("nested_50_nodes", |b| {
        b.iter(|| parse_yaml(black_box(&medium_yaml)));
    });

    // Large array (100 items)
    let array_yaml = generate_array_yaml(100);
    group.bench_function("array_100_items", |b| {
        b.iter(|| parse_yaml(black_box(&array_yaml)));
    });

    // Large array (1000 items)
    let array_yaml_1k = generate_array_yaml(1000);
    group.bench_function("array_1000_items", |b| {
        b.iter(|| parse_yaml(black_box(&array_yaml_1k)));
    });

    // Complex document (mixed types, nesting, arrays)
    let complex_yaml = r#"
database:
  primary:
    host: "db.nexvigilant.com"
    port: 5432
    credentials:
      username: admin
      password_ref: vault://secrets/db
    pool:
      min_connections: 5
      max_connections: 100
      idle_timeout_ms: 30000
  replicas:
    - host: "replica-1.nexvigilant.com"
      port: 5432
      read_only: true
    - host: "replica-2.nexvigilant.com"
      port: 5432
      read_only: true
    - host: "replica-3.nexvigilant.com"
      port: 5432
      read_only: true
signal_detection:
  algorithms:
    - name: PRR
      threshold: 2.0
      chi_square_min: 3.841
      min_cases: 3
    - name: ROR
      threshold: 2.0
      lower_ci_min: 1.0
      min_cases: 3
    - name: IC
      ic025_threshold: 0.0
      min_cases: 3
    - name: EBGM
      threshold: 2.0
      eb05_threshold: 2.0
      min_cases: 3
  batch:
    parallel: true
    chunk_size: 1000
    timeout_ms: 30000
  output:
    format: json
    include_ci: true
    include_chi_square: true
monitoring:
  enabled: true
  interval_seconds: 60
  alerts:
    slack_webhook: "https://hooks.slack.com/..."
    email_recipients:
      - safety@nexvigilant.com
      - regulatory@nexvigilant.com
  metrics:
    - name: signal_count
      type: gauge
    - name: processing_time
      type: histogram
      buckets: [10, 50, 100, 500, 1000]
"#;
    group.bench_function("complex_production_config", |b| {
        b.iter(|| parse_yaml(black_box(complex_yaml)));
    });

    // Deeply nested (10 levels)
    let deep_yaml = generate_deep_yaml(10);
    group.bench_function("deep_10_levels", |b| {
        b.iter(|| parse_yaml(black_box(&deep_yaml)));
    });

    // Deeply nested (20 levels)
    let deep_yaml_20 = generate_deep_yaml(20);
    group.bench_function("deep_20_levels", |b| {
        b.iter(|| parse_yaml(black_box(&deep_yaml_20)));
    });

    group.finish();
}

/// Generate a nested YAML document with given breadth and depth.
fn generate_nested_yaml(depth: usize, breadth: usize) -> String {
    fn build_level(depth: usize, breadth: usize, indent: usize) -> String {
        if depth == 0 {
            return format!("{}value: \"leaf\"", " ".repeat(indent));
        }
        let mut result = String::new();
        for i in 0..breadth {
            result.push_str(&format!("{}key_{}:\n", " ".repeat(indent), i));
            result.push_str(&build_level(depth - 1, breadth, indent + 2));
            result.push('\n');
        }
        result
    }
    build_level(depth, breadth, 0)
}

/// Generate a YAML document with an array of N items.
fn generate_array_yaml(n: usize) -> String {
    let mut yaml = String::from("items:\n");
    for i in 0..n {
        yaml.push_str(&format!(
            "  - id: {}\n    name: \"item_{}\"\n    value: {}\n    active: {}\n",
            i,
            i,
            i * 10,
            i % 2 == 0
        ));
    }
    yaml
}

/// Generate a deeply nested YAML document.
fn generate_deep_yaml(depth: usize) -> String {
    let mut yaml = String::new();
    for i in 0..depth {
        yaml.push_str(&format!("{}level_{}:\n", "  ".repeat(i), i));
    }
    yaml.push_str(&format!("{}value: \"deep_leaf\"", "  ".repeat(depth)));
    yaml
}

// ============================================================================
// Criterion Configuration
// ============================================================================

criterion_group!(
    benches,
    bench_levenshtein_distance,
    bench_levenshtein_full,
    bench_fuzzy_search,
    bench_sha256,
    bench_yaml_parse,
);

criterion_main!(benches);
