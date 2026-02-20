# AI Guidance — vr-marketplace

Marketplace coordination and service discovery engine.

## Use When
- Adding new service providers or updating their ratings.
- managing the catalog of available research models or CRO services.
- Implementing order management logic for marketplace transactions.
- Computing revenue shares or performance scores for providers.

## Grounding Patterns
- **State Transition (ς)**: Always use `validate_order_transition` before updating an order status to prevent illegal lifecycle jumps.
- **Fair Scoring (κ)**: Use the `scoring` module to normalize CRO performance metrics; do not implement ad-hoc ranking logic.
- **T1 Primitives**:
  - `μ + ς`: Root primitives for mapping services to stateful orders.
  - `κ + →`: Root primitives for provider comparison and causal settlement.

## Maintenance SOPs
- **Revenue Share**: Model revenue sharing logic in `src/models.rs` is sensitive; any changes must be verified against the `vr-billing` commission logic.
- **Catalog Schema**: Every `CatalogEntry` MUST include a `PricingModel` to be discoverable by the billing engine.
- **Vetting**: New providers start in `ProviderStatus::Pending`. A transition to `Active` requires a manual or automated compliance audit.

## Key Entry Points
- `src/catalog.rs`: Service definitions and pricing estimation.
- `src/ordering.rs`: Order lifecycle and commission triggers.
- `src/providers.rs`: Provider registration and metadata.
