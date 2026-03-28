//! Drug analysis trait.
//!
//! `DrugAnalysis` is the query interface over a [`Drug`] aggregate.
//! Implementors gain a standard set of analytical views: signal portfolio,
//! strongest signal, on-label / off-label signal split, and cross-drug
//! signal comparison.
//!
//! The trait is object-safe — all methods return owned or reference types
//! with no associated types or generic parameters.

use crate::{Drug, SignalEntry};

/// Result of comparing a single adverse event across two drugs.
///
/// When a drug has no signal for the event, `drug_a_prr` or `drug_b_prr`
/// is `None`. The `advantage` field records which drug has the lower PRR
/// (i.e. the better safety profile for that event).
///
/// # Examples
///
/// ```
/// use nexcore_drug::analysis::{ComparisonResult, SignalComparison};
///
/// let cmp = SignalComparison {
///     event: "Pancreatitis".to_string(),
///     drug_a_prr: Some(3.02),
///     drug_b_prr: Some(2.41),
///     advantage: ComparisonResult::DrugB,
/// };
/// assert_eq!(cmp.event, "Pancreatitis");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalComparison {
    /// Adverse event being compared
    pub event: String,
    /// PRR for drug A (`None` if no signal data)
    pub drug_a_prr: Option<f64>,
    /// PRR for drug B (`None` if no signal data)
    pub drug_b_prr: Option<f64>,
    /// Which drug has the more favourable (lower) PRR for this event
    pub advantage: ComparisonResult,
}

/// Outcome of a per-event PRR comparison between two drugs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComparisonResult {
    /// Drug A has the lower PRR for this event
    DrugA,
    /// Drug B has the lower PRR for this event
    DrugB,
    /// PRRs are equal or both absent — no advantage
    Neutral,
}

use serde::{Deserialize, Serialize};

/// Analytical query interface over a drug aggregate.
///
/// Provides domain-level signal views without requiring callers to traverse
/// the raw [`Drug`] fields directly.
///
/// # Implementing
///
/// The `DefaultDrugAnalysis` struct in this module implements the trait over
/// `&Drug` for convenience. For richer implementations (e.g. with cached
/// indexes), implement the trait on your own wrapper type.
///
/// # Examples
///
/// ```
/// use nexcore_drug::{Drug, DrugClass, DrugId, LabelStatus};
/// use nexcore_drug::analysis::{DefaultDrugAnalysis, DrugAnalysis};
///
/// let drug = Drug {
///     id: DrugId::new("example"),
///     generic_name: "example".to_string(),
///     brand_names: vec![],
///     rxcui: None,
///     mechanism: "Unknown".to_string(),
///     drug_class: DrugClass::Other("Unknown".to_string()),
///     indications: vec![],
///     contraindications: vec![],
///     safety_signals: vec![],
///     label_status: LabelStatus::default(),
///     owner: None,
/// };
///
/// let analysis = DefaultDrugAnalysis::new(&drug);
/// assert!(analysis.signal_portfolio().is_empty());
/// assert!(analysis.strongest_signal().is_none());
/// ```
pub trait DrugAnalysis {
    /// Returns a reference to the underlying drug aggregate.
    fn drug(&self) -> &Drug;

    /// Returns all signal entries in the portfolio.
    fn signal_portfolio(&self) -> &[SignalEntry];

    /// Returns the signal with the highest PRR, if any.
    fn strongest_signal(&self) -> Option<&SignalEntry>;

    /// Returns all on-label signals (known labelled safety events).
    fn on_label_signals(&self) -> Vec<&SignalEntry>;

    /// Returns all off-label signals (potential new safety signals).
    fn off_label_signals(&self) -> Vec<&SignalEntry>;

    /// Compares the signal portfolio of this drug against another implementor,
    /// returning a per-event comparison for every event present in either drug.
    fn compare_signals(&self, other: &dyn DrugAnalysis) -> Vec<SignalComparison>;
}

/// Default implementation of [`DrugAnalysis`] that operates directly
/// over a `&Drug` reference without additional indexing.
pub struct DefaultDrugAnalysis<'a> {
    drug: &'a Drug,
}

impl<'a> DefaultDrugAnalysis<'a> {
    /// Wrap a drug reference for analysis.
    pub fn new(drug: &'a Drug) -> Self {
        Self { drug }
    }
}

impl<'a> DrugAnalysis for DefaultDrugAnalysis<'a> {
    fn drug(&self) -> &Drug {
        self.drug
    }

    fn signal_portfolio(&self) -> &[SignalEntry] {
        &self.drug.safety_signals
    }

    fn strongest_signal(&self) -> Option<&SignalEntry> {
        self.drug.strongest_signal()
    }

    fn on_label_signals(&self) -> Vec<&SignalEntry> {
        self.drug.on_label_signals()
    }

    fn off_label_signals(&self) -> Vec<&SignalEntry> {
        self.drug.off_label_signals()
    }

