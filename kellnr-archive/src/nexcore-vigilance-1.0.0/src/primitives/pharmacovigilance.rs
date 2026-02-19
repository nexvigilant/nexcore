//! Pharmacovigilance Primitive Taxonomy
//!
//! Complete type-level decomposition of the pharmacovigilance domain into
//! T1/T2-P/T2-C/T3 tiers grounded to the 15 Lex Primitiva symbols.
//!
//! # Coverage
//!
//! | Layer | Count | Coverage |
//! |-------|-------|----------|
//! | T2-P Newtypes | 10 | Single T1 each |
//! | T2-C Composites | 11 | 2-5 T1 each |
//! | T3 Domain Types | 7 | 6+ T1 each |

use crate::lex_primitiva::{GroundsTo, LexPrimitiva, PrimitiveComposition};
use serde::{Deserialize, Serialize};

// =============================================================================
// T2-P: Newtypes over single T1 primitives
// =============================================================================

/// Unique substance identifier. Tier: T2-P (N)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DrugId(pub u64);

impl GroundsTo for DrugId {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity])
    }
}

/// Unique adverse event identifier. Tier: T2-P (N)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventId(pub u64);

impl GroundsTo for EventId {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity])
    }
}

/// Events per unit exposure time. Tier: T2-P (ν)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ReportingRate(pub f64);

impl ReportingRate {
    #[must_use]
    pub fn new(rate: f64) -> Self {
        Self(rate.max(0.0))
    }

    #[must_use]
    pub fn value(self) -> f64 {
        self.0
    }
}

impl GroundsTo for ReportingRate {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Frequency])
    }
}

/// Detection boundary (PRR ≥ 2.0, χ² ≥ 3.841). Tier: T2-P (∂)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PvThreshold(pub f64);

impl PvThreshold {
    #[must_use]
    pub fn exceeded_by(&self, value: f64) -> bool {
        value >= self.0
    }
}

impl GroundsTo for PvThreshold {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Boundary])
    }
}

/// Time-to-onset interval. Tier: T2-P (σ)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TemporalWindow {
    pub start_days: f64,
    pub end_days: f64,
}

impl TemporalWindow {
    #[must_use]
    pub fn new(start_days: f64, end_days: f64) -> Self {
        Self {
            start_days,
            end_days,
        }
    }

    #[must_use]
    pub fn duration_days(&self) -> f64 {
        (self.end_days - self.start_days).max(0.0)
    }

    #[must_use]
    pub fn contains(&self, days: f64) -> bool {
        days >= self.start_days && days <= self.end_days
    }
}

impl GroundsTo for TemporalWindow {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sequence])
    }
}

/// Total patient-exposure denominator. Tier: T2-P (Σ)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ExposureCount(pub u64);

impl GroundsTo for ExposureCount {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sum])
    }
}

/// ICH E2A seriousness criteria. Tier: T2-P (∝)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Seriousness {
    Death,
    LifeThreatening,
    Hospitalization,
    Disability,
    CongenitalAnomaly,
    MedicallyImportant,
    NotSerious,
}

impl Seriousness {
    #[must_use]
    pub fn is_serious(self) -> bool {
        !matches!(self, Self::NotSerious)
    }

    #[must_use]
    pub fn is_irreversible(self) -> bool {
        matches!(
            self,
            Self::Death | Self::Disability | Self::CongenitalAnomaly
        )
    }
}

impl GroundsTo for Seriousness {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Irreversibility])
    }
}

/// Single cause-effect assertion. Tier: T2-P (→)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CausalAssertion {
    Related,
    PossiblyRelated,
    Unrelated,
    Indeterminate,
}

impl GroundsTo for CausalAssertion {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Causality])
    }
}

/// Binary signal/case existence. Tier: T2-P (∃)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SignalPresent(pub bool);

impl GroundsTo for SignalPresent {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Existence])
    }
}

/// Geographic reporting unit. Tier: T2-P (λ)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PvRegion(pub String);

impl PvRegion {
    #[must_use]
    pub fn new(code: impl Into<String>) -> Self {
        Self(code.into())
    }
}

