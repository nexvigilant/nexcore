//! Pharmacovigilance types — signal results, guardian status
//!
//! Mirrors the types from wksp-api-client that come from nexcore-api.

use serde::{Deserialize, Serialize};

/// Signal detection result (from nexcore-api)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalResult {
    pub a: u64,
    pub b: u64,
    pub c: u64,
    pub d: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prr: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ror: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ic: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ic025: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ebgm: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eb05: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chi_squared: Option<f64>,
    pub signal_detected: bool,
}

/// Guardian homeostasis status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardianStatus {
    pub state: GuardianState,
    pub tick_count: u64,
    pub last_risk_score: f64,
    pub threat_level: ThreatLevel,
}

/// Guardian operational state
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum GuardianState {
    Running,
    Paused,
    Stopped,
    Alert,
}

/// Threat level classification
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ThreatLevel {
    None,
    Low,
    Medium,
    High,
    Critical,
}

/// Naranjo causality assessment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NaranjoResult {
    pub score: i32,
    pub category: String,
    pub answers: Vec<i32>,
}

/// WHO-UMC causality category
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum WhoUmcCategory {
    Certain,
    Probable,
    Possible,
    Unlikely,
    Conditional,
    Unassessable,
}

/// QBRI (Quantitative Benefit-Risk Index) result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QbriResult {
    pub index: f64,
    pub benefit_score: f64,
    pub risk_score: f64,
    pub recommendation: String,
}
