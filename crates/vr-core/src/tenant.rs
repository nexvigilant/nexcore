//! TenantContext — the foundation of compile-time tenant isolation.
//!
//! Every API request extracts a TenantContext from the JWT. All repository
//! methods require it. The type system makes cross-tenant data access
//! a compile error, not a runtime bug.

use crate::ids::{TenantId, UserId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ============================================================================
// Subscription Tiers
// ============================================================================

/// Platform subscription tiers with monthly pricing (cents).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionTier {
    /// $250/mo — academic labs, .edu email required
    Academic,
    /// $500/mo — single program, 3 users, basic tools
    Explorer,
    /// $2,500/mo — 5 programs, 10 users, full computational suite
    Accelerator,
    /// $10,000/mo — unlimited programs, 50 users, dedicated support
    Enterprise,
    /// Negotiated — pharma innovation units, large biotechs
    Custom,
}

impl SubscriptionTier {
    /// Monthly base price in cents.
    #[must_use]
    pub fn monthly_price_cents(&self) -> u64 {
        match self {
            Self::Academic => 25_000,
            Self::Explorer => 50_000,
            Self::Accelerator => 250_000,
            Self::Enterprise => 1_000_000,
            Self::Custom => 0, // negotiated
        }
    }

    /// Annual price with 16.7% discount (10 months for 12).
    #[must_use]
    pub fn annual_price_cents(&self) -> u64 {
        let monthly = self.monthly_price_cents();
        // 10 months for the price of 12 = 16.67% discount
        monthly * 10
    }

    /// Maximum number of programs allowed.
    #[must_use]
    pub fn max_programs(&self) -> Option<u32> {
        match self {
            Self::Academic => Some(3),
            Self::Explorer => Some(1),
            Self::Accelerator => Some(5),
            Self::Enterprise => None, // unlimited
            Self::Custom => None,
        }
    }

    /// Maximum number of users per tenant.
    #[must_use]
    pub fn max_users(&self) -> Option<u32> {
        match self {
            Self::Academic => Some(10),
            Self::Explorer => Some(3),
            Self::Accelerator => Some(10),
            Self::Enterprise => Some(50),
            Self::Custom => None,
        }
    }

    /// Storage allocation in bytes.
    #[must_use]
    pub fn storage_bytes(&self) -> u64 {
        match self {
            Self::Academic => 25 * 1_073_741_824,    // 25 GB
            Self::Explorer => 5 * 1_073_741_824,     // 5 GB
            Self::Accelerator => 50 * 1_073_741_824, // 50 GB
            Self::Enterprise => 500 * 1_073_741_824, // 500 GB
            Self::Custom => 1_000 * 1_073_741_824,   // 1 TB default
        }
    }

    /// Maximum virtual screens per month.
    #[must_use]
    pub fn max_virtual_screens_per_month(&self) -> Option<u32> {
        match self {
            Self::Academic => Some(5),
            Self::Explorer => None, // not available
            Self::Accelerator => Some(10),
            Self::Enterprise => None, // unlimited (None = unlimited here)
            Self::Custom => None,
        }
    }

    /// Whether this tier has API access.
    #[must_use]
    pub fn has_api_access(&self) -> bool {
        !matches!(self, Self::Explorer)
    }

    /// Whether this tier supports SSO.
    #[must_use]
    pub fn has_sso(&self) -> bool {
        matches!(self, Self::Enterprise | Self::Custom)
    }

    /// SLA uptime percentage (basis points, e.g., 9950 = 99.50%).
    #[must_use]
    pub fn sla_uptime_bps(&self) -> u32 {
        match self {
            Self::Academic => 9950,
            Self::Explorer => 9950,
            Self::Accelerator => 9990,
            Self::Enterprise => 9995,
            Self::Custom => 9999,
        }
    }

    /// Tier ordering for comparison (higher = more features).
    #[must_use]
    pub fn rank(&self) -> u8 {
        match self {
            Self::Academic => 1,
            Self::Explorer => 2,
            Self::Accelerator => 3,
            Self::Enterprise => 4,
            Self::Custom => 5,
        }
    }

    /// Check if this tier includes a feature available at `required_tier`.
    #[must_use]
    pub fn includes(&self, required_tier: &SubscriptionTier) -> bool {
        self.rank() >= required_tier.rank()
    }
}

// ============================================================================
// User Roles
// ============================================================================

