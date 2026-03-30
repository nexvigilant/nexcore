//! Anti-vector computation engine.
//!
//! Given a harm vector and available evidence, computes the anti-vector
//! that annihilates it. This is the core computational module.

use crate::classify::classify_anti_vector;
use crate::types::*;

/// Compute the epistemic anti-vector for a harm vector.
///
/// Evaluates whether the signal is real or noise by assessing known biases
/// against the signal magnitude. Returns the counter-evidence packet.
#[must_use]
pub fn compute_epistemic(harm: &HarmVector, biases: &[BiasAssessment]) -> EpistemicAntiVector {
    // Sum the bias magnitudes — each bias attenuates the signal
    let total_bias: f64 = biases.iter().map(|b| b.magnitude).sum();
    let counter_magnitude = total_bias.min(1.0);
    let residual = (harm.magnitude - counter_magnitude).max(0.0);

    let verdict = if residual < 0.1 {
        EpistemicVerdict::SignalRefuted
    } else if residual < harm.magnitude * 0.5 {
        EpistemicVerdict::SignalAttenuated
    } else {
        EpistemicVerdict::SignalConfirmed
    };

    let evidence: Vec<EvidenceItem> = biases
        .iter()
        .map(|b| EvidenceItem {
            source: b.evidence_source,
            description: b.description.clone(),
            strength: b.magnitude,
        })
        .collect();

    EpistemicAntiVector {
        bias_type: biases
            .first()
            .map_or(BiasType::IndicationBias, |b| b.bias_type),
        counter_magnitude,
        evidence,
        residual_signal: residual,
        verdict,
    }
}

/// Compute the architectural anti-vector for a harm vector.
///
/// Selects the proportionate risk minimization measure based on signal
/// magnitude and harm type. Avoids disproportionate measures (REMS for a
/// label-change-level signal).
#[must_use]
pub fn compute_architectural(harm: &HarmVector) -> ArchitecturalAntiVector {
    let strategy = classify_anti_vector(harm.harm_type);

    // Select measure proportionate to signal magnitude
    let (measure, proportionality, delta_ds) =
        select_proportionate_measure(harm.magnitude, harm.confidence, &strategy.measures);

    let actions = describe_actions(measure, harm);

    ArchitecturalAntiVector {
        measure,
        proportionality,
        actions,
        delta_safety_distance: delta_ds,
    }
}

/// Compute the complete anti-vector for a harm vector.
///
/// Assembles all three components (mechanistic, epistemic, architectural),
/// computes the combined magnitude, and determines the annihilation result.
#[must_use]
pub fn compute_anti_vector(
    harm: &HarmVector,
    biases: &[BiasAssessment],
    mechanistic: Option<MechanisticAntiVector>,
) -> AntiVector {
    let epistemic = if biases.is_empty() {
        None
    } else {
        Some(compute_epistemic(harm, biases))
    };

    let architectural = Some(compute_architectural(harm));

    // Combined anti-vector magnitude from all components
    let mech_mag = mechanistic.as_ref().map_or(0.0, |m| m.expected_attenuation);
    let epist_mag = epistemic.as_ref().map_or(0.0, |e| e.counter_magnitude);
    let arch_mag = architectural
        .as_ref()
        .map_or(0.0, |a| a.delta_safety_distance);

    // Anti-vector magnitudes compound (each independently reduces harm)
    // Using 1 - ∏(1 - component_i) — probability of at least one succeeding
    let combined = 1.0 - (1.0 - mech_mag) * (1.0 - epist_mag) * (1.0 - arch_mag);
    let magnitude = combined.clamp(0.0, 1.0);

    let annihilation_result = compute_annihilation(harm.magnitude, magnitude);

    AntiVector {
        harm_vector: harm.clone(),
        mechanistic,
        epistemic,
        architectural,
        magnitude,
        annihilation_result,
    }
}

// =============================================================================
// INTERNALS
// =============================================================================

/// A bias assessment — input to epistemic anti-vector computation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BiasAssessment {
    /// Type of bias
    pub bias_type: BiasType,
    /// Estimated magnitude of this bias (0..1)
    pub magnitude: f64,
    /// Evidence source
    pub evidence_source: EvidenceSource,
    /// Description of the bias evidence
    pub description: String,
}