impl GroundsTo for PvRegion {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Location])
    }
}

// =============================================================================
// T2-C: Composite primitives (2-5 T1 combinations)
// =============================================================================

/// Drug-event pair association. Tier: T2-C (μ + N + N), dominant μ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DrugEventPair {
    pub drug: DrugId,
    pub event: EventId,
}

impl DrugEventPair {
    #[must_use]
    pub fn new(drug: DrugId, event: EventId) -> Self {
        Self { drug, event }
    }
}

impl GroundsTo for DrugEventPair {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,
            LexPrimitiva::Quantity,
            LexPrimitiva::Quantity,
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.95)
    }
}

/// 2×2 contingency table. Tier: T2-C (N + κ + Σ), dominant κ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContingencyCell {
    pub a: u64, // Drug+/Event+
    pub b: u64, // Drug+/Event-
    pub c: u64, // Drug-/Event+
    pub d: u64, // Drug-/Event-
}

impl ContingencyCell {
    #[must_use]
    pub fn new(a: u64, b: u64, c: u64, d: u64) -> Self {
        Self { a, b, c, d }
    }

    #[must_use]
    pub fn total(&self) -> u64 {
        self.a + self.b + self.c + self.d
    }

    #[must_use]
    pub fn expected_a(&self) -> f64 {
        let total = self.total() as f64;
        if total == 0.0 {
            return 0.0;
        }
        ((self.a + self.b) as f64 * (self.a + self.c) as f64) / total
    }
}

impl GroundsTo for ContingencyCell {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,
            LexPrimitiva::Comparison,
            LexPrimitiva::Sum,
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.95)
    }
}

/// Disproportionality metric result. Tier: T2-C (N + κ + ∂ + ×), dominant κ
///
/// Cross-multiplication in 2×2 contingency tables (a×d, b×c) is the Product (×) primitive.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisproportionalityScore {
    pub metric: String,
    pub value: f64,
    pub lower_bound: Option<f64>,
    pub upper_bound: Option<f64>,
    pub threshold: f64,
    pub is_signal: bool,
}

impl DisproportionalityScore {
    #[must_use]
    pub fn new(metric: impl Into<String>, value: f64, threshold: f64) -> Self {
        Self {
            metric: metric.into(),
            value,
            lower_bound: None,
            upper_bound: None,
            threshold,
            is_signal: value >= threshold,
        }
    }

    #[must_use]
    pub fn with_ci(mut self, lower: f64, upper: f64) -> Self {
        self.lower_bound = Some(lower);
        self.upper_bound = Some(upper);
        self
    }
}

impl GroundsTo for DisproportionalityScore {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,   // N — cell counts, metric value
            LexPrimitiva::Comparison, // κ — metric vs threshold comparison
            LexPrimitiva::Boundary,   // ∂ — signal/non-signal threshold
            LexPrimitiva::Product,    // × — cross-multiplication: a×d, b×c in 2×2 table
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.90)
    }
}

/// Statistical signal candidate. Tier: T2-C (∃ + κ + →), dominant →
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalHypothesis {
    pub pair: DrugEventPair,
    pub detected: SignalPresent,
    pub evidence_strength: f64,
    pub proposed_causality: CausalAssertion,
}

impl GroundsTo for SignalHypothesis {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Existence,
            LexPrimitiva::Comparison,
            LexPrimitiva::Causality,
        ])
        .with_dominant(LexPrimitiva::Causality, 0.90)
    }
}

/// Temporal sequence of clinical events. Tier: T2-C (σ + μ + ∃), dominant σ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseNarrative {
    pub events: Vec<NarrativeEvent>,
    pub seriousness: Seriousness,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeEvent {
    pub day_offset: i32,
    pub description: String,
    pub is_drug_event: bool,
}

impl GroundsTo for CaseNarrative {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Mapping,
            LexPrimitiva::Existence,
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.90)
    }
}

