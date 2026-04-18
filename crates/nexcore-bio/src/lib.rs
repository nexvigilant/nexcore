// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # nexcore-bio — Biological Systems Aggregator
//!
//! Single umbrella crate that re-exports all NexCore biological crates as Rust
//! modules AND exposes them to Python through PyO3 as `nexcore_bio`.
//!
//! ## Two modes
//!
//! - **Rust library** (default): pull in any subsystem via
//!   `use nexcore_bio::{dna, cytokine, immunity};`
//! - **Python extension** (`--features python`): `import nexcore_bio` and
//!   access `nexcore_bio.dna`, `nexcore_bio.metabolite`, etc.
//!
//! ## Map of included crates
//!
//! | Layer          | Modules |
//! |----------------|---------|
//! | Organ systems  | `anatomy`, `cardiovascular`, `circulatory`, `cns`, `cortex`, `digestive`, `integumentary`, `lymphatic`, `muscular`, `nervous`, `reproductive`, `respiratory`, `skeletal`, `synapse`, `urinary` |
//! | Molecular      | `antibodies`, `antivector`, `dna`, `dna_ml`, `metabolite` |
//! | Signaling      | `cytokine`, `hormones`, `hormone_types` |
//! | Homeostasis    | `homeostasis`, `homeostasis_memory`, `homeostasis_primitives`, `homeostasis_sensing`, `homeostasis_storm`, `homeostat` |
//! | Immune         | `immunity` |
//! | Integrator     | `guardian` |
//! | Infrastructure | `organize`, `stem_bio` |
//!
//! Full mesh inside the organism is intentional: each submodule is reachable
//! without importing the upstream crate individually.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

// ============================================================================
// Rust re-exports — 31 biological crates as flat submodules
// ============================================================================

// Organ systems
pub use nexcore_anatomy as anatomy;
pub use nexcore_cardiovascular as cardiovascular;
pub use nexcore_circulatory as circulatory;
pub use nexcore_cns as cns;
pub use nexcore_cortex as cortex;
pub use nexcore_digestive as digestive;
pub use nexcore_integumentary as integumentary;
pub use nexcore_lymphatic as lymphatic;
pub use nexcore_muscular as muscular;
pub use nexcore_nervous as nervous;
pub use nexcore_reproductive as reproductive;
pub use nexcore_respiratory as respiratory;
pub use nexcore_skeletal as skeletal;
pub use nexcore_synapse as synapse;
pub use nexcore_urinary as urinary;

// Molecular / adaptive defense
pub use nexcore_antibodies as antibodies;
pub use nexcore_antivector as antivector;
pub use nexcore_dna as dna;
pub use nexcore_dna_ml as dna_ml;
pub use nexcore_energy as energy;
pub use nexcore_metabolite as metabolite;
pub use nexcore_phenotype as phenotype;
pub use nexcore_ribosome as ribosome;
pub use nexcore_spliceosome as spliceosome;
pub use nexcore_transcriptase as transcriptase;

// Signaling
pub use nexcore_cytokine as cytokine;
pub use nexcore_hormone_types as hormone_types;
pub use nexcore_hormones as hormones;

// Homeostasis
pub use nexcore_homeostasis as homeostasis;
pub use nexcore_homeostasis_memory as homeostasis_memory;
pub use nexcore_homeostasis_primitives as homeostasis_primitives;
pub use nexcore_homeostasis_sensing as homeostasis_sensing;
pub use nexcore_homeostasis_storm as homeostasis_storm;
// nexcore-homeostat is a standalone binary (no library target), so it is not
// re-exported as a Rust module here — it still appears in AGGREGATED_CRATES
// and its binary is reachable through cargo.

// Immune + integrator
pub use nexcore_guardian_engine as guardian;
pub use nexcore_immunity as immunity;

// Infrastructure
pub use nexcore_organize as organize;
pub use stem_bio;

/// The crates this umbrella aggregates, as a const slice for introspection.
pub const AGGREGATED_CRATES: &[&str] = &[
    "nexcore-anatomy",
    "nexcore-antibodies",
    "nexcore-antivector",
    "nexcore-cardiovascular",
    "nexcore-circulatory",
    "nexcore-cns",
    "nexcore-cortex",
    "nexcore-cytokine",
    "nexcore-digestive",
    "nexcore-dna",
    "nexcore-dna-ml",
    "nexcore-energy",
    "nexcore-guardian-engine",
    "nexcore-homeostasis",
    "nexcore-homeostasis-memory",
    "nexcore-homeostasis-primitives",
    "nexcore-homeostasis-sensing",
    "nexcore-homeostasis-storm",
    "nexcore-homeostat",
    "nexcore-hormone-types",
    "nexcore-hormones",
    "nexcore-immunity",
    "nexcore-integumentary",
    "nexcore-lymphatic",
    "nexcore-metabolite",
    "nexcore-muscular",
    "nexcore-nervous",
    "nexcore-organize",
    "nexcore-phenotype",
    "nexcore-reproductive",
    "nexcore-respiratory",
    "nexcore-ribosome",
    "nexcore-skeletal",
    "nexcore-spliceosome",
    "nexcore-synapse",
    "nexcore-transcriptase",
    "nexcore-urinary",
    "stem-bio",
];

// ============================================================================
// Python bindings — only compiled with `--features python`
// ============================================================================

