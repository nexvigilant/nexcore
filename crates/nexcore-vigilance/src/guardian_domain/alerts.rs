//! Alert models for healthcare security incidents.
//!
//! These models represent security alerts and compliance incidents in the
//! healthcare environment, including metadata for regulatory audit trails
//! and incident response.

use chrono::{DateTime, Utc};
use nexcore_id::NexId;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;

use super::error::{GuardianError, GuardianResult};

/// Get the source validation regex (compiled once).
fn source_regex() -> Option<&'static Regex> {
    static REGEX: OnceLock<Option<Regex>> = OnceLock::new();
    REGEX
        .get_or_init(|| Regex::new(r"^[a-zA-Z0-9._-]+$").ok())
        .as_ref()
}

/// Get the tenant ID validation regex (compiled once).
fn tenant_id_regex() -> Option<&'static Regex> {
    static REGEX: OnceLock<Option<Regex>> = OnceLock::new();
    REGEX
        .get_or_init(|| Regex::new(r"^[a-zA-Z0-9-]+$").ok())
        .as_ref()
}

/// Get the HTML tag sanitization regex (compiled once).
fn html_tag_regex() -> Option<&'static Regex> {
    static REGEX: OnceLock<Option<Regex>> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"<[^>]*>").ok()).as_ref()
}

/// Get the JavaScript protocol sanitization regex (compiled once).
fn js_protocol_regex() -> Option<&'static Regex> {
    static REGEX: OnceLock<Option<Regex>> = OnceLock::new();
    REGEX
        .get_or_init(|| Regex::new(r"(?i)javascript:").ok())
        .as_ref()
}

/// Get the event handler sanitization regex (compiled once).
fn event_handler_regex() -> Option<&'static Regex> {
    static REGEX: OnceLock<Option<Regex>> = OnceLock::new();
    REGEX
        .get_or_init(|| Regex::new(r"(?i)on\w+\s*=").ok())
        .as_ref()
}

/// Alert severity levels for healthcare security incidents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AlertSeverity {
    /// Critical severity - immediate attention required.
    Critical,
    /// Warning severity - potential issue detected.
    Warning,
    /// Informational - for logging/audit purposes.
    #[default]
    Info,
}

/// Alert status for incident management tracking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AlertStatus {
    /// Alert is active and requires attention.
    #[default]
    Active,
    /// Alert has been acknowledged by an operator.
    Acknowledged,
    /// Alert has been resolved.
    Resolved,
}

/// Healthcare security alert.
///
/// Represents security alerts and compliance incidents in the healthcare environment.
/// Includes metadata for regulatory audit trails and incident response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    /// Unique alert identifier.
    pub id: NexId,
    /// Alert severity level.
    pub severity: AlertSeverity,
    /// Current alert status.
    #[serde(default)]
    pub status: AlertStatus,
    /// Alert message content (1-2000 chars, sanitized).
    pub message: String,
    /// Alert source system.
    pub source: String,
    /// Timestamp when alert was created.
    #[serde(default = "Utc::now")]
    pub timestamp: DateTime<Utc>,
    /// Additional alert metadata.
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
    /// Tenant identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
    /// User who acknowledged the alert.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acknowledged_by: Option<String>,
    /// Timestamp when alert was resolved.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_at: Option<DateTime<Utc>>,
    /// Firestore TTL field for automatic expiration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
}

impl Alert {
    /// Create a new alert with validation.
    pub fn new(
        severity: AlertSeverity,
        message: impl Into<String>,
        source: impl Into<String>,
    ) -> GuardianResult<Self> {
        let message = sanitize_message(&message.into())?;
        let source = validate_source(&source.into())?;

        Ok(Self {
            id: NexId::v4(),
            severity,
            status: AlertStatus::Active,
            message,
            source,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
            tenant_id: None,
            acknowledged_by: None,
            resolved_at: None,
            expires_at: None,
        })
    }

    /// Set the tenant ID with validation.
    pub fn with_tenant_id(mut self, tenant_id: impl Into<String>) -> GuardianResult<Self> {
        let tenant_id = tenant_id.into().trim().to_string();
        if !tenant_id.is_empty() {
            validate_tenant_id(&tenant_id)?;
            self.tenant_id = Some(tenant_id);
        }
        Ok(self)
    }

    /// Set metadata with validation.
    pub fn with_metadata(
        mut self,
        metadata: HashMap<String, serde_json::Value>,
    ) -> GuardianResult<Self> {
        let sanitized = sanitize_metadata(metadata)?;
        self.metadata = sanitized;
        Ok(self)
    }
}

/// Model for updating alert status and acknowledgment.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AlertUpdate {
    /// New status for the alert.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<AlertStatus>,
    /// User acknowledging the alert.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acknowledged_by: Option<String>,
}