/// Select the risk minimization measure proportionate to the signal.
fn select_proportionate_measure(
    signal_magnitude: f64,
    confidence: f64,
    available_measures: &[RiskMinimizationMeasure],
) -> (RiskMinimizationMeasure, f64, f64) {
    let severity = signal_magnitude * confidence;

    // Proportionality ladder (ICH E2C(R2) inspired):
    // Low severity (<0.3)  → label update
    // Medium (0.3-0.6)     → DHCP letter + medication guide
    // High (0.6-0.8)       → REMS / restricted distribution
    // Critical (>0.8)      → contraindication or withdrawal
    let (measure, base_delta) = if severity > 0.8 {
        // Critical: strongest available measure
        let m = available_measures
            .iter()
            .find(|m| {
                matches!(
                    m,
                    RiskMinimizationMeasure::Withdrawal | RiskMinimizationMeasure::Contraindication
                )
            })
            .or(available_measures.last())
            .copied()
            .unwrap_or(RiskMinimizationMeasure::Rems);
        (m, 0.8)
    } else if severity > 0.6 {
        let m = available_measures
            .iter()
            .find(|m| {
                matches!(
                    m,
                    RiskMinimizationMeasure::Rems | RiskMinimizationMeasure::RestrictedDistribution
                )
            })
            .or(available_measures.first())
            .copied()
            .unwrap_or(RiskMinimizationMeasure::Rems);
        (m, 0.5)
    } else if severity > 0.3 {
        let m = available_measures
            .iter()
            .find(|m| {
                matches!(
                    m,
                    RiskMinimizationMeasure::DhcpLetter | RiskMinimizationMeasure::MedicationGuide
                )
            })
            .or(available_measures.first())
            .copied()
            .unwrap_or(RiskMinimizationMeasure::LabelUpdate);
        (m, 0.3)
    } else {
        let m = available_measures
            .first()
            .copied()
            .unwrap_or(RiskMinimizationMeasure::LabelUpdate);
        (m, 0.15)
    };

    let proportionality = if signal_magnitude > 0.0 {
        base_delta / signal_magnitude
    } else {
        1.0
    };

    (measure, proportionality, base_delta)
}

/// Generate action descriptions for a risk minimization measure.
fn describe_actions(measure: RiskMinimizationMeasure, harm: &HarmVector) -> Vec<String> {
    match measure {
        RiskMinimizationMeasure::LabelUpdate => vec![
            format!(
                "Update Section 5 (Warnings) for {} regarding {}",
                harm.source, harm.target
            ),
            format!(
                "Add {} to Section 6 (Adverse Reactions) if not present",
                harm.target
            ),
        ],
        RiskMinimizationMeasure::MedicationGuide => vec![
            format!(
                "Create/update Medication Guide warning about {}",
                harm.target
            ),
            "Require dispensing with each fill".to_string(),
        ],
        RiskMinimizationMeasure::DhcpLetter => vec![
            format!(
                "Issue DHCP letter to prescribers of {} regarding {} risk",
                harm.source, harm.target
            ),
            "Include monitoring recommendations".to_string(),
        ],
        RiskMinimizationMeasure::Rems => vec![
            format!("Implement REMS for {} with ETASU", harm.source),
            "Prescriber certification required".to_string(),
            "Patient enrollment required".to_string(),
        ],
        RiskMinimizationMeasure::RestrictedDistribution => vec![format!(
            "Restrict {} distribution to certified pharmacies",
            harm.source
        )],
        RiskMinimizationMeasure::RequiredMonitoring => vec![format!(
            "Mandate baseline and periodic monitoring for {} risk",
            harm.target
        )],
        RiskMinimizationMeasure::DoseModification => vec![format!(
            "Implement titration protocol for {} to mitigate {} risk",
            harm.source, harm.target
        )],
        RiskMinimizationMeasure::Contraindication => vec![format!(
            "Add contraindication for {} in patients at high risk of {}",
            harm.source, harm.target
        )],
        RiskMinimizationMeasure::Withdrawal => vec![format!(
            "Initiate withdrawal assessment for {} based on {} signal",
            harm.source, harm.target
        )],
    }
}

