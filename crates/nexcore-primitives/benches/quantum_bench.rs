use criterion::{Criterion, black_box, criterion_group, criterion_main};
use nexcore_primitives::quantum::{
    Amplitude, Decoherence, Entanglement, Interference, Phase, Qubit, Superposition,
};

fn amplitude_energy_bench(c: &mut Criterion) {
    let a = Amplitude::new(2.0, 50.0);
    c.bench_function("amplitude_energy", |b| b.iter(|| black_box(&a).energy()));
}

fn phase_interference_bench(c: &mut Criterion) {
    let p = Phase::new(1.5, 0.0);
    c.bench_function("phase_interference_factor", |b| {
        b.iter(|| black_box(&p).interference_factor())
    });
}

fn superposition_entropy_bench(c: &mut Criterion) {
    let weights = vec![0.1; 10]; // 10 states
    let labels = (0..10).map(|i| i.to_string()).collect();
    let s = Superposition::new(weights, labels);
    c.bench_function("superposition_entropy_10_states", |b| {
        b.iter(|| black_box(&s).entropy())
    });
}

fn interference_resultant_bench(c: &mut Criterion) {
    let amplitudes = vec![1.0; 100];
    let phases: Vec<f64> = (0..100).map(|i| (i as f64) * 0.1).collect();
    let i = Interference::new(amplitudes, phases);
    c.bench_function("interference_resultant_100_components", |b| {
        b.iter(|| black_box(&i).resultant_amplitude())
    });
}

fn entanglement_entropy_bench(c: &mut Criterion) {
    let e = Entanglement::new("A", "B", 0.7);
    c.bench_function("entanglement_entropy", |b| {
        b.iter(|| black_box(&e).entropy())
    });
}

fn decoherence_decay_bench(c: &mut Criterion) {
    let d = Decoherence::new(10.0, "T2");
    c.bench_function("decoherence_at_t", |b| {
        b.iter(|| black_box(&d).coherence_at(black_box(5.0)))
    });
}

fn qubit_normalization_bench(c: &mut Criterion) {
    let q = Qubit::new(0.6, 0.8);
    c.bench_function("qubit_is_normalized", |b| {
        b.iter(|| black_box(&q).is_normalized(black_box(1e-9)))
    });
}

criterion_group!(
    benches,
    amplitude_energy_bench,
    phase_interference_bench,
    superposition_entropy_bench,
    interference_resultant_bench,
    entanglement_entropy_bench,
    decoherence_decay_bench,
    qubit_normalization_bench
);
criterion_main!(benches);
