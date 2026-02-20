# AI Guidance — nexcore-harm-taxonomy

Formal harm classification taxonomy (ToV §9).

## Use When
- Classifying a system failure or safety violation into formal categories.
- Mapping a harm event to the specific conservation law being violated.
- Determining the hierarchical level at which a specific type of harm is observable.
- Selecting an intervention strategy (e.g., circuit breaker, load shedding) based on harm type.

## Grounding Patterns
- **Combinatorial Basis**: Always refer to Proposition 9.0.1 (multiplicity, temporal, determinism) when justifying a classification.
- **Manifestation Levels**: Use `ManifestationLevel::for_type()` to verify if a detection signal is being monitored at the correct hierarchical level.
- **T1 Primitives**:
  - `κ + Σ`: Root primitives for combinatorial classification.
  - `∂ + →`: Root primitives for boundary manifestation and propagation.

## Maintenance SOPs
- **Extension Safety**: Type I (Goal Misalignment) is an extension. When adding further extensions (Type J, K), you MUST justify them using a similar {G, V, R} or T1-based framework.
- **Law Mapping**: Every new harm type MUST be linked to at least one of the 11 conservation laws from ToV §8.
- **Theta Space**: Remember that Types E and H are θ-space phenomena; do not attempt to map them to physical conservation laws.

## Key Entry Points
- `src/lib.rs`: `HarmTypeId`, `HarmCharacteristics`, and `classify_harm_event()` logic.
- `src/grounding.rs`: T1 grounding for taxonomic types.
