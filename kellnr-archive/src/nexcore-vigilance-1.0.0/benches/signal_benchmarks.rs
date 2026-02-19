//! Criterion benchmarks for signal detection algorithms.
//!
//! Comprehensive pharmacovigilance signal detection benchmarks covering:
//! - Individual algorithm performance (PRR, ROR, IC, EBGM)
//! - Complete evaluation (all 4 algorithms together)
//! - Batch processing (sequential vs parallel)
//! - Parallel optimization across 10,000+ contingency tables
//! - Algorithm comparisons across signal scenarios
//! - Chi-square p-value batch computation
//! - EBGM with custom priors
//! - Contingency table building
//!
//! Run with: cargo bench --package nexcore-vigilance
//! For HTML reports: cargo bench --package nexcore-vigilance -- --plotting-backend gnuplot

use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use nexcore_vigilance::pv::signals::{
    BatchContingencyTables, batch_chi_square_p_values, batch_chi_square_p_values_sequential,
    batch_complete_parallel, batch_ebgm_parallel, batch_ic_parallel, batch_prr_parallel,
    batch_ror_parallel, build_contingency_tables, evaluate_signal_complete,
};
use nexcore_vigilance::pv::{
    ContingencyTable, SignalCriteria, calculate_ebgm, calculate_ic, calculate_prr, calculate_ror,
};

/// Standard test case: strong signal
fn strong_signal_table() -> ContingencyTable {
    ContingencyTable::new(10, 90, 100, 9800)
}

/// Test case: no signal (close to independence)
fn no_signal_table() -> ContingencyTable {
    ContingencyTable::new(100, 900, 1000, 8000)
}

/// Test case: sparse data (few cases)
fn sparse_table() -> ContingencyTable {
    ContingencyTable::new(3, 97, 300, 9600)
}

/// Test case: large database
fn large_table() -> ContingencyTable {
    ContingencyTable::new(500, 49_500, 10_000, 940_000)
}

fn bench_prr(c: &mut Criterion) {
    let mut group = c.benchmark_group("PRR");
    let criteria = SignalCriteria::evans();

    for (name, table) in [
        ("strong_signal", strong_signal_table()),
        ("no_signal", no_signal_table()),
        ("sparse", sparse_table()),
        ("large_db", large_table()),
    ] {
        group.throughput(Throughput::Elements(1));
        group.bench_with_input(BenchmarkId::new("calculate", name), &table, |b, table| {
            b.iter(|| calculate_prr(black_box(table), black_box(&criteria)));
        });
    }
    group.finish();
}

fn bench_ror(c: &mut Criterion) {
    let mut group = c.benchmark_group("ROR");
    let criteria = SignalCriteria::evans();

    for (name, table) in [
        ("strong_signal", strong_signal_table()),
        ("no_signal", no_signal_table()),
        ("sparse", sparse_table()),
        ("large_db", large_table()),
    ] {
        group.throughput(Throughput::Elements(1));
        group.bench_with_input(BenchmarkId::new("calculate", name), &table, |b, table| {
            b.iter(|| calculate_ror(black_box(table), black_box(&criteria)));
        });
    }
    group.finish();
}

fn bench_ic(c: &mut Criterion) {
    let mut group = c.benchmark_group("IC");
    let criteria = SignalCriteria::evans();

    for (name, table) in [
        ("strong_signal", strong_signal_table()),
        ("no_signal", no_signal_table()),
        ("sparse", sparse_table()),
        ("large_db", large_table()),
    ] {
        group.throughput(Throughput::Elements(1));
        group.bench_with_input(BenchmarkId::new("calculate", name), &table, |b, table| {
            b.iter(|| calculate_ic(black_box(table), black_box(&criteria)));
        });
    }
    group.finish();
}

fn bench_ebgm(c: &mut Criterion) {
    let mut group = c.benchmark_group("EBGM");
    let criteria = SignalCriteria::evans();

    for (name, table) in [
        ("strong_signal", strong_signal_table()),
        ("no_signal", no_signal_table()),
        ("sparse", sparse_table()),
        ("large_db", large_table()),
    ] {
        group.throughput(Throughput::Elements(1));
        group.bench_with_input(BenchmarkId::new("calculate", name), &table, |b, table| {
            b.iter(|| calculate_ebgm(black_box(table), black_box(&criteria)));
        });
    }
    group.finish();
}

