// Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! NMD (Nonsense-Mediated Decay) cytokine signal family.
//!
//! Provides specialized cytokine constructors for the NMD surveillance system.
//! These signals coordinate the degradation response when structural anomalies
//! are detected during co-translational monitoring.
//!
//! ## Biology Analog
//!
//! In biology, NMD triggers mRNA degradation through SMG proteins:
//! - **SMG5/7**: Recruit decapping and 5'->3' exonuclease (kill the mRNA)
//! - **SMG6**: Endonucleolytic cleavage (direct cut)
//!
//! ## Signal Mapping
//!
//! | NMD Action | SMG Analog | Cytokine Family | Effect |
//! |------------|------------|-----------------|--------|
//! | Abort pipeline | SMG5 | TNF-alpha | Terminate execution |
//! | Flag source | SMG6 | IFN-gamma | Mark artifact for review |
//! | Adaptive update | SMG7 | TGF-beta | Update spliceosome templates |
//!
//! ## Primitive Grounding: ->(Causality) + void(Void) + state(State)

use crate::{Cytokine, CytokineFamily, Scope, ThreatLevel};

impl Cytokine {
    /// NMD abort signal — terminate pipeline execution.
    ///
    /// Maps to SMG5: decapping + exonuclease degradation.
    /// Emitted when the UPF complex detects a critical structural violation
    /// that cannot be recovered from.
    ///
    /// Family: TNF-alpha (destruction), Scope: Systemic, Severity: Critical
    pub fn nmd_abort(reason: impl Into<String>) -> Self {
        let reason_str = reason.into();
        Self::new(CytokineFamily::TnfAlpha, "nmd_abort")
            .with_severity(ThreatLevel::Critical)
            .with_scope(Scope::Systemic)
            .with_source("upf_complex")
            .with_payload(serde_json::json!({
                "reason": reason_str,
                "smg_analog": "SMG5",
                "action": "abort_pipeline"
            }))
    }

    /// NMD flag source signal — mark source artifact for review.
    ///
    /// Maps to SMG6: endonucleolytic cleavage (kill the mRNA, not just the protein).
    /// Emitted to flag the original task specification or prompt that produced
    /// the structural violation, preventing it from being reused.
    ///
    /// Family: IFN-gamma (activation), Scope: Endocrine, Severity: High
    pub fn nmd_flag_source(source: impl Into<String>, reason: impl Into<String>) -> Self {
        let source_str = source.into();
        let reason_str = reason.into();
        Self::new(CytokineFamily::IfnGamma, "nmd_flag_source")
            .with_severity(ThreatLevel::High)
            .with_scope(Scope::Endocrine)
            .with_source("upf_complex")
            .with_payload(serde_json::json!({
                "flagged_source": source_str,
                "reason": reason_str,
                "smg_analog": "SMG6",
                "action": "flag_source_artifact"
            }))
    }

    /// NMD adaptive update signal — update spliceosome templates.
    ///
    /// Maps to SMG7: adaptive immunity feedback loop.
    /// Emitted when an NMD event reveals that the spliceosome's structural
    /// expectations need refinement for a task category.
    ///
    /// Family: TGF-beta (regulation), Scope: Paracrine, Severity: Medium
    pub fn nmd_adaptive_update(category: impl Into<String>, update: &serde_json::Value) -> Self {
        let category_str = category.into();
        Self::new(CytokineFamily::TgfBeta, "nmd_adaptive_update")
            .with_severity(ThreatLevel::Medium)
            .with_scope(Scope::Paracrine)
            .with_source("smg_complex")
            .with_payload(serde_json::json!({
                "category": category_str,
                "template_update": update,
                "smg_analog": "SMG7",
                "action": "update_spliceosome_templates"
            }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nmd_abort_signal() {
        let signal = Cytokine::nmd_abort("phase order violation detected");
        assert_eq!(signal.family, CytokineFamily::TnfAlpha);
        assert_eq!(signal.severity, ThreatLevel::Critical);
        assert_eq!(signal.scope, Scope::Systemic);
        assert_eq!(signal.name, "nmd_abort");
        assert_eq!(signal.source, Some("upf_complex".to_string()));
        assert!(signal.payload.get("smg_analog").is_some());
    }

    #[test]
    fn test_nmd_flag_source_signal() {
        let signal = Cytokine::nmd_flag_source("task-spec-123", "persistent tool drift");
        assert_eq!(signal.family, CytokineFamily::IfnGamma);
        assert_eq!(signal.severity, ThreatLevel::High);
        assert_eq!(signal.scope, Scope::Endocrine);
        assert_eq!(signal.name, "nmd_flag_source");
        let payload = &signal.payload;
        assert_eq!(
            payload.get("flagged_source").and_then(|v| v.as_str()),
            Some("task-spec-123")
        );
    }

    #[test]
    fn test_nmd_adaptive_update_signal() {
        let update = serde_json::json!({"new_threshold": 0.8});
        let signal = Cytokine::nmd_adaptive_update("Compute", &update);
        assert_eq!(signal.family, CytokineFamily::TgfBeta);
        assert_eq!(signal.severity, ThreatLevel::Medium);
        assert_eq!(signal.scope, Scope::Paracrine);
        assert_eq!(signal.name, "nmd_adaptive_update");
        assert_eq!(signal.source, Some("smg_complex".to_string()));
    }

    #[test]
    fn test_nmd_signals_are_cascadable() {
        let abort = Cytokine::nmd_abort("test");
        let flag = Cytokine::nmd_flag_source("src", "test");
        let update = Cytokine::nmd_adaptive_update("cat", &serde_json::json!({}));
        assert!(abort.cascadable);
        assert!(flag.cascadable);
        assert!(update.cascadable);
    }

    #[test]
    fn test_nmd_abort_not_expired_immediately() {
        let signal = Cytokine::nmd_abort("test");
        assert!(!signal.is_expired());
    }
}
