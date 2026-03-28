use serde::{Deserialize, Serialize};

/// Aggregate safety burden across all drugs approved for a disease.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SafetyBurden {
    /// Total number of approved drugs for this indication.
    pub total_drugs_approved: u32,
    /// How many carry a boxed warning.
    pub drugs_with_boxed_warnings: u32,
    /// How many have an active REMS.
    pub drugs_with_rems: u32,
    /// Known class effects shared across drug classes.
    pub class_effects: Vec<ClassEffect>,
    /// Drugs withdrawn from market for safety reasons.
    pub notable_withdrawals: Vec<DrugWithdrawal>,
}

/// A safety signal shared across a drug class.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClassEffect {
    /// Drug class name (e.g., "GLP-1 Receptor Agonist").
    pub drug_class: String,
    /// The shared adverse event.
    pub event: String,
    /// Strength of evidence.
    pub evidence_strength: String,
}

/// A drug withdrawn from market.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DrugWithdrawal {
    /// Generic drug name.
    pub drug_name: String,
    /// Year of withdrawal.
    pub year: u16,
    /// Reason for withdrawal.
    pub reason: String,
}

impl SafetyBurden {
    /// Fraction of approved drugs carrying boxed warnings.
    pub fn boxed_warning_rate(&self) -> f64 {
        if self.total_drugs_approved == 0 {
            return 0.0;
        }
        f64::from(self.drugs_with_boxed_warnings) / f64::from(self.total_drugs_approved)
    }
}