fn bench_complete_evaluation(c: &mut Criterion) {
    let mut group = c.benchmark_group("Complete");
    let criteria = SignalCriteria::evans();

    for (name, table) in [
        ("strong_signal", strong_signal_table()),
        ("no_signal", no_signal_table()),
        ("sparse", sparse_table()),
        ("large_db", large_table()),
    ] {
        group.throughput(Throughput::Elements(4)); // 4 algorithms
        group.bench_with_input(BenchmarkId::new("all_methods", name), &table, |b, table| {
            b.iter(|| evaluate_signal_complete(black_box(table), &criteria));
        });
    }
    group.finish();
}

fn bench_batch_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("Batch");

    // Simulate batch processing of multiple drug-event pairs
    let tables: Vec<ContingencyTable> = (0..1000)
        .map(|i| {
            ContingencyTable::new(
                ((i % 50) as u64) + 1,
                100 - (((i % 50) as u64) + 1),
                (((i * 3) % 500) as u64) + 50,
                10000 - (((i * 3) % 500) as u64) - 50,
            )
        })
        .collect();

    group.throughput(Throughput::Elements(1000));

    group.bench_function("prr_1000", |b| {
        b.iter(|| {
            for table in &tables {
                // Benchmarking calculation only, not error handling
                #[allow(unused_results)]
                {
                    calculate_prr(black_box(table), black_box(&SignalCriteria::evans()));
                }
            }
        });
    });

    group.bench_function("ror_1000", |b| {
        b.iter(|| {
            for table in &tables {
                #[allow(unused_results)]
                {
                    calculate_ror(black_box(table), black_box(&SignalCriteria::evans()));
                }
            }
        });
    });

    group.bench_function("ic_1000", |b| {
        b.iter(|| {
            for table in &tables {
                #[allow(unused_results)]
                {
                    calculate_ic(black_box(table), black_box(&SignalCriteria::evans()));
                }
            }
        });
    });

    group.bench_function("ebgm_1000", |b| {
        b.iter(|| {
            for table in &tables {
                #[allow(unused_results)]
                {
                    calculate_ebgm(black_box(table), black_box(&SignalCriteria::evans()));
                }
            }
        });
    });

    group.bench_function("complete_1000", |b| {
        b.iter(|| {
            for table in &tables {
                // evaluate_signal_complete returns CompleteSignalResult (not Result)
                // and is #[must_use], so we capture the value to silence compiler
                let _result = evaluate_signal_complete(black_box(table), &SignalCriteria::evans());
            }
        });
    });

    group.finish();
}

fn bench_parallel_batch_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("Parallel_Batch");

    // Generate 10,000 contingency tables for realistic workload
    let n = 10_000;
    let a: Vec<u64> = (0..n).map(|i| ((i % 50) as u64) + 1).collect();
    let b: Vec<u64> = (0..n).map(|i| 100 - (((i % 50) as u64) + 1)).collect();
    let c_vals: Vec<u64> = (0..n).map(|i| (((i * 3) % 500) as u64) + 50).collect();
    let d: Vec<u64> = (0..n)
        .map(|i| 10000 - (((i * 3) % 500) as u64) - 50)
        .collect();

    let batch = BatchContingencyTables::new(a.clone(), b.clone(), c_vals.clone(), d.clone());

    group.throughput(Throughput::Elements(n as u64));

    // Sequential baseline for PRR
    let tables: Vec<ContingencyTable> = (0..n as usize)
        .map(|i| ContingencyTable::new(batch.a[i], batch.b[i], batch.c[i], batch.d[i]))
        .collect();

    group.bench_function("prr_sequential_10k", |bench| {
        bench.iter(|| {
            for table in &tables {
                #[allow(unused_results)]
                {
                    calculate_prr(black_box(table), black_box(&SignalCriteria::evans()));
                }
            }
        });
    });

    // Parallel PRR
    group.bench_function("prr_parallel_10k", |bench| {
        bench.iter(|| {
            #[allow(unused_results)]
            {
                batch_prr_parallel(black_box(&batch));
            }
        });
    });

    // Parallel ROR
    group.bench_function("ror_parallel_10k", |bench| {
        bench.iter(|| {
            #[allow(unused_results)]
            {
                batch_ror_parallel(black_box(&batch));
            }
        });
    });

    // Parallel IC
    group.bench_function("ic_parallel_10k", |bench| {
        bench.iter(|| {
            #[allow(unused_results)]
            {
                batch_ic_parallel(black_box(&batch));
            }
        });
    });

    // Parallel EBGM
    group.bench_function("ebgm_parallel_10k", |bench| {
        bench.iter(|| {
            #[allow(unused_results)]
            {
                batch_ebgm_parallel(black_box(&batch));
            }
        });
    });

    // Complete parallel (all 4 algorithms)
    group.bench_function("complete_parallel_10k", |bench| {
        bench.iter(|| {
            #[allow(unused_results)]
            {
                batch_complete_parallel(black_box(&batch));
            }
        });
    });

    group.finish();
}

