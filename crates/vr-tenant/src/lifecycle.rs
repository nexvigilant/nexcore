//! Tenant lifecycle management — suspend, reactivate, offboard, export.

use nexcore_chrono::DateTime;
use serde::{Deserialize, Serialize};
use vr_core::{Tenant, TenantId, TenantStatus, VrError, VrResult};

/// Lifecycle transition request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleTransition {
    pub tenant_id: TenantId,
    pub from_status: TenantStatus,
    pub to_status: TenantStatus,
    pub reason: String,
    pub initiated_by: String,
    pub timestamp: DateTime,
}

/// Valid state transitions for tenant lifecycle.
///
/// Trial → Active (subscription started)
/// Trial → Offboarding (trial expired, no conversion)
/// Active → PastDue (payment failed)
/// Active → Suspended (admin action)
/// Active → Offboarding (cancellation requested)
/// PastDue → Active (payment recovered)
/// PastDue → Suspended (grace period expired)
/// Suspended → Active (reactivation)
/// Suspended → Offboarding (final deprovisioning)
/// Offboarding → Deprovisioned (data archived)
pub fn validate_transition(from: &TenantStatus, to: &TenantStatus) -> VrResult<()> {
    let valid = matches!(
        (from, to),
        (TenantStatus::Trial, TenantStatus::Active)
            | (TenantStatus::Trial, TenantStatus::Offboarding)
            | (TenantStatus::Active, TenantStatus::PastDue)
            | (TenantStatus::Active, TenantStatus::Suspended)
            | (TenantStatus::Active, TenantStatus::Offboarding)
            | (TenantStatus::PastDue, TenantStatus::Active)
            | (TenantStatus::PastDue, TenantStatus::Suspended)
            | (TenantStatus::Suspended, TenantStatus::Active)
            | (TenantStatus::Suspended, TenantStatus::Offboarding)
            | (TenantStatus::Offboarding, TenantStatus::Deprovisioned)
    );

    if valid {
        Ok(())
    } else {
        Err(VrError::InvalidInput {
            message: format!("invalid lifecycle transition: {:?} → {:?}", from, to),
        })
    }
}

/// Create a lifecycle transition record.
pub fn create_transition(
    tenant: &Tenant,
    to_status: TenantStatus,
    reason: &str,
    initiated_by: &str,
) -> VrResult<LifecycleTransition> {
    validate_transition(&tenant.status, &to_status)?;

    Ok(LifecycleTransition {
        tenant_id: tenant.id,
        from_status: tenant.status,
        to_status,
        reason: reason.to_string(),
        initiated_by: initiated_by.to_string(),
        timestamp: DateTime::now(),
    })
}

/// Data retention policy per tenant status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    /// Days to retain data after entering this status.
    pub retention_days: u32,
    /// Whether data export is available.
    pub export_available: bool,
    /// Whether data is read-only (no new writes).
    pub read_only: bool,
}

impl RetentionPolicy {
    /// Get the retention policy for a given tenant status.
    pub fn for_status(status: &TenantStatus) -> Self {
        match status {
            TenantStatus::Trial | TenantStatus::Active => Self {
                retention_days: u32::MAX, // indefinite
                export_available: true,
                read_only: false,
            },
            TenantStatus::PastDue => Self {
                retention_days: 30, // 30-day grace period
                export_available: true,
                read_only: false, // can still work
            },
            TenantStatus::Suspended => Self {
                retention_days: 90, // 90 days to reactivate
                export_available: true,
                read_only: true,
            },
            TenantStatus::Offboarding => Self {
                retention_days: 30, // 30 days to export
                export_available: true,
                read_only: true,
            },
            TenantStatus::Deprovisioned => Self {
                retention_days: 0,
                export_available: false,
                read_only: true,
            },
            _ => Self {
                retention_days: 30,
                export_available: true,
                read_only: true,
            },
        }
    }
}

/// Offboarding checklist — steps required before full deprovisioning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OffboardingChecklist {
    pub tenant_id: TenantId,
    pub data_exported: bool,
    pub active_orders_completed: bool,
    pub marketplace_listings_removed: bool,
    pub api_keys_revoked: bool,
    pub team_members_notified: bool,
    pub billing_finalized: bool,
    pub storage_archived: bool,
}