impl AlertUpdate {
    /// Create a new alert update.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the new status.
    pub fn with_status(mut self, status: AlertStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// Set the acknowledging user with sanitization.
    pub fn with_acknowledged_by(mut self, user: impl Into<String>) -> Self {
        let sanitized = sanitize_user_identifier(&user.into());
        if !sanitized.is_empty() {
            self.acknowledged_by = Some(sanitized);
        }
        self
    }
}

/// Sanitize alert message by removing XSS vectors.
fn sanitize_message(message: &str) -> GuardianResult<String> {
    let trimmed = message.trim();
    if trimmed.is_empty() {
        return Err(GuardianError::Validation(
            "Alert message cannot be empty".to_string(),
        ));
    }
    if trimmed.len() > 2000 {
        return Err(GuardianError::FieldTooLong {
            field: "message".to_string(),
            max_length: 2000,
        });
    }

    let mut cleaned = trimmed.to_string();

    // Apply sanitization if regexes are available
    if let Some(re) = html_tag_regex() {
        cleaned = re.replace_all(&cleaned, "").to_string();
    }
    if let Some(re) = js_protocol_regex() {
        cleaned = re.replace_all(&cleaned, "").to_string();
    }
    if let Some(re) = event_handler_regex() {
        cleaned = re.replace_all(&cleaned, "").to_string();
    }

    Ok(cleaned.trim().to_string())
}

/// Validate alert source format.
fn validate_source(source: &str) -> GuardianResult<String> {
    let trimmed = source.trim();
    if trimmed.is_empty() {
        return Err(GuardianError::Validation(
            "Alert source cannot be empty".to_string(),
        ));
    }
    if trimmed.len() > 200 {
        return Err(GuardianError::FieldTooLong {
            field: "source".to_string(),
            max_length: 200,
        });
    }

    // Validate format if regex is available
    if let Some(re) = source_regex() {
        if !re.is_match(trimmed) {
            return Err(GuardianError::InvalidFormat {
                field: "source".to_string(),
                reason: "Must contain only alphanumeric characters, dots, hyphens, and underscores"
                    .to_string(),
            });
        }
    }

    Ok(trimmed.to_string())
}

/// Validate tenant ID format.
fn validate_tenant_id(tenant_id: &str) -> GuardianResult<()> {
    if tenant_id.len() > 100 {
        return Err(GuardianError::FieldTooLong {
            field: "tenant_id".to_string(),
            max_length: 100,
        });
    }

    // Validate format if regex is available
    if let Some(re) = tenant_id_regex() {
        if !re.is_match(tenant_id) {
            return Err(GuardianError::InvalidFormat {
                field: "tenant_id".to_string(),
                reason: "Must contain only alphanumeric characters and hyphens".to_string(),
            });
        }
    }

    Ok(())
}

/// Sanitize metadata values.
fn sanitize_metadata(
    metadata: HashMap<String, serde_json::Value>,
) -> GuardianResult<HashMap<String, serde_json::Value>> {
    let serialized = serde_json::to_string(&metadata)?;
    if serialized.len() > 10000 {
        return Err(GuardianError::FieldTooLong {
            field: "metadata".to_string(),
            max_length: 10000,
        });
    }

    let mut sanitized = HashMap::new();
    for (key, value) in metadata {
        if key.len() > 100 {
            return Err(GuardianError::FieldTooLong {
                field: "metadata key".to_string(),
                max_length: 100,
            });
        }

        let sanitized_value = if let serde_json::Value::String(s) = value {
            let mut cleaned = s;
            if let Some(re) = html_tag_regex() {
                cleaned = re.replace_all(&cleaned, "").to_string();
            }
            if let Some(re) = js_protocol_regex() {
                cleaned = re.replace_all(&cleaned, "").to_string();
            }
            serde_json::Value::String(cleaned)
        } else {
            value
        };

        sanitized.insert(key, sanitized_value);
    }

    Ok(sanitized)
}

/// Sanitize user identifier by removing potential injection vectors.
fn sanitize_user_identifier(user: &str) -> String {
    let trimmed = user.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    // Remove angle brackets, quotes
    trimmed
        .chars()
        .filter(|c| !matches!(*c, '<' | '>' | '"' | '\''))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alert_creation() -> GuardianResult<()> {
        let alert = Alert::new(AlertSeverity::Critical, "Test message", "test-source")?;
        assert_eq!(alert.severity, AlertSeverity::Critical);
        assert_eq!(alert.message, "Test message");
        assert_eq!(alert.source, "test-source");
        assert_eq!(alert.status, AlertStatus::Active);
        Ok(())
    }

    #[test]
    fn test_alert_xss_sanitization() -> GuardianResult<()> {
        let alert = Alert::new(
            AlertSeverity::Info,
            "<script>alert('xss')</script>Test",
            "source",
        )?;
        assert!(!alert.message.contains("<script>"));
        Ok(())
    }

    #[test]
    fn test_invalid_source() {
        let result = Alert::new(AlertSeverity::Info, "Test", "invalid source!");
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_message() {
        let result = Alert::new(AlertSeverity::Info, "   ", "source");
        assert!(result.is_err());
    }

    #[test]
    fn test_alert_update() {
        let update = AlertUpdate::new()
            .with_status(AlertStatus::Acknowledged)
            .with_acknowledged_by("user<script>");

        assert_eq!(update.status, Some(AlertStatus::Acknowledged));
        // Sanitization removes < and > but leaves the text between them
        assert_eq!(update.acknowledged_by, Some("userscript".to_string()));
    }

    #[test]
    fn test_severity_default() {
        assert_eq!(AlertSeverity::default(), AlertSeverity::Info);
    }

    #[test]
    fn test_status_default() {
        assert_eq!(AlertStatus::default(), AlertStatus::Active);
    }
}