fn bench_algorithms_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("Algorithm_Comparison");
    let criteria = SignalCriteria::evans();

    // Create a single representative table for comparison
    let table = strong_signal_table();

    group.throughput(Throughput::Elements(1));

    group.bench_function("prr_strong_signal", |b| {
        b.iter(|| calculate_prr(black_box(&table), black_box(&criteria)));
    });

    group.bench_function("ror_strong_signal", |b| {
        b.iter(|| calculate_ror(black_box(&table), black_box(&criteria)));
    });

    group.bench_function("ic_strong_signal", |b| {
        b.iter(|| calculate_ic(black_box(&table), black_box(&criteria)));
    });

    group.bench_function("ebgm_strong_signal", |b| {
        b.iter(|| calculate_ebgm(black_box(&table), black_box(&criteria)));
    });

    group.bench_function("complete_strong_signal", |b| {
        b.iter(|| evaluate_signal_complete(black_box(&table), &criteria));
    });

    group.finish();
}

fn bench_chi_square_p_values(c: &mut Criterion) {
    let mut group = c.benchmark_group("Chi_Square_P_Values");

    // Generate test data: chi-square values from 0.1 to 100
    let sizes = [100, 1_000, 10_000, 100_000];

    for size in sizes {
        let chi_squares: Vec<f64> = (0..size)
            .map(|i| 0.1 + ((i as f64) / (size as f64)) * 99.9)
            .collect();

        group.throughput(Throughput::Elements(size as u64));

        // Sequential version (baseline)
        group.bench_with_input(
            BenchmarkId::new("sequential", size),
            &chi_squares,
            |bench, chi_squares| {
                bench.iter(|| batch_chi_square_p_values_sequential(black_box(chi_squares)));
            },
        );

        // Parallel version (Rayon)
        group.bench_with_input(
            BenchmarkId::new("parallel", size),
            &chi_squares,
            |bench, chi_squares| {
                bench.iter(|| batch_chi_square_p_values(black_box(chi_squares)));
            },
        );
    }

    group.finish();
}

fn bench_contingency_table_building(c: &mut Criterion) {
    let mut group = c.benchmark_group("Table_Building");

    let n = 100_000;
    let drug_counts: Vec<u32> = (0..n).map(|i| ((i % 1000) as u32) + 100).collect();
    let event_counts: Vec<u32> = (0..n).map(|i| ((i % 500) as u32) + 50).collect();
    let drug_event_counts: Vec<u32> = (0..n).map(|i| (i % 50) as u32).collect();
    let total = 1_000_000;

    group.throughput(Throughput::Elements(n as u64));

    group.bench_function("build_100k", |bench| {
        bench.iter(|| {
            build_contingency_tables(
                black_box(&drug_counts),
                black_box(&event_counts),
                black_box(&drug_event_counts),
                black_box(total),
            )
        });
    });

    group.finish();
}

fn bench_ebgm_custom_priors(c: &mut Criterion) {
    let mut group = c.benchmark_group("EBGM_Custom_Priors");

    // Generate realistic test data for 10,000 contingency tables
    let size = 10_000;
    let a: Vec<u64> = (0..size).map(|i| ((i % 20) as u64) + 1).collect();
    let b: Vec<u64> = (0..size).map(|i| ((i % 100) as u64) + 10).collect();
    let c: Vec<u64> = (0..size).map(|i| ((i % 200) as u64) + 50).collect();
    let d: Vec<u64> = (0..size)
        .map(|i| {
            let a_val = a[i];
            let b_val = b[i];
            let c_val = c[i];
            (10000_u64)
                .saturating_sub(a_val)
                .saturating_sub(b_val)
                .saturating_sub(c_val)
        })
        .collect();

    let batch = BatchContingencyTables::new(a, b, c, d);

    group.throughput(Throughput::Elements(size as u64));

    // Standard EBGM (default DuMouchel priors)
    group.bench_function("default_priors_10k", |bench| {
        bench.iter(|| {
            #[allow(unused_results)]
            {
                batch_ebgm_parallel(black_box(&batch));
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_prr,
    bench_ror,
    bench_ic,
    bench_ebgm,
    bench_complete_evaluation,
    bench_batch_processing,
    bench_parallel_batch_processing,
    bench_algorithms_comparison,
    bench_chi_square_p_values,
    bench_contingency_table_building,
    bench_ebgm_custom_priors
);

criterion_main!(benches);
