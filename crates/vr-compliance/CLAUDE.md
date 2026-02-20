# AI Guidance — vr-compliance

Regulatory compliance and audit infrastructure.

## Use When
- Implementing audit logging for new platform features.
- Handling Data Subject Requests (DSRs) or consent management (GDPR).
- Screening data or compounds for export control compliance.
- Tracking evidence or control status for SOC 2 Type II audits.

## Grounding Patterns
- **Immutability (π)**: Audit events MUST be append-only. Never implement logic that deletes or modifies existing audit records.
- **Deadline Enforcement (σ)**: GDPR requests carry a strict 30-day clock; always prioritize these tasks in the `Sequence`.
- **T1 Primitives**:
  - `π + σ`: Root primitives for sequential, immutable logging.
  - `∂ + κ`: Root primitives for boundary screening and control comparison.

## Maintenance SOPs
- **Sensitivity**: Compliance data often contains PII. Ensure that the `audit` module uses proper redaction for sensitive fields.
- **Dual-Use Indicators**: Update the `export_control` chemical lists whenever new CWC (Chemical Weapons Convention) schedules are released.
- **Evidence Verification**: Every `Soc2Control` must link to at least one `EvidenceType` (e.g., Log, Config, Screenshot) to be considered valid.

## Key Entry Points
- `src/audit.rs`: Event definitions and query logic.
- `src/gdpr.rs`: Data subject rights and consent records.
- `src/export_control.rs`: Screening logic for sanctioned countries and compounds.