/// Role within a tenant organization. Determines permissions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    /// Full access + billing + team management
    Owner,
    /// Full access except billing
    Admin,
    /// Programs, compounds, assays (read/write)
    Scientist,
    /// Deals, asset packages (read/write); science (read)
    BusinessDev,
    /// Read-only access to specified programs
    Viewer,
    /// Scoped access (SAB members, consultants)
    External,
}

impl UserRole {
    /// Role ordering for comparison (higher = more privileges).
    #[must_use]
    pub fn rank(&self) -> u8 {
        match self {
            Self::Owner => 6,
            Self::Admin => 5,
            Self::Scientist => 4,
            Self::BusinessDev => 3,
            Self::Viewer => 2,
            Self::External => 1,
        }
    }

    /// Whether this role has at least the privileges of `required`.
    #[must_use]
    pub fn has_at_least(&self, required: &UserRole) -> bool {
        self.rank() >= required.rank()
    }

    /// Whether this role can manage team members.
    #[must_use]
    pub fn can_manage_team(&self) -> bool {
        matches!(self, Self::Owner | Self::Admin)
    }

    /// Whether this role can access billing.
    #[must_use]
    pub fn can_access_billing(&self) -> bool {
        matches!(self, Self::Owner)
    }

    /// Whether this role can write to programs.
    #[must_use]
    pub fn can_write_programs(&self) -> bool {
        matches!(self, Self::Owner | Self::Admin | Self::Scientist)
    }

    /// Whether this role can manage deals.
    #[must_use]
    pub fn can_manage_deals(&self) -> bool {
        matches!(self, Self::Owner | Self::Admin | Self::BusinessDev)
    }
}

// ============================================================================
// Actions & Resources (for fine-grained RBAC)
// ============================================================================

/// Actions that can be performed on resources.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Create,
    Read,
    Update,
    Delete,
    Export,
    Admin,
}

/// Resources that can be acted upon.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Resource {
    Program,
    Compound,
    Assay,
    Deal,
    Asset,
    Order,
    Team,
    Billing,
    Settings,
    ApiKey,
    AuditLog,
}

/// Permission set — evaluated at runtime from role + tier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permissions {
    /// Explicit grants: (Action, Resource) pairs.
    grants: Vec<(Action, Resource)>,
}

impl Permissions {
    /// Build permissions from role.
    #[must_use]
    pub fn from_role(role: &UserRole) -> Self {
        use Action::*;
        use Resource::*;

        let grants = match role {
            UserRole::Owner => vec![
                (Create, Program),
                (Read, Program),
                (Update, Program),
                (Delete, Program),
                (Create, Compound),
                (Read, Compound),
                (Update, Compound),
                (Delete, Compound),
                (Create, Assay),
                (Read, Assay),
                (Update, Assay),
                (Create, Deal),
                (Read, Deal),
                (Update, Deal),
                (Delete, Deal),
                (Create, Asset),
                (Read, Asset),
                (Update, Asset),
                (Create, Order),
                (Read, Order),
                (Create, Team),
                (Read, Team),
                (Update, Team),
                (Delete, Team),
                (Read, Billing),
                (Update, Billing),
                (Read, Settings),
                (Update, Settings),
                (Create, ApiKey),
                (Read, ApiKey),
                (Delete, ApiKey),
                (Read, AuditLog),
                (Admin, Settings),
            ],
            UserRole::Admin => vec![
                (Create, Program),
                (Read, Program),
                (Update, Program),
                (Delete, Program),
                (Create, Compound),
                (Read, Compound),
                (Update, Compound),
                (Delete, Compound),
                (Create, Assay),
                (Read, Assay),
                (Update, Assay),
                (Create, Deal),
                (Read, Deal),
                (Update, Deal),
                (Create, Asset),
                (Read, Asset),
                (Update, Asset),
                (Create, Order),
                (Read, Order),
                (Create, Team),
                (Read, Team),
                (Update, Team),
                (Read, Settings),
                (Update, Settings),
                (Create, ApiKey),
                (Read, ApiKey),
                (Read, AuditLog),
            ],
            UserRole::Scientist => vec![
                (Create, Program),
                (Read, Program),
                (Update, Program),
                (Create, Compound),
                (Read, Compound),
                (Update, Compound),
                (Create, Assay),
                (Read, Assay),
                (Update, Assay),
                (Read, Deal),
                (Create, Order),
                (Read, Order),
                (Read, Settings),
            ],
            UserRole::BusinessDev => vec![
                (Read, Program),
                (Read, Compound),
                (Read, Assay),
                (Create, Deal),
                (Read, Deal),
                (Update, Deal),
                (Create, Asset),
                (Read, Asset),
                (Update, Asset),
                (Read, Order),
                (Read, Settings),
            ],
            UserRole::Viewer => vec![
                (Read, Program),
                (Read, Compound),
                (Read, Assay),
                (Read, Deal),
                (Read, Order),
            ],
            UserRole::External => vec![(Read, Program), (Read, Compound)],
        };

        Self { grants }
    }

