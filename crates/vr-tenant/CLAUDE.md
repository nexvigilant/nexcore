# AI Guidance — vr-tenant

Multi-tenant lifecycle and RBAC management.

## Use When
- Provisioning new tenants or managing their lifecycle transitions.
- Enforcing feature gates based on subscription tiers.
- Validating team operations, user invitations, or role changes.
- Implementing usage limits (quota checks) for specific resources.

## Grounding Patterns
- **Isolation (∂)**: Always verify tenant context before performing any data operation. Never leak information between tenant IDs.
- **Role Enforcement (μ)**: Use the `teams` module to validate permissions; do not hardcode role-string checks in business logic.
- **T1 Primitives**:
  - `∂ + ς`: Root primitives for isolated state management.
  - `μ + N`: Root primitives for role mapping and quota enforcement.

## Maintenance SOPs
- **Tier Invariant**: When adding a new platform feature, you MUST add a corresponding variant to the `Feature` enum and define its availability in `src/tiers.rs`.
- **Retention Compliance**: Offboarding must follow the `OffboardingChecklist` to ensure data retention laws (e.g., GDPR, HIPAA) are respected.
- **No Unsafe**: Strictly enforce `#![forbid(unsafe_code)]`.

## Key Entry Points
- `src/tiers.rs`: Feature definitions and limit enforcement logic.
- `src/teams.rs`: RBAC and member management.
- `src/provisioning.rs`: Tenant initialization flows.
