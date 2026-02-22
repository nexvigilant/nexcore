# AI Guidance — nexcore-spliceosome

Pre-translation structural expectation generator for NMD pipeline surveillance.

## Use When
- Generating EJC markers before pipeline execution begins.
- Classifying task specifications into categories (Explore/Mutate/Orchestrate/Compute/Verify/Browse/Mixed).
- Providing structural expectations for the UPF surveillance complex in `nexcore-immunity`.

## Biology Analog
The biological spliceosome processes pre-mRNA and deposits EJC markers. This crate analyzes task specs and deposits structural expectations. It is cognitively independent from the pipeline it monitors.

## Grounding Patterns
- **Orthogonality Invariant**: This crate has ZERO dependencies on brain, immunity, cytokine, or ribosome. It must stay Foundation-layer.
- **Rules Engine**: Classification is keyword-based, not ML. Keep it fast and deterministic.
- **T1 Primitives**:
  - `sigma + d`: Ordered phase sequences with boundary constraints.
  - `mu + kappa`: Keyword mapping with comparison scoring.

## Maintenance SOPs
- **Template Updates**: When adding new task categories, update `templates.rs` AND add corresponding tests.
- **Classifier Keywords**: Derived from empirical tool usage data (Phase 0 analysis). Update only with new telemetry evidence.
- **Grounding Thresholds**: `grounding_confidence_threshold` is `f32` (0.0-1.0), NOT bool. Higher = more external validation needed.

## Key Entry Points
- `src/engine.rs`: `Spliceosome::splice()` — the main API.
- `src/classifier.rs`: `TaskClassifier::classify()` — keyword-based categorization.
- `src/templates.rs`: Default EJC marker templates per category.
- `src/types.rs`: `EjcMarker`, `TaskCategory`, `TranscriptExpectation`.
