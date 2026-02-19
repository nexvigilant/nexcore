//! # Fertility — System Reproductive Capacity Assessment
//!
//! Evaluates the overall ability of the system to produce healthy offspring
//! (artifacts, deployments, sessions). Combines metrics from all reproductive
//! subsystems into a single fertility score.
//!
//! ## Biological Mapping
//!
//! | Concept | Biological | Claude Code |
//! |---------|-----------|-------------|
//! | Fertility | Ability to conceive | System capacity to produce working artifacts |
//! | Fecundity | Offspring rate | Deployment frequency |
//! | Viability | Offspring survival | Deployment success rate |
//! | Genetic Load | Harmful mutations | Accumulated tech debt |
//! | Inbreeding Depression | Reduced fitness from low diversity | Single-approach over-specialization |

use serde::{Deserialize, Serialize};

/// Fertility assessment for the reproductive system.
///
/// **T1 grounding**: ς(State) + N(Quantity) + κ(Comparison) + ∂(Boundary)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FertilityAssessment {
    /// Fecundity: rate of artifact/deployment production (per time unit).
    pub fecundity: f64,
    /// Viability: fraction of offspring that survive (deploy successfully).
    pub viability: f64,
    /// Genetic diversity: variety of approaches/patterns used (0.0 to 1.0).
    pub genetic_diversity: f64,
    /// Genetic load: accumulated harmful mutations (tech debt score, 0.0 = clean).
    pub genetic_load: f64,
    /// Mutation rate: rate of beneficial changes (feature branches merged / total).
    pub beneficial_mutation_rate: f64,
    /// Pipeline health: fraction of CI gates currently passing.
    pub pipeline_health: f64,
    /// Session continuity: fraction of knowledge successfully transferred across sessions.
    pub session_continuity: f64,
}

impl FertilityAssessment {
    /// Computes the overall fertility score (0.0 to 1.0).
    ///
    /// Weighted combination:
    /// - 25% viability (most important — offspring must survive)
    /// - 20% pipeline health (infrastructure must work)
    /// - 15% fecundity (must be productive, capped at 1.0)
    /// - 15% session continuity (knowledge must transfer)
    /// - 10% genetic diversity (avoid monoculture)
    /// - 10% beneficial mutation rate (must innovate)
    /// - 5% genetic load penalty (tech debt hurts)
    pub fn fertility_score(&self) -> f64 {
        let fecundity_normalized = self.fecundity.min(1.0);
        let load_penalty = (1.0 - self.genetic_load).max(0.0);

        let score = (self.viability * 0.25)
            + (self.pipeline_health * 0.20)
            + (fecundity_normalized * 0.15)
            + (self.session_continuity * 0.15)
            + (self.genetic_diversity * 0.10)
            + (self.beneficial_mutation_rate * 0.10)
            + (load_penalty * 0.05);

        score.clamp(0.0, 1.0)
    }

    /// Returns a classification based on the fertility score.
    pub fn classification(&self) -> FertilityClass {
        let score = self.fertility_score();
        if score >= 0.85 {
            FertilityClass::Excellent
        } else if score >= 0.70 {
            FertilityClass::Good
        } else if score >= 0.50 {
            FertilityClass::Fair
        } else if score >= 0.30 {
            FertilityClass::Poor
        } else {
            FertilityClass::Infertile
        }
    }

    /// Identifies the weakest factor limiting fertility.
    pub fn limiting_factor(&self) -> LimitingFactor {
        let factors = [
            (self.viability, LimitingFactor::LowViability),
            (self.pipeline_health, LimitingFactor::PipelineFailure),
            (self.fecundity.min(1.0), LimitingFactor::LowFecundity),
            (self.session_continuity, LimitingFactor::KnowledgeLoss),
            (self.genetic_diversity, LimitingFactor::InbreedingDepression),
            (
                self.beneficial_mutation_rate,
                LimitingFactor::StagnantEvolution,
            ),
            (
                (1.0 - self.genetic_load).max(0.0),
                LimitingFactor::GeneticLoadExcess,
            ),
        ];

        factors
            .into_iter()
            .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(_, f)| f)
            .unwrap_or(LimitingFactor::LowViability)
    }

    /// Returns recommendations for improving fertility.
    pub fn recommendations(&self) -> Vec<String> {
        let mut recs = Vec::new();

        if self.viability < 0.9 {
            recs.push("Improve deployment success rate — review rollback procedures".into());
        }
        if self.pipeline_health < 0.95 {
            recs.push("Fix failing CI gates — run guard-program pipeline".into());
        }
        if self.genetic_diversity < 0.3 {
            recs.push(
                "Increase approach diversity — try alternative patterns for recurring problems"
                    .into(),
            );
        }
        if self.genetic_load > 0.3 {
            recs.push("Reduce tech debt — run clean-program pipeline".into());
        }
        if self.session_continuity < 0.8 {
            recs.push("Improve session handoffs — verify brain artifacts persist".into());
        }
        if self.beneficial_mutation_rate < 0.1 {
            recs.push("Increase innovation rate — more feature branches, less maintenance".into());
        }

        recs
    }
}

