//! Annihilation engine: applies anti-vectors to harm vectors and reports results.
//!
//! This module handles the collision semantics — what happens when a harm vector
//! meets its anti-vector. The key insight: annihilation releases knowledge.

use crate::types::*;

/// Apply an anti-vector to its harm vector and produce a detailed report.
///
/// This is the terminal operation: after computing the anti-vector, this
/// function describes what happened in human-readable terms.
#[must_use]
pub fn annihilation_report(av: &AntiVector) -> AnnihilationReport {
    let harm = &av.harm_vector;

    let mechanistic_summary = av.mechanistic.as_ref().map(|m| {
        format!(
            "Pathway [{}] targeted by [{}]. Expected attenuation: {:.0}%.",
            m.pathway_target,
            m.intervention,
            m.expected_attenuation * 100.0,
        )
    });

    let epistemic_summary = av.epistemic.as_ref().map(|e| {
        let verdict_str = match e.verdict {
            EpistemicVerdict::SignalConfirmed => "CONFIRMED — signal survives counter-evidence",
            EpistemicVerdict::SignalAttenuated => "ATTENUATED — signal partially explained by bias",
            EpistemicVerdict::SignalRefuted => "REFUTED — signal likely noise",
        };
        format!(
            "Bias [{:?}] counter-magnitude {:.2}. Residual signal: {:.2}. Verdict: {}.",
            e.bias_type, e.counter_magnitude, e.residual_signal, verdict_str,
        )
    });

    let architectural_summary = av.architectural.as_ref().map(|a| {
        let prop_str = if a.proportionality < 0.8 {
            "INSUFFICIENT"
        } else if a.proportionality <= 1.5 {
            "PROPORTIONATE"
        } else {
            "DISPROPORTIONATE"
        };
        format!(
            "Measure: {:?}. Proportionality: {:.2} ({}). Δd(s) = +{:.2}. Actions: {}.",
            a.measure,
            a.proportionality,
            prop_str,
            a.delta_safety_distance,
            a.actions.join("; "),
        )
    });

    let outcome = match &av.annihilation_result {
        AnnihilationResult::ResidualHarm { residual, gap } => {
            format!("RESIDUAL HARM: {residual:.2} remaining. {gap}")
        }
        AnnihilationResult::Annihilated { knowledge } => {
            format!("ANNIHILATED. Knowledge released: {knowledge}")
        }
        AnnihilationResult::SurplusProtection {
            surplus,
            disproportionate,
        } => {
            if *disproportionate {
                format!(
                    "SURPLUS PROTECTION: +{surplus:.2}. WARNING: measure may be disproportionate. Consider de-escalation."
                )
            } else {
                format!("SURPLUS PROTECTION: +{surplus:.2}. Safety margin expanded.")
            }
        }
    };

    AnnihilationReport {
        drug: harm.source.clone(),
        event: harm.target.clone(),
        harm_type: harm.harm_type,
        harm_magnitude: harm.magnitude,
        anti_vector_magnitude: av.magnitude,
        mechanistic_summary,
        epistemic_summary,
        architectural_summary,
        outcome,
    }
}

/// Human-readable annihilation report.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AnnihilationReport {
    /// Drug/intervention name
    pub drug: String,
    /// Adverse event
    pub event: String,
    /// Harm classification
    pub harm_type: nexcore_harm_taxonomy::HarmTypeId,
    /// Original harm signal magnitude
    pub harm_magnitude: f64,
    /// Anti-vector combined magnitude
    pub anti_vector_magnitude: f64,
    /// Mechanistic component summary (if computed)
    pub mechanistic_summary: Option<String>,
    /// Epistemic component summary (if computed)
    pub epistemic_summary: Option<String>,
    /// Architectural component summary (if computed)
    pub architectural_summary: Option<String>,
    /// Final outcome description
    pub outcome: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compute::*;
    use nexcore_harm_taxonomy::HarmTypeId;

    #[test]
    fn report_includes_all_components() {
        let harm = HarmVector {
            source: "semaglutide".to_string(),
            target: "pancreatitis".to_string(),
            harm_type: HarmTypeId::A,
            magnitude: 0.6,
            confidence: 0.75,
            pathway: Some("GLP-1R pathway".to_string()),
        };

        let biases = vec![BiasAssessment {
            bias_type: BiasType::IndicationBias,
            magnitude: 0.15,
            evidence_source: EvidenceSource::DatabaseAnalysis,
            description: "T2DM comorbidity".to_string(),
        }];

        let mechanistic = Some(MechanisticAntiVector {
            pathway_target: "GLP-1R".to_string(),
            intervention: "dose titration".to_string(),
            mechanism_of_action: "gradual adaptation".to_string(),
            expected_attenuation: 0.4,
            evidence: vec![],
        });

        let av = compute_anti_vector(&harm, &biases, mechanistic);
        let report = annihilation_report(&av);

        assert_eq!(report.drug, "semaglutide");
        assert_eq!(report.event, "pancreatitis");
        assert!(report.mechanistic_summary.is_some());
        assert!(report.epistemic_summary.is_some());
        assert!(report.architectural_summary.is_some());
        assert!(!report.outcome.is_empty());
    }
}
