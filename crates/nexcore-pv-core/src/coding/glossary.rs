//! ICH E2B/E2C glossary support.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Glossary entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlossaryEntry {
    /// Term
    pub term: String,
    /// Definition
    pub definition: String,
    /// Source
    pub source: String,
}

impl GlossaryEntry {
    /// Create new entry
    #[must_use]
    pub fn new(term: &str, definition: &str, source: &str) -> Self {
        Self {
            term: term.into(),
            definition: definition.into(),
            source: source.into(),
        }
    }
}

/// ICH Glossary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ICHGlossary {
    entries: HashMap<String, GlossaryEntry>,
}

impl ICHGlossary {
    /// Create new glossary
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with E2B core terms
    #[must_use]
    pub fn with_e2b_core() -> Self {
        let mut g = Self::new();
        g.add(GlossaryEntry::new(
            "Adverse Event",
            "Any untoward medical occurrence in a patient",
            "ICH E2A",
        ));
        g.add(GlossaryEntry::new(
            "ADR",
            "Adverse Drug Reaction - noxious and unintended response",
            "ICH E2A",
        ));
        g.add(GlossaryEntry::new(
            "Serious",
            "Death, life-threatening, hospitalization, disability",
            "ICH E2A",
        ));
        g.add(GlossaryEntry::new(
            "Signal",
            "New potentially causal association",
            "ICH E2C",
        ));
        g.add(GlossaryEntry::new(
            "ICSR",
            "Individual Case Safety Report",
            "ICH E2B",
        ));
        g.add(GlossaryEntry::new(
            "PSUR",
            "Periodic Safety Update Report",
            "ICH E2C",
        ));
        g.add(GlossaryEntry::new(
            "PBRER",
            "Periodic Benefit-Risk Evaluation Report",
            "ICH E2C(R2)",
        ));
        g
    }

    /// Add entry
    pub fn add(&mut self, entry: GlossaryEntry) {
        self.entries.insert(entry.term.to_lowercase(), entry);
    }

    /// Get entry
    #[must_use]
    pub fn get(&self, term: &str) -> Option<&GlossaryEntry> {
        self.entries.get(&term.to_lowercase())
    }

    /// Search entries
    #[must_use]
    pub fn search(&self, query: &str) -> Vec<&GlossaryEntry> {
        let q = query.to_lowercase();
        self.entries
            .values()
            .filter(|e| {
                e.term.to_lowercase().contains(&q) || e.definition.to_lowercase().contains(&q)
            })
            .collect()
    }

    /// Count entries
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Outcome code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutcomeCode {
    /// Death
    DE,
    /// Life-threatening
    LT,
    /// Hospitalization
    HO,
    /// Disability
    DS,
    /// Congenital anomaly
    CA,
    /// Required intervention
    RI,
    /// Other serious
    OT,
}

impl OutcomeCode {
    /// Parse from string
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "DE" => Some(Self::DE),
            "LT" => Some(Self::LT),
            "HO" => Some(Self::HO),
            "DS" => Some(Self::DS),
            "CA" => Some(Self::CA),
            "RI" => Some(Self::RI),
            "OT" => Some(Self::OT),
            _ => None,
        }
    }
    /// Is fatal
    #[must_use]
    pub fn is_fatal(&self) -> bool {
        matches!(self, Self::DE)
    }
}

/// Seriousness criteria
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SeriousnessCriteria {
    /// Death
    pub death: bool,
    /// Life-threatening
    pub life_threatening: bool,
    /// Hospitalization
    pub hospitalization: bool,
    /// Disability
    pub disability: bool,
}

impl SeriousnessCriteria {
    /// Check if serious
    #[must_use]
    pub fn is_serious(&self) -> bool {
        self.death || self.life_threatening || self.hospitalization || self.disability
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_glossary() {
        let g = ICHGlossary::with_e2b_core();
        assert!(g.get("ICSR").is_some());
        assert!(!g.search("safety").is_empty());
    }
    #[test]
    fn test_outcome() {
        assert_eq!(OutcomeCode::parse("DE"), Some(OutcomeCode::DE));
        assert!(OutcomeCode::DE.is_fatal());
    }
}
