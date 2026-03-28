//! # nexcore-disease-t2dm
//!
//! Type 2 Diabetes Mellitus — epidemiology, treatment landscape, safety burden,
//! and unmet needs for the NexVigilant disease knowledge graph.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod catalog;

use nexcore_disease::{Disease, DiseaseAnalysis};

/// Type 2 Diabetes Mellitus disease model.
pub struct T2dm {
    data: Disease,
}

impl T2dm {
    /// Load the canonical T2DM disease model.
    pub fn load() -> Self {
        Self {
            data: catalog::disease(),
        }
    }
}

impl DiseaseAnalysis for T2dm {
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
        let d = T2dm::load();
        assert_eq!(d.disease().id.as_str(), "t2dm");
        assert_eq!(d.disease().icd10_codes, vec!["E11"]);
    }

    #[test]
    fn has_first_line_treatment() {
        let d = T2dm::load();
        let first_line = d.drugs_by_line(&LineOfTherapy::First);
        assert!(!first_line.is_empty(), "T2DM must have first-line drugs");
        assert!(
            first_line.contains(&"metformin"),
            "Metformin must be first-line"
        );
    }
}