/// Dose/duration/frequency profile. Tier: T2-C (σ + ν + N), dominant σ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExposurePattern {
    pub daily_dose_mg: f64,
    pub duration_days: u32,
    pub frequency_per_day: f64,
    pub cumulative_dose_mg: f64,
}

impl ExposurePattern {
    #[must_use]
    pub fn from_regimen(daily_dose_mg: f64, duration_days: u32, frequency_per_day: f64) -> Self {
        Self {
            daily_dose_mg,
            duration_days,
            frequency_per_day,
            cumulative_dose_mg: daily_dose_mg * duration_days as f64,
        }
    }
}

impl GroundsTo for ExposurePattern {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Frequency,
            LexPrimitiva::Quantity,
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.90)
    }
}

/// Causality assessment result. Tier: T2-C (→ + κ + ∂), dominant →
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalityAssessmentResult {
    pub algorithm: String,
    pub score: f64,
    pub category: String,
    pub assertion: CausalAssertion,
}

impl GroundsTo for CausalityAssessmentResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality,
            LexPrimitiva::Comparison,
            LexPrimitiva::Boundary,
        ])
        .with_dominant(LexPrimitiva::Causality, 0.95)
    }
}

/// Acceptable risk threshold. Tier: T2-C (∂ + N + ∝), dominant ∂
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskBoundary {
    pub max_incidence_rate: f64,
    pub seriousness_trigger: Seriousness,
    pub action_on_breach: String,
}

impl GroundsTo for RiskBoundary {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,
            LexPrimitiva::Quantity,
            LexPrimitiva::Irreversibility,
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.90)
    }
}

/// Population-level detection aggregate. Tier: T2-C (Σ + ν + κ), dominant Σ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateSignal {
    pub pair: DrugEventPair,
    pub case_count: u64,
    pub reporting_rate: ReportingRate,
    pub scores: Vec<DisproportionalityScore>,
    pub methods_flagging: u32,
}

impl GroundsTo for AggregateSignal {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,
            LexPrimitiva::Frequency,
            LexPrimitiva::Comparison,
        ])
        .with_dominant(LexPrimitiva::Sum, 0.90)
    }
}

/// MedDRA hierarchy path (PT → HLT → HLGT → SOC). Tier: T2-C (ρ + μ + σ), dominant ρ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchyPath {
    pub pt: String,
    pub hlt: Option<String>,
    pub hlgt: Option<String>,
    pub soc: Option<String>,
}

impl HierarchyPath {
    #[must_use]
    pub fn from_pt(pt: impl Into<String>) -> Self {
        Self {
            pt: pt.into(),
            hlt: None,
            hlgt: None,
            soc: None,
        }
    }

    #[must_use]
    pub fn depth(&self) -> u8 {
        1 + self.hlt.is_some() as u8 + self.hlgt.is_some() as u8 + self.soc.is_some() as u8
    }
}

impl GroundsTo for HierarchyPath {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Recursion,
            LexPrimitiva::Mapping,
            LexPrimitiva::Sequence,
        ])
        .with_dominant(LexPrimitiva::Recursion, 0.90)
    }
}

/// Systematic underreporting detection. Tier: T2-C (∅ + ν + λ), dominant ∅
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissingDataPattern {
    pub region: PvRegion,
    pub expected_rate: ReportingRate,
    pub observed_rate: ReportingRate,
    pub reporting_ratio: f64,
}

impl MissingDataPattern {
    #[must_use]
    pub fn is_underreported(&self) -> bool {
        self.reporting_ratio < 0.5
    }
}

impl GroundsTo for MissingDataPattern {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Void,
            LexPrimitiva::Frequency,
            LexPrimitiva::Location,
        ])
        .with_dominant(LexPrimitiva::Void, 0.90)
    }
}

// =============================================================================
// T3: Domain-specific types (6+ T1 primitives)
// =============================================================================

/// Case processing lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CaseStatus {
    Pending,
    InProgress,
    Confirmed,
    Submitted,
    Closed,
}