impl OffboardingChecklist {
    /// Create a new checklist with all items unchecked.
    pub fn new(tenant_id: TenantId) -> Self {
        Self {
            tenant_id,
            data_exported: false,
            active_orders_completed: false,
            marketplace_listings_removed: false,
            api_keys_revoked: false,
            team_members_notified: false,
            billing_finalized: false,
            storage_archived: false,
        }
    }

    /// Check if all required steps are complete.
    pub fn is_complete(&self) -> bool {
        self.active_orders_completed
            && self.marketplace_listings_removed
            && self.api_keys_revoked
            && self.team_members_notified
            && self.billing_finalized
            && self.storage_archived
    }

    /// Count completed steps.
    pub fn completed_count(&self) -> u32 {
        let checks = [
            self.data_exported,
            self.active_orders_completed,
            self.marketplace_listings_removed,
            self.api_keys_revoked,
            self.team_members_notified,
            self.billing_finalized,
            self.storage_archived,
        ];
        checks.iter().filter(|&&c| c).count() as u32
    }

    /// Total number of steps (data_exported is optional).
    pub fn total_required(&self) -> u32 {
        6 // all except data_exported which is optional
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vr_core::SubscriptionTier;

    fn make_tenant(status: TenantStatus) -> Tenant {
        Tenant::new(
            TenantId::new(),
            "Test".into(),
            "test".into(),
            SubscriptionTier::Accelerator,
            status,
            None,
            serde_json::json!({}),
            DateTime::now(),
            DateTime::now(),
        )
    }

    #[test]
    fn valid_transitions() {
        assert!(validate_transition(&TenantStatus::Trial, &TenantStatus::Active).is_ok());
        assert!(validate_transition(&TenantStatus::Active, &TenantStatus::PastDue).is_ok());
        assert!(validate_transition(&TenantStatus::PastDue, &TenantStatus::Active).is_ok());
        assert!(validate_transition(&TenantStatus::Suspended, &TenantStatus::Active).is_ok());
        assert!(
            validate_transition(&TenantStatus::Offboarding, &TenantStatus::Deprovisioned).is_ok()
        );
    }

    #[test]
    fn invalid_transitions() {
        // Can't go from trial directly to suspended
        assert!(validate_transition(&TenantStatus::Trial, &TenantStatus::Suspended).is_err());
        // Can't go backwards from deprovisioned
        assert!(validate_transition(&TenantStatus::Deprovisioned, &TenantStatus::Active).is_err());
        // Can't go from active to trial
        assert!(validate_transition(&TenantStatus::Active, &TenantStatus::Trial).is_err());
    }

    #[test]
    fn create_transition_validates() {
        let tenant = make_tenant(TenantStatus::Active);
        let result = create_transition(&tenant, TenantStatus::PastDue, "payment failed", "system");
        assert!(result.is_ok());

        let result = create_transition(&tenant, TenantStatus::Trial, "??", "system");
        assert!(result.is_err());
    }

    #[test]
    fn retention_policies() {
        let active = RetentionPolicy::for_status(&TenantStatus::Active);
        assert!(!active.read_only);
        assert!(active.export_available);

        let suspended = RetentionPolicy::for_status(&TenantStatus::Suspended);
        assert!(suspended.read_only);
        assert_eq!(suspended.retention_days, 90);

        let offboarding = RetentionPolicy::for_status(&TenantStatus::Offboarding);
        assert_eq!(offboarding.retention_days, 30);
    }

    #[test]
    fn offboarding_checklist() {
        let mut checklist = OffboardingChecklist::new(TenantId::new());
        assert!(!checklist.is_complete());
        assert_eq!(checklist.completed_count(), 0);

        checklist.active_orders_completed = true;
        checklist.marketplace_listings_removed = true;
        checklist.api_keys_revoked = true;
        checklist.team_members_notified = true;
        checklist.billing_finalized = true;
        assert!(!checklist.is_complete()); // storage not archived

        checklist.storage_archived = true;
        assert!(checklist.is_complete());
        assert_eq!(checklist.completed_count(), 6); // data_exported still false but it's optional
    }
}
