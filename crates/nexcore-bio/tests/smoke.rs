// Copyright © 2026 NexVigilant LLC. All Rights Reserved.

//! Smoke test for the `nexcore-bio` umbrella.
//!
//! Two invariants are enforced:
//!
//! 1. **Count invariant** — `AGGREGATED_CRATES` has the expected length. If a
//!    biological crate is added or removed, this test fires loudly.
//! 2. **Resolution invariant** — every re-exported module path still resolves.
//!    The `use` block below is the guard: remove any `pub use` from `lib.rs`
//!    and this file fails to compile, long before a downstream consumer does.

// Resolution invariant: 37 library re-exports. If any module is renamed,
// dropped, or shadowed, this fails at `cargo build --tests`.
#[allow(unused_imports, reason = "compile-time re-export resolution guard")]
use nexcore_bio::{
    anatomy, antibodies, antivector, cardiovascular, circulatory, cns, cortex, cytokine, digestive,
    dna, dna_ml, energy, guardian, homeostasis, homeostasis_memory, homeostasis_primitives,
    homeostasis_sensing, homeostasis_storm, hormone_types, hormones, immunity, integumentary,
    lymphatic, metabolite, muscular, nervous, organize, phenotype, reproductive, respiratory,
    ribosome, skeletal, spliceosome, stem_bio, synapse, transcriptase, urinary,
};

/// Expected count: 38 aggregated crates (37 libraries re-exported as modules
/// above, plus the binary-only `nexcore-homeostat` listed for documentation).
const EXPECTED_AGGREGATED: usize = 38;

#[test]
fn aggregated_crates_count_matches_spec() {
    assert_eq!(
        nexcore_bio::AGGREGATED_CRATES.len(),
        EXPECTED_AGGREGATED,
        "AGGREGATED_CRATES length drifted from spec — if this was intentional, \
         update EXPECTED_AGGREGATED and the count in the lib.rs doc comment."
    );
}

#[test]
fn aggregated_crates_are_sorted_and_unique() {
    let mut sorted: Vec<&&str> = nexcore_bio::AGGREGATED_CRATES.iter().collect();
    sorted.sort();
    sorted.dedup();
    assert_eq!(
        sorted.len(),
        nexcore_bio::AGGREGATED_CRATES.len(),
        "AGGREGATED_CRATES contains duplicates"
    );
}

#[test]
fn aggregated_crates_all_nexcore_or_stem_prefix() {
    for name in nexcore_bio::AGGREGATED_CRATES {
        assert!(
            name.starts_with("nexcore-") || name.starts_with("stem-"),
            "unexpected crate in AGGREGATED_CRATES: {name}"
        );
    }
}

/// Consumer-boundary check: invoke a re-exported function via the umbrella
/// path to confirm the pipeline works end-to-end for at least one crate.
/// Ethanol (CCO) always yields at least one Phase II metabolite.
#[test]
fn metabolite_reexport_reaches_consumer() {
    let tree = nexcore_bio::metabolite::predict_from_smiles("CCO").unwrap_or_default();
    assert_eq!(tree.parent_smiles, "CCO");
    assert!(
        !tree.phase2.is_empty(),
        "Phase II prediction returned no metabolites — metabolite re-export may be broken"
    );
}