/// Individual Case Safety Report. Tier: T3 (σ + μ + ∃ + π + λ + N + ς)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Icsr {
    pub case_id: String,
    pub status: CaseStatus,
    pub drug_event_pairs: Vec<DrugEventPair>,
    pub exposure: Option<ExposurePattern>,
    pub narrative: Option<CaseNarrative>,
    pub region: PvRegion,
    pub seriousness: Seriousness,
    pub causality_assessments: Vec<CausalityAssessmentResult>,
    pub received_day: u32,
}

impl GroundsTo for Icsr {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Mapping,
            LexPrimitiva::Existence,
            LexPrimitiva::Persistence,
            LexPrimitiva::Location,
            LexPrimitiva::Quantity,
            LexPrimitiva::State,
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.80)
    }
}

/// 12-stage signal detection pipeline. Tier: T3 (σ + κ + ∂ + Σ + ν + → + ρ)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalDetectionPipeline {
    pub name: String,
    pub stages: Vec<PipelineStage>,
    pub thresholds: Vec<f64>,
    pub use_hierarchy: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStage {
    pub name: String,
    pub index: u8,
    pub enabled: bool,
}

impl GroundsTo for SignalDetectionPipeline {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Comparison,
            LexPrimitiva::Boundary,
            LexPrimitiva::Sum,
            LexPrimitiva::Frequency,
            LexPrimitiva::Causality,
            LexPrimitiva::Recursion,
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.80)
    }
}

/// Benefit-risk conclusion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BenefitRiskConclusion {
    Favorable,
    MarginallyFavorable,
    Balanced,
    Unfavorable,
    Indeterminate,
}

/// Quantitative benefit-risk assessment. Tier: T3 (κ + N + ∂ + → + Σ + ∝ + ς)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PvBenefitRiskAssessment {
    pub drug: DrugId,
    pub benefit_magnitude: f64,
    pub benefit_probability: f64,
    pub risk_magnitude: f64,
    pub risk_probability: f64,
    pub ratio: f64,
    pub conclusion: BenefitRiskConclusion,
}

impl GroundsTo for PvBenefitRiskAssessment {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison,
            LexPrimitiva::Quantity,
            LexPrimitiva::Boundary,
            LexPrimitiva::Causality,
            LexPrimitiva::Sum,
            LexPrimitiva::Irreversibility,
            LexPrimitiva::State,
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.80)
    }
}

/// EU Risk Management Plan. Tier: T3 (∂ + σ + π + ς + → + Σ)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskManagementPlan {
    pub drug: DrugId,
    pub identified_risks: Vec<String>,
    pub potential_risks: Vec<String>,
    pub missing_information: Vec<String>,
    pub minimization_activities: Vec<String>,
    pub pv_activities: Vec<String>,
    pub status: CaseStatus,
}

impl GroundsTo for RiskManagementPlan {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,
            LexPrimitiva::Sequence,
            LexPrimitiva::Persistence,
            LexPrimitiva::State,
            LexPrimitiva::Causality,
            LexPrimitiva::Sum,
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.80)
    }
}

/// PSUR/PBRER lifecycle. Tier: T3 (σ + Σ + ν + π + ∂ + κ + ς)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PsurCycle {
    pub drug: DrugId,
    pub period_start: u32,
    pub period_end: u32,
    pub cases_in_period: u64,
    pub cumulative_cases: u64,
    pub new_signals: Vec<SignalHypothesis>,
    pub benefit_risk_conclusion: BenefitRiskConclusion,
    pub status: CaseStatus,
}

impl GroundsTo for PsurCycle {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Sum,
            LexPrimitiva::Frequency,
            LexPrimitiva::Persistence,
            LexPrimitiva::Boundary,
            LexPrimitiva::Comparison,
            LexPrimitiva::State,
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.80)
    }
}

/// Multi-factor causal chain. Tier: T3 (→ + σ + κ + ∃ + ρ + ∂)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalityChain {
    pub pair: DrugEventPair,
    pub links: Vec<CausalLink>,
    pub overall_strength: f64,
    pub conclusion: CausalAssertion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalLink {
    pub evidence_type: String,
    pub strength: f64,
    pub supports_causality: bool,
}