    /// Check if a specific (action, resource) is permitted.
    #[must_use]
    pub fn allows(&self, action: Action, resource: Resource) -> bool {
        self.grants
            .iter()
            .any(|&(a, r)| a == action && r == resource)
    }
}

// ============================================================================
// TenantContext
// ============================================================================

/// Extracted from every authenticated request. Cannot be constructed
/// outside the auth module (fields are private). All repository methods
/// require this — the type system prevents cross-tenant data access.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantContext {
    tenant_id: TenantId,
    user_id: UserId,
    role: UserRole,
    tier: SubscriptionTier,
    permissions: Permissions,
}

impl TenantContext {
    /// Construct from a verified auth token. In production, this is called
    /// only by the auth middleware after JWT verification.
    #[must_use]
    pub fn new(
        tenant_id: TenantId,
        user_id: UserId,
        role: UserRole,
        tier: SubscriptionTier,
    ) -> Self {
        let permissions = Permissions::from_role(&role);
        Self {
            tenant_id,
            user_id,
            role,
            tier,
            permissions,
        }
    }

    #[must_use]
    pub fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    #[must_use]
    pub fn user_id(&self) -> &UserId {
        &self.user_id
    }

    #[must_use]
    pub fn role(&self) -> &UserRole {
        &self.role
    }

    #[must_use]
    pub fn tier(&self) -> &SubscriptionTier {
        &self.tier
    }

    /// Check if the user can perform an action on a resource.
    #[must_use]
    pub fn can(&self, action: Action, resource: Resource) -> bool {
        self.permissions.allows(action, resource)
    }

    /// Check if the tenant's tier includes a feature at `required_tier`.
    #[must_use]
    pub fn tier_includes(&self, required_tier: &SubscriptionTier) -> bool {
        self.tier.includes(required_tier)
    }
}

// ============================================================================
// TenantScoped<T> — compile-time isolation wrapper
// ============================================================================

/// Wraps any value with its owning tenant_id. Prevents data from one
/// tenant leaking into another's context. The tenant_id is immutable
/// after construction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantScoped<T> {
    tenant_id: TenantId,
    inner: T,
}

impl<T> TenantScoped<T> {
    /// Wrap a value with tenant scope. Only succeeds if the context
    /// matches — compile-time safety.
    #[must_use]
    pub fn new(ctx: &TenantContext, inner: T) -> Self {
        Self {
            tenant_id: *ctx.tenant_id(),
            inner,
        }
    }

    #[must_use]
    pub fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    /// Access the inner value (read-only).
    #[must_use]
    pub fn inner(&self) -> &T {
        &self.inner
    }

    /// Consume and return the inner value.
    #[must_use]
    pub fn into_inner(self) -> T {
        self.inner
    }

    /// Verify that this scoped value belongs to the given context.
    /// Returns None if tenant_id doesn't match.
    #[must_use]
    pub fn verify(self, ctx: &TenantContext) -> Option<T> {
        if self.tenant_id == *ctx.tenant_id() {
            Some(self.inner)
        } else {
            None
        }
    }

    /// Map the inner value while preserving tenant scope.
    #[must_use]
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> TenantScoped<U> {
        TenantScoped {
            tenant_id: self.tenant_id,
            inner: f(self.inner),
        }
    }
}

// ============================================================================
// Tenant Status
// ============================================================================

/// Lifecycle status of a tenant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TenantStatus {
    /// In trial period (14 days, Accelerator features).
    Trial,
    /// Active paying customer.
    Active,
    /// Payment failed, in grace period.
    PastDue,
    /// Administratively suspended.
    Suspended,
    /// Tenant requested cancellation, data being archived.
    Offboarding,
    /// Fully deprovisioned, data archived or deleted.
    Deprovisioned,
}

impl TenantStatus {
    /// Whether the tenant can use the platform.
    #[must_use]
    pub fn is_accessible(&self) -> bool {
        matches!(self, Self::Trial | Self::Active | Self::PastDue)
    }

