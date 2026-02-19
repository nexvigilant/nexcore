//! Tenant provisioning — self-service signup, resource allocation, onboarding.
//!
//! Target: signup to first compound registration in under 15 minutes.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use vr_core::{SubscriptionTier, Tenant, TenantId, TenantStatus, UserId};

/// Self-service signup request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignupRequest {
    pub organization_name: String,
    pub admin_email: String,
    pub admin_name: String,
    pub firebase_uid: String,
    pub therapeutic_area: Option<String>,
    pub team_size_estimate: Option<u32>,
}

/// Resources provisioned for a new tenant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisioningPlan {
    pub tenant_id: TenantId,
    pub admin_user_id: UserId,
    pub tier: SubscriptionTier,
    pub trial_ends_at: DateTime<Utc>,
    pub storage_prefix: String,
    pub steps: Vec<ProvisioningStep>,
}

/// Individual step in the provisioning pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisioningStep {
    pub name: String,
    pub status: StepStatus,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

/// Generate a URL-safe slug from an organization name.
pub fn generate_slug(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_ascii_lowercase()
            } else if c == ' ' || c == '-' || c == '_' {
                '-'
            } else {
                '_'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Create a provisioning plan for a new tenant signup.
/// All new tenants start on a 14-day trial with Accelerator features.
pub fn create_provisioning_plan(request: &SignupRequest) -> ProvisioningPlan {
    let tenant_id = TenantId::new();
    let admin_user_id = UserId::new();
    let now = Utc::now();
    let trial_duration = Duration::days(14);
    let trial_ends_at = now + trial_duration;

    let slug = generate_slug(&request.organization_name);
    let storage_prefix = format!("tenants/{}/", tenant_id.as_uuid());

    let steps = vec![
        ProvisioningStep {
            name: "create_tenant_record".into(),
            status: StepStatus::Pending,
            description: "Create tenant record in database".into(),
        },
        ProvisioningStep {
            name: "create_admin_user".into(),
            status: StepStatus::Pending,
            description: "Create admin user and link Firebase UID".into(),
        },
        ProvisioningStep {
            name: "provision_storage".into(),
            status: StepStatus::Pending,
            description: format!("Create storage prefix: {}", storage_prefix),
        },
        ProvisioningStep {
            name: "initialize_firestore".into(),
            status: StepStatus::Pending,
            description: "Initialize Firestore tenant collection".into(),
        },
        ProvisioningStep {
            name: "configure_feature_flags".into(),
            status: StepStatus::Pending,
            description: "Set Accelerator-tier feature flags for trial".into(),
        },
        ProvisioningStep {
            name: "create_sample_program".into(),
            status: StepStatus::Pending,
            description: "Create template program based on therapeutic area".into(),
        },
    ];

    ProvisioningPlan {
        tenant_id,
        admin_user_id,
        tier: SubscriptionTier::Accelerator, // trial gets Accelerator features
        trial_ends_at,
        storage_prefix,
        steps,
    }
}

/// Build the initial Tenant record from a provisioning plan.
pub fn build_tenant_record(plan: &ProvisioningPlan, request: &SignupRequest) -> Tenant {
    let now = Utc::now();
    Tenant {
        id: plan.tenant_id,
        name: request.organization_name.clone(),
        slug: generate_slug(&request.organization_name),
        tier: plan.tier,
        status: TenantStatus::Trial,
        trial_ends_at: Some(plan.trial_ends_at),
        settings: serde_json::json!({
            "therapeutic_area": request.therapeutic_area,
            "team_size_estimate": request.team_size_estimate,
            "onboarding_completed": false,
        }),
        created_at: now,
        updated_at: now,
    }
}

/// Check if a trial has expired.
pub fn is_trial_expired(tenant: &Tenant) -> bool {
    if tenant.status != TenantStatus::Trial {
        return false;
    }
    match tenant.trial_ends_at {
        Some(ends_at) => Utc::now() > ends_at,
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_generation() {
        assert_eq!(generate_slug("Acme Therapeutics"), "acme-therapeutics");
        assert_eq!(generate_slug("BioGen Labs Inc."), "biogen-labs-inc_");
        assert_eq!(generate_slug("  Multiple   Spaces  "), "multiple-spaces");
    }

    #[test]
    fn provisioning_plan_has_correct_steps() {
        let request = SignupRequest {
            organization_name: "Test Lab".into(),
            admin_email: "admin@testlab.com".into(),
            admin_name: "Dr. Admin".into(),
            firebase_uid: "uid_123".into(),
            therapeutic_area: Some("oncology".into()),
            team_size_estimate: Some(5),
        };

        let plan = create_provisioning_plan(&request);
        assert_eq!(plan.steps.len(), 6);
        assert_eq!(plan.tier, SubscriptionTier::Accelerator);
        assert!(plan.trial_ends_at > Utc::now());

        // Trial should be ~14 days from now
        let duration = plan.trial_ends_at - Utc::now();
        assert!(duration.num_days() >= 13 && duration.num_days() <= 14);
    }

    #[test]
    fn tenant_record_from_plan() {
        let request = SignupRequest {
            organization_name: "Pharma Co".into(),
            admin_email: "admin@pharma.co".into(),
            admin_name: "Admin".into(),
            firebase_uid: "uid_456".into(),
            therapeutic_area: None,
            team_size_estimate: None,
        };

        let plan = create_provisioning_plan(&request);
        let tenant = build_tenant_record(&plan, &request);

        assert_eq!(tenant.name, "Pharma Co");
        assert_eq!(tenant.slug, "pharma-co");
        assert_eq!(tenant.status, TenantStatus::Trial);
        assert!(tenant.trial_ends_at.is_some());
    }

    #[test]
    fn trial_expiry_check() {
        let mut tenant = Tenant {
            id: TenantId::new(),
            name: "Test".into(),
            slug: "test".into(),
            tier: SubscriptionTier::Accelerator,
            status: TenantStatus::Trial,
            trial_ends_at: Some(Utc::now() - Duration::days(1)),
            settings: serde_json::json!({}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert!(is_trial_expired(&tenant));

        // Future trial — not expired
        tenant.trial_ends_at = Some(Utc::now() + Duration::days(7));
        assert!(!is_trial_expired(&tenant));

        // Active tenant — not a trial
        tenant.status = TenantStatus::Active;
        assert!(!is_trial_expired(&tenant));
    }
}