/// Fertility classification.
///
/// **T1 grounding**: Σ(Sum) + κ(Comparison)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FertilityClass {
    /// Score >= 0.85 — system is highly productive and healthy.
    Excellent,
    /// Score >= 0.70 — system is productive with minor issues.
    Good,
    /// Score >= 0.50 — system has significant reproductive challenges.
    Fair,
    /// Score >= 0.30 — system is struggling to produce viable offspring.
    Poor,
    /// Score < 0.30 — system cannot reliably produce offspring.
    Infertile,
}

/// The weakest factor limiting reproductive capacity.
///
/// **T1 grounding**: Σ(Sum) + →(Causality)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LimitingFactor {
    /// Deployments frequently fail.
    LowViability,
    /// CI pipeline is broken.
    PipelineFailure,
    /// Not producing artifacts fast enough.
    LowFecundity,
    /// Knowledge lost between sessions.
    KnowledgeLoss,
    /// Over-reliance on single patterns.
    InbreedingDepression,
    /// Not enough innovation/feature work.
    StagnantEvolution,
    /// Too much tech debt accumulated.
    GeneticLoadExcess,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn healthy_system() -> FertilityAssessment {
        FertilityAssessment {
            fecundity: 0.95,
            viability: 0.98,
            genetic_diversity: 0.7,
            genetic_load: 0.05,
            beneficial_mutation_rate: 0.3,
            pipeline_health: 0.99,
            session_continuity: 0.95,
        }
    }

    fn struggling_system() -> FertilityAssessment {
        FertilityAssessment {
            fecundity: 0.3,
            viability: 0.5,
            genetic_diversity: 0.1,
            genetic_load: 0.7,
            beneficial_mutation_rate: 0.05,
            pipeline_health: 0.6,
            session_continuity: 0.4,
        }
    }

    #[test]
    fn test_healthy_fertility_score() {
        let a = healthy_system();
        let score = a.fertility_score();
        assert!(
            score > 0.8,
            "healthy system should score > 0.8, got {score}"
        );
        assert_eq!(a.classification(), FertilityClass::Excellent);
    }

    #[test]
    fn test_struggling_fertility_score() {
        let a = struggling_system();
        let score = a.fertility_score();
        assert!(
            score < 0.5,
            "struggling system should score < 0.5, got {score}"
        );
        assert!(matches!(
            a.classification(),
            FertilityClass::Poor | FertilityClass::Fair
        ));
    }

    #[test]
    fn test_limiting_factor_identification() {
        let mut a = healthy_system();
        a.genetic_diversity = 0.05; // Very low diversity
        assert_eq!(a.limiting_factor(), LimitingFactor::InbreedingDepression);
    }

    #[test]
    fn test_recommendations_generated() {
        let a = struggling_system();
        let recs = a.recommendations();
        assert!(!recs.is_empty());
        // Should recommend fixing multiple things
        assert!(recs.len() >= 3);
    }

    #[test]
    fn test_perfect_system() {
        let a = FertilityAssessment {
            fecundity: 1.0,
            viability: 1.0,
            genetic_diversity: 1.0,
            genetic_load: 0.0,
            beneficial_mutation_rate: 1.0,
            pipeline_health: 1.0,
            session_continuity: 1.0,
        };
        assert_eq!(a.fertility_score(), 1.0);
        assert_eq!(a.classification(), FertilityClass::Excellent);
        assert!(a.recommendations().is_empty());
    }

    #[test]
    fn test_score_clamping() {
        let a = FertilityAssessment {
            fecundity: 10.0, // Way over 1.0
            viability: 1.0,
            genetic_diversity: 1.0,
            genetic_load: 0.0,
            beneficial_mutation_rate: 1.0,
            pipeline_health: 1.0,
            session_continuity: 1.0,
        };
        assert!(a.fertility_score() <= 1.0);
    }

    #[test]
    fn test_all_fertility_classes() {
        let classes = [
            (0.90, FertilityClass::Excellent),
            (0.75, FertilityClass::Good),
            (0.55, FertilityClass::Fair),
            (0.35, FertilityClass::Poor),
            (0.10, FertilityClass::Infertile),
        ];

        for (target_score, expected_class) in classes {
            // Build an assessment that hits roughly the target score
            // viability * 0.25 = target_score (simplification)
            let a = FertilityAssessment {
                fecundity: target_score,
                viability: target_score,
                genetic_diversity: target_score,
                genetic_load: 1.0 - target_score,
                beneficial_mutation_rate: target_score,
                pipeline_health: target_score,
                session_continuity: target_score,
            };
            assert_eq!(
                a.classification(),
                expected_class,
                "score {} should classify as {:?}",
                a.fertility_score(),
                expected_class
            );
        }
    }
}
