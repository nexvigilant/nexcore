//! # Harm Synthesis — PV-Domain Adversarial Test Generation
//!
//! Extends the phenotype mutation engine with pharmacovigilance-specific
//! harm scenarios. Generates adversarial test fixtures that simulate
//! real-world data integrity failures in drug safety records.
//!
//! ## Innovation Scan 001 — Goal 4 (Score: 7.85)
//!
//! ```text
//! HarmScenario → PvHarmMutation → Phenotype { data, expected_drifts }
//!                                      ↓
//!                      pv_contracts::evaluate_drift → PvDriftAction
//! ```
//!
//! ## ToV Alignment: V4 Safety Manifold
//! d(s) > 0 — these mutations simulate the exact failure modes that
//! threaten patient safety, ensuring detection systems catch them.
//!
//! ## Tier: T2-C (∂ + μ + κ + N + →)

use nexcore_ribosome::DriftType;
use serde::{Deserialize, Serialize};

// ─── Harm Scenarios ──────────────────────────────────────────────────────────

/// PV-domain harm scenarios that map to real-world data integrity failures.
///
/// Each scenario represents a specific way drug safety data can be
/// corrupted, mislabeled, or manipulated — and what detection systems
/// should catch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HarmScenario {
    /// Adverse event term replaced with a less severe synonym.
    /// Example: "hepatotoxicity" → "nausea" (severity downgrade).
    /// Impact: Underreporting of serious adverse events.
    SeverityDowngrade,

    /// Seriousness criteria fields removed or set to false.
    /// Example: `is_serious: true` → field missing.
    /// Impact: Serious cases not flagged for expedited reporting.
    MissingSeriousness,

    /// Drug name altered to a different substance.
    /// Example: "warfarin" → "aspirin" (attribution error).
    /// Impact: Signal attributed to wrong drug.
    DrugMisattribution,

    /// Case count artificially reduced.
    /// Example: `case_count: 47` → `case_count: 3`.
    /// Impact: PRR/ROR denominator manipulation, signal suppression.
    CaseCountDeflation,

    /// Duplicate records injected with slightly different keys.
    /// Example: Same case, different `case_id` values.
    /// Impact: Artificially inflated signal strength.
    DuplicateInjection,

    /// Narrative/description field truncated below useful length.
    /// Example: 500-char narrative → 10-char stub.
    /// Impact: Loss of clinical context for causality assessment.
    NarrativeTruncation,

    /// Reporting date shifted to avoid time-based signal windows.
    /// Example: `report_date: 2025-01-15` → `report_date: 2023-01-15`.
    /// Impact: Case excluded from temporal signal analysis.
    TemporalShift,

    /// Outcome field changed to mask fatal outcomes.
    /// Example: `outcome: "death"` → `outcome: "recovered"`.
    /// Impact: Fatal cases invisible in mortality signal detection.
    OutcomeMasking,
}

impl HarmScenario {
    /// All available harm scenarios.
    pub const ALL: &[Self] = &[
        Self::SeverityDowngrade,
        Self::MissingSeriousness,
        Self::DrugMisattribution,
        Self::CaseCountDeflation,
        Self::DuplicateInjection,
        Self::NarrativeTruncation,
        Self::TemporalShift,
        Self::OutcomeMasking,
    ];

    /// Which drift types this scenario should trigger in the ribosome.
    #[must_use]
    pub fn expected_drift_types(self) -> Vec<DriftType> {
        match self {
            Self::SeverityDowngrade
            | Self::DrugMisattribution
            | Self::TemporalShift
            | Self::OutcomeMasking => vec![DriftType::TypeMismatch],
            Self::MissingSeriousness => vec![DriftType::MissingField],
            Self::CaseCountDeflation => vec![DriftType::RangeContraction],
            Self::DuplicateInjection => vec![DriftType::ExtraField, DriftType::ArraySizeChange],
            Self::NarrativeTruncation => vec![DriftType::LengthChange],
        }
    }

    /// Patient safety impact level (1-5).
    /// 5 = directly threatens patient life.
    #[must_use]
    pub const fn safety_impact(&self) -> u8 {
        match self {
            Self::OutcomeMasking => 5,
            Self::SeverityDowngrade | Self::MissingSeriousness | Self::DrugMisattribution => 4,
            Self::CaseCountDeflation | Self::TemporalShift => 3,
            Self::DuplicateInjection | Self::NarrativeTruncation => 2,
        }
    }

