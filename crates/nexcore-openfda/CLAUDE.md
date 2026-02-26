# AI Guidance — nexcore-openfda

OpenFDA REST API client and search bridge.

## Use When
- Querying real-time drug labels or adverse event counts from the FDA.
- Searching for medical device 510k clearances or PMA status.
- Retrieving recall information (enforcement) for food or drugs.
- Performing concurrent searches across multiple regulatory domains.

## Grounding Patterns
- **Address Selection (λ)**: Ensure the correct endpoint is selected based on the domain (e.g., `/drug/event.json` vs `/device/event.json`).
- **Mapping (μ)**: Always use the typed structs in `src/types/` rather than raw `serde_json::Value` to maintain grounding and type safety.
- **T1 Primitives**:
  - `μ + λ`: Root primitives for API mapping and addressing.
  - `→ + ∅`: Root primitives for async causality and null-result handling.

## Maintenance SOPs
- **API Rate Limits**: Respect the OpenFDA public rate limits (typically 240 requests per minute). Use the `MAX_LIMIT` (1_000) for large page sizes.
- **Error Handling**: Map all HTTP errors to `OpenFdaError` to ensure the vigilance kernel can differentiate between networking issues and "no results."
- **Schema Updates**: The FDA periodically updates JSON keys; when adding support for a new field, ensure it is added to the `OpenFdaEnrichment` struct.

## Key Entry Points
- `src/client.rs`: `OpenFdaClient` and `QueryParams` implementation.
- `src/endpoints/`: Domain-specific API call wrappers.
- `src/search.rs`: Concurrent fan-out search logic.
