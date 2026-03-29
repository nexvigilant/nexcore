#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod catalog;

use nexcore_disease::{Disease, DiseaseAnalysis};

pub struct DiseaseModel {
    data: Disease,
}

impl DiseaseModel {
    pub fn load() -> Self {
        Self {
            data: catalog::disease(),
        }
    }
}

impl DiseaseAnalysis for DiseaseModel {
    fn disease(&self) -> &Disease {
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_disease::DiseaseAnalysis;

    #[test]
    fn model_loads() {
        let model = DiseaseModel::load();
        let disease = model.disease();
        assert!(!disease.name.is_empty());
    }

    #[test]
    fn has_icd10_codes() {
        let model = DiseaseModel::load();
        assert!(!model.disease().icd10_codes.is_empty());
    }

    #[test]
    fn has_standard_of_care() {
        let model = DiseaseModel::load();
        assert!(!model.disease().standard_of_care.is_empty());
    }

    #[test]
    fn has_safety_burden() {
        let model = DiseaseModel::load();
        assert!(
            model.disease().safety_burden.total_drugs_approved > 0
                || !model.disease().safety_burden.class_effects.is_empty()
        );
    }

    #[test]
    fn has_unmet_needs() {
        let model = DiseaseModel::load();
        assert!(!model.disease().unmet_needs.is_empty());
    }
}
