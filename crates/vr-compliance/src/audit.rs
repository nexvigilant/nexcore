//! Audit trail capture and querying.
//!
//! Every significant action in the platform generates an [`AuditEvent`].
//! Events are immutable once created and can be queried via [`AuditQuery`]
//! for compliance reporting, incident investigation, and SOC 2 evidence.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use vr_core::{TenantId, UserId};

// ============================================================================
// Event Types
// ============================================================================

/// Classification of audit events for filtering and reporting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventType {
    /// User accessed data (read operations on sensitive resources).
    DataAccess,
    /// User modified data (create, update operations).
    DataModification,
    /// User deleted data (delete operations, GDPR erasure).
    DataDeletion,
    /// Authentication event (login, logout, failed attempt, MFA).
    AuthEvent,
    /// Configuration change (settings, integrations, permissions).
    ConfigChange,
    /// Data export request (CSV, API bulk, GDPR portability).
    ExportRequest,
    /// API key lifecycle (created, rotated, revoked).
    ApiKeyEvent,
    /// Team membership change (invite, remove, role change).
    TeamChange,
    /// Billing event (subscription change, payment, invoice).
    BillingEvent,
    /// Compliance-specific event (consent change, DSR, audit).
    ComplianceEvent,
}

// ============================================================================
// Audit Event
// ============================================================================

/// An immutable record of a significant action in the platform.
///
/// Audit events are append-only — once created, they cannot be modified
/// or deleted (even by GDPR erasure requests, per legal basis of
/// legitimate interest in security audit trails).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Unique identifier for this event.
    pub id: Uuid,
    /// Tenant that generated this event.
    pub tenant_id: TenantId,
    /// User who performed the action.
    pub user_id: UserId,
    /// When the event occurred.
    pub timestamp: DateTime<Utc>,
    /// Classification of the event.
    pub event_type: AuditEventType,
    /// The type of resource acted upon (e.g., "compound", "program").
    pub resource_type: String,
    /// The ID of the specific resource (e.g., compound UUID).
    pub resource_id: String,
    /// Human-readable description of the action (e.g., "updated assay protocol").
    pub action: String,
    /// Structured details about the event (before/after values, parameters).
    pub details: serde_json::Value,
    /// IP address of the client, if available.
    pub ip_address: Option<String>,
    /// User-Agent header of the client, if available.
    pub user_agent: Option<String>,
}

// ============================================================================
// Query
// ============================================================================

/// Filter criteria for querying audit events.
///
/// All filter fields are optional — omitted fields match everything.
/// Multiple filters are combined with AND logic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditQuery {
    /// Required: tenant isolation boundary.
    pub tenant_id: TenantId,
    /// Filter by event type(s). None matches all types.
    pub event_types: Option<Vec<AuditEventType>>,
    /// Filter events on or after this timestamp.
    pub from_date: Option<DateTime<Utc>>,
    /// Filter events on or before this timestamp.
    pub to_date: Option<DateTime<Utc>>,
    /// Filter by the user who performed the action.
    pub user_id: Option<UserId>,
    /// Filter by the type of resource acted upon.
    pub resource_type: Option<String>,
    /// Maximum number of events to return.
    pub limit: u32,
}

impl Default for AuditQuery {
    fn default() -> Self {
        Self {
            tenant_id: TenantId::new(),
            event_types: None,
            from_date: None,
            to_date: None,
            user_id: None,
            resource_type: None,
            limit: 100,
        }
    }
}

// ============================================================================
// Query Matching
// ============================================================================

/// Check whether a single audit event matches the given query criteria.
///
/// Returns `true` if the event satisfies ALL specified filters.
/// Unset (None) filters are treated as wildcards that match everything.
#[must_use]
pub fn matches_query(event: &AuditEvent, query: &AuditQuery) -> bool {
    // Tenant isolation is mandatory
    if event.tenant_id != query.tenant_id {
        return false;
    }

    // Event type filter
    if let Some(ref types) = query.event_types {
        if !types.contains(&event.event_type) {
            return false;
        }
    }

    // Date range filter
    if let Some(ref from) = query.from_date {
        if event.timestamp < *from {
            return false;
        }
    }

    if let Some(ref to) = query.to_date {
        if event.timestamp > *to {
            return false;
        }
    }

    // User filter
    if let Some(ref uid) = query.user_id {
        if event.user_id != *uid {
            return false;
        }
    }

    // Resource type filter
    if let Some(ref rtype) = query.resource_type {
        if event.resource_type != *rtype {
            return false;
        }
    }

    true
}

