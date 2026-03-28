//! # nexcore-disease-ra
//!
//! Rheumatoid Arthritis — epidemiology, treatment landscape, safety burden,
//! and unmet needs for the NexVigilant disease knowledge graph.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod catalog;

use nexcore_disease::{Disease, DiseaseAnalysis};

/// Rheumatoid Arthritis disease model.
pub struct Ra {
    data: Disease,
}

impl Ra {
    /// Load the canonical Rheumatoid Arthritis disease model.
    pub fn load() -> Self {
        Self {
            data: catalog::disease(),
        }
    }
}

impl DiseaseAnalysis for Ra {
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
        let d = Ra::load();
        assert_eq!(d.disease().id.as_str(), "ra");
        assert!(d.disease().icd10_codes.contains(&"M05".to_string()));
    }

    #[test]
    fn has_first_line_methotrexate() {
        let d = Ra::load();
        let first_line = d.drugs_by_line(&LineOfTherapy::First);
        assert!(!first_line.is_empty(), "RA must have first-line drugs");
        assert!(
            first_line.contains(&"methotrexate"),
            "Methotrexate must be first-line"
        );
    }
}