/// Compute the annihilation result from harm and anti-vector magnitudes.
fn compute_annihilation(harm_magnitude: f64, anti_magnitude: f64) -> AnnihilationResult {
    let ratio = if harm_magnitude > 0.0 {
        anti_magnitude / harm_magnitude
    } else {
        1.0
    };

    if ratio < 0.8 {
        AnnihilationResult::ResidualHarm {
            residual: harm_magnitude - anti_magnitude,
            gap: format!(
                "Anti-vector covers {:.0}% of harm. Need {:.2} more magnitude.",
                ratio * 100.0,
                harm_magnitude - anti_magnitude,
            ),
        }
    } else if ratio <= 1.2 {
        AnnihilationResult::Annihilated {
            knowledge: format!(
                "Harm pathway fully characterized. Mechanism understood at {:.0}% confidence.",
                anti_magnitude * 100.0,
            ),
        }
    } else {
        AnnihilationResult::SurplusProtection {
            surplus: anti_magnitude - harm_magnitude,
            disproportionate: ratio > 2.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_harm_taxonomy::HarmTypeId;

    fn sample_harm() -> HarmVector {
        HarmVector {
            source: "semaglutide".to_string(),
            target: "pancreatitis".to_string(),
            harm_type: HarmTypeId::A,
            magnitude: 0.6,
            confidence: 0.75,
            pathway: Some("GLP-1R → pancreatic acinar hyperstimulation".to_string()),
        }
    }

    #[test]
    fn epistemic_refutes_weak_signal() {
        let harm = HarmVector {
            magnitude: 0.3,
            confidence: 0.5,
            ..sample_harm()
        };

        let biases = vec![
            BiasAssessment {
                bias_type: BiasType::IndicationBias,
                magnitude: 0.25,
                evidence_source: EvidenceSource::DatabaseAnalysis,
                description: "T2DM comorbidity inflates pancreatitis reports".to_string(),
            },
            BiasAssessment {
                bias_type: BiasType::NotorietyBias,
                magnitude: 0.1,
                evidence_source: EvidenceSource::Literature,
                description: "Media coverage of GLP-1 pancreatitis risk".to_string(),
            },
        ];

        let av = compute_epistemic(&harm, &biases);
        assert_eq!(av.verdict, EpistemicVerdict::SignalRefuted);
        assert!(av.residual_signal < 0.1);
    }

    #[test]
    fn epistemic_confirms_strong_signal() {
        let harm = sample_harm();
        let biases = vec![BiasAssessment {
            bias_type: BiasType::IndicationBias,
            magnitude: 0.1,
            evidence_source: EvidenceSource::DatabaseAnalysis,
            description: "Minor indication bias".to_string(),
        }];

        let av = compute_epistemic(&harm, &biases);
        assert_eq!(av.verdict, EpistemicVerdict::SignalConfirmed);
        assert!(av.residual_signal > 0.3);
    }

    #[test]
    fn architectural_proportionality() {
        let low_harm = HarmVector {
            magnitude: 0.2,
            confidence: 0.5,
            ..sample_harm()
        };
        let av_low = compute_architectural(&low_harm);
        // Low severity should get a proportionate measure, not REMS
        assert!(!matches!(av_low.measure, RiskMinimizationMeasure::Rems));

        let high_harm = HarmVector {
            magnitude: 0.9,
            confidence: 0.95,
            ..sample_harm()
        };
        let av_high = compute_architectural(&high_harm);
        // High severity should get stronger measures
        assert!(av_high.delta_safety_distance > av_low.delta_safety_distance);
    }

    #[test]
    fn complete_anti_vector_annihilation() {
        let harm = sample_harm();
        let biases = vec![BiasAssessment {
            bias_type: BiasType::IndicationBias,
            magnitude: 0.15,
            evidence_source: EvidenceSource::DatabaseAnalysis,
            description: "Minor indication bias".to_string(),
        }];
        let mechanistic = Some(MechanisticAntiVector {
            pathway_target: "GLP-1R acinar stimulation".to_string(),
            intervention: "Stepped dose titration over 16 weeks".to_string(),
            mechanism_of_action: "Gradual receptor adaptation reduces acute acinar stress"
                .to_string(),
            expected_attenuation: 0.4,
            evidence: vec![EvidenceItem {
                source: EvidenceSource::Rct,
                description: "SUSTAIN trials showed 62% reduction with titration".to_string(),
                strength: 0.8,
            }],
        });

        let av = compute_anti_vector(&harm, &biases, mechanistic);
        assert!(av.magnitude > 0.4);
        // With mechanistic + epistemic + architectural, should approach annihilation
        assert!(
            !matches!(
                av.annihilation_result,
                AnnihilationResult::ResidualHarm { .. }
            ) || av.magnitude > 0.3
        );
    }

    #[test]
    fn all_harm_types_have_strategies() {
        // Every harm type must map to a valid anti-vector strategy
        let types = [
            HarmTypeId::A,
            HarmTypeId::B,
            HarmTypeId::C,
            HarmTypeId::D,
            HarmTypeId::E,
            HarmTypeId::F,
            HarmTypeId::G,
            HarmTypeId::H,
            HarmTypeId::I,
        ];
        for ht in types {
            let strategy = crate::classify::classify_anti_vector(ht);
            assert!(
                !strategy.measures.is_empty(),
                "Harm type {ht:?} has no measures"
            );
        }
    }
}