/// Filter a slice of audit events using the given query criteria.
///
/// Returns references to matching events, up to `query.limit` results.
/// Events are returned in the order they appear in the input slice.
#[must_use]
pub fn filter_events<'a>(events: &'a [AuditEvent], query: &AuditQuery) -> Vec<&'a AuditEvent> {
    events
        .iter()
        .filter(|e| matches_query(e, query))
        .take(query.limit as usize)
        .collect()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn make_event(tenant_id: TenantId, user_id: UserId, event_type: AuditEventType) -> AuditEvent {
        AuditEvent {
            id: Uuid::new_v4(),
            tenant_id,
            user_id,
            timestamp: Utc::now(),
            event_type,
            resource_type: "compound".to_string(),
            resource_id: Uuid::new_v4().to_string(),
            action: "test action".to_string(),
            details: serde_json::json!({}),
            ip_address: Some("192.168.1.1".to_string()),
            user_agent: Some("test-agent/1.0".to_string()),
        }
    }

    #[test]
    fn matches_query_filters_by_tenant() {
        let tenant_a = TenantId::new();
        let tenant_b = TenantId::new();
        let user = UserId::new();

        let event = make_event(tenant_a, user, AuditEventType::DataAccess);

        let query_a = AuditQuery {
            tenant_id: tenant_a,
            ..AuditQuery::default()
        };
        let query_b = AuditQuery {
            tenant_id: tenant_b,
            ..AuditQuery::default()
        };

        assert!(matches_query(&event, &query_a));
        assert!(!matches_query(&event, &query_b));
    }

    #[test]
    fn matches_query_filters_by_event_type() {
        let tenant = TenantId::new();
        let user = UserId::new();

        let event = make_event(tenant, user, AuditEventType::DataModification);

        let query_match = AuditQuery {
            tenant_id: tenant,
            event_types: Some(vec![
                AuditEventType::DataModification,
                AuditEventType::DataDeletion,
            ]),
            ..AuditQuery::default()
        };
        let query_no_match = AuditQuery {
            tenant_id: tenant,
            event_types: Some(vec![AuditEventType::AuthEvent]),
            ..AuditQuery::default()
        };

        assert!(matches_query(&event, &query_match));
        assert!(!matches_query(&event, &query_no_match));
    }

    #[test]
    fn matches_query_filters_by_date_range() {
        let tenant = TenantId::new();
        let user = UserId::new();

        let mut event = make_event(tenant, user, AuditEventType::DataAccess);
        let now = Utc::now();
        event.timestamp = now;

        // Event is within range
        let query_in_range = AuditQuery {
            tenant_id: tenant,
            from_date: Some(now - Duration::hours(1)),
            to_date: Some(now + Duration::hours(1)),
            ..AuditQuery::default()
        };
        assert!(matches_query(&event, &query_in_range));

        // Event is before from_date
        let query_too_early = AuditQuery {
            tenant_id: tenant,
            from_date: Some(now + Duration::hours(1)),
            ..AuditQuery::default()
        };
        assert!(!matches_query(&event, &query_too_early));

        // Event is after to_date
        let query_too_late = AuditQuery {
            tenant_id: tenant,
            to_date: Some(now - Duration::hours(1)),
            ..AuditQuery::default()
        };
        assert!(!matches_query(&event, &query_too_late));
    }

    #[test]
    fn matches_query_filters_by_user() {
        let tenant = TenantId::new();
        let user_a = UserId::new();
        let user_b = UserId::new();

        let event = make_event(tenant, user_a, AuditEventType::ConfigChange);

        let query_match = AuditQuery {
            tenant_id: tenant,
            user_id: Some(user_a),
            ..AuditQuery::default()
        };
        let query_no_match = AuditQuery {
            tenant_id: tenant,
            user_id: Some(user_b),
            ..AuditQuery::default()
        };

        assert!(matches_query(&event, &query_match));
        assert!(!matches_query(&event, &query_no_match));
    }

    #[test]
    fn matches_query_filters_by_resource_type() {
        let tenant = TenantId::new();
        let user = UserId::new();

        let event = make_event(tenant, user, AuditEventType::DataAccess);

        let query_match = AuditQuery {
            tenant_id: tenant,
            resource_type: Some("compound".to_string()),
            ..AuditQuery::default()
        };
        let query_no_match = AuditQuery {
            tenant_id: tenant,
            resource_type: Some("program".to_string()),
            ..AuditQuery::default()
        };

        assert!(matches_query(&event, &query_match));
        assert!(!matches_query(&event, &query_no_match));
    }

    #[test]
    fn filter_events_respects_limit() {
        let tenant = TenantId::new();
        let user = UserId::new();

        let events: Vec<AuditEvent> = (0..10)
            .map(|_| make_event(tenant, user, AuditEventType::DataAccess))
            .collect();

        let query = AuditQuery {
            tenant_id: tenant,
            limit: 3,
            ..AuditQuery::default()
        };

        let results = filter_events(&events, &query);
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn filter_events_returns_matching_only() {
        let tenant = TenantId::new();
        let other_tenant = TenantId::new();
        let user = UserId::new();

        let events = vec![
            make_event(tenant, user, AuditEventType::DataAccess),
            make_event(other_tenant, user, AuditEventType::DataAccess),
            make_event(tenant, user, AuditEventType::AuthEvent),
        ];

        let query = AuditQuery {
            tenant_id: tenant,
            event_types: Some(vec![AuditEventType::DataAccess]),
            ..AuditQuery::default()
        };

        let results = filter_events(&events, &query);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].tenant_id, tenant);
        assert_eq!(results[0].event_type, AuditEventType::DataAccess);
    }

    #[test]
    fn audit_event_serialization_roundtrip() {
        let event = make_event(
            TenantId::new(),
            UserId::new(),
            AuditEventType::ComplianceEvent,
        );
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: AuditEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, event.id);
        assert_eq!(deserialized.event_type, AuditEventType::ComplianceEvent);
    }

    #[test]
    fn default_query_limit_is_100() {
        let query = AuditQuery::default();
        assert_eq!(query.limit, 100);
    }

    #[test]
    fn none_filters_match_everything() {
        let tenant = TenantId::new();
        let user = UserId::new();

        let event = make_event(tenant, user, AuditEventType::BillingEvent);

        // Query with no optional filters — should match
        let query = AuditQuery {
            tenant_id: tenant,
            event_types: None,
            from_date: None,
            to_date: None,
            user_id: None,
            resource_type: None,
            limit: 100,
        };

        assert!(matches_query(&event, &query));
    }
}
