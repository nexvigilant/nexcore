//! # nexcore-disease-nsclc
//!
//! Non-Small Cell Lung Cancer — epidemiology, treatment landscape, safety burden,
//! and unmet needs for the NexVigilant disease knowledge graph.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod catalog;

use nexcore_disease::{Disease, DiseaseAnalysis};

/// Non-Small Cell Lung Cancer disease model.
pub struct Nsclc {
    data: Disease,
}

impl Nsclc {
    /// Load the canonical NSCLC disease model.
    pub fn load() -> Self {
        Self {
            data: catalog::disease(),
        }
    }
}

impl DiseaseAnalysis for Nsclc {
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
        let d = Nsclc::load();
        assert_eq!(d.disease().id.as_str(), "nsclc");
        assert!(d.disease().icd10_codes.contains(&"C34".to_string()));
    }

    #[test]
    fn has_first_line_checkpoint_inhibitors() {
        let d = Nsclc::load();
        let first_line = d.drugs_by_line(&LineOfTherapy::First);
        assert!(!first_line.is_empty(), "NSCLC must have first-line drugs");
        assert!(
            first_line.contains(&"pembrolizumab"),
            "Pembrolizumab must be first-line"
        );
    }
}
