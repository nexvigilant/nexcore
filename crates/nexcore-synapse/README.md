# nexcore-synapse

Mathematical foundation for pattern reinforcement and learning in the NexVigilant Core kernel. It implements the **Amplitude Growth Learning Model**, using quantum-grounded formulas to track the strength and decay of interaction patterns over time.

## Intent
To enable AI agents to "remember" and reinforce successful patterns (e.g., tool sequences, formatting preferences) while allowing unused or low-confidence patterns to naturally decay. It provides a formal basis for Hebbian-style learning in the AI context.

## T1 Grounding (Lex Primitiva)
Dominant Primitives:
- **ν (Frequency)**: Tracks the number of observations for a given pattern.
- **Σ (Sum)**: Accumulates the total weighted learning signal.
- **π (Persistence)**: Manages cross-session retention of learned patterns.
- **∝ (Irreversibility)**: Implements the exponential decay of amplitude over time.
- **∂ (Boundary)**: Gating threshold for pattern consolidation.

## Core Formula
`α(t+1) = α(t)·e^(-λt) + η·confidence·relevance`
- `α`: Amplitude (Strength)
- `λ`: Decay constant (derived from half-life)
- `η`: Learning rate

## SOPs for Use
### Observing a Learning Signal
```rust
use nexcore_synapse::{Synapse, AmplitudeConfig, LearningSignal};

let mut synapse = Synapse::new("edit-pattern", AmplitudeConfig::FAST);
synapse.observe(LearningSignal::new(0.9, 1.0)); // high confidence, high relevance

if synapse.is_consolidated() {
    // Pattern is strong enough to be considered "learned"
}
```

### Managing Multiple Patterns
Use the `SynapseBank` to manage a collection of learning targets (e.g., one per tool or one per user preference).

## Key Components
- **Synapse**: The primary state container for a single learning target.
- **LearningSignal**: A quantified observation with confidence and relevance scores.
- **SaturationKinetics**: Michaelis-Menten bounded growth (prevents infinite accumulation).
- **ConsolidationStatus**: Tracks if a pattern is `Accumulating`, `Consolidated`, or `Decayed`.

## License
Proprietary. Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
