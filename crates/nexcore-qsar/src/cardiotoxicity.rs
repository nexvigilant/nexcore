// Copyright © 2026 NexVigilant LLC. All Rights Reserved.

//! Rule-based cardiotoxicity (hERG channel inhibition) prediction.
//!
//! ## Model rationale
//!
//! hERG (Kv11.1) potassium channel inhibition is the primary mechanism of
//! drug-induced QT prolongation and potentially fatal arrhythmias.  Published
//! hERG QSAR models (Aronov 2006; Sanguinetti & Tristani-Firouzi 2006) show
//! that lipophilicity, molecular planarity, and basicity are the dominant
//! physicochemical drivers.
//!
//! Phase 1 rules:
//!
//! | Signal | Score contribution |
//! |--------|--------------------|
//! | LogP > 3.0 | +0.20 |
//! | LogP > 4.5 | +0.15 (cumulative) |
//! | MW in hERG-sweet-spot 250–500 Da | +0.10 |
//! | ≥ 2 aromatic rings | +0.15 |
//! | TPSA < 75 Å² (and > 0) | +0.10 |
//!
//! Probability is clamped to [0, 1].

use nexcore_molcore::descriptor::Descriptors;

use crate::types::{PredictionResult, ToxClass};

/// Predict hERG-channel cardiotoxicity from molecular descriptors.
///
/// No structural alert count is accepted for this endpoint; descriptor-based
/// rules alone capture the dominant physicochemical signal in Phase 1.
///
/// # Examples
///
/// ```rust
/// use nexcore_molcore::descriptor::calculate_descriptors;
/// use nexcore_molcore::graph::MolGraph;
/// use nexcore_molcore::smiles::parse;
/// use nexcore_qsar::cardiotoxicity::predict_cardiotoxicity;
/// use nexcore_qsar::types::ToxClass;
///
/// // Ethanol — very low LogP, no rings → Negative.
/// let mol = parse("CCO").unwrap_or_default();
/// let g = MolGraph::from_molecule(mol);
/// let d = calculate_descriptors(&g);
/// let result = predict_cardiotoxicity(&d);
/// assert_eq!(result.classification, ToxClass::Negative);
/// ```
#[must_use]
pub fn predict_cardiotoxicity(descriptors: &Descriptors) -> PredictionResult {
    let mut score = 0.0_f64;

    // Lipophilicity — primary driver of hERG binding.
    if descriptors.logp > 3.0 {
        score += 0.20;
    }
    if descriptors.logp > 4.5 {
        score += 0.15;
    }

    // MW in the typical hERG-binder range (Aronov 2006).
    if descriptors.molecular_weight > 250.0 && descriptors.molecular_weight < 500.0 {
        score += 0.10;
    }

    // Multiple aromatic rings → planar scaffold → hERG channel access.
    if descriptors.num_aromatic_rings >= 2 {
        score += 0.15;
    }

    // Low TPSA → high cardiac membrane permeability.
    if descriptors.tpsa < 75.0 && descriptors.tpsa > 0.0 {
        score += 0.10;
    }

    let probability = score.clamp(0.0, 1.0);
    let classification = classify(probability);

    PredictionResult {
        probability,
        classification,
        confidence: 0.40,
        in_domain: true,
        model_version: "rule-based-v1".to_string(),
    }
}

fn classify(probability: f64) -> ToxClass {
    if probability >= 0.5 {
        ToxClass::Positive
    } else if probability >= 0.3 {
        ToxClass::Inconclusive
    } else {
        ToxClass::Negative
    }
}
