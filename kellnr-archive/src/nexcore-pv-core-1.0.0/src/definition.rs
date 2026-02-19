//! # Computational Definition of Pharmacovigilance
//!
//! WHO defines pharmacovigilance as:
//! > "The science and activities relating to the **detection**, **assessment**,
//! > **understanding**, and **prevention** of adverse effects or any other
//! > drug-related problem."
//!
//! We extend this with **collect** — the prerequisite data acquisition phase
//! that feeds all downstream verbs. No detection without data.
//!
//! ## PV Verb → nexcore Subsystem
//!
//! | Verb | Method | Subsystem |
//! |------|--------|-----------|
//! | Collect | `collect()` | FAERS ETL, E2B ingest, spontaneous reports |
//! | Detect | `detect()` | PRR, ROR, IC, EBGM, MaxSPRT, CuSum |
//! | Assess | `assess()` | Naranjo, WHO-UMC, RUCAM, UCAS, QBRI |
//! | Understand | `understand()` | ToV, Conservation Laws, Minesweeper-PV |
//! | Prevent | `prevent()` | REMS/RMP, risk minimization, threshold monitoring |
//!
//! ## T1 Grounding
//!
//! | PV Verb | T1 Primitive | Symbol |
//! |---------|-------------|--------|
//! | Collect | Sequence | σ |
//! | Detect | Existence | ∃ |
//! | Assess | Causality | → |
//! | Understand | Recursion | ρ |
//! | Prevent | Boundary | ∂ |

use serde::{Deserialize, Serialize};

use super::icsr::Icsr;

// ═══════════════════════════════════════════════════════════════════════════
// COLLECTION OUTPUT
// ═══════════════════════════════════════════════════════════════════════════

/// Data acquisition result from `collect()`.
///
/// Represents the outcome of ingesting safety reports from any source:
/// FAERS, EudraVigilance, VigiBase, spontaneous reports, literature.
///
/// # Tier: T2-C (composed from T1: Sequence + Existence + State)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionResult {
    /// ICSRs successfully ingested
    pub cases: Vec<Icsr>,
    /// Source of the data (e.g., "FAERS-Q4-2024", "EudraVigilance", "Spontaneous")
    pub source: String,
    /// Total records attempted
    pub records_attempted: u32,
    /// Records successfully parsed into ICSRs
    pub records_ingested: u32,
    /// Records rejected (parse errors, duplicates, incomplete)
    pub records_rejected: u32,
    /// Duplicate case IDs detected and deduplicated
    pub duplicates_removed: u32,
}

impl CollectionResult {
    /// Ingestion success rate [0.0, 1.0].
    #[must_use]
    pub fn success_rate(&self) -> f64 {
        if self.records_attempted == 0 {
            return 0.0;
        }
        f64::from(self.records_ingested) / f64::from(self.records_attempted)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// DETECTION OUTPUT
// ═══════════════════════════════════════════════════════════════════════════

/// Signal detection result from `detect()`.
///
/// Wraps the 2×2 contingency table disproportionality output.
///
/// # Tier: T2-C (composed from T1: Existence + Quantity + Comparison)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionResult {
    /// Drug-event pair identifier
    pub drug: String,
    /// Adverse event (MedDRA preferred term)
    pub event: String,
    /// Case count supporting this signal
    pub case_count: u32,
    /// PRR point estimate
    pub prr: f64,
    /// ROR point estimate
    pub ror: f64,
    /// IC (Information Component) point estimate
    pub ic: f64,
    /// EBGM (Empirical Bayes Geometric Mean)
    pub ebgm: f64,
    /// Whether signal exceeds detection threshold
    pub signal_detected: bool,
}

// ═══════════════════════════════════════════════════════════════════════════
// ASSESSMENT OUTPUT
// ═══════════════════════════════════════════════════════════════════════════

/// Causality + benefit-risk assessment from `assess()`.
///
/// # Tier: T2-C (composed from T1: Causality + Comparison + Sequence)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentResult {
    /// Naranjo score (-4 to +13)
    pub naranjo_score: Option<i32>,
    /// WHO-UMC category
    pub who_umc_category: Option<String>,
    /// RUCAM score (-4 to +14) for hepatotoxicity
    pub rucam_score: Option<i32>,
    /// Overall causality classification
    pub causality: CausalityLevel,
    /// Benefit-risk ratio (QBRI) if benefit data available
    pub benefit_risk_ratio: Option<f64>,
}

