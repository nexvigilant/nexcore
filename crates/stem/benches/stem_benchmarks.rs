//! STEM Benchmarks: Newtype overhead vs raw f64 operations
//!
//! Measures the zero-cost abstraction claim — STEM T2-P newtypes
//! should have no overhead vs raw f64 in release builds.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use stem::prelude::*;

// ============================================================================
// Core: Confidence operations
// ============================================================================

fn bench_confidence_create(c: &mut Criterion) {
    c.bench_function("confidence_create", |b| {
        b.iter(|| Confidence::new(black_box(0.85)));
    });
}

fn bench_confidence_combine(c: &mut Criterion) {
    let a = Confidence::new(0.8);
    let b = Confidence::new(0.9);
    c.bench_function("confidence_combine", |b2| {
        b2.iter(|| black_box(a).combine(black_box(b)));
    });
}

fn bench_raw_f64_clamp(c: &mut Criterion) {
    c.bench_function("raw_f64_clamp", |b| {
        b.iter(|| black_box(0.85_f64).clamp(0.0, 1.0));
    });
}

fn bench_raw_f64_multiply(c: &mut Criterion) {
    c.bench_function("raw_f64_multiply", |b| {
        b.iter(|| black_box(0.8_f64) * black_box(0.9_f64));
    });
}

// ============================================================================
// Chemistry: Ratio and Balance operations
// ============================================================================

fn bench_ratio_create(c: &mut Criterion) {
    c.bench_function("ratio_create", |b| {
        b.iter(|| Ratio::new(black_box(2.5)));
    });
}

fn bench_balance_create(c: &mut Criterion) {
    c.bench_function("balance_create", |b| {
        b.iter(|| Balance::new(Rate::new(black_box(2.0)), Rate::new(black_box(1.0))));
    });
}

fn bench_balance_equilibrium_check(c: &mut Criterion) {
    let balance = Balance::new(Rate::new(1.001), Rate::new(1.0));
    c.bench_function("balance_equilibrium_check", |b| {
        b.iter(|| black_box(&balance).is_equilibrium(black_box(0.01)));
    });
}

fn bench_raw_division(c: &mut Criterion) {
    c.bench_function("raw_f64_division", |b| {
        b.iter(|| {
            let fwd = black_box(2.0_f64);
            let rev = black_box(1.0_f64);
            if rev > 0.0 { fwd / rev } else { f64::INFINITY }
        });
    });
}

// ============================================================================
// Physics: Force and Acceleration
// ============================================================================

fn bench_acceleration_fma(c: &mut Criterion) {
    let force = Force::new(10.0);
    let mass = Mass::new(2.0);
    c.bench_function("acceleration_from_fma", |b| {
        b.iter(|| Acceleration::from_force_and_mass(black_box(force), black_box(mass)));
    });
}

fn bench_raw_fma(c: &mut Criterion) {
    c.bench_function("raw_fma", |b| {
        b.iter(|| {
            let f = black_box(10.0_f64);
            let m = black_box(2.0_f64);
            if m > 0.0 { f / m } else { 0.0 }
        });
    });
}

fn bench_quantity_conservation(c: &mut Criterion) {
    let q1 = Quantity::new(100.0);
    let q2 = Quantity::new(100.001);
    c.bench_function("quantity_conservation_check", |b| {
        b.iter(|| black_box(&q1).conserved_with(&black_box(q2), black_box(0.01)));
    });
}

fn bench_quantity_arithmetic(c: &mut Criterion) {
    let a = Quantity::new(100.0);
    let b = Quantity::new(50.0);
    c.bench_function("quantity_add_sub", |b2| {
        b2.iter(|| {
            let sum = black_box(a) + black_box(b);
            let diff = black_box(a) - black_box(b);
            (sum, diff)
        });
    });
}

// ============================================================================
// Math: Bounded operations
// ============================================================================

fn bench_bounded_in_bounds(c: &mut Criterion) {
    let b_val = Bounded::new(5.0, Some(0.0), Some(10.0));
    c.bench_function("bounded_in_bounds", |b| {
        b.iter(|| black_box(&b_val).in_bounds());
    });
}

fn bench_bounded_clamp(c: &mut Criterion) {
    let b_val = Bounded::new(15.0, Some(0.0), Some(10.0));
    c.bench_function("bounded_clamp", |b| {
        b.iter(|| black_box(&b_val).clamp());
    });
}

fn bench_raw_bounds_check(c: &mut Criterion) {
    c.bench_function("raw_bounds_check", |b| {
        b.iter(|| {
            let v = black_box(5.0_f64);
            v >= 0.0 && v <= 10.0
        });
    });
}

// ============================================================================
// Measured<T> overhead
// ============================================================================

fn bench_measured_create(c: &mut Criterion) {
    c.bench_function("measured_create", |b| {
        b.iter(|| Measured::new(black_box(42.0), Confidence::new(black_box(0.9))));
    });
}

fn bench_measured_map(c: &mut Criterion) {
    let m = Measured::new(42.0, Confidence::new(0.9));
    c.bench_function("measured_map", |b| {
        b.iter(|| black_box(m.clone()).map(|x| x * 2.0));
    });
}

// ============================================================================
// Derive macro newtype overhead
// ============================================================================

fn bench_fraction_create(c: &mut Criterion) {
    c.bench_function("fraction_create", |b| {
        b.iter(|| Fraction::new(black_box(0.75)));
    });
}

// ============================================================================
// Groups
// ============================================================================

criterion_group!(
    core_benches,
    bench_confidence_create,
    bench_confidence_combine,
    bench_raw_f64_clamp,
    bench_raw_f64_multiply,
);

criterion_group!(
    chem_benches,
    bench_ratio_create,
    bench_balance_create,
    bench_balance_equilibrium_check,
    bench_raw_division,
);

criterion_group!(
    phys_benches,
    bench_acceleration_fma,
    bench_raw_fma,
    bench_quantity_conservation,
    bench_quantity_arithmetic,
);

criterion_group!(
    math_benches,
    bench_bounded_in_bounds,
    bench_bounded_clamp,
    bench_raw_bounds_check,
);

criterion_group!(
    overhead_benches,
    bench_measured_create,
    bench_measured_map,
    bench_fraction_create,
);

criterion_main!(
    core_benches,
    chem_benches,
    phys_benches,
    math_benches,
    overhead_benches
);