    fn compare_signals(&self, other: &dyn DrugAnalysis) -> Vec<SignalComparison> {
        use std::collections::HashMap;

        // Index self signals by event name
        let self_map: HashMap<&str, f64> = self
            .drug
            .safety_signals
            .iter()
            .map(|s| (s.event.as_str(), s.prr))
            .collect();

        // Index other signals by event name
        let other_map: HashMap<&str, f64> = other
            .signal_portfolio()
            .iter()
            .map(|s| (s.event.as_str(), s.prr))
            .collect();

        // Union of all event names
        let mut events: Vec<&str> = self_map.keys().copied().collect();
        for key in other_map.keys() {
            if !self_map.contains_key(key) {
                events.push(key);
            }
        }
        events.sort_unstable();

        events
            .into_iter()
            .map(|event| {
                let a_prr = self_map.get(event).copied();
                let b_prr = other_map.get(event).copied();
                let advantage = match (a_prr, b_prr) {
                    (Some(a), Some(b)) => {
                        if a < b {
                            ComparisonResult::DrugA
                        } else if b < a {
                            ComparisonResult::DrugB
                        } else {
                            ComparisonResult::Neutral
                        }
                    }
                    (Some(_), None) => ComparisonResult::DrugB, // other has no signal → other safer for this event
                    (None, Some(_)) => ComparisonResult::DrugA, // self has no signal → self safer for this event
                    (None, None) => ComparisonResult::Neutral,
                };
                SignalComparison {
                    event: event.to_string(),
                    drug_a_prr: a_prr,
                    drug_b_prr: b_prr,
                    advantage,
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ContingencyTable, DrugClass, DrugId, LabelStatus, SignalEntry, SignalVerdict};

    fn make_signal(event: &str, prr: f64, on_label: bool) -> SignalEntry {
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
            verdict: if prr >= 3.0 {
                SignalVerdict::Strong
            } else if prr >= 2.0 {
                SignalVerdict::Moderate
            } else {
                SignalVerdict::Weak
            },
        }
    }

    fn make_drug(id: &str, signals: Vec<SignalEntry>) -> Drug {
        Drug {
            id: DrugId::new(id),
            generic_name: id.to_string(),
            brand_names: vec![],
            rxcui: None,
            mechanism: "test".to_string(),
            drug_class: DrugClass::Other("test".to_string()),
            indications: vec![],
            contraindications: vec![],
            safety_signals: signals,
            label_status: LabelStatus::default(),
            owner: None,
        }
    }

    #[test]
    fn signal_portfolio_empty() {
        let d = make_drug("empty", vec![]);
        let analysis = DefaultDrugAnalysis::new(&d);
        assert!(analysis.signal_portfolio().is_empty());
        assert!(analysis.strongest_signal().is_none());
    }

    #[test]
    fn signal_portfolio_returns_all() {
        let d = make_drug(
            "multi",
            vec![
                make_signal("nausea", 2.5, true),
                make_signal("pancreatitis", 3.0, false),
            ],
        );
        let analysis = DefaultDrugAnalysis::new(&d);
        assert_eq!(analysis.signal_portfolio().len(), 2);
    }

    #[test]
    fn strongest_signal_picks_highest_prr() {
        let d = make_drug(
            "drug",
            vec![
                make_signal("nausea", 2.5, true),
                make_signal("pancreatitis", 3.0, false),
                make_signal("alopecia", 1.8, false),
            ],
        );
        let analysis = DefaultDrugAnalysis::new(&d);
        let s = analysis.strongest_signal().expect("has signals");
        assert_eq!(s.event, "pancreatitis");
    }

    #[test]
    fn on_off_label_split_correct() {
        let d = make_drug(
            "drug",
            vec![
                make_signal("nausea", 2.5, true),
                make_signal("pancreatitis", 3.0, false),
            ],
        );
        let analysis = DefaultDrugAnalysis::new(&d);
        assert_eq!(analysis.on_label_signals().len(), 1);
        assert_eq!(analysis.off_label_signals().len(), 1);
    }

    #[test]
    fn compare_signals_shared_event() {
        let d_a = make_drug("drug-a", vec![make_signal("nausea", 2.5, true)]);
        let d_b = make_drug("drug-b", vec![make_signal("nausea", 3.5, true)]);
        let analysis_a = DefaultDrugAnalysis::new(&d_a);
        let analysis_b = DefaultDrugAnalysis::new(&d_b);
        let comparisons = analysis_a.compare_signals(&analysis_b);
        assert_eq!(comparisons.len(), 1);
        assert_eq!(comparisons[0].event, "nausea");
        assert_eq!(comparisons[0].advantage, ComparisonResult::DrugA); // lower PRR → drug A better
    }

    #[test]
    fn compare_signals_unique_events() {
        let d_a = make_drug("drug-a", vec![make_signal("nausea", 2.5, true)]);
        let d_b = make_drug("drug-b", vec![make_signal("pancreatitis", 3.0, false)]);
        let analysis_a = DefaultDrugAnalysis::new(&d_a);
        let analysis_b = DefaultDrugAnalysis::new(&d_b);
        let comparisons = analysis_a.compare_signals(&analysis_b);
        // Both events should appear; unique events give advantage to the drug without signal
        assert_eq!(comparisons.len(), 2);
        let nausea = comparisons
            .iter()
            .find(|c| c.event == "nausea")
            .expect("nausea present");
        assert_eq!(nausea.advantage, ComparisonResult::DrugB); // B has no nausea signal → B safer for nausea
        let panc = comparisons
            .iter()
            .find(|c| c.event == "pancreatitis")
            .expect("pancreatitis present");
        assert_eq!(panc.advantage, ComparisonResult::DrugA); // A has no pancreatitis signal → A safer for pancreatitis
    }

    #[test]
    fn compare_signals_neutral_when_equal_prr() {
        let d_a = make_drug("drug-a", vec![make_signal("nausea", 2.5, true)]);
        let d_b = make_drug("drug-b", vec![make_signal("nausea", 2.5, true)]);
        let analysis_a = DefaultDrugAnalysis::new(&d_a);
        let analysis_b = DefaultDrugAnalysis::new(&d_b);
        let comparisons = analysis_a.compare_signals(&analysis_b);
        assert_eq!(comparisons[0].advantage, ComparisonResult::Neutral);
    }
}
