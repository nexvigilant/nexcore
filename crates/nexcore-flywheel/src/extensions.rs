// Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Extension loops — additional feedback loops that run alongside the core 5.
//!
//! Extension loops contribute to the Reality Gradient without modifying the
//! core cascade. Each extension produces an [`ExtensionResult`] with evidence
//! quality and target achievement, which gets folded into the VDAG grading.
//!
//! ## Current Extensions
//!
//! | Loop | Source | What It Measures |
//! |------|--------|-----------------|
//! | Trust | nexcore-trust | Accumulated trust score from verified operations |
//! | Immunity | nexcore-immunity | Antibody coverage and threat response |
//! | Skill Maturation | skill ecosystem | Skill count, coverage, and enforcement |
//!
//! ## T1 Primitive Grounding: μ(Mapping) + ∂(Boundary) + κ(Comparison)

use crate::vdag::{EvidenceQuality, LoopEvidence};
use serde::{Deserialize, Serialize};

/// Input for the Trust extension loop.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TrustInput {
    /// Global trust score (0.0-1.0) from the trust engine.
    pub trust_score: f64,
    /// Number of verified operations contributing to trust.
    pub verified_operations: u32,
    /// Number of trust violations detected.
    pub violations: u32,
}

/// Input for the Immunity extension loop.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ImmunityInput {
    /// Total antibodies in the registry.
    pub antibody_count: u32,
    /// Critical-severity antibodies (active threats).
    pub critical_count: u32,
    /// PAMP (pathogen-associated) antibodies.
    pub pamp_count: u32,
    /// DAMP (damage-associated) antibodies.
    pub damp_count: u32,
}

/// Input for the Skill Maturation extension loop.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SkillMaturationInput {
    /// Total skills in the ecosystem.
    pub skill_count: u32,
    /// Skills with Diamond compliance level.
    pub diamond_skills: u32,
    /// Skill enforcement ratio (0.0-1.0).
    pub enforcement_ratio: f64,
}

/// Combined extension loop inputs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExtensionInputs {
    /// Trust loop input (optional — omit if trust data unavailable).
    pub trust: Option<TrustInput>,
    /// Immunity loop input (optional).
    pub immunity: Option<ImmunityInput>,
    /// Skill maturation loop input (optional).
    pub skill_maturation: Option<SkillMaturationInput>,
}

/// Evaluate the Trust extension loop.
fn grade_trust(input: &TrustInput) -> EvidenceQuality {
    if input.violations > 5 {
        return EvidenceQuality::Strong; // Strong evidence of failure
    }
    if input.verified_operations == 0 {
        return EvidenceQuality::None;
    }
    if input.trust_score > 0.8 {
        EvidenceQuality::Strong
    } else if input.trust_score > 0.5 {
        EvidenceQuality::Moderate
    } else {
        EvidenceQuality::Weak
    }
}

/// Check if trust achieved its target (healthy = high trust, low violations).
fn trust_achieved_target(input: &TrustInput) -> bool {
    input.trust_score > 0.7 && input.violations <= 2
}

/// Evaluate the Immunity extension loop.
fn grade_immunity(input: &ImmunityInput) -> EvidenceQuality {
    if input.antibody_count == 0 {
        return EvidenceQuality::None;
    }
    // High critical count relative to total = strong evidence of active threats
    let critical_ratio = if input.antibody_count > 0 {
        f64::from(input.critical_count) / f64::from(input.antibody_count)
    } else {
        0.0
    };
    if critical_ratio > 0.5 {
        EvidenceQuality::Strong // Strong evidence of threat
    } else if input.antibody_count >= 10 {
        EvidenceQuality::Strong // Strong coverage
    } else if input.antibody_count >= 5 {
        EvidenceQuality::Moderate
    } else {
        EvidenceQuality::Weak
    }
}

/// Check if immunity achieved its target (good coverage, low critical ratio).
fn immunity_achieved_target(input: &ImmunityInput) -> bool {
    let critical_ratio = if input.antibody_count > 0 {
        f64::from(input.critical_count) / f64::from(input.antibody_count)
    } else {
        1.0 // No antibodies = not achieved
    };
    input.antibody_count >= 5 && critical_ratio < 0.5
}

/// Evaluate the Skill Maturation extension loop.
fn grade_skill_maturation(input: &SkillMaturationInput) -> EvidenceQuality {
    if input.skill_count == 0 {
        return EvidenceQuality::None;
    }
    if input.enforcement_ratio > 0.8 && input.diamond_skills > 10 {
        EvidenceQuality::Strong
    } else if input.enforcement_ratio > 0.5 {
        EvidenceQuality::Moderate
    } else {
        EvidenceQuality::Weak
    }
}

/// Check if skill maturation achieved its target.
fn skill_maturation_achieved_target(input: &SkillMaturationInput) -> bool {
    input.enforcement_ratio > 0.6 && input.skill_count >= 50
}

