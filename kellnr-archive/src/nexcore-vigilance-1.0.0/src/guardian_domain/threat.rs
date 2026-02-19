//! NIST Zero Trust Threat Response Models.
//!
//! Automated security threat detection and response per NIST SP 800-207.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Threat severity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ThreatLevel {
    Low,
    #[default]
    Medium,
    High,
    Critical,
}

/// Types of security threats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ThreatType {
    #[default]
    FailedLogin,
    BruteForce,
    SuspiciousIp,
    PolicyViolation,
    UnauthorizedAccess,
    DataExfiltration,
    MfaBypassAttempt,
    AnomalousBehavior,
}

/// Automated response actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ResponseAction {
    AccountLockout,
    IpBlock,
    SessionTermination,
    MfaStepUp,
    AlertAdmin,
    LogIncident,
}

/// Account lockout status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AccountLockoutStatus {
    #[default]
    Active,
    Unlocked,
    Expired,
}

/// IP block status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IPBlockStatus {
    #[default]
    Active,
    Unblocked,
    Expired,
}

/// Security threat event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatEvent {
    pub id: String,
    pub threat_type: ThreatType,
    pub threat_level: ThreatLevel,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_email: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    pub ip_address: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    /// Risk score (0-100).
    pub risk_score: i32,
    /// Detection confidence (0.0-1.0).
    pub confidence: f64,
    #[serde(default)]
    pub response_actions: Vec<ResponseAction>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_timestamp: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_details: Option<HashMap<String, serde_json::Value>>,
    #[serde(default = "Utc::now")]
    pub detected_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_by: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    pub tenant_id: String,
}

/// Account lockout record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountLockout {
    pub id: String,
    pub user_id: String,
    pub user_email: String,
    pub reason: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub threat_event_id: Option<String>,
    #[serde(default = "Utc::now")]
    pub locked_at: DateTime<Utc>,
    #[serde(default = "default_system")]
    pub locked_by: String,
    /// Lockout duration in minutes (None = indefinite).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_minutes: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unlock_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub status: AccountLockoutStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unlocked_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unlocked_by: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unlock_reason: Option<String>,
    pub tenant_id: String,
}

fn default_system() -> String {
    "system".to_string()
}

/// IP address block record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IPBlock {
    pub id: String,
    pub ip_address: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ip_range: Option<String>,
    pub reason: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub threat_event_id: Option<String>,
    #[serde(default = "Utc::now")]
    pub blocked_at: DateTime<Utc>,
    #[serde(default = "default_system")]
    pub blocked_by: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_minutes: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unblock_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub status: IPBlockStatus,
    #[serde(default)]
    pub blocked_requests: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_attempt_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unblocked_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unblocked_by: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unblock_reason: Option<String>,
    pub tenant_id: String,
}

/// Session termination record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionTermination {
    pub id: String,
    pub session_id: String,
    pub user_id: String,
    pub user_email: String,
    pub reason: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub threat_event_id: Option<String>,
    #[serde(default = "Utc::now")]
    pub terminated_at: DateTime<Utc>,
    #[serde(default = "default_system")]
    pub terminated_by: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    pub tenant_id: String,
}

/// MFA step-up requirement record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MFAStepUp {
    pub id: String,
    pub user_id: String,
    pub user_email: String,
    pub session_id: String,
    pub reason: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub threat_event_id: Option<String>,
    #[serde(default = "Utc::now")]
    pub required_at: DateTime<Utc>,
    pub mfa_method: String,
    #[serde(default)]
    pub challenge_sent: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub challenge_sent_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub verified: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verified_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub attempts: i32,
    pub expires_at: DateTime<Utc>,
    #[serde(default)]
    pub expired: bool,
    pub tenant_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_threat_level_default() {
        assert_eq!(ThreatLevel::default(), ThreatLevel::Medium);
    }

    #[test]
    fn test_threat_type_default() {
        assert_eq!(ThreatType::default(), ThreatType::FailedLogin);
    }

    #[test]
    fn test_lockout_status_default() {
        assert_eq!(
            AccountLockoutStatus::default(),
            AccountLockoutStatus::Active
        );
    }
}
