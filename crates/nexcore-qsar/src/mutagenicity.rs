// Copyright © 2026 NexVigilant LLC. All Rights Reserved.

//! Rule-based mutagenicity prediction (Ames-test surrogate).
//!
//! ## Model rationale
//!
//! The Ames test detects point mutations caused by direct DNA reactivity or
//! generation of reactive metabolites.  Published QSAR models (e.g., Kazius
//! et al. 2005) show that **structural alerts** — substructures associated
//! with DNA reactivity — are the dominant predictor, outperforming global
//! physicochemical descriptors alone.
//!
//! Phase 1 rules:
//!
//! | Signal | Score contribution |
//! |--------|--------------------|
//! | Each structural alert | +0.25 |
//! | LogP > 3.0 **and** aromatic ring present | +0.15 |
//! | LogP > 5.0 | +0.10 |
//! | MW < 150 and zero alerts | −0.20 |
//!
//! Probability is clamped to [0, 1].

use nexcore_molcore::descriptor::Descriptors;

use crate::types::{PredictionResult, ToxClass};

/// Predict mutagenicity from molecular descriptors and a structural alert count.
///
/// `structural_alert_count` should be provided by the caller from a dedicated
/// structural-alert scanning module (e.g., a substructure search over the
/// Kazius/Benigni alert library).
///
/// # Examples
///
/// ```rust
/// use nexcore_molcore::descriptor::calculate_descriptors;
/// use nexcore_molcore::graph::MolGraph;
/// use nexcore_molcore::smiles::parse;
/// use nexcore_qsar::mutagenicity::predict_mutagenicity;
/// use nexcore_qsar::types::ToxClass;
///
/// // Benzene with three alerts → Positive.
/// let mol = parse("c1ccccc1").unwrap_or_default();
/// let g = MolGraph::from_molecule(mol);
/// let d = calculate_descriptors(&g);
/// let result = predict_mutagenicity(&d, 3);
/// assert_eq!(result.classification, ToxClass::Positive);
/// ```
#[must_use]
pub fn predict_mutagenicity(
    descriptors: &Descriptors,
    structural_alert_count: usize,
) -> PredictionResult {
    let mut score = 0.0_f64;

    // Structural alerts are the strongest signal.
    score += (structural_alert_count as f64) * 0.25;

    // High LogP combined with an aromatic ring increases electrophilic risk.
    if descriptors.logp > 3.0 && descriptors.num_aromatic_rings > 0 {
        score += 0.15;
    }

    // Very high lipophilicity is an independent risk factor.
    if descriptors.logp > 5.0 {
        score += 0.10;
    }

    // Small, featureless molecules are rarely mutagenic.
    if descriptors.molecular_weight < 150.0 && structural_alert_count == 0 {
        score -= 0.20;
    }

    let probability = score.clamp(0.0, 1.0);
    let classification = classify(probability);

    // Higher confidence when structural alerts drive the prediction.
    let confidence = if structural_alert_count > 0 { 0.75 } else { 0.50 };

    PredictionResult {
        probability,
        classification,
        confidence,
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
