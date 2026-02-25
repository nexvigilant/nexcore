//! Team management — user invitations, roles, activity within a tenant.

use nexcore_chrono::DateTime;
use serde::{Deserialize, Serialize};
use vr_core::{Action, Resource, TenantContext, TenantId, UserId, UserRole, VrError, VrResult};

/// A user within a tenant organization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMember {
    pub id: UserId,
    pub tenant_id: TenantId,
    pub firebase_uid: String,
    pub email: String,
    pub display_name: Option<String>,
    pub role: UserRole,
    pub status: MemberStatus,
    pub invited_by: Option<UserId>,
    pub created_at: DateTime,
    pub last_active_at: Option<DateTime>,
}

/// Status of a team member.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemberStatus {
    /// Invitation sent, not yet accepted.
    Invited,
    /// Active member.
    Active,
    /// Temporarily deactivated.
    Deactivated,
    /// Removed from team.
    Removed,
}

/// Request to invite a new team member.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InviteMemberRequest {
    pub email: String,
    pub role: UserRole,
    pub display_name: Option<String>,
}

/// Validate an invitation request against tenant context and limits.
pub fn validate_invitation(
    ctx: &TenantContext,
    request: &InviteMemberRequest,
    current_member_count: u32,
) -> VrResult<()> {
    // Check permission
    if !ctx.can(Action::Create, Resource::Team) {
        return Err(VrError::PermissionDenied {
            action: Action::Create,
            resource: Resource::Team,
        });
    }

    // Check user limit
    let check = crate::tiers::check_user_limit(ctx.tier(), current_member_count);
    if !check.is_allowed() {
        return Err(VrError::LimitExceeded {
            resource: "users".into(),
            used: current_member_count as u64,
            limit: ctx.tier().max_users().unwrap_or(u32::MAX) as u64,
        });
    }

    // Validate email format (basic check)
    if !request.email.contains('@') || request.email.len() < 5 {
        return Err(VrError::InvalidInput {
            message: "invalid email address".into(),
        });
    }

    // Cannot invite someone with a higher role than yourself
    if request.role.rank() > ctx.role().rank() {
        return Err(VrError::PermissionDenied {
            action: Action::Create,
            resource: Resource::Team,
        });
    }

    Ok(())
}

/// Check if a role change is allowed.
pub fn validate_role_change(
    ctx: &TenantContext,
    target_member: &TeamMember,
    new_role: UserRole,
) -> VrResult<()> {
    // Must have team management permission
    if !ctx.can(Action::Update, Resource::Team) {
        return Err(VrError::PermissionDenied {
            action: Action::Update,
            resource: Resource::Team,
        });
    }

    // Cannot change your own role
    if target_member.id == *ctx.user_id() {
        return Err(VrError::InvalidInput {
            message: "cannot change your own role".into(),
        });
    }

    // Cannot promote to a role higher than your own
    if new_role.rank() > ctx.role().rank() {
        return Err(VrError::PermissionDenied {
            action: Action::Update,
            resource: Resource::Team,
        });
    }

    // Cannot demote someone with a higher role
    if target_member.role.rank() > ctx.role().rank() {
        return Err(VrError::PermissionDenied {
            action: Action::Update,
            resource: Resource::Team,
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use vr_core::SubscriptionTier;

    fn make_ctx(role: UserRole, tier: SubscriptionTier) -> TenantContext {
        TenantContext::new(TenantId::new(), UserId::new(), role, tier)
    }

    #[test]
    fn owner_can_invite_scientist() {
        let ctx = make_ctx(UserRole::Owner, SubscriptionTier::Accelerator);
        let req = InviteMemberRequest {
            email: "scientist@lab.com".into(),
            role: UserRole::Scientist,
            display_name: Some("Dr. Smith".into()),
        };
        assert!(validate_invitation(&ctx, &req, 5).is_ok());
    }

    #[test]
    fn scientist_cannot_invite() {
        let ctx = make_ctx(UserRole::Scientist, SubscriptionTier::Accelerator);
        let req = InviteMemberRequest {
            email: "new@lab.com".into(),
            role: UserRole::Viewer,
            display_name: None,
        };
        assert!(validate_invitation(&ctx, &req, 5).is_err());
    }

    #[test]
    fn admin_cannot_invite_owner() {
        let ctx = make_ctx(UserRole::Admin, SubscriptionTier::Accelerator);
        let req = InviteMemberRequest {
            email: "owner@lab.com".into(),
            role: UserRole::Owner,
            display_name: None,
        };
        assert!(validate_invitation(&ctx, &req, 5).is_err());
    }

    #[test]
    fn invitation_blocked_at_user_limit() {
        let ctx = make_ctx(UserRole::Owner, SubscriptionTier::Explorer);
        let req = InviteMemberRequest {
            email: "fourth@lab.com".into(),
            role: UserRole::Viewer,
            display_name: None,
        };
        // Explorer limit is 3 users
        assert!(validate_invitation(&ctx, &req, 3).is_err());
    }

    #[test]
    fn cannot_change_own_role() {
        let ctx = make_ctx(UserRole::Owner, SubscriptionTier::Enterprise);
        let member = TeamMember {
            id: *ctx.user_id(),
            tenant_id: *ctx.tenant_id(),
            firebase_uid: "uid123".into(),
            email: "owner@lab.com".into(),
            display_name: None,
            role: UserRole::Owner,
            status: MemberStatus::Active,
            invited_by: None,
            created_at: DateTime::now(),
            last_active_at: None,
        };
        assert!(validate_role_change(&ctx, &member, UserRole::Admin).is_err());
    }
}
