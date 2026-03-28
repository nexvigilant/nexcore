//! # nexcore-disease-obesity
//!
//! Obesity — epidemiology, treatment landscape, safety burden,
//! and unmet needs for the NexVigilant disease knowledge graph.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod catalog;

use nexcore_disease::{Disease, DiseaseAnalysis};

/// Obesity disease model.
pub struct Obesity {
    data: Disease,
}

impl Obesity {
    /// Load the canonical Obesity disease model.
    pub fn load() -> Self {
        Self {
            data: catalog::disease(),
        }
    }
}

impl DiseaseAnalysis for Obesity {
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
        let d = Obesity::load();
        assert_eq!(d.disease().id.as_str(), "obesity");
        assert!(d.disease().icd10_codes.contains(&"E66".to_string()));
    }

    #[test]
    fn has_second_line_glp1_treatment() {
        let d = Obesity::load();
        let second_line = d.drugs_by_line(&LineOfTherapy::Second);
        assert!(
            !second_line.is_empty(),
            "Obesity must have second-line drugs"
        );
        assert!(
            second_line.contains(&"semaglutide 2.4mg"),
            "Semaglutide 2.4mg must be second-line"
        );
    }
}
