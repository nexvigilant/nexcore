# AI Guidance — vr-billing

Billing, metering, and marketplace commissions engine.

## Use When
- Recording metered platform usage (e.g., number of signal detections).
- Calculating subscription or usage-based charges.
- Generating invoices for tenant billing cycles.
- Computing marketplace commissions for third-party providers.

## Grounding Patterns
- **No Float**: Never use floating point for monetary values. Use `vr_core::Money`.
- **Basis Points (bps)**: Use `percent_bps` for all percentage calculations to prevent rounding errors in high-volume transactions.
- **T1 Primitives**:
  - `N + Σ`: Root primitives for quantity accumulation and aggregation.
  - `μ + ∝`: Root primitives for pricing mapping and proportional splits.

## Maintenance SOPs
- **Auditability**: All metering events MUST be immutable once recorded. Use `UsageAggregation` to track history.
- **Rounding**: Follow the standard "round half to even" banker's rounding for any intermediate fractional currency steps.
- **Currency Isolation**: Ensure that invoices are multi-currency capable by using the `Currency` field in all `Money` objects.

## Key Entry Points
- `src/metering.rs`: Consumption tracking and aggregation.
- `src/pricing.rs`: Rate tables and volume discount logic.
- `src/commission.rs`: Split-payment logic for the marketplace.
