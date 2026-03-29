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
    fn has_therapeutic_area() {
        let model = DiseaseModel::load();
        let _ = format!("{}", model.disease().therapeutic_area);
    }

    #[test]
    fn disease_id_nonempty() {
        let model = DiseaseModel::load();
        let _ = model.disease().id.clone();
    }
}