    /// Human-readable label.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::SeverityDowngrade => "Severity Downgrade",
            Self::MissingSeriousness => "Missing Seriousness",
            Self::DrugMisattribution => "Drug Misattribution",
            Self::CaseCountDeflation => "Case Count Deflation",
            Self::DuplicateInjection => "Duplicate Injection",
            Self::NarrativeTruncation => "Narrative Truncation",
            Self::TemporalShift => "Temporal Shift",
            Self::OutcomeMasking => "Outcome Masking",
        }
    }
}

impl std::fmt::Display for HarmScenario {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

// ─── PV Harm Mutation ────────────────────────────────────────────────────────

/// A PV-domain mutation with full traceability.
///
/// Records exactly what was changed, why, and what the detection
/// system should flag.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PvHarmMutation {
    /// Which harm scenario this mutation simulates.
    pub scenario: HarmScenario,
    /// Which field was targeted.
    pub target_field: String,
    /// Original value (before mutation).
    pub original_value: serde_json::Value,
    /// Mutated value (after mutation).
    pub mutated_value: serde_json::Value,
    /// Expected patient safety impact (1-5).
    pub safety_impact: u8,
    /// Expected drift types the ribosome should detect.
    pub expected_drifts: Vec<DriftType>,
}

// ─── PV Harm Phenotype ───────────────────────────────────────────────────────

/// A PV-domain phenotype: mutated drug safety record with harm metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PvHarmPhenotype {
    /// The mutated FAERS-like record.
    pub data: serde_json::Value,
    /// All harm mutations applied.
    pub mutations: Vec<PvHarmMutation>,
    /// Aggregate safety impact (max of all mutations).
    pub max_safety_impact: u8,
}

// ─── Baseline Record ─────────────────────────────────────────────────────────

/// Generate a baseline FAERS-like drug safety record.
///
/// This represents a "healthy" record with all fields present and valid.
/// Mutations are applied against this baseline.
#[must_use]
pub fn baseline_drug_record() -> serde_json::Value {
    serde_json::json!({
        "case_id": "CASE-2025-00001",
        "report_date": "2025-06-15",
        "drug_name": "warfarin",
        "drug_role": "primary_suspect",
        "indication": "atrial_fibrillation",
        "adverse_event": "hepatotoxicity",
        "adverse_event_code": "10019851",
        "is_serious": true,
        "seriousness_criteria": {
            "death": false,
            "life_threatening": true,
            "hospitalization": true,
            "disability": false,
            "congenital_anomaly": false,
            "other_medically_important": true
        },
        "outcome": "hospitalization",
        "case_count": 47,
        "reporter_type": "healthcare_professional",
        "narrative": "A 67-year-old male patient on warfarin therapy for atrial fibrillation presented with elevated liver enzymes (ALT 5x ULN, AST 4x ULN) after 3 weeks of treatment. Warfarin was discontinued and liver function returned to normal within 2 weeks. Positive dechallenge supports causal relationship.",
        "age": 67,
        "sex": "male",
        "weight_kg": 82.5
    })
}

// ─── Harm Synthesis Engine ───────────────────────────────────────────────────

/// Apply a single harm scenario to a baseline record.
///
/// Returns a `PvHarmPhenotype` with the mutation applied and full
/// traceability metadata.
#[must_use]
pub fn synthesize_harm(baseline: &serde_json::Value, scenario: HarmScenario) -> PvHarmPhenotype {
    let mut data = baseline.clone();
    let mutation = apply_harm_mutation(&mut data, scenario);
    let safety_impact = mutation.safety_impact;

    PvHarmPhenotype {
        data,
        mutations: vec![mutation],
        max_safety_impact: safety_impact,
    }
}

/// Apply all harm scenarios to a baseline record, producing one phenotype each.
#[must_use]
pub fn synthesize_all_harms(baseline: &serde_json::Value) -> Vec<PvHarmPhenotype> {
    HarmScenario::ALL
        .iter()
        .map(|s| synthesize_harm(baseline, *s))
        .collect()
}

/// Apply multiple harm scenarios to a single record (compound mutation).
///
/// Produces a single record with multiple mutations — simulates
/// sophisticated data manipulation.
#[must_use]
pub fn synthesize_compound_harm(
    baseline: &serde_json::Value,
    scenarios: &[HarmScenario],
) -> PvHarmPhenotype {
    let mut data = baseline.clone();
    let mut mutations = Vec::with_capacity(scenarios.len());
    let mut max_impact = 0u8;

    for scenario in scenarios {
        let mutation = apply_harm_mutation(&mut data, *scenario);
        if mutation.safety_impact > max_impact {
            max_impact = mutation.safety_impact;
        }
        mutations.push(mutation);
    }

    PvHarmPhenotype {
        data,
        mutations,
        max_safety_impact: max_impact,
    }
}

