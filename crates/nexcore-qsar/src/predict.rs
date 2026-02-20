// Copyright © 2026 NexVigilant LLC. All Rights Reserved.

//! Top-level prediction API for full toxicity profiling.
//!
//! This module composes the individual endpoint models
//! (`mutagenicity`, `hepatotoxicity`, `cardiotoxicity`) with the
//! applicability domain assessment to produce a [`ToxProfile`].
//!
//! ## Entry points
//!
//! | Function | Use when |
//! |----------|----------|
//! | [`predict_from_smiles`] | Input is a SMILES string |
//! | [`predict_toxicity`] | Input is a pre-built [`MolGraph`] |
//! | [`predict_from_descriptors`] | Descriptors already computed |

use nexcore_molcore::descriptor::{calculate_descriptors, Descriptors};
use nexcore_molcore::graph::MolGraph;
use nexcore_molcore::smiles::parse;

use crate::applicability::assess_domain;
use crate::cardiotoxicity::predict_cardiotoxicity;
use crate::error::{QsarError, QsarResult};
use crate::hepatotoxicity::predict_hepatotoxicity;
use crate::mutagenicity::predict_mutagenicity;
use crate::types::{RiskLevel, ToxProfile};

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Predict the full toxicity profile for a molecule provided as a SMILES string.
///
/// Returns `Err(QsarError::SmilesParse)` if the SMILES cannot be parsed.
///
/// # Examples
///
/// ```rust
/// use nexcore_qsar::predict::predict_from_smiles;
/// use nexcore_qsar::types::{RiskLevel, ToxClass};
///
/// let profile = predict_from_smiles("CCO", 0, 0).unwrap_or_default();
/// assert_eq!(profile.mutagenicity.classification, ToxClass::Negative);
/// ```
pub fn predict_from_smiles(
    smiles: &str,
    structural_alert_count: usize,
    reactive_metabolite_alerts: usize,
) -> QsarResult<ToxProfile> {
    let mol = parse(smiles).map_err(|e| QsarError::SmilesParse(e.to_string()))?;
    let graph = MolGraph::from_molecule(mol);
    Ok(predict_toxicity(
        &graph,
        structural_alert_count,
        reactive_metabolite_alerts,
    ))
}

/// Predict the full toxicity profile for a molecule represented as a [`MolGraph`].
///
/// Descriptors are computed internally; use [`predict_from_descriptors`] when
/// they have already been calculated to avoid redundant work.
///
/// # Examples
///
/// ```rust
/// use nexcore_molcore::graph::MolGraph;
/// use nexcore_molcore::smiles::parse;
/// use nexcore_qsar::predict::predict_toxicity;
///
/// let mol = parse("c1ccccc1").unwrap_or_default();
/// let g = MolGraph::from_molecule(mol);
/// let profile = predict_toxicity(&g, 0, 0);
/// assert!(!profile.mutagenicity.model_version.is_empty());
/// ```
#[must_use]
pub fn predict_toxicity(
    graph: &MolGraph,
    structural_alert_count: usize,
    reactive_metabolite_alerts: usize,
) -> ToxProfile {
    let descriptors = calculate_descriptors(graph);
    predict_from_descriptors(&descriptors, structural_alert_count, reactive_metabolite_alerts)
}

