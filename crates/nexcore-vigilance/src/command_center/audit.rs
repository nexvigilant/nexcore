//! Audit domain types: activity logging and tracking.

use chrono::{DateTime, Utc};
use nexcore_id::NexId;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::enums::AuditAction;

/// Audit log entry for complete activity tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    /// Unique identifier.
    pub id: NexId,

    /// User who performed the action (optional for system actions).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<NexId>,

    /// Associated LLC ID (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub llc_id: Option<NexId>,

    /// Action performed.
    pub action: AuditAction,

    /// Entity type (tasks, transactions, users, etc.).
    pub entity_type: String,

    /// Entity ID (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity_id: Option<NexId>,

    /// Human-readable description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Before/after values for changes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub changes: Option<Value>,

    /// Client IP address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,

    /// Client user agent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,

    /// Request path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_path: Option<String>,

    /// Timestamp (when action occurred).
    pub created_at: DateTime<Utc>,
}

impl AuditLog {
    /// Create a new audit log entry.
    #[must_use]
    pub fn new(action: AuditAction, entity_type: impl Into<String>) -> Self {
        Self {
            id: NexId::v4(),
            user_id: None,
            llc_id: None,
            action,
            entity_type: entity_type.into(),
            entity_id: None,
            description: None,
            changes: None,
            ip_address: None,
            user_agent: None,
            request_path: None,
            created_at: Utc::now(),
        }
    }

    /// Builder method: set user ID.
    #[must_use]
    pub fn with_user(mut self, user_id: NexId) -> Self {
        self.user_id = Some(user_id);
        self
    }

    /// Builder method: set LLC ID.
    #[must_use]
    pub fn with_llc(mut self, llc_id: NexId) -> Self {
        self.llc_id = Some(llc_id);
        self
    }

    /// Builder method: set entity ID.
    #[must_use]
    pub fn with_entity(mut self, entity_id: NexId) -> Self {
        self.entity_id = Some(entity_id);
        self
    }

    /// Builder method: set description.
    #[must_use]
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Builder method: set changes.
    #[must_use]
    pub fn with_changes(mut self, changes: Value) -> Self {
        self.changes = Some(changes);
        self
    }

    /// Builder method: set request context.
    #[must_use]
    pub fn with_request_context(
        mut self,
        ip_address: Option<String>,
        user_agent: Option<String>,
        request_path: Option<String>,
    ) -> Self {
        self.ip_address = ip_address;
        self.user_agent = user_agent;
        self.request_path = request_path;
        self
    }
}

/// Create audit log entry with before/after tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditChanges {
    /// Field values before the change.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<Value>,

    /// Field values after the change.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<Value>,

    /// List of fields that changed.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub changed_fields: Vec<String>,
}

impl AuditChanges {
    /// Create a new changes record.
    #[must_use]
    pub fn new(before: Option<Value>, after: Option<Value>) -> Self {
        Self {
            before,
            after,
            changed_fields: Vec::new(),
        }
    }

    /// Create changes for a create action (no before state).
    #[must_use]
    pub fn for_create(after: Value) -> Self {
        Self::new(None, Some(after))
    }

    /// Create changes for a delete action (no after state).
    #[must_use]
    pub fn for_delete(before: Value) -> Self {
        Self::new(Some(before), None)
    }

    /// Create changes for an update action.
    #[must_use]
    pub fn for_update(before: Value, after: Value) -> Self {
        Self::new(Some(before), Some(after))
    }

    /// Add a changed field.
    pub fn add_changed_field(&mut self, field: impl Into<String>) {
        self.changed_fields.push(field.into());
    }

    /// Convert to JSON Value for storage.
    #[must_use]
    pub fn to_value(&self) -> Value {
        serde_json::to_value(self).unwrap_or_default()
    }
}

/// Convenience constructors for common audit scenarios.
impl AuditLog {
    /// Create a login audit entry.
    #[must_use]
    pub fn login(user_id: NexId) -> Self {
        Self::new(AuditAction::Login, "user")
            .with_user(user_id)
            .with_entity(user_id)
            .with_description("User logged in")
    }

    /// Create a logout audit entry.
    #[must_use]
    pub fn logout(user_id: NexId) -> Self {
        Self::new(AuditAction::Logout, "user")
            .with_user(user_id)
            .with_entity(user_id)
            .with_description("User logged out")
    }