/// Evaluate all extension loops and produce [`LoopEvidence`] entries.
///
/// Returns a Vec of evidence entries for each provided extension.
/// Absent extensions (None) are skipped — they don't dilute the score.
pub fn evaluate_extensions(inputs: &ExtensionInputs) -> Vec<LoopEvidence> {
    let mut evidence = Vec::new();

    if let Some(trust) = &inputs.trust {
        evidence.push(LoopEvidence {
            loop_name: "trust".to_string(),
            quality: grade_trust(trust),
            weight: 0.15,
            achieved_target: trust_achieved_target(trust),
        });
    }

    if let Some(immunity) = &inputs.immunity {
        evidence.push(LoopEvidence {
            loop_name: "immunity".to_string(),
            quality: grade_immunity(immunity),
            weight: 0.10,
            achieved_target: immunity_achieved_target(immunity),
        });
    }

    if let Some(skills) = &inputs.skill_maturation {
        evidence.push(LoopEvidence {
            loop_name: "skill_maturation".to_string(),
            quality: grade_skill_maturation(skills),
            weight: 0.10,
            achieved_target: skill_maturation_achieved_target(skills),
        });
    }

    evidence
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trust_grading() {
        let high = TrustInput {
            trust_score: 0.9,
            verified_operations: 100,
            violations: 0,
        };
        assert_eq!(grade_trust(&high), EvidenceQuality::Strong);
        assert!(trust_achieved_target(&high));

        let low = TrustInput {
            trust_score: 0.3,
            verified_operations: 10,
            violations: 0,
        };
        assert_eq!(grade_trust(&low), EvidenceQuality::Weak);
        assert!(!trust_achieved_target(&low));

        let none = TrustInput::default();
        assert_eq!(grade_trust(&none), EvidenceQuality::None);
    }

    #[test]
    fn immunity_grading() {
        let good = ImmunityInput {
            antibody_count: 14,
            critical_count: 4,
            pamp_count: 2,
            damp_count: 12,
        };
        assert_eq!(grade_immunity(&good), EvidenceQuality::Strong);
        assert!(immunity_achieved_target(&good));

        let threat = ImmunityInput {
            antibody_count: 10,
            critical_count: 8,
            pamp_count: 5,
            damp_count: 5,
        };
        assert_eq!(grade_immunity(&threat), EvidenceQuality::Strong);
        assert!(!immunity_achieved_target(&threat)); // High critical ratio
    }

    #[test]
    fn skill_maturation_grading() {
        let mature = SkillMaturationInput {
            skill_count: 256,
            diamond_skills: 20,
            enforcement_ratio: 0.9,
        };
        assert_eq!(grade_skill_maturation(&mature), EvidenceQuality::Strong);
        assert!(skill_maturation_achieved_target(&mature));

        let early = SkillMaturationInput {
            skill_count: 10,
            diamond_skills: 0,
            enforcement_ratio: 0.3,
        };
        assert_eq!(grade_skill_maturation(&early), EvidenceQuality::Weak);
    }

    #[test]
    fn evaluate_extensions_only_includes_present() {
        let inputs = ExtensionInputs {
            trust: Some(TrustInput {
                trust_score: 0.9,
                verified_operations: 50,
                violations: 0,
            }),
            immunity: None,
            skill_maturation: None,
        };
        let evidence = evaluate_extensions(&inputs);
        assert_eq!(evidence.len(), 1);
        assert_eq!(evidence[0].loop_name, "trust");
    }

    #[test]
    fn evaluate_all_extensions() {
        let inputs = ExtensionInputs {
            trust: Some(TrustInput {
                trust_score: 0.85,
                verified_operations: 100,
                violations: 1,
            }),
            immunity: Some(ImmunityInput {
                antibody_count: 14,
                critical_count: 4,
                pamp_count: 2,
                damp_count: 12,
            }),
            skill_maturation: Some(SkillMaturationInput {
                skill_count: 256,
                diamond_skills: 15,
                enforcement_ratio: 0.85,
            }),
        };
        let evidence = evaluate_extensions(&inputs);
        assert_eq!(evidence.len(), 3);

        let names: Vec<&str> = evidence.iter().map(|e| e.loop_name.as_str()).collect();
        assert_eq!(names, vec!["trust", "immunity", "skill_maturation"]);

        // All should achieve targets with these healthy inputs
        for e in &evidence {
            assert!(
                e.achieved_target,
                "Loop {} should achieve target",
                e.loop_name
            );
        }
    }

    #[test]
    fn empty_extensions() {
        let inputs = ExtensionInputs::default();
        let evidence = evaluate_extensions(&inputs);
        assert!(evidence.is_empty());
    }

    #[test]
    fn weight_sum_is_reasonable() {
        let inputs = ExtensionInputs {
            trust: Some(TrustInput::default()),
            immunity: Some(ImmunityInput::default()),
            skill_maturation: Some(SkillMaturationInput::default()),
        };
        let evidence = evaluate_extensions(&inputs);
        let total_weight: f64 = evidence.iter().map(|e| e.weight).sum();
        // 0.15 + 0.10 + 0.10 = 0.35 — leaves 0.65 for core loops
        assert!((total_weight - 0.35).abs() < f64::EPSILON);
    }
}
