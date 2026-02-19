//! FAERS advanced analytics params
use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FaersStandardPrr {
    pub drug: String,
    pub event: String,
    pub prr: f64,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FaersOutcomeCase {
    pub drug: String,
    pub event: String,
    pub outcome_code: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FaersOutcomeConditionedParams {
    pub cases: Vec<FaersOutcomeCase>,
    pub standard_prrs: Vec<FaersStandardPrr>,
    #[serde(default)]
    pub prr_threshold: Option<f64>,
    #[serde(default)]
    pub min_cases: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FaersTemporalCase {
    pub drug: String,
    pub event: String,
    pub receipt_date: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FaersSignalVelocityParams {
    pub cases: Vec<FaersTemporalCase>,
    pub known_prrs: Vec<FaersStandardPrr>,
    #[serde(default)]
    pub min_cases: Option<u32>,
    #[serde(default)]
    pub min_months: Option<usize>,
    #[serde(default)]
    pub acceleration_threshold: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FaersSeriousnessCase {
    pub drug: String,
    pub event: String,
    pub receipt_date: String,
    #[serde(default)]
    pub seriousness_death: Option<String>,
    #[serde(default)]
    pub seriousness_hospitalization: Option<String>,
    #[serde(default)]
    pub seriousness_disabling: Option<String>,
    #[serde(default)]
    pub seriousness_congenital: Option<String>,
    #[serde(default)]
    pub seriousness_life_threatening: Option<String>,
    #[serde(default)]
    pub seriousness_other: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FaersSeriousnessCascadeParams {
    pub cases: Vec<FaersSeriousnessCase>,
    #[serde(default)]
    pub min_cases: Option<u32>,
    #[serde(default)]
    pub death_rate_threshold: Option<f64>,
}

/// A single drug entry in a polypharmacy case.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FaersPolypharmacyDrug {
    /// Drug name
    pub name: String,
    /// Drug characterization code
    pub characterization: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FaersPolypharmacyCase {
    pub case_id: String,
    pub drugs: Vec<FaersPolypharmacyDrug>,
    pub event: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FaersPolypharmacyParams {
    pub cases: Vec<FaersPolypharmacyCase>,
    #[serde(default)]
    pub min_pair_count: Option<u32>,
    #[serde(default)]
    pub interaction_threshold: Option<f64>,
}
