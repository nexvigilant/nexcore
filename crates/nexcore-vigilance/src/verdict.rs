//! # Verdict — RSK Engine/Guardian Interface Contract
//!
//! The typed driveshaft between the RSK microgram engine and the Guardian navigator.
//! The RSK engine produces `Verdict` values; Guardian absorbs them for prioritization,
//! aggregation, and routing. The engine is domain-agnostic — it just produces Verdicts.
//!
//! ## T1 Primitive Grounding
//! - Σ (Sum/Coproduct): `SignalStrength`, `CausalityLevel`, `RegulatoryAction`, `SourceType`
//! - ς (State): `Verdict` struct
//! - μ (Mapping): `from_chain_output`, `FromStr`, `Display`
//! - ∂ (Boundary): `Result<T, NexError>` at every parse boundary
//! - κ (Comparison): `match` arms in `FromStr` impls
//! - π (Persistence): `Serialize`, `Deserialize`

use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

use nexcore_error::{NexError, Result};
use serde::{Deserialize, Serialize};

// ─────────────────────────────────────────────────────────────────────────────
// Supporting Enums (Σ — Sum/Coproduct)
// ─────────────────────────────────────────────────────────────────────────────

/// Signal strength from the PV disproportionality analysis.
///
/// Maps to PRR/ROR/IC thresholds from the pv-signal-to-action chain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerdictSignalStrength {
    /// PRR ≥ 2, IC ≥ 0, N ≥ 3 — strong pharmacovigilance signal
    Strong,
    /// PRR ≥ 1.5, suggestive but not definitive
    Moderate,
    /// Below standard thresholds but numerically present
    Weak,
    /// Signal collapsed — contradicted by denominator data
    Collapsed,
    /// No signal detected at any threshold
    NoSignal,
}

impl fmt::Display for VerdictSignalStrength {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Strong => write!(f, "strong"),
            Self::Moderate => write!(f, "moderate"),
            Self::Weak => write!(f, "weak"),
            Self::Collapsed => write!(f, "collapsed"),
            Self::NoSignal => write!(f, "no_signal"),
        }
    }
}

impl FromStr for VerdictSignalStrength {
    type Err = NexError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "strong" => Ok(Self::Strong),
            "moderate" => Ok(Self::Moderate),
            "weak" => Ok(Self::Weak),
            "collapsed" => Ok(Self::Collapsed),
            "no_signal" | "nosignal" | "none" => Ok(Self::NoSignal),
            other => Err(NexError::msg(format!("unknown signal strength: {other:?}"))),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────

/// WHO-UMC causality classification for adverse event assessment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CausalityLevel {
    /// Definite causal relationship — temporal plausibility + dechallenge + rechallenge
    Definite,
    /// Probable — temporal plausibility + dechallenge, no rechallenge
    Probable,
    /// Possible — temporal plausibility, insufficient dechallenge data
    Possible,
    /// Unlikely — temporal relationship doubtful; other drugs/disease explain
    Unlikely,
    /// Conditional/Unclassified — more data needed before final classification
    Conditional,
    /// Unassessable — insufficient data to classify
    Unassessable,
}

impl fmt::Display for CausalityLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Definite => write!(f, "definite"),
            Self::Probable => write!(f, "probable"),
            Self::Possible => write!(f, "possible"),
            Self::Unlikely => write!(f, "unlikely"),
            Self::Conditional => write!(f, "conditional"),
            Self::Unassessable => write!(f, "unassessable"),
        }
    }
}

impl FromStr for CausalityLevel {
    type Err = NexError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "definite" => Ok(Self::Definite),
            "probable" => Ok(Self::Probable),
            "possible" => Ok(Self::Possible),
            "unlikely" => Ok(Self::Unlikely),
            "conditional" | "unclassified" => Ok(Self::Conditional),
            "unassessable" | "unclassifiable" => Ok(Self::Unassessable),
            other => Err(NexError::msg(format!("unknown causality level: {other:?}"))),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────

/// Regulatory action recommended by the PV decision chain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RegulatoryAction {
    /// Expedited (15-day) MedWatch/EudraVigilance report required
    ExpeditedReport,
    /// Add to active surveillance program; periodic signal review
    Monitor,
    /// Log in periodic safety report; no immediate action required
    Document,
    /// Below threshold for any regulatory action
    NoAction,
}

