# vr-tenant

Tenant lifecycle management for the Vigilant Research (VR) platform. This crate provides the infrastructure for provisioning, tier enforcement, team management, and RBAC (Role-Based Access Control) in a multi-tenant pharmaceutical research environment.

## Intent
To provide a secure and scalable multi-tenancy model. It ensures that tenant data is isolated, feature usage is gated by subscription tier, and team roles are strictly enforced across the platform.

## T1 Grounding (Lex Primitiva)
Dominant Primitives:
- **∂ (Boundary)**: The primary primitive for multi-tenant isolation and security boundaries.
- **ς (State)**: Manages the tenant lifecycle state (Active, Suspended, Trial).
- **μ (Mapping)**: Maps users to teams and roles via RBAC logic.
- **N (Quantity)**: Tracks and enforces usage limits (user counts, storage, program runs).

## Core Modules
- **tiers**: Feature gating and usage limit enforcement (Free, Pro, Enterprise).
- **teams**: User invitation, role management, and team-level access control.
- **provisioning**: Automated tenant creation and resource allocation.
- **lifecycle**: Management of tenant transitions (offboarding, suspension, retention).

## SOPs for Use
### Checking Feature Access
```rust
use vr_tenant::tiers::{Feature, require_feature};
if require_feature(&tenant_config, Feature::SignalAdvanced).is_allowed() {
    // Execute advanced signal logic
}
```

### Validating an Invitation
```rust
use vr_tenant::teams::validate_invitation;
let result = validate_invitation(&current_team, &invite_request)?;
```

## License
Proprietary. Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
