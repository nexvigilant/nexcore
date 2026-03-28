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