impl fmt::Display for RegulatoryAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ExpeditedReport => write!(f, "expedited_report"),
            Self::Monitor => write!(f, "monitor"),
            Self::Document => write!(f, "document"),
            Self::NoAction => write!(f, "no_action"),
        }
    }
}

impl FromStr for RegulatoryAction {
    type Err = NexError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "expedited_report" | "expedited" | "15day" | "15-day" => Ok(Self::ExpeditedReport),
            "monitor" | "monitoring" => Ok(Self::Monitor),
            "document" | "psur" | "pbrer" => Ok(Self::Document),
            "no_action" | "noaction" | "none" => Ok(Self::NoAction),
            other => Err(NexError::msg(format!(
                "unknown regulatory action: {other:?}"
            ))),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────

/// Source type governing under-reporting correction factors.
///
/// Each source type carries a different expected reporting fraction `F`
/// and correction factor `C` used in signal strength calculation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    /// Spontaneous reports (FAERS, EudraVigilance, VigiBase) — F ≈ 0.05
    Spontaneous,
    /// Clinical trial SAE reports — F ≈ 0.95 (near-complete capture)
    ClinicalTrial,
    /// Published case reports or series — F ≈ 0.10
    Literature,
    /// Disease registry or observational cohort — F ≈ 0.70
    Registry,
}

impl fmt::Display for SourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Spontaneous => write!(f, "spontaneous"),
            Self::ClinicalTrial => write!(f, "clinical_trial"),
            Self::Literature => write!(f, "literature"),
            Self::Registry => write!(f, "registry"),
        }
    }
}

impl FromStr for SourceType {
    type Err = NexError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "spontaneous" | "faers" | "eudravigilance" | "vigibase" => Ok(Self::Spontaneous),
            "clinical_trial" | "clinicaltrial" | "trial" | "rct" => Ok(Self::ClinicalTrial),
            "literature" | "case_report" | "published" => Ok(Self::Literature),
            "registry" | "cohort" | "observational" => Ok(Self::Registry),
            other => Err(NexError::msg(format!("unknown source type: {other:?}"))),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Verdict (ς — State struct)
// ─────────────────────────────────────────────────────────────────────────────

/// Formal interface contract between the RSK engine and Guardian navigator.
///
/// The RSK microgram engine (chains like `pv-signal-to-action`) produces this
/// typed output. Guardian absorbs it for prioritization, aggregation, and routing.
/// The engine itself is domain-agnostic — it operates on JSON and produces Verdicts.
///
/// ## Field Semantics
///
/// - `correction_factor`: Bioavailability correction C. Default 20.0 for spontaneous reports
///   (1/F where F=0.05). Override per `source_type`.
/// - `reporting_fraction`: F value — estimated fraction of events that reach the database.
///   Spontaneous default: 0.05 (5% reporting fraction, per FAERS literature).
/// - `deadline_days`: Calendar days to regulatory action from signal detection date.
///
/// ## Guardian Contract
///
/// Guardian reads `signal_strength` for triage priority, `regulatory_action` for routing,
/// and `deadline_days` for SLA scheduling. The `drug`/`event` pair is the routing key.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Verdict {
    /// Drug name (INN preferred, brand accepted)
    pub drug: String,
    /// Adverse event description (MedDRA PT preferred)
    pub event: String,
    /// Signal strength from disproportionality analysis
    pub signal_strength: VerdictSignalStrength,
    /// WHO-UMC causality classification
    pub causality: CausalityLevel,
    /// Recommended regulatory action
    pub regulatory_action: RegulatoryAction,
    /// Days to regulatory deadline from signal detection
    pub deadline_days: u32,
    /// Bioavailability correction factor C (default: 20.0 for spontaneous)
    pub correction_factor: f64,
    /// Reporting fraction F (default: 0.05 for spontaneous)
    pub reporting_fraction: f64,
    /// Evidence source type (governs correction factors)
    pub source_type: SourceType,
    /// ISO 8601 timestamp of verdict generation
    pub timestamp: String,
}