/// Predict the full toxicity profile from pre-computed molecular descriptors.
///
/// Use this when descriptors have already been calculated (e.g., for batch
/// screening where descriptors are reused across multiple models).
///
/// # Examples
///
/// ```rust
/// use nexcore_molcore::descriptor::calculate_descriptors;
/// use nexcore_molcore::graph::MolGraph;
/// use nexcore_molcore::smiles::parse;
/// use nexcore_qsar::predict::predict_from_descriptors;
///
/// let mol = parse("CCO").unwrap_or_default();
/// let g = MolGraph::from_molecule(mol);
/// let d = calculate_descriptors(&g);
/// let profile = predict_from_descriptors(&d, 0, 0);
/// assert!(profile.mutagenicity.probability >= 0.0);
/// ```
#[must_use]
pub fn predict_from_descriptors(
    descriptors: &Descriptors,
    structural_alert_count: usize,
    reactive_metabolite_alerts: usize,
) -> ToxProfile {
    let mutagenicity = predict_mutagenicity(descriptors, structural_alert_count);
    let hepatotoxicity = predict_hepatotoxicity(descriptors, reactive_metabolite_alerts);
    let cardiotoxicity = predict_cardiotoxicity(descriptors);
    let applicability_domain = assess_domain(descriptors);

    let overall_risk = compute_overall_risk(
        mutagenicity.probability,
        hepatotoxicity.probability,
        cardiotoxicity.probability,
    );

    ToxProfile {
        mutagenicity,
        hepatotoxicity,
        cardiotoxicity,
        off_target_binding: Vec::new(), // Phase 2
        applicability_domain,
        overall_risk,
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Derive the overall risk level from the worst-case endpoint probability.
fn compute_overall_risk(mutagen: f64, hepato: f64, cardio: f64) -> RiskLevel {
    let max_prob = mutagen.max(hepato).max(cardio);
    if max_prob >= 0.7 {
        RiskLevel::VeryHigh
    } else if max_prob >= 0.5 {
        RiskLevel::High
    } else if max_prob >= 0.3 {
        RiskLevel::Medium
    } else {
        RiskLevel::Low
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{DomainStatus, ToxClass};

    #[test]
    fn test_ethanol_low_risk() {
        let profile = predict_from_smiles("CCO", 0, 0).unwrap_or_default();
        // Ethanol: small, no alerts → Negative mutagenicity, Low overall risk.
        assert_eq!(profile.mutagenicity.classification, ToxClass::Negative);
        assert_eq!(profile.overall_risk, RiskLevel::Low);
    }

    #[test]
    fn test_structural_alerts_increase_mutagenicity_probability() {
        let no_alerts = predict_from_smiles("c1ccccc1", 0, 0).unwrap_or_default();
        let with_alerts = predict_from_smiles("c1ccccc1", 3, 0).unwrap_or_default();
        assert!(
            with_alerts.mutagenicity.probability > no_alerts.mutagenicity.probability,
            "three alerts should raise probability above zero-alert baseline"
        );
    }

    #[test]
    fn test_domain_assessment_ethanol_out_of_domain() {
        let profile = predict_from_smiles("CCO", 0, 0).unwrap_or_default();
        // Ethanol MW ≈ 46 Da < 100 Da → out-of-domain or borderline.
        let is_flagged = matches!(
            profile.applicability_domain,
            DomainStatus::OutOfDomain { .. } | DomainStatus::Borderline { .. }
        );
        assert!(is_flagged, "ethanol should be flagged as out-of-domain or borderline");
    }

    #[test]
    fn test_aspirin_in_domain() {
        let profile = predict_from_smiles("CC(=O)Oc1ccccc1C(=O)O", 0, 0).unwrap_or_default();
        // Aspirin MW ≈ 180 Da, LogP ≈ 1.2 → within all descriptor bounds.
        assert!(
            matches!(profile.applicability_domain, DomainStatus::InDomain { .. }),
            "aspirin should be in-domain"
        );
    }

    #[test]
    fn test_invalid_smiles_returns_error() {
        let result = predict_from_smiles("INVALID$$", 0, 0);
        assert!(result.is_err(), "invalid SMILES should return Err");
    }

    #[test]
    fn test_risk_levels_are_ordered() {
        assert!(RiskLevel::Low < RiskLevel::Medium);
        assert!(RiskLevel::Medium < RiskLevel::High);
        assert!(RiskLevel::High < RiskLevel::VeryHigh);
    }

    #[test]
    fn test_prediction_probabilities_are_bounded() {
        let profile = predict_from_smiles("c1ccccc1", 5, 3).unwrap_or_default();
        assert!(
            (0.0..=1.0).contains(&profile.mutagenicity.probability),
            "mutagenicity probability out of [0, 1]"
        );
        assert!(
            (0.0..=1.0).contains(&profile.hepatotoxicity.probability),
            "hepatotoxicity probability out of [0, 1]"
        );
        assert!(
            (0.0..=1.0).contains(&profile.cardiotoxicity.probability),
            "cardiotoxicity probability out of [0, 1]"
        );
    }

    #[test]
    fn test_model_version_is_non_empty() {
        let profile = predict_from_smiles("CCO", 0, 0).unwrap_or_default();
        assert!(
            !profile.mutagenicity.model_version.is_empty(),
            "model_version must be set"
        );
        assert!(
            !profile.hepatotoxicity.model_version.is_empty(),
            "model_version must be set"
        );
        assert!(
            !profile.cardiotoxicity.model_version.is_empty(),
            "model_version must be set"
        );
    }

    #[test]
    fn test_off_target_binding_empty_in_phase1() {
        let profile = predict_from_smiles("CCO", 0, 0).unwrap_or_default();
        assert!(
            profile.off_target_binding.is_empty(),
            "off-target binding should be empty in Phase 1"
        );
    }

    #[test]
    fn test_very_high_risk_with_many_alerts() {
        // 3 structural alerts → mutagenicity score = 0.75 → VeryHigh
        let profile = predict_from_smiles("c1ccccc1", 3, 0).unwrap_or_default();
        assert_eq!(profile.overall_risk, RiskLevel::VeryHigh);
    }
}