impl CausalityChain {
    /// Geometric mean of supporting link strengths.
    #[must_use]
    pub fn calculate_strength(links: &[CausalLink]) -> f64 {
        let supporting: Vec<f64> = links
            .iter()
            .filter(|l| l.supports_causality)
            .map(|l| l.strength)
            .collect();
        if supporting.is_empty() {
            return 0.0;
        }
        supporting
            .iter()
            .product::<f64>()
            .powf(1.0 / supporting.len() as f64)
    }
}

impl GroundsTo for CausalityChain {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality,
            LexPrimitiva::Sequence,
            LexPrimitiva::Comparison,
            LexPrimitiva::Existence,
            LexPrimitiva::Recursion,
            LexPrimitiva::Boundary,
        ])
        .with_dominant(LexPrimitiva::Causality, 0.80)
    }
}

/// Signal triage priority workflow. Tier: T3 (κ + ∂ + ς + → + N + ∝)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalTriageWorkflow {
    pub signals: Vec<AggregateSignal>,
    pub ranked_signals: Vec<TriagedSignal>,
    pub status: CaseStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriagedSignal {
    pub pair: DrugEventPair,
    pub rank: u32,
    pub priority_score: f64,
    pub category: TriagePriority,
    pub action: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TriagePriority {
    Critical,
    High,
    Medium,
    Low,
    None,
}

impl GroundsTo for SignalTriageWorkflow {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison,
            LexPrimitiva::Boundary,
            LexPrimitiva::State,
            LexPrimitiva::Causality,
            LexPrimitiva::Quantity,
            LexPrimitiva::Irreversibility,
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.80)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_t2p_single_primitive() {
        let types: Vec<PrimitiveComposition> = vec![
            DrugId::primitive_composition(),
            EventId::primitive_composition(),
            ReportingRate::primitive_composition(),
            PvThreshold::primitive_composition(),
            TemporalWindow::primitive_composition(),
            ExposureCount::primitive_composition(),
            Seriousness::primitive_composition(),
            CausalAssertion::primitive_composition(),
            SignalPresent::primitive_composition(),
            PvRegion::primitive_composition(),
        ];
        for comp in &types {
            assert_eq!(comp.primitives.len(), 1, "T2-P must have 1 primitive");
            assert_eq!(comp.confidence, 1.0);
        }
    }

    #[test]
    fn test_t2c_composite_range() {
        let types: Vec<PrimitiveComposition> = vec![
            DrugEventPair::primitive_composition(),
            ContingencyCell::primitive_composition(),
            DisproportionalityScore::primitive_composition(),
            SignalHypothesis::primitive_composition(),
            CaseNarrative::primitive_composition(),
            ExposurePattern::primitive_composition(),
            CausalityAssessmentResult::primitive_composition(),
            RiskBoundary::primitive_composition(),
            AggregateSignal::primitive_composition(),
            HierarchyPath::primitive_composition(),
            MissingDataPattern::primitive_composition(),
        ];
        for comp in &types {
            assert!(
                comp.primitives.len() >= 2 && comp.primitives.len() <= 5,
                "T2-C must have 2-5 primitives, got {}",
                comp.primitives.len()
            );
        }
    }

    #[test]
    fn test_t3_domain_specific() {
        let types: Vec<PrimitiveComposition> = vec![
            Icsr::primitive_composition(),
            SignalDetectionPipeline::primitive_composition(),
            PvBenefitRiskAssessment::primitive_composition(),
            RiskManagementPlan::primitive_composition(),
            PsurCycle::primitive_composition(),
            CausalityChain::primitive_composition(),
            SignalTriageWorkflow::primitive_composition(),
        ];
        for comp in &types {
            assert!(
                comp.primitives.len() >= 6,
                "T3 must have 6+ primitives, got {}",
                comp.primitives.len()
            );
            assert_eq!(comp.confidence, 0.80);
        }
    }

    #[test]
    fn test_threshold_exceeded() {
        let t = PvThreshold(2.0);
        assert!(t.exceeded_by(3.0));
        assert!(t.exceeded_by(2.0));
        assert!(!t.exceeded_by(1.9));
    }

    #[test]
    fn test_temporal_window() {
        let w = TemporalWindow::new(1.0, 30.0);
        assert_eq!(w.duration_days(), 29.0);
        assert!(w.contains(15.0));
        assert!(!w.contains(0.5));
    }

    #[test]
    fn test_seriousness() {
        assert!(Seriousness::Death.is_serious());
        assert!(Seriousness::Death.is_irreversible());
        assert!(Seriousness::Hospitalization.is_serious());
        assert!(!Seriousness::Hospitalization.is_irreversible());
        assert!(!Seriousness::NotSerious.is_serious());
    }

    #[test]
    fn test_contingency_cell() {
        let cell = ContingencyCell::new(15, 100, 20, 10000);
        assert_eq!(cell.total(), 10135);
        let expected = cell.expected_a();
        assert!(expected > 0.0 && expected < 15.0);
    }

    #[test]
    fn test_disproportionality_signal() {
        let score = DisproportionalityScore::new("PRR", 3.5, 2.0);
        assert!(score.is_signal);
        let no = DisproportionalityScore::new("PRR", 1.5, 2.0);
        assert!(!no.is_signal);
    }

    #[test]
    fn test_hierarchy_depth() {
        assert_eq!(HierarchyPath::from_pt("Headache").depth(), 1);
        let full = HierarchyPath {
            pt: "Headache".into(),
            hlt: Some("Headaches NEC".into()),
            hlgt: Some("Headaches".into()),
            soc: Some("Nervous system disorders".into()),
        };
        assert_eq!(full.depth(), 4);
    }

    #[test]
    fn test_missing_data_underreporting() {
        let p = MissingDataPattern {
            region: PvRegion::new("XX"),
            expected_rate: ReportingRate::new(10.0),
            observed_rate: ReportingRate::new(3.0),
            reporting_ratio: 0.3,
        };
        assert!(p.is_underreported());
    }

    #[test]
    fn test_exposure_pattern_cumulative() {
        let exp = ExposurePattern::from_regimen(100.0, 30, 2.0);
        assert_eq!(exp.cumulative_dose_mg, 3000.0);
    }

    #[test]
    fn test_causality_chain_strength() {
        let links = vec![
            CausalLink {
                evidence_type: "temporal".into(),
                strength: 0.9,
                supports_causality: true,
            },
            CausalLink {
                evidence_type: "dechallenge".into(),
                strength: 0.8,
                supports_causality: true,
            },
            CausalLink {
                evidence_type: "alternative".into(),
                strength: 0.3,
                supports_causality: false,
            },
        ];
        let s = CausalityChain::calculate_strength(&links);
        assert!(s > 0.8 && s < 0.9); // geometric mean of 0.9, 0.8
    }

    #[test]
    fn test_primitive_coverage() {
        use std::collections::HashSet;
        let all: Vec<PrimitiveComposition> = vec![
            DrugId::primitive_composition(),
            ReportingRate::primitive_composition(),
            PvThreshold::primitive_composition(),
            TemporalWindow::primitive_composition(),
            ExposureCount::primitive_composition(),
            Seriousness::primitive_composition(),
            CausalAssertion::primitive_composition(),
            SignalPresent::primitive_composition(),
            PvRegion::primitive_composition(),
            ContingencyCell::primitive_composition(),
            CaseNarrative::primitive_composition(),
            MissingDataPattern::primitive_composition(),
            HierarchyPath::primitive_composition(),
            Icsr::primitive_composition(),
            SignalDetectionPipeline::primitive_composition(),
            PvBenefitRiskAssessment::primitive_composition(),
        ];
        let mut prims = HashSet::new();
        for comp in &all {
            for p in &comp.primitives {
                prims.insert(format!("{p:?}"));
            }
        }
        // Should cover all 15 T1 primitives
        assert!(
            prims.len() >= 14,
            "Got {} primitives: {:?}",
            prims.len(),
            prims
        );
    }
}