    /// Create a failed login audit entry.
    #[must_use]
    pub fn failed_login(email: impl Into<String>) -> Self {
        Self::new(AuditAction::FailedLogin, "user")
            .with_description(format!("Failed login attempt for: {}", email.into()))
    }

    /// Create an entity created audit entry.
    #[must_use]
    pub fn entity_created(
        user_id: NexId,
        entity_type: impl Into<String>,
        entity_id: NexId,
    ) -> Self {
        let entity_type = entity_type.into();
        Self::new(AuditAction::Create, &entity_type)
            .with_user(user_id)
            .with_entity(entity_id)
            .with_description(format!("Created {} {}", entity_type, entity_id))
    }

    /// Create an entity updated audit entry.
    #[must_use]
    pub fn entity_updated(
        user_id: NexId,
        entity_type: impl Into<String>,
        entity_id: NexId,
        changes: AuditChanges,
    ) -> Self {
        let entity_type = entity_type.into();
        Self::new(AuditAction::Update, &entity_type)
            .with_user(user_id)
            .with_entity(entity_id)
            .with_description(format!("Updated {} {}", entity_type, entity_id))
            .with_changes(changes.to_value())
    }

    /// Create an entity deleted audit entry.
    #[must_use]
    pub fn entity_deleted(
        user_id: NexId,
        entity_type: impl Into<String>,
        entity_id: NexId,
    ) -> Self {
        let entity_type = entity_type.into();
        Self::new(AuditAction::Delete, &entity_type)
            .with_user(user_id)
            .with_entity(entity_id)
            .with_description(format!("Deleted {} {}", entity_type, entity_id))
    }

    /// Create an export audit entry.
    #[must_use]
    pub fn data_exported(user_id: NexId, entity_type: impl Into<String>) -> Self {
        let entity_type = entity_type.into();
        Self::new(AuditAction::Export, &entity_type)
            .with_user(user_id)
            .with_description(format!("Exported {} data", entity_type))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_log_new() {
        let log = AuditLog::new(AuditAction::Create, "task");

        assert_eq!(log.action, AuditAction::Create);
        assert_eq!(log.entity_type, "task");
        assert!(log.user_id.is_none());
    }

    #[test]
    fn test_audit_log_builder() {
        let user_id = NexId::v4();
        let llc_id = NexId::v4();
        let entity_id = NexId::v4();

        let log = AuditLog::new(AuditAction::Update, "transaction")
            .with_user(user_id)
            .with_llc(llc_id)
            .with_entity(entity_id)
            .with_description("Updated transaction amount");

        assert_eq!(log.user_id, Some(user_id));
        assert_eq!(log.llc_id, Some(llc_id));
        assert_eq!(log.entity_id, Some(entity_id));
        assert_eq!(
            log.description,
            Some("Updated transaction amount".to_string())
        );
    }

    #[test]
    fn test_audit_changes() {
        let before = serde_json::json!({ "amount": 100 });
        let after = serde_json::json!({ "amount": 150 });

        let mut changes = AuditChanges::for_update(before, after);
        changes.add_changed_field("amount");

        assert!(changes.before.is_some());
        assert!(changes.after.is_some());
        assert_eq!(changes.changed_fields, vec!["amount"]);
    }

    #[test]
    fn test_convenience_constructors() {
        let user_id = NexId::v4();

        let login = AuditLog::login(user_id);
        assert_eq!(login.action, AuditAction::Login);
        assert_eq!(login.user_id, Some(user_id));

        let logout = AuditLog::logout(user_id);
        assert_eq!(logout.action, AuditAction::Logout);

        let failed = AuditLog::failed_login("test@example.com");
        assert_eq!(failed.action, AuditAction::FailedLogin);
        assert!(failed.description.unwrap().contains("test@example.com"));
    }

    #[test]
    fn test_entity_audit() {
        let user_id = NexId::v4();
        let entity_id = NexId::v4();

        let created = AuditLog::entity_created(user_id, "task", entity_id);
        assert_eq!(created.action, AuditAction::Create);

        let deleted = AuditLog::entity_deleted(user_id, "task", entity_id);
        assert_eq!(deleted.action, AuditAction::Delete);
    }
}
