# vr-compliance

Regulatory compliance engine for the Vigilant Research (VR) platform. This crate provides the essential infrastructure for maintaining an immutable audit trail, managing GDPR requests, enforcing export controls, and tracking SOC 2 compliance across the platform.

## Intent
To ensure the platform meets the highest standards of regulatory and security compliance. It automates the collection of evidence for audits and provides built-in mechanisms for protecting data privacy and preventing illegal data exports.

## T1 Grounding (Lex Primitiva)
Dominant Primitives:
- **π (Persistence)**: The core primitive for the immutable audit trail and consent records.
- **∂ (Boundary)**: Enforces export control boundaries and data privacy limits (GDPR).
- **κ (Comparison)**: Used to evaluate current platform state against SOC 2 controls.
- **σ (Sequence)**: Manages the temporal sequence of audit events and the GDPR response timeline.

## Core Modules
- **audit**: Capture and query the immutable record of all significant platform actions.
- **gdpr**: Manage Data Subject Requests (DSRs) and track user consent.
- **export_control**: Screen data exports against sanction lists and dual-use chemical indicators.
- **soc2**: Dashboard and scorecard for SOC 2 Type II control compliance.

## SOPs for Use
### Recording an Audit Event
```rust
use vr_compliance::audit::{AuditEvent, AuditEventType};
let event = AuditEvent::new(actor_id, AuditEventType::TenantCreated, details);
// Persist to immutable storage...
```

### Screening an Export
```rust
use vr_compliance::export_control::ExportRisk;
let result = export_control::screen_payload(&compound_data, target_country);
if result.risk == ExportRisk::High {
    // Block export
}
```

## License
Proprietary. Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