/// Core dispatch: apply a specific harm mutation to a mutable JSON value.
fn apply_harm_mutation(data: &mut serde_json::Value, scenario: HarmScenario) -> PvHarmMutation {
    match scenario {
        HarmScenario::SeverityDowngrade => severity_downgrade(data),
        HarmScenario::MissingSeriousness => missing_seriousness(data),
        HarmScenario::DrugMisattribution => drug_misattribution(data),
        HarmScenario::CaseCountDeflation => case_count_deflation(data),
        HarmScenario::DuplicateInjection => duplicate_injection(data),
        HarmScenario::NarrativeTruncation => narrative_truncation(data),
        HarmScenario::TemporalShift => temporal_shift(data),
        HarmScenario::OutcomeMasking => outcome_masking(data),
    }
}

// ─── Mutation Implementations ────────────────────────────────────────────────

fn severity_downgrade(data: &mut serde_json::Value) -> PvHarmMutation {
    let original = data
        .get("adverse_event")
        .cloned()
        .unwrap_or(serde_json::Value::Null);

    // Replace serious AE term with a mild one
    if let Some(obj) = data.as_object_mut() {
        obj.insert("adverse_event".to_string(), serde_json::json!("nausea"));
        obj.insert(
            "adverse_event_code".to_string(),
            serde_json::json!("10028813"),
        );
    }

    PvHarmMutation {
        scenario: HarmScenario::SeverityDowngrade,
        target_field: "adverse_event".to_string(),
        original_value: original,
        mutated_value: serde_json::json!("nausea"),
        safety_impact: HarmScenario::SeverityDowngrade.safety_impact(),
        expected_drifts: HarmScenario::SeverityDowngrade.expected_drift_types(),
    }
}

fn missing_seriousness(data: &mut serde_json::Value) -> PvHarmMutation {
    let original = data
        .get("is_serious")
        .cloned()
        .unwrap_or(serde_json::Value::Null);

    // Remove seriousness-related fields
    if let Some(obj) = data.as_object_mut() {
        obj.remove("is_serious");
        obj.remove("seriousness_criteria");
    }

    PvHarmMutation {
        scenario: HarmScenario::MissingSeriousness,
        target_field: "is_serious".to_string(),
        original_value: original,
        mutated_value: serde_json::Value::Null,
        safety_impact: HarmScenario::MissingSeriousness.safety_impact(),
        expected_drifts: HarmScenario::MissingSeriousness.expected_drift_types(),
    }
}

fn drug_misattribution(data: &mut serde_json::Value) -> PvHarmMutation {
    let original = data
        .get("drug_name")
        .cloned()
        .unwrap_or(serde_json::Value::Null);

    if let Some(obj) = data.as_object_mut() {
        obj.insert("drug_name".to_string(), serde_json::json!("aspirin"));
    }

    PvHarmMutation {
        scenario: HarmScenario::DrugMisattribution,
        target_field: "drug_name".to_string(),
        original_value: original,
        mutated_value: serde_json::json!("aspirin"),
        safety_impact: HarmScenario::DrugMisattribution.safety_impact(),
        expected_drifts: HarmScenario::DrugMisattribution.expected_drift_types(),
    }
}

fn case_count_deflation(data: &mut serde_json::Value) -> PvHarmMutation {
    let original = data
        .get("case_count")
        .cloned()
        .unwrap_or(serde_json::Value::Null);

    if let Some(obj) = data.as_object_mut() {
        obj.insert("case_count".to_string(), serde_json::json!(1));
    }

    PvHarmMutation {
        scenario: HarmScenario::CaseCountDeflation,
        target_field: "case_count".to_string(),
        original_value: original,
        mutated_value: serde_json::json!(1),
        safety_impact: HarmScenario::CaseCountDeflation.safety_impact(),
        expected_drifts: HarmScenario::CaseCountDeflation.expected_drift_types(),
    }
}

fn duplicate_injection(data: &mut serde_json::Value) -> PvHarmMutation {
    let original = data
        .get("case_id")
        .cloned()
        .unwrap_or(serde_json::Value::Null);

    // Add a duplicate_cases array to simulate injection
    if let Some(obj) = data.as_object_mut() {
        obj.insert(
            "duplicate_cases".to_string(),
            serde_json::json!([
                "CASE-2025-00001-DUP1",
                "CASE-2025-00001-DUP2",
                "CASE-2025-00001-DUP3"
            ]),
        );
        obj.insert("_injected_flag".to_string(), serde_json::json!(true));
    }

    PvHarmMutation {
        scenario: HarmScenario::DuplicateInjection,
        target_field: "case_id".to_string(),
        original_value: original,
        mutated_value: serde_json::json!("CASE-2025-00001 + 3 duplicates"),
        safety_impact: HarmScenario::DuplicateInjection.safety_impact(),
        expected_drifts: HarmScenario::DuplicateInjection.expected_drift_types(),
    }
}

