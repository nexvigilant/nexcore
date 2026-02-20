# AI Guidance — vr-core

Foundational types for the Vigilant Research (VR) platform.

## Use When
- Defining new platform entities that require a `TenantId`.
- Implementing financial transactions or pricing logic.
- Verifying user permissions or tenant status.
- Creating new type-safe identifiers for domain objects.

## Grounding Patterns
- **Compile-Time Isolation**: Always prefer using `TenantContext` in function signatures to prove that the operation is tenant-aware.
- **Identity (λ)**: Use the `id_struct!` macro (from `nexcore-id`) to define new IDs.
- **T1 Primitives**:
  - `λ + ∂`: Root primitives for identity and boundary isolation.
  - `N + κ`: Root primitives for financial quantity and permission comparison.

## Maintenance SOPs
- **Integer Cents**: Never use `f64` for money. Always use `Money` to ensure arithmetic consistency.
- **Tier Invariant**: When adding a new `SubscriptionTier`, ensure it is reflected in the `vr-tenant` crate's feature gating.
- **Idempotency**: Identifiers should be generated using `nexcore_id` (V4 UUIDs) to ensure collision-free addressing.

## Key Entry Points
- `src/tenant.rs`: `TenantContext` and `UserRole` definitions.
- `src/money.rs`: Financial types and currency logic.
- `src/ids.rs`: Platform-wide type-safe ID declarations.