#[cfg(feature = "python")]
// PyO3 0.22's `#[pyfunction]` macro emits calls to the unsafe
// `unwrap_required_argument` helper without wrapping them in an explicit
// `unsafe` block. Under Rust Edition 2024 this trips `unsafe_op_in_unsafe_fn`.
// The upstream fix ships in pyo3 0.23+, but a 0.22→0.23+ migration breaks
// several binding APIs (`Bound<'_, _>`, `wrap_pyfunction`, module signatures).
// Suppress the macro-expansion noise module-locally until the workspace
// migrates to pyo3 ≥ 0.23. Tracking: see ~/TODO.md "nexcore-bio pyo3 upgrade".
#[allow(unsafe_op_in_unsafe_fn)]
mod py {
    use pyo3::prelude::*;

    /// Return the list of aggregated crate names — smoke test for the binding.
    #[pyfunction]
    fn aggregated_crates() -> Vec<String> {
        super::AGGREGATED_CRATES
            .iter()
            .map(|s| (*s).to_owned())
            .collect()
    }

    /// Predict Phase I/II metabolites for a SMILES string.
    /// Wraps `nexcore_metabolite::predict_from_smiles`; returns one line per
    /// metabolite: `"<Transformation> @ <site_description> (p=<probability>)"`.
    #[pyfunction]
    fn metabolite_predict(smiles: &str) -> PyResult<Vec<String>> {
        match nexcore_metabolite::predict_from_smiles(smiles) {
            Ok(tree) => {
                let mut out = Vec::new();
                for m in tree.phase1.iter().chain(tree.phase2.iter()) {
                    out.push(format!(
                        "{:?} @ {} (p={:.2})",
                        m.transformation, m.site_description, m.probability
                    ));
                }
                Ok(out)
            }
            Err(e) => Err(pyo3::exceptions::PyValueError::new_err(format!("{e:?}"))),
        }
    }

    // ------------------------------------------------------------------
    // DNA codec — nexcore_dna::storage
    // ------------------------------------------------------------------

    /// Encode a UTF-8 string into a DNA strand (ACGT alphabet).
    #[pyfunction]
    fn dna_encode_str(text: &str) -> String {
        nexcore_dna::storage::encode_str(text).to_string()
    }

    /// Decode an `ACGT` strand back to the original UTF-8 string.
    #[pyfunction]
    fn dna_decode_str(strand: &str) -> PyResult<String> {
        let parsed = nexcore_dna::types::Strand::parse(strand)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("{e:?}")))?;
        nexcore_dna::storage::decode_str(&parsed)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("{e:?}")))
    }

    // ------------------------------------------------------------------
    // CNS base-9 — nexcore_cns::digit
    // ------------------------------------------------------------------

    /// Return the conjugate partner of a CNS digit (0..=8). The conjugate
    /// of N is `8 - N`; each conjugate pair sums to the full scale.
    #[pyfunction]
    fn cns_conjugate(digit: u8) -> PyResult<u8> {
        use nexcore_cns::CnsDigit;
        let d = match digit {
            0 => CnsDigit::Void,
            1 => CnsDigit::I,
            2 => CnsDigit::II,
            3 => CnsDigit::III,
            4 => CnsDigit::IV,
            5 => CnsDigit::V,
            6 => CnsDigit::VI,
            7 => CnsDigit::VII,
            8 => CnsDigit::VIII,
            n => {
                return Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "digit {n} out of base-9 range 0..=8"
                )));
            }
        };
        Ok(nexcore_cns::conjugate_pair(d) as u8)
    }

    // ------------------------------------------------------------------
    // Immunity — nexcore_immunity::ImmunityScanner
    // ------------------------------------------------------------------

    /// Scan a code snippet for threat patterns using the default registry.
    /// Returns one line per detected threat: `"<severity> <antibody> @ <file?>"`.
    #[pyfunction]
    #[pyo3(signature = (content, file_path=None))]
    fn immunity_scan(content: &str, file_path: Option<&str>) -> PyResult<Vec<String>> {
        let registry = nexcore_immunity::load_default_registry()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("{e:?}")))?;
        let scanner = nexcore_immunity::ImmunityScanner::new(&registry)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("{e:?}")))?;
        let result = scanner.scan(content, file_path);
        Ok(result
            .threats
            .iter()
            .map(|t| format!("{:?} {}", t.severity, t.antibody_name))
            .collect())
    }

    /// Top-level `nexcore_bio` module. Subsystems are attached as submodules
    /// so `import nexcore_bio; nexcore_bio.metabolite_predict("CCO")` works,
    /// and `from nexcore_bio import metabolite_predict` works too.
    #[pymodule]
    fn nexcore_bio(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
        m.add_function(wrap_pyfunction!(aggregated_crates, m)?)?;
        m.add_function(wrap_pyfunction!(metabolite_predict, m)?)?;
        m.add_function(wrap_pyfunction!(dna_encode_str, m)?)?;
        m.add_function(wrap_pyfunction!(dna_decode_str, m)?)?;
        m.add_function(wrap_pyfunction!(cns_conjugate, m)?)?;
        m.add_function(wrap_pyfunction!(immunity_scan, m)?)?;
        m.add("__version__", env!("CARGO_PKG_VERSION"))?;
        m.add(
            "__doc__",
            "nexcore-bio: unified Python surface for all NexCore biological crates.",
        )?;
        Ok(())
    }
}