fn narrative_truncation(data: &mut serde_json::Value) -> PvHarmMutation {
    let original = data
        .get("narrative")
        .cloned()
        .unwrap_or(serde_json::Value::Null);

    if let Some(obj) = data.as_object_mut() {
        obj.insert("narrative".to_string(), serde_json::json!("Truncated."));
    }

    PvHarmMutation {
        scenario: HarmScenario::NarrativeTruncation,
        target_field: "narrative".to_string(),
        original_value: original,
        mutated_value: serde_json::json!("Truncated."),
        safety_impact: HarmScenario::NarrativeTruncation.safety_impact(),
        expected_drifts: HarmScenario::NarrativeTruncation.expected_drift_types(),
    }
}

fn temporal_shift(data: &mut serde_json::Value) -> PvHarmMutation {
    let original = data
        .get("report_date")
        .cloned()
        .unwrap_or(serde_json::Value::Null);

    if let Some(obj) = data.as_object_mut() {
        obj.insert("report_date".to_string(), serde_json::json!("2020-01-01"));
    }

    PvHarmMutation {
        scenario: HarmScenario::TemporalShift,
        target_field: "report_date".to_string(),
        original_value: original,
        mutated_value: serde_json::json!("2020-01-01"),
        safety_impact: HarmScenario::TemporalShift.safety_impact(),
        expected_drifts: HarmScenario::TemporalShift.expected_drift_types(),
    }
}

