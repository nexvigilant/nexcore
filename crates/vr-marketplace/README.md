# vr-marketplace

Marketplace engine for the Vigilant Research (VR) platform. This crate manages the lifecycle of third-party service providers (CROs, ML model creators, experts) and the service catalog where tenants discover and purchase specialized research capabilities.

## Intent
To facilitate a high-trust ecosystem for pharmaceutical research services. It provides standardized mechanisms for provider registration, service cataloging, transparent pricing, and performance-based scoring.

## T1 Grounding (Lex Primitiva)
Dominant Primitives:
- **μ (Mapping)**: The primary primitive for mapping providers to services and catalog entries to pricing models.
- **κ (Comparison)**: Used for provider performance scoring and ranking.
- **→ (Causality)**: Manages the sequential order-to-settlement lifecycle.
- **ς (State)**: Represents the mutable state of orders and provider statuses.

## Core Modules
- **providers**: Registration, vetting, and reputation management for service providers.
- **catalog**: Unified service directory with technical specs and pricing.
- **ordering**: Order management state machine with built-in commission logic.
- **models**: Specialized marketplace for machine learning models and revenue sharing.
- **scoring**: Performance metrics and tier classification for CROs.

## SOPs for Use
### Estimating Service Cost
```rust
use vr_marketplace::catalog::{CatalogEntry, PricingModel, estimate_order_cost};
let estimate = estimate_order_cost(&entry, requested_units)?;
```

### Transitioning an Order
```rust
use vr_marketplace::ordering::{OrderStatus, validate_order_transition};
if validate_order_transition(current_status, new_status).is_valid() {
    // Execute state change...
}
```

## License
Proprietary. Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