impl Verdict {
    /// Parse a `Verdict` from the JSON output map of the `pv-signal-to-action` chain.
    ///
    /// The chain emits a flat `HashMap<String, serde_json::Value>` where each key
    /// corresponds to a microgram output variable. This constructor extracts and
    /// type-coerces each field, returning a well-typed `Verdict`.
    ///
    /// # Errors
    ///
    /// Returns `Err` if any required field is absent or cannot be parsed.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::collections::HashMap;
    /// use serde_json::json;
    /// use nexcore_vigilance::verdict::Verdict;
    ///
    /// let mut output = HashMap::new();
    /// output.insert("drug".into(), json!("warfarin"));
    /// output.insert("event".into(), json!("intracranial_hemorrhage"));
    /// output.insert("signal_strength".into(), json!("strong"));
    /// output.insert("causality".into(), json!("probable"));
    /// output.insert("regulatory_action".into(), json!("expedited_report"));
    /// output.insert("deadline_days".into(), json!(15));
    /// output.insert("correction_factor".into(), json!(20.0));
    /// output.insert("reporting_fraction".into(), json!(0.05));
    /// output.insert("source_type".into(), json!("spontaneous"));
    /// output.insert("timestamp".into(), json!("2026-04-11T00:00:00Z"));
    ///
    /// let verdict = Verdict::from_chain_output(&output).unwrap();
    /// assert_eq!(verdict.drug, "warfarin");
    /// ```
    pub fn from_chain_output(output: &HashMap<String, serde_json::Value>) -> Result<Self> {
        // μ (Mapping): extract string fields
        let drug = extract_string(output, "drug")?;
        let event = extract_string(output, "event")?;

        // Σ (Sum): parse enum fields through FromStr
        let signal_strength = extract_string(output, "signal_strength")?
            .parse::<VerdictSignalStrength>()
            .map_err(|e| NexError::msg(format!("signal_strength: {e}")))?;

        let causality = extract_string(output, "causality")?
            .parse::<CausalityLevel>()
            .map_err(|e| NexError::msg(format!("causality: {e}")))?;

        let regulatory_action = extract_string(output, "regulatory_action")?
            .parse::<RegulatoryAction>()
            .map_err(|e| NexError::msg(format!("regulatory_action: {e}")))?;

        let source_type = extract_string(output, "source_type")?
            .parse::<SourceType>()
            .map_err(|e| NexError::msg(format!("source_type: {e}")))?;

        // N (Quantity): numeric fields
        let deadline_days = extract_u32(output, "deadline_days")?;
        let correction_factor = extract_f64_with_default(output, "correction_factor", 20.0);
        let reporting_fraction = extract_f64_with_default(output, "reporting_fraction", 0.05);

        let timestamp = extract_string(output, "timestamp")?;

        Ok(Self {
            drug,
            event,
            signal_strength,
            causality,
            regulatory_action,
            deadline_days,
            correction_factor,
            reporting_fraction,
            source_type,
            timestamp,
        })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Private extraction helpers (∂ — Boundary functions)
// ─────────────────────────────────────────────────────────────────────────────

fn extract_string(map: &HashMap<String, serde_json::Value>, key: &str) -> Result<String> {
    match map.get(key) {
        Some(serde_json::Value::String(s)) => Ok(s.clone()),
        Some(other) => Ok(other.to_string().trim_matches('"').to_owned()),
        None => Err(NexError::msg(format!(
            "missing required field {key:?} in chain output"
        ))),
    }
}

fn extract_u32(map: &HashMap<String, serde_json::Value>, key: &str) -> Result<u32> {
    match map.get(key) {
        Some(serde_json::Value::Number(n)) => n
            .as_u64()
            .map(|v| v as u32)
            .ok_or_else(|| NexError::msg(format!("field {key:?} is not a non-negative integer"))),
        Some(serde_json::Value::String(s)) => s
            .parse::<u32>()
            .map_err(|e| NexError::msg(format!("field {key:?} parse error: {e}"))),
        Some(other) => Err(NexError::msg(format!(
            "field {key:?} expected number, got {other}"
        ))),
        None => Err(NexError::msg(format!(
            "missing required field {key:?} in chain output"
        ))),
    }
}

fn extract_f64_with_default(
    map: &HashMap<String, serde_json::Value>,
    key: &str,
    default: f64,
) -> f64 {
    match map.get(key) {
        Some(serde_json::Value::Number(n)) => n.as_f64().unwrap_or(default),
        Some(serde_json::Value::String(s)) => s.parse::<f64>().unwrap_or(default),
        _ => default,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sample_chain_output() -> HashMap<String, serde_json::Value> {
        let mut m = HashMap::new();
        m.insert("drug".into(), json!("warfarin"));
        m.insert("event".into(), json!("intracranial_hemorrhage"));
        m.insert("signal_strength".into(), json!("strong"));
        m.insert("causality".into(), json!("probable"));
        m.insert("regulatory_action".into(), json!("expedited_report"));
        m.insert("deadline_days".into(), json!(15u32));
        m.insert("correction_factor".into(), json!(20.0f64));
        m.insert("reporting_fraction".into(), json!(0.05f64));
        m.insert("source_type".into(), json!("spontaneous"));
        m.insert("timestamp".into(), json!("2026-04-11T00:00:00Z"));
        m
    }

    #[test]
    fn test_from_chain_output_round_trips_all_fields() {
        let output = sample_chain_output();
        let verdict = Verdict::from_chain_output(&output).expect("parse failed");

        assert_eq!(verdict.drug, "warfarin");
        assert_eq!(verdict.event, "intracranial_hemorrhage");
        assert_eq!(verdict.signal_strength, VerdictSignalStrength::Strong);
        assert_eq!(verdict.causality, CausalityLevel::Probable);
        assert_eq!(verdict.regulatory_action, RegulatoryAction::ExpeditedReport);
        assert_eq!(verdict.deadline_days, 15);
        assert!((verdict.correction_factor - 20.0).abs() < f64::EPSILON);
        assert!((verdict.reporting_fraction - 0.05).abs() < f64::EPSILON);
        assert_eq!(verdict.source_type, SourceType::Spontaneous);
        assert_eq!(verdict.timestamp, "2026-04-11T00:00:00Z");
    }

    #[test]
    fn test_from_chain_output_defaults_correction_fields() {
        let mut output = sample_chain_output();
        output.remove("correction_factor");
        output.remove("reporting_fraction");

        let verdict = Verdict::from_chain_output(&output).expect("parse failed");
        assert!((verdict.correction_factor - 20.0).abs() < f64::EPSILON);
        assert!((verdict.reporting_fraction - 0.05).abs() < f64::EPSILON);
    }

    #[test]
    fn test_from_chain_output_missing_required_field_errors() {
        let mut output = sample_chain_output();
        output.remove("drug");

        let result = Verdict::from_chain_output(&output);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("drug"),
            "error should name the missing field: {msg}"
        );
    }

    #[test]
    fn test_signal_strength_fromstr_all_variants() {
        assert_eq!(
            "strong".parse::<VerdictSignalStrength>().unwrap(),
            VerdictSignalStrength::Strong
        );
        assert_eq!(
            "moderate".parse::<VerdictSignalStrength>().unwrap(),
            VerdictSignalStrength::Moderate
        );
        assert_eq!(
            "weak".parse::<VerdictSignalStrength>().unwrap(),
            VerdictSignalStrength::Weak
        );
        assert_eq!(
            "collapsed".parse::<VerdictSignalStrength>().unwrap(),
            VerdictSignalStrength::Collapsed
        );
        assert_eq!(
            "no_signal".parse::<VerdictSignalStrength>().unwrap(),
            VerdictSignalStrength::NoSignal
        );
        assert!("bogus".parse::<VerdictSignalStrength>().is_err());
    }

    #[test]
    fn test_causality_fromstr_all_variants() {
        assert_eq!(
            "definite".parse::<CausalityLevel>().unwrap(),
            CausalityLevel::Definite
        );
        assert_eq!(
            "probable".parse::<CausalityLevel>().unwrap(),
            CausalityLevel::Probable
        );
        assert_eq!(
            "possible".parse::<CausalityLevel>().unwrap(),
            CausalityLevel::Possible
        );
        assert_eq!(
            "unlikely".parse::<CausalityLevel>().unwrap(),
            CausalityLevel::Unlikely
        );
        assert_eq!(
            "conditional".parse::<CausalityLevel>().unwrap(),
            CausalityLevel::Conditional
        );
        assert_eq!(
            "unassessable".parse::<CausalityLevel>().unwrap(),
            CausalityLevel::Unassessable
        );
        assert!("unknown".parse::<CausalityLevel>().is_err());
    }

    #[test]
    fn test_regulatory_action_fromstr_all_variants() {
        assert_eq!(
            "expedited_report".parse::<RegulatoryAction>().unwrap(),
            RegulatoryAction::ExpeditedReport
        );
        assert_eq!(
            "15day".parse::<RegulatoryAction>().unwrap(),
            RegulatoryAction::ExpeditedReport
        );
        assert_eq!(
            "monitor".parse::<RegulatoryAction>().unwrap(),
            RegulatoryAction::Monitor
        );
        assert_eq!(
            "document".parse::<RegulatoryAction>().unwrap(),
            RegulatoryAction::Document
        );
        assert_eq!(
            "no_action".parse::<RegulatoryAction>().unwrap(),
            RegulatoryAction::NoAction
        );
        assert!("invalid".parse::<RegulatoryAction>().is_err());
    }

    #[test]
    fn test_source_type_fromstr_all_variants() {
        assert_eq!(
            "spontaneous".parse::<SourceType>().unwrap(),
            SourceType::Spontaneous
        );
        assert_eq!(
            "faers".parse::<SourceType>().unwrap(),
            SourceType::Spontaneous
        );
        assert_eq!(
            "clinical_trial".parse::<SourceType>().unwrap(),
            SourceType::ClinicalTrial
        );
        assert_eq!(
            "rct".parse::<SourceType>().unwrap(),
            SourceType::ClinicalTrial
        );
        assert_eq!(
            "literature".parse::<SourceType>().unwrap(),
            SourceType::Literature
        );
        assert_eq!(
            "registry".parse::<SourceType>().unwrap(),
            SourceType::Registry
        );
        assert!("unknown".parse::<SourceType>().is_err());
    }

    #[test]
    fn test_display_round_trips() {
        assert_eq!(VerdictSignalStrength::Strong.to_string(), "strong");
        assert_eq!(CausalityLevel::Probable.to_string(), "probable");
        assert_eq!(
            RegulatoryAction::ExpeditedReport.to_string(),
            "expedited_report"
        );
        assert_eq!(SourceType::Spontaneous.to_string(), "spontaneous");
    }

    #[test]
    fn test_verdict_serializes_to_json() {
        let output = sample_chain_output();
        let verdict = Verdict::from_chain_output(&output).expect("parse failed");
        let json = serde_json::to_string(&verdict).expect("serialize failed");
        assert!(json.contains("warfarin"));
        assert!(json.contains("strong"));
        assert!(json.contains("expedited_report"));
    }

    #[test]
    fn test_verdict_clinical_trial_source() {
        let mut output = sample_chain_output();
        output.insert("source_type".into(), json!("clinical_trial"));
        output.insert("reporting_fraction".into(), json!(0.95f64));
        output.insert("correction_factor".into(), json!(1.053f64));

        let verdict = Verdict::from_chain_output(&output).expect("parse failed");
        assert_eq!(verdict.source_type, SourceType::ClinicalTrial);
        assert!((verdict.reporting_fraction - 0.95).abs() < 0.001);
    }

    #[test]
    fn test_no_signal_verdict() {
        let mut output = sample_chain_output();
        output.insert("signal_strength".into(), json!("no_signal"));
        output.insert("causality".into(), json!("unassessable"));
        output.insert("regulatory_action".into(), json!("no_action"));
        output.insert("deadline_days".into(), json!(0u32));

        let verdict = Verdict::from_chain_output(&output).expect("parse failed");
        assert_eq!(verdict.signal_strength, VerdictSignalStrength::NoSignal);
        assert_eq!(verdict.regulatory_action, RegulatoryAction::NoAction);
        assert_eq!(verdict.deadline_days, 0);
    }
}
