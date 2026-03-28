//! # NexVigilant Core — Disease
//!
//! Disease domain models: epidemiology, treatment landscape, safety burden, and unmet needs.
//! Part of the Company × Drug × Disease knowledge graph.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod analysis;
mod biomarker;
mod disease;
mod epidemiology;
mod id;
mod safety_burden;
mod treatment;
mod unmet_need;

pub use analysis::DiseaseAnalysis;
pub use biomarker::{Biomarker, BiomarkerType};
pub use disease::Disease;
pub use epidemiology::{Demographics, Epidemiology, Trend};
pub use id::DiseaseId;
pub use safety_burden::{ClassEffect, DrugWithdrawal, SafetyBurden};
pub use treatment::{EvidenceLevel, LineOfTherapy, TreatmentLine};
pub use unmet_need::{NeedSeverity, UnmetNeed};

/// Therapeutic area classification.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum TherapeuticArea {
    Oncology,
    Immunology,
    Neuroscience,
    Cardiovascular,
    Metabolic,
    RareDisease,
    Respiratory,
    Infectious,
    Hematology,
    Ophthalmology,
    Dermatology,
    Gastroenterology,
    Other(String),
}

impl std::fmt::Display for TherapeuticArea {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Oncology => write!(f, "Oncology"),
            Self::Immunology => write!(f, "Immunology"),
            Self::Neuroscience => write!(f, "Neuroscience"),
            Self::Cardiovascular => write!(f, "Cardiovascular"),
            Self::Metabolic => write!(f, "Metabolic"),
            Self::RareDisease => write!(f, "Rare Disease"),
            Self::Respiratory => write!(f, "Respiratory"),
            Self::Infectious => write!(f, "Infectious Disease"),
            Self::Hematology => write!(f, "Hematology"),
            Self::Ophthalmology => write!(f, "Ophthalmology"),
            Self::Dermatology => write!(f, "Dermatology"),
            Self::Gastroenterology => write!(f, "Gastroenterology"),
            Self::Other(s) => write!(f, "{s}"),
        }
    }
}
