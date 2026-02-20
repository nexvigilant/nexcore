# vr-core

Foundational domain types for the Vigilant Research (VR) platform. This crate provides the type-safe identifiers, tenant isolation primitives, and monetary types used across the entire platform ecosystem.

## Intent
To establish a "Single Source of Truth" for platform-wide data structures. It enforces tenant isolation at the type level (`TenantScoped<T>`) and provides a unified financial model for all billing and marketplace operations.

## T1 Grounding (Lex Primitiva)
Dominant Primitives:
- **λ (Address)**: The core primitive for type-safe identifiers (`TenantId`, `UserId`).
- **∂ (Boundary)**: Enforces strict isolation via `TenantContext` and `TenantScoped`.
- **N (Quantity)**: Represents financial values through the `Money` type (integer cents).
- **κ (Comparison)**: Used for permission checks and role evaluations.

## Core Types
- **TenantContext**: Essential metadata extracted from authenticated requests to ensure isolation.
- **TenantScoped<T>**: A container that binds a value to a specific `TenantId`.
- **Money**: Type-safe financial calculations in integer cents to avoid floating-point errors.
- **User Roles**: The permission model (Reader, Contributor, Admin, Owner).

## SOPs for Use
### Using Tenant Isolation
```rust
use vr_core::tenant::{TenantContext, TenantScoped};

fn save_data<T>(ctx: &TenantContext, data: T) {
    let scoped = TenantScoped::new(ctx.tenant_id, data);
    // Persist scoped data...
}
```

### Financial Calculations
```rust
use vr_core::money::{Money, Currency};
let price = Money::cents(4900); // $49.00
let discount = Money::cents(500);
let total = price - discount;
```

## License
Proprietary. Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
