# AI Guidance — nexcore-social-types

Shared social media domain types extracted from `nexcore-social`.

## Use When
- Domain-layer crates need `Post` without pulling in the Service-layer API client.
- Adding new social media data types that should be consumable across holds.

## Extraction Rationale
`nexcore-social` (mcp-service/Service) contains HTTP clients, OAuth2, rate limiters.
`nexcore-value-mining` (business-strategy/Domain) only needs the `Post` data struct.
This crate breaks the Domain → Service direction violation (DV7).

## Grounding Patterns
- **Post**: T3 (σ + N + μ + λ + ς + κ), dominant σ (Sequence)
- All types implement `GroundsTo` in `src/grounding.rs`

## Maintenance
- When adding types, ensure `nexcore-social` re-exports them via `pub use nexcore_social_types::*`
- Keep dependencies minimal — no HTTP clients, no async runtime
- Follow `nexcore-hormone-types` extraction pattern