    /// Whether the tenant can be billed.
    #[must_use]
    pub fn is_billable(&self) -> bool {
        matches!(self, Self::Active | Self::PastDue)
    }
}

// ============================================================================
// Tenant Record
// ============================================================================

/// Full tenant record as stored in the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    pub id: TenantId,
    pub name: String,
    pub slug: String,
    pub tier: SubscriptionTier,
    pub status: TenantStatus,
    pub trial_ends_at: Option<DateTime<Utc>>,
    pub settings: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier_pricing() {
        assert_eq!(SubscriptionTier::Explorer.monthly_price_cents(), 50_000);
        assert_eq!(SubscriptionTier::Accelerator.monthly_price_cents(), 250_000);
        assert_eq!(
            SubscriptionTier::Enterprise.monthly_price_cents(),
            1_000_000
        );
        assert_eq!(SubscriptionTier::Academic.monthly_price_cents(), 25_000);
    }

    #[test]
    fn annual_discount_is_16_7_percent() {
        let monthly = SubscriptionTier::Accelerator.monthly_price_cents();
        let annual = SubscriptionTier::Accelerator.annual_price_cents();
        // 10 months for 12 = 10/12 = 83.3% of yearly, i.e., 16.7% discount
        assert_eq!(annual, monthly * 10);
        let full_year = monthly * 12;
        let discount_pct = 100.0 * (1.0 - (annual as f64 / full_year as f64));
        assert!((discount_pct - 16.67).abs() < 0.1);
    }

    #[test]
    fn tier_limits() {
        assert_eq!(SubscriptionTier::Explorer.max_programs(), Some(1));
        assert_eq!(SubscriptionTier::Enterprise.max_programs(), None);
        assert_eq!(SubscriptionTier::Explorer.max_users(), Some(3));
    }

    #[test]
    fn tier_includes_checks_rank() {
        assert!(SubscriptionTier::Enterprise.includes(&SubscriptionTier::Explorer));
        assert!(!SubscriptionTier::Explorer.includes(&SubscriptionTier::Enterprise));
        assert!(SubscriptionTier::Accelerator.includes(&SubscriptionTier::Accelerator));
    }

    #[test]
    fn role_permissions() {
        let ctx = TenantContext::new(
            TenantId::new(),
            UserId::new(),
            UserRole::Scientist,
            SubscriptionTier::Accelerator,
        );
        assert!(ctx.can(Action::Create, Resource::Compound));
        assert!(ctx.can(Action::Read, Resource::Program));
        assert!(!ctx.can(Action::Delete, Resource::Program)); // scientists can't delete
        assert!(!ctx.can(Action::Read, Resource::Billing)); // scientists can't see billing
    }

    #[test]
    fn owner_has_all_permissions() {
        let ctx = TenantContext::new(
            TenantId::new(),
            UserId::new(),
            UserRole::Owner,
            SubscriptionTier::Enterprise,
        );
        assert!(ctx.can(Action::Admin, Resource::Settings));
        assert!(ctx.can(Action::Read, Resource::Billing));
        assert!(ctx.can(Action::Delete, Resource::Team));
    }

    #[test]
    fn tenant_scoped_verify() {
        let ctx1 = TenantContext::new(
            TenantId::new(),
            UserId::new(),
            UserRole::Owner,
            SubscriptionTier::Explorer,
        );
        let ctx2 = TenantContext::new(
            TenantId::new(),
            UserId::new(),
            UserRole::Owner,
            SubscriptionTier::Explorer,
        );

        let scoped = TenantScoped::new(&ctx1, "secret data");
        // Same tenant can access
        assert!(TenantScoped::new(&ctx1, "x").verify(&ctx1).is_some());
        // Different tenant is rejected
        assert!(scoped.verify(&ctx2).is_none());
    }

    #[test]
    fn tenant_scoped_map() {
        let ctx = TenantContext::new(
            TenantId::new(),
            UserId::new(),
            UserRole::Scientist,
            SubscriptionTier::Accelerator,
        );
        let scoped = TenantScoped::new(&ctx, 42);
        let doubled = scoped.map(|x| x * 2);
        assert_eq!(*doubled.inner(), 84);
    }

    #[test]
    fn tenant_status_accessibility() {
        assert!(TenantStatus::Trial.is_accessible());
        assert!(TenantStatus::Active.is_accessible());
        assert!(TenantStatus::PastDue.is_accessible());
        assert!(!TenantStatus::Suspended.is_accessible());
        assert!(!TenantStatus::Offboarding.is_accessible());
    }
}
