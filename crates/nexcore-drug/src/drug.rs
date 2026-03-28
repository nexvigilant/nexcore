//! Core Drug aggregate.
//!
//! `Drug` is the central domain type in `nexcore-drug`. It aggregates
//! identity, classification, indications, contraindications, safety signals,
//! label status, and the owning company reference into a single owned struct.
//!
//! Drug and Company are **peer aggregates** — `Drug` stores the company name
//! as a plain `Option<String>` rather than a `CompanyId`. The strategy crate
//! composes the two peers when cross-aggregate analysis is required.

use serde::{Deserialize, Serialize};

use crate::{DrugClass, DrugId, Indication, LabelStatus, SignalEntry};

/// A pharmaceutical drug with its full domain profile.
///
/// Aggregates marketed identity (names, RxCUI), pharmacological classification,
/// approved indications, contraindications, pharmacovigilance signal portfolio,
/// regulatory label status, and the owning manufacturer.
///
/// # Examples
///
/// ```
/// use nexcore_drug::{Drug, DrugClass, DrugId, LabelStatus};
///
/// let drug = Drug {
///     id: DrugId::new("tirzepatide"),
///     generic_name: "tirzepatide".to_string(),
///     brand_names: vec!["Mounjaro".to_string(), "Zepbound".to_string()],
///     rxcui: Some("2200644".to_string()),
///     mechanism: "GLP-1 and GIP dual receptor agonist".to_string(),
///     drug_class: DrugClass::GLP1GIPDualAgonist,
///     indications: vec![],
///     contraindications: vec!["Personal or family history of MTC".to_string()],
///     safety_signals: vec![],
///     label_status: LabelStatus::default(),
///     owner: Some("Eli Lilly and Company".to_string()),
/// };
/// assert_eq!(drug.id.as_str(), "tirzepatide");
/// assert_eq!(drug.signal_count(), 0);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Drug {
    /// Stable machine-readable identifier
    pub id: DrugId,
    /// INN / generic drug name
    pub generic_name: String,
    /// All brand names under which the drug is marketed
    pub brand_names: Vec<String>,
    /// RxNorm concept unique identifier, if known
    pub rxcui: Option<String>,
    /// Primary mechanism of action description
    pub mechanism: String,
    /// Primary pharmacological class
    pub drug_class: DrugClass,
    /// Approved regulatory indications
    pub indications: Vec<Indication>,
    /// Absolute contraindications (plain-English summaries)
    pub contraindications: Vec<String>,
    /// Pharmacovigilance signal portfolio
    pub safety_signals: Vec<SignalEntry>,
    /// Regulatory label safety status
    pub label_status: LabelStatus,
    /// Owning manufacturer (plain name; links to nexcore-pharma Company)
    pub owner: Option<String>,
}

impl Drug {
    /// Number of safety signals in the portfolio.
    pub fn signal_count(&self) -> usize {
        self.safety_signals.len()
    }

    /// Number of approved indications.
    pub fn indication_count(&self) -> usize {
        self.indications.len()
    }

    /// Returns `true` if the drug carries a boxed warning.
    pub fn has_boxed_warning(&self) -> bool {
        self.label_status.boxed_warning
    }

    /// Returns `true` if the drug requires a REMS program.
    pub fn has_rems(&self) -> bool {
        self.label_status.rems
    }

    /// Returns the primary brand name, if any.
    pub fn primary_brand(&self) -> Option<&str> {
        self.brand_names.first().map(|s| s.as_str())
    }

    /// Returns all signals classified as off-label (potential new signals).
    pub fn off_label_signals(&self) -> Vec<&SignalEntry> {
        self.safety_signals.iter().filter(|s| !s.on_label).collect()
    }

    /// Returns all signals classified as on-label (known safety events).
    pub fn on_label_signals(&self) -> Vec<&SignalEntry> {
        self.safety_signals.iter().filter(|s| s.on_label).collect()
    }