/// Causality strength levels (ordered by confidence).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CausalityLevel {
    /// Insufficient data
    Unassessable,
    /// Timing incompatible or better explanation exists
    Unlikely,
    /// Possible but not confirmed
    Possible,
    /// Reasonable temporal + pharmacological relationship
    Probable,
    /// Positive rechallenge or unequivocal evidence
    Certain,
}

// ═══════════════════════════════════════════════════════════════════════════
// UNDERSTANDING OUTPUT
// ═══════════════════════════════════════════════════════════════════════════

/// Mechanistic understanding from `understand()`.
///
/// # Tier: T2-C (composed from T1: Recursion + Mapping + State)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnderstandingResult {
    /// Risk factors identified (patient, drug, interaction)
    pub risk_factors: Vec<String>,
    /// Proposed mechanism of action for the adverse effect
    pub mechanism: Option<String>,
    /// Population subgroups at elevated risk
    pub vulnerable_populations: Vec<String>,
    /// Conservation laws validated (from comppv 11-law framework)
    pub conservation_laws_checked: u8,
    /// Conservation law violations detected
    pub conservation_violations: Vec<String>,
    /// ToV safety level mapping
    pub tov_level: Option<String>,
}

// ═══════════════════════════════════════════════════════════════════════════
// PREVENTION OUTPUT
// ═══════════════════════════════════════════════════════════════════════════

/// Risk minimization actions from `prevent()`.
///
/// # Tier: T2-C (composed from T1: Boundary + State + Causality)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreventionResult {
    /// Recommended regulatory actions
    pub actions: Vec<RegulatoryAction>,
    /// Whether a REMS/RMP is warranted
    pub risk_management_required: bool,
    /// Signal monitoring thresholds updated
    pub threshold_adjustments: Vec<ThresholdAdjustment>,
}

/// Regulatory action types for risk prevention.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RegulatoryAction {
    /// Update product labeling/SmPC
    LabelUpdate { section: String, text: String },
    /// Dear Healthcare Professional Communication
    DhpcLetter,
    /// Restriction of indication or population
    Restriction { description: String },
    /// Suspension or withdrawal from market
    Withdrawal,
    /// Require additional monitoring
    AdditionalMonitoring,
    /// Require a Risk Evaluation and Mitigation Strategy
    Rems,
    /// Require a Risk Management Plan
    Rmp,
}

/// Signal threshold adjustment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdAdjustment {
    /// Algorithm name (PRR, ROR, IC, EBGM)
    pub algorithm: String,
    /// Previous threshold
    pub previous: f64,
    /// New threshold
    pub updated: f64,
    /// Reason for adjustment
    pub reason: String,
}

// ═══════════════════════════════════════════════════════════════════════════
// THE PHARMACOVIGILANCE TRAIT
// ═══════════════════════════════════════════════════════════════════════════

/// The computational definition of pharmacovigilance.
///
/// Implements the WHO definition through four methods corresponding to
/// the four verbs: detect, assess, understand, prevent.
///
/// Each method operates on ICSRs — the atomic unit of PV.
///
/// # Tier: T3 (Domain-Specific)
///
/// # Example
///
/// ```rust,ignore
/// use nexcore_vigilance::pv::definition::Pharmacovigilance;
///
/// struct MyPvSystem;
/// impl Pharmacovigilance for MyPvSystem {
///     fn detect(&self, cases: &[Icsr]) -> Vec<DetectionResult> { todo!() }
///     fn assess(&self, cases: &[Icsr], signal: &DetectionResult) -> AssessmentResult { todo!() }
///     fn understand(&self, cases: &[Icsr], signal: &DetectionResult) -> UnderstandingResult { todo!() }
///     fn prevent(&self, assessment: &AssessmentResult) -> PreventionResult { todo!() }
/// }
/// ```
pub trait Pharmacovigilance {
    /// **COLLECT** — Acquire and normalize safety data into ICSRs.
    ///
    /// Input: Raw data source identifier.
    /// Output: Validated, deduplicated ICSRs ready for analysis.
    ///
    /// T1 grounding: σ (Sequence) — ordered ingestion pipeline.
    fn collect(&self, source: &str) -> CollectionResult;

