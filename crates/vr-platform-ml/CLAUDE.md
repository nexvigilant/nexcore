# AI Guidance — vr-platform-ml

Platform-level machine learning infrastructure.

## Use When
- Aggregating data across multiple tenants for training (must use differential privacy).
- Implementing model training or re-training triggers.
- Routing inference requests to the optimal model based on cost or accuracy.
- Running standardized benchmarks for performance comparison.
- Applying active learning techniques to improve data efficiency.

## Grounding Patterns
- **Privacy Gating (∂)**: Never aggregate data without applying the `add_noise` (differential privacy) transformation first.
- **Metric Standard (κ)**: Favor the MCC (Matthews Correlation Coefficient) over simple Accuracy when evaluating unbalanced safety datasets.
- **T1 Primitives**:
  - `ρ + μ`: Root primitives for learning loops and inference mapping.
  - `∂ + κ`: Root primitives for privacy boundaries and benchmarking comparison.

## Maintenance SOPs
- **Model Promotion**: A model MUST pass the `BenchmarkSuite` with a "SILVER" grade or higher before being promoted to `ServingStatus::Production`.
- **Anonymization Invariant**: Differential privacy parameters (ε) should be chosen to maintain a high privacy budget across epochs.
- **Inference Stability**: Multi-model routing should implement a circuit breaker to fall back to a "safe" baseline model if the primary fails.

## Key Entry Points
- `src/benchmarking.rs`: Evaluation metrics and suites.
- `src/aggregation.rs`: Privacy-preserving data collection.
- `src/training.rs`: HPO (Hyperparameter Optimization) and training lifecycle.
