// Copyright © 2026 NexVigilant LLC. All Rights Reserved.

//! Rule-based hepatotoxicity (drug-induced liver injury) prediction.
//!
//! ## Model rationale
//!
//! Drug-induced liver injury (DILI) is a leading cause of post-market drug
//! withdrawal.  The dominant physicochemical risk factors identified in
//! published literature (Chen et al. 2013; Xu et al. 2015) are high
//! lipophilicity (LogP) and the formation of reactive metabolites.
//!
//! Phase 1 rules:
//!
//! | Signal | Score contribution |
//! |--------|--------------------|
//! | LogP > 3.0 | +0.15 |
//! | LogP > 5.0 | +0.15 (cumulative) |
//! | TPSA < 75 Å² (and > 0) | +0.10 |
//! | MW > 500 Da | +0.10 |
//! | Each reactive metabolite alert | +0.20 |
//!
//! Probability is clamped to [0, 1].  The in-domain flag is set to `false`
//! for very large molecules (MW > 1000 Da) which are outside the training
//! space of most published DILI models.

use nexcore_molcore::descriptor::Descriptors;

use crate::types::{PredictionResult, ToxClass};

/// Predict hepatotoxicity from molecular descriptors and reactive metabolite alert count.
///
/// `reactive_metabolite_alerts` should be provided by the caller from a
/// structural-alert scan focused on reactive metabolite-forming substructures
/// (e.g., Michael acceptors, acyl glucuronides, epoxides).
///
/// # Examples
///
/// ```rust
/// use nexcore_molcore::descriptor::calculate_descriptors;
/// use nexcore_molcore::graph::MolGraph;
/// use nexcore_molcore::smiles::parse;
/// use nexcore_qsar::hepatotoxicity::predict_hepatotoxicity;
/// use nexcore_qsar::types::ToxClass;
///
/// // Small molecule, no alerts → Negative.
/// let mol = parse("CCO").unwrap_or_default();
/// let g = MolGraph::from_molecule(mol);
/// let d = calculate_descriptors(&g);
/// let result = predict_hepatotoxicity(&d, 0);
/// assert_eq!(result.classification, ToxClass::Negative);
/// ```
#[must_use]
pub fn predict_hepatotoxicity(
    descriptors: &Descriptors,
    reactive_metabolite_alerts: usize,
) -> PredictionResult {
    let mut score = 0.0_f64;

    // Lipophilicity — primary driver of liver accumulation.
    if descriptors.logp > 3.0 {
        score += 0.15;
    }
    if descriptors.logp > 5.0 {
        score += 0.15;
    }

    // Low TPSA → high membrane permeability → increased hepatocyte uptake.
    if descriptors.tpsa < 75.0 && descriptors.tpsa > 0.0 {
        score += 0.10;
    }

    // High MW → potential for saturation of metabolic pathways.
    if descriptors.molecular_weight > 500.0 {
        score += 0.10;
    }

    // Reactive metabolite alerts are strong DILI predictors.
    score += (reactive_metabolite_alerts as f64) * 0.20;

    let probability = score.clamp(0.0, 1.0);
    let classification = classify(probability);

    // Rule-based models have inherently limited confidence for DILI.
    PredictionResult {
        probability,
        classification,
        confidence: 0.45,
        in_domain: descriptors.molecular_weight < 1000.0,
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