    /// **DETECT** — Identify statistical signals of disproportionate reporting.
    ///
    /// Input: ICSR corpus.
    /// Output: Drug-event pairs exceeding detection thresholds.
    ///
    /// T1 grounding: ∃ (Existence) — does a signal exist?
    fn detect(&self, cases: &[Icsr]) -> Vec<DetectionResult>;

    /// **ASSESS** — Evaluate causality strength and benefit-risk balance.
    ///
    /// Input: Cases + detected signal.
    /// Output: Causality level + QBRI score.
    ///
    /// T1 grounding: → (Causality) — did the drug cause this?
    fn assess(&self, cases: &[Icsr], signal: &DetectionResult) -> AssessmentResult;

    /// **UNDERSTAND** — Investigate mechanism, risk factors, vulnerable populations.
    ///
    /// Input: Cases + detected signal.
    /// Output: Mechanistic insight + conservation law validation.
    ///
    /// T1 grounding: ρ (Recursion) — recursive deepening of knowledge.
    fn understand(&self, cases: &[Icsr], signal: &DetectionResult) -> UnderstandingResult;

    /// **PREVENT** — Determine risk minimization measures.
    ///
    /// Input: Assessment result.
    /// Output: Regulatory actions + threshold adjustments.
    ///
    /// T1 grounding: ∂ (Boundary) — enforce safety boundaries.
    fn prevent(&self, assessment: &AssessmentResult) -> PreventionResult;

    /// Full PV cycle: collect → detect → assess → understand → prevent.
    ///
    /// Convenience method that chains all five PV verbs.
    fn full_cycle(&self, source: &str) -> Vec<PvCycleResult> {
        let collection = self.collect(source);
        let cases = &collection.cases;
        let signals = self.detect(cases);
        signals
            .iter()
            .map(|signal| {
                let assessment = self.assess(cases, signal);
                let understanding = self.understand(cases, signal);
                let prevention = self.prevent(&assessment);
                PvCycleResult {
                    source: collection.source.clone(),
                    detection: signal.clone(),
                    assessment,
                    understanding,
                    prevention,
                }
            })
            .collect()
    }
}