fn outcome_masking(data: &mut serde_json::Value) -> PvHarmMutation {
    let original = data
        .get("outcome")
        .cloned()
        .unwrap_or(serde_json::Value::Null);

    if let Some(obj) = data.as_object_mut() {
        obj.insert("outcome".to_string(), serde_json::json!("recovered"));
    }

    PvHarmMutation {
        scenario: HarmScenario::OutcomeMasking,
        target_field: "outcome".to_string(),
        original_value: original,
        mutated_value: serde_json::json!("recovered"),
        safety_impact: HarmScenario::OutcomeMasking.safety_impact(),
        expected_drifts: HarmScenario::OutcomeMasking.expected_drift_types(),
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> serde_json::Value {
        baseline_drug_record()
    }

    // ── Scenario coverage ─────────────────────────────────────────────────

    #[test]
    fn test_all_scenarios_count() {
        assert_eq!(HarmScenario::ALL.len(), 8);
    }

    #[test]
    fn test_all_scenarios_have_expected_drifts() {
        for scenario in HarmScenario::ALL {
            let drifts = scenario.expected_drift_types();
            assert!(!drifts.is_empty(), "{scenario} has no expected drift types");
        }
    }

    #[test]
    fn test_all_scenarios_have_safety_impact() {
        for scenario in HarmScenario::ALL {
            let impact = scenario.safety_impact();
            assert!(
                (1..=5).contains(&impact),
                "{scenario} has invalid safety impact: {impact}"
            );
        }
    }

    // ── Baseline record ───────────────────────────────────────────────────

    #[test]
    fn test_baseline_has_required_fields() {
        let b = baseline();
        assert!(b.get("case_id").is_some());
        assert!(b.get("drug_name").is_some());
        assert!(b.get("adverse_event").is_some());
        assert!(b.get("is_serious").is_some());
        assert!(b.get("outcome").is_some());
        assert!(b.get("case_count").is_some());
        assert!(b.get("narrative").is_some());
        assert!(b.get("report_date").is_some());
    }

    #[test]
    fn test_baseline_is_object() {
        assert!(baseline().is_object());
    }

    // ── Individual mutations ──────────────────────────────────────────────

    #[test]
    fn test_severity_downgrade() {
        let phenotype = synthesize_harm(&baseline(), HarmScenario::SeverityDowngrade);
        assert_eq!(phenotype.data["adverse_event"], "nausea");
        assert_eq!(phenotype.mutations[0].target_field, "adverse_event");
        assert_eq!(phenotype.max_safety_impact, 4);
    }

    #[test]
    fn test_missing_seriousness() {
        let phenotype = synthesize_harm(&baseline(), HarmScenario::MissingSeriousness);
        assert!(phenotype.data.get("is_serious").is_none());
        assert!(phenotype.data.get("seriousness_criteria").is_none());
    }

    #[test]
    fn test_drug_misattribution() {
        let phenotype = synthesize_harm(&baseline(), HarmScenario::DrugMisattribution);
        assert_eq!(phenotype.data["drug_name"], "aspirin");
        assert_ne!(
            phenotype.mutations[0].original_value,
            phenotype.mutations[0].mutated_value
        );
    }

    #[test]
    fn test_case_count_deflation() {
        let phenotype = synthesize_harm(&baseline(), HarmScenario::CaseCountDeflation);
        assert_eq!(phenotype.data["case_count"], 1);
    }

    #[test]
    fn test_duplicate_injection() {
        let phenotype = synthesize_harm(&baseline(), HarmScenario::DuplicateInjection);
        assert!(phenotype.data.get("duplicate_cases").is_some());
        assert!(phenotype.data.get("_injected_flag").is_some());
    }

    #[test]
    fn test_narrative_truncation() {
        let phenotype = synthesize_harm(&baseline(), HarmScenario::NarrativeTruncation);
        let narrative = phenotype.data["narrative"].as_str().unwrap_or("");
        assert!(
            narrative.len() < 20,
            "Narrative should be truncated, got len={}",
            narrative.len()
        );
    }

    #[test]
    fn test_temporal_shift() {
        let phenotype = synthesize_harm(&baseline(), HarmScenario::TemporalShift);
        assert_eq!(phenotype.data["report_date"], "2020-01-01");
    }

    #[test]
    fn test_outcome_masking() {
        let phenotype = synthesize_harm(&baseline(), HarmScenario::OutcomeMasking);
        assert_eq!(phenotype.data["outcome"], "recovered");
        assert_eq!(phenotype.max_safety_impact, 5);
    }

    // ── Batch operations ──────────────────────────────────────────────────

    #[test]
    fn test_synthesize_all_harms() {
        let phenotypes = synthesize_all_harms(&baseline());
        assert_eq!(phenotypes.len(), 8);
    }

    #[test]
    fn test_compound_harm() {
        let phenotype = synthesize_compound_harm(
            &baseline(),
            &[
                HarmScenario::SeverityDowngrade,
                HarmScenario::OutcomeMasking,
            ],
        );
        assert_eq!(phenotype.mutations.len(), 2);
        assert_eq!(phenotype.max_safety_impact, 5); // OutcomeMasking is 5
        assert_eq!(phenotype.data["adverse_event"], "nausea");
        assert_eq!(phenotype.data["outcome"], "recovered");
    }

    #[test]
    fn test_compound_harm_all_scenarios() {
        let phenotype = synthesize_compound_harm(&baseline(), HarmScenario::ALL);
        assert_eq!(phenotype.mutations.len(), 8);
        assert_eq!(phenotype.max_safety_impact, 5);
    }

    // ── Traceability ──────────────────────────────────────────────────────

    #[test]
    fn test_mutation_records_original_value() {
        let phenotype = synthesize_harm(&baseline(), HarmScenario::DrugMisattribution);
        assert_eq!(phenotype.mutations[0].original_value, "warfarin");
    }

    #[test]
    fn test_mutation_records_mutated_value() {
        let phenotype = synthesize_harm(&baseline(), HarmScenario::DrugMisattribution);
        assert_eq!(phenotype.mutations[0].mutated_value, "aspirin");
    }

    #[test]
    fn test_mutation_records_scenario() {
        let phenotype = synthesize_harm(&baseline(), HarmScenario::TemporalShift);
        assert_eq!(phenotype.mutations[0].scenario, HarmScenario::TemporalShift);
    }

    // ── Display ───────────────────────────────────────────────────────────

    #[test]
    fn test_harm_scenario_display() {
        assert_eq!(HarmScenario::OutcomeMasking.to_string(), "Outcome Masking");
        assert_eq!(
            HarmScenario::SeverityDowngrade.to_string(),
            "Severity Downgrade"
        );
    }

    // ── Safety impact ordering ────────────────────────────────────────────

    #[test]
    fn test_outcome_masking_is_highest_impact() {
        let max_impact = HarmScenario::ALL
            .iter()
            .map(|s| s.safety_impact())
            .max()
            .unwrap_or(0);
        assert_eq!(max_impact, 5);
        assert_eq!(HarmScenario::OutcomeMasking.safety_impact(), max_impact);
    }

    #[test]
    fn test_narrative_truncation_is_lower_impact() {
        assert!(
            HarmScenario::NarrativeTruncation.safety_impact()
                < HarmScenario::OutcomeMasking.safety_impact()
        );
    }
}
