# vr-billing

Billing and monetization engine for the Vigilant Research (VR) platform. This crate manages the end-to-end financial lifecycle, including usage metering, pricing evaluation, invoice generation, and marketplace commission processing.

## Intent
To provide a precise and transparent billing system. It ensures that every platform interaction is metered accurately and that financial settlements for marketplace participants (CROs, experts, AI model providers) are handled with high precision.

## T1 Grounding (Lex Primitiva)
Dominant Primitives:
- **N (Quantity)**: The core primitive for all currency and usage counts.
- **μ (Mapping)**: Maps usage events (`MeterEvent`) to financial charges via pricing rates.
- **Σ (Sum)**: Aggregates metered events over billing periods.
- **∝ (Proportion)**: Calculates commissions and volume-based discounts using basis points.

## Core Modules
- **metering**: Ingests and aggregates platform usage events (e.g., CPU time, API calls).
- **pricing**: Implements tiered pricing and volume discount logic.
- **invoicing**: Generates structured, line-item invoices for tenants.
- **commission**: Handles complex split-payment logic for the VR Marketplace.

## SOPs for Use
### Recording Usage
```rust
use vr_billing::metering::{MeterEvent, MeterType};
let event = MeterEvent::new(tenant_id, MeterType::ApiCall, 1);
// Emit to metering queue...
```

### Generating an Invoice
```rust
use vr_billing::invoicing::generate_invoice;
let invoice = generate_invoice(tenant_id, &aggregated_usage, &rates)?;
```

## License
Proprietary. Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