/// Complete result from running the full PV cycle on a single signal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PvCycleResult {
    /// Data source this cycle operated on
    pub source: String,
    /// Detection: what was found
    pub detection: DetectionResult,
    /// Assessment: how strong is the evidence
    pub assessment: AssessmentResult,
    /// Understanding: why does this happen
    pub understanding: UnderstandingResult,
    /// Prevention: what to do about it
    pub prevention: PreventionResult,
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::icsr::{IcsrBuilder, ReactionOutcome};

    /// Minimal PV system for testing the trait.
    struct TestPvSystem;

    impl Pharmacovigilance for TestPvSystem {
        fn collect(&self, source: &str) -> CollectionResult {
            CollectionResult {
                cases: vec![
                    IcsrBuilder::new("CASE-001")
                        .suspect_drug("TEST-DRUG")
                        .reaction("Test event", ReactionOutcome::Recovered)
                        .build(),
                    IcsrBuilder::new("CASE-002")
                        .suspect_drug("TEST-DRUG")
                        .reaction("Test event", ReactionOutcome::Recovering)
                        .build(),
                ],
                source: source.into(),
                records_attempted: 3,
                records_ingested: 2,
                records_rejected: 1,
                duplicates_removed: 0,
            }
        }

        fn detect(&self, cases: &[Icsr]) -> Vec<DetectionResult> {
            if cases.is_empty() {
                return vec![];
            }
            // Simple: any drug with 2+ cases gets a signal
            vec![DetectionResult {
                drug: "TEST-DRUG".into(),
                event: "Test event".into(),
                case_count: cases.len() as u32,
                prr: 2.5,
                ror: 3.0,
                ic: 1.2,
                ebgm: 2.1,
                signal_detected: true,
            }]
        }

        fn assess(&self, _cases: &[Icsr], signal: &DetectionResult) -> AssessmentResult {
            AssessmentResult {
                naranjo_score: Some(6),
                who_umc_category: Some("Probable".into()),
                rucam_score: None,
                causality: if signal.prr > 2.0 {
                    CausalityLevel::Probable
                } else {
                    CausalityLevel::Possible
                },
                benefit_risk_ratio: Some(0.8),
            }
        }

        fn understand(&self, _cases: &[Icsr], _signal: &DetectionResult) -> UnderstandingResult {
            UnderstandingResult {
                risk_factors: vec!["Age > 65".into()],
                mechanism: Some("COX-1 inhibition".into()),
                vulnerable_populations: vec!["Elderly".into()],
                conservation_laws_checked: 11,
                conservation_violations: vec![],
                tov_level: Some("C-SystemFailure".into()),
            }
        }

        fn prevent(&self, assessment: &AssessmentResult) -> PreventionResult {
            let mut actions = vec![RegulatoryAction::AdditionalMonitoring];
            if assessment.causality >= CausalityLevel::Probable {
                actions.push(RegulatoryAction::LabelUpdate {
                    section: "4.4".into(),
                    text: "Risk of GI bleeding in elderly".into(),
                });
            }
            PreventionResult {
                actions,
                risk_management_required: assessment.benefit_risk_ratio.is_some_and(|r| r < 1.0),
                threshold_adjustments: vec![],
            }
        }
    }

    #[test]
    fn full_pv_cycle() {
        let system = TestPvSystem;

        let results = system.full_cycle("FAERS-TEST");
        assert_eq!(results.len(), 1);

        let r = &results[0];
        assert_eq!(r.source, "FAERS-TEST");
        assert!(r.detection.signal_detected);
        assert_eq!(r.assessment.causality, CausalityLevel::Probable);
        assert!(r.understanding.mechanism.is_some());
        assert!(r.prevention.risk_management_required);
        assert!(r.prevention.actions.len() >= 2);
    }

    #[test]
    fn collect_success_rate() {
        let system = TestPvSystem;
        let result = system.collect("FAERS-Q4-2024");
        assert_eq!(result.cases.len(), 2);
        assert_eq!(result.records_attempted, 3);
        let rate = result.success_rate();
        assert!(rate > 0.6 && rate < 0.7); // 2/3 ≈ 0.667
    }

    #[test]
    fn detect_empty_corpus() {
        let system = TestPvSystem;
        let results = system.detect(&[]);
        assert!(results.is_empty());
    }

    #[test]
    fn causality_level_ordering() {
        assert!(CausalityLevel::Certain > CausalityLevel::Probable);
        assert!(CausalityLevel::Probable > CausalityLevel::Possible);
        assert!(CausalityLevel::Possible > CausalityLevel::Unlikely);
        assert!(CausalityLevel::Unlikely > CausalityLevel::Unassessable);
    }

    #[test]
    fn regulatory_action_equality() {
        assert_eq!(RegulatoryAction::DhpcLetter, RegulatoryAction::DhpcLetter);
        assert_ne!(RegulatoryAction::Withdrawal, RegulatoryAction::Rems);
    }

    #[test]
    fn prevention_triggers_rmp_on_low_benefit_risk() {
        let system = TestPvSystem;
        let assessment = AssessmentResult {
            naranjo_score: Some(7),
            who_umc_category: Some("Probable".into()),
            rucam_score: None,
            causality: CausalityLevel::Probable,
            benefit_risk_ratio: Some(0.5),
        };
        let prevention = system.prevent(&assessment);
        assert!(prevention.risk_management_required);
    }

    #[test]
    fn prevention_no_rmp_on_favorable_ratio() {
        let system = TestPvSystem;
        let assessment = AssessmentResult {
            naranjo_score: Some(3),
            who_umc_category: Some("Possible".into()),
            rucam_score: None,
            causality: CausalityLevel::Possible,
            benefit_risk_ratio: Some(5.0),
        };
        let prevention = system.prevent(&assessment);
        assert!(!prevention.risk_management_required);
    }
}
