//! # nexcore-disease-alzheimers
//!
//! Alzheimer's Disease — epidemiology, treatment landscape, safety burden,
//! and unmet needs for the NexVigilant disease knowledge graph.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod catalog;

use nexcore_disease::{Disease, DiseaseAnalysis};

/// Alzheimer's Disease disease model.
pub struct Alzheimers {
    data: Disease,
}

impl Alzheimers {
    /// Load the canonical Alzheimer's Disease disease model.
    pub fn load() -> Self {
        Self {
            data: catalog::disease(),
        }
    }
}

impl DiseaseAnalysis for Alzheimers {
    fn disease(&self) -> &Disease {
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_disease::LineOfTherapy;

    #[test]
    fn disease_loads_with_correct_id() {
        let d = Alzheimers::load();
        assert_eq!(d.disease().id.as_str(), "alzheimers");
        assert_eq!(d.disease().icd10_codes, vec!["G30", "G30.0", "G30.9"]);
    }

    #[test]
    fn has_first_line_treatment() {
        let d = Alzheimers::load();
        let first_line = d.drugs_by_line(&LineOfTherapy::First);
        assert!(
            !first_line.is_empty(),
            "Alzheimer's must have first-line drugs"
        );
        assert!(
            first_line.contains(&"donepezil"),
            "Donepezil must be first-line"
        );
    }
}
