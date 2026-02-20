# nexcore-foundry

The core assembly line architecture for the NexVigilant platform. It implements the **Dual-Pipeline (Builder + Analyst)** model, providing a structured, 14-station workflow for designing, implementing, validating, and reasoning about artifacts.

## Intent
To provide a deterministic, stage-gated process for complex technical deliverables. It separates the act of construction (Builder) from the act of critique and measurement (Analyst), with explicit "Bridge" stations for cross-pipeline coordination.

## T1 Grounding (Lex Primitiva)
Dominant Primitives:
- **σ (Sequence)**: Manages the ordered progression of stations from B1 to BridgeFeedback.
- **→ (Causality)**: Defines the causal dependency between pipeline stages (e.g., B2 depends on B1).
- **κ (Comparison)**: Used in the B3 Finish station to validate deliverables against quality gates.
- **μ (Mapping)**: Maps raw construction signals into high-level analytical patterns.

## The Dual-Pipeline Model
| Pipeline | Stages | Purpose |
| :--- | :--- | :--- |
| **Builder** | B1, B2, B3 | Blueprinting, Framing, and Finishing a deliverable. |
| **Analyst** | A1, A2, A3 | Measuring, Pattern-matching, and Reasoning about a deliverable. |
| **Bridge** | 7 Bridges | Handoff, Codification, Extraction, and Feedback loops. |

## VDAG Full Pipeline Order
1. **B1 (Blueprint)**: Design the spec.
2. **BridgeCodify**: Encode the spec.
3. **B2 (Frame)**: Implement the spec.
4. **BridgeVerify**: Cross-check the implementation.
5. **B3 (Finish)**: Run quality gates.
6. **BridgeExtract (1-3)**: Extract quantitative signals.
7. **A1 (Measure)**: Quantify the deliverable.
8. **BridgeCrystal**: Crystallize measurements.
9. **A2 (Pattern)**: Identify structural patterns.
10. **BridgeInfer**: Carry patterns to reasoning.
11. **A3 (Reason)**: Actionable conclusions.
12. **BridgeFeedback**: Close the loop.

## SOPs for Use
### Executing a Pipeline Run
```rust
use nexcore_foundry::station::PipelineOrder;

let order = PipelineOrder::vdag_full();
// Iterate through order.stages and dispatch to corresponding agents
```

### Adding a new Bridge
1. Define the bridge variant in `src/station.rs`.
2. Implement the bridge transformation logic in `src/bridge.rs`.
3. Update the `vdag_full` sequence if the bridge is systemic.

## License
Proprietary. Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