    /// Returns the signal with the highest PRR, if any signals exist.
    pub fn strongest_signal(&self) -> Option<&SignalEntry> {
        self.safety_signals.iter().max_by(|a, b| {
            a.prr
                .partial_cmp(&b.prr)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ContingencyTable, DrugClass, DrugId, LabelStatus, SignalEntry, SignalVerdict};

    fn make_signal(event: &str, prr: f64, on_label: bool, verdict: SignalVerdict) -> SignalEntry {
        SignalEntry {
            event: event.to_string(),
            contingency: ContingencyTable {
                a: 50,
                b: 1_000,
                c: 200,
                d: 5_000_000,
            },
            prr,
            ror: prr * 1.05,
            ic: prr.log2(),
            cases: 50,
            on_label,
            verdict,
        }
    }

    fn minimal_drug() -> Drug {
        Drug {
            id: DrugId::new("testdrug"),
            generic_name: "testdrug".to_string(),
            brand_names: vec!["BrandA".to_string()],
            rxcui: None,
            mechanism: "Test mechanism".to_string(),
            drug_class: DrugClass::Other("Test class".to_string()),
            indications: vec![],
            contraindications: vec![],
            safety_signals: vec![],
            label_status: LabelStatus::default(),
            owner: None,
        }
    }

    #[test]
    fn drug_construction_minimal() {
        let d = minimal_drug();
        assert_eq!(d.id.as_str(), "testdrug");
        assert_eq!(d.signal_count(), 0);
        assert_eq!(d.indication_count(), 0);
        assert!(!d.has_boxed_warning());
        assert!(!d.has_rems());
    }

    #[test]
    fn primary_brand_returns_first() {
        let d = minimal_drug();
        assert_eq!(d.primary_brand(), Some("BrandA"));
    }

    #[test]
    fn primary_brand_none_when_empty() {
        let mut d = minimal_drug();
        d.brand_names.clear();
        assert!(d.primary_brand().is_none());
    }

    #[test]
    fn signal_count_tracks_vec_length() {
        let mut d = minimal_drug();
        d.safety_signals
            .push(make_signal("nausea", 2.5, true, SignalVerdict::Moderate));
        assert_eq!(d.signal_count(), 1);
    }

    #[test]
    fn on_off_label_split() {
        let mut d = minimal_drug();
        d.safety_signals
            .push(make_signal("nausea", 2.5, true, SignalVerdict::Moderate));
        d.safety_signals.push(make_signal(
            "pancreatitis",
            3.0,
            false,
            SignalVerdict::Strong,
        ));
        assert_eq!(d.on_label_signals().len(), 1);
        assert_eq!(d.off_label_signals().len(), 1);
    }

    #[test]
    fn strongest_signal_picks_highest_prr() {
        let mut d = minimal_drug();
        d.safety_signals
            .push(make_signal("nausea", 2.5, true, SignalVerdict::Moderate));
        d.safety_signals.push(make_signal(
            "pancreatitis",
            3.0,
            false,
            SignalVerdict::Strong,
        ));
        d.safety_signals.push(make_signal(
            "gastroparesis",
            1.8,
            false,
            SignalVerdict::Weak,
        ));
        let strongest = d.strongest_signal().expect("signals not empty");
        assert_eq!(strongest.event, "pancreatitis");
    }

    #[test]
    fn strongest_signal_none_when_empty() {
        let d = minimal_drug();
        assert!(d.strongest_signal().is_none());
    }

    #[test]
    fn drug_serializes_round_trip() {
        let d = minimal_drug();
        let json = serde_json::to_string(&d).expect("serialization cannot fail on valid struct");
        let parsed: Drug =
            serde_json::from_str(&json).expect("deserialization cannot fail on valid JSON");
        assert_eq!(parsed.id.as_str(), "testdrug");
        assert_eq!(parsed.generic_name, d.generic_name);
    }
}
