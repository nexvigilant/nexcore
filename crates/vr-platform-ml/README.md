# vr-platform-ml

Platform-level machine learning (ML) infrastructure for the Vigilant Research (VR) environment. This crate provides the tools for cross-tenant data aggregation with differential privacy, model training orchestration, standardized benchmarking, and inference routing.

## Intent
To enable collaborative, privacy-preserving machine learning. It allows the platform to "learn" from anonymized data across tenants while strictly enforcing data isolation and providing objective performance benchmarks for all hosted models.

## T1 Grounding (Lex Primitiva)
Dominant Primitives:
- **ρ (Recursion)**: The primary primitive for the iterative active learning loops and Bayesian optimization.
- **μ (Mapping)**: Maps input data to inference results and model versions to performance metrics.
- **∂ (Boundary)**: Enforces differential privacy limits and tenant isolation boundaries during aggregation.
- **κ (Comparison)**: Used for model benchmarking and ranking (F1, MCC scores).

## Core Modules
- **aggregation**: Collects anonymized training data using differential privacy (ε-δ privacy).
- **training**: Manages the model training lifecycle, versioning, and promotion.
- **serving**: Handles high-performance multi-model inference routing and cost estimation.
- **benchmarking**: Standardized evaluation suite for model performance.
- **active_learning**: Implements uncertainty sampling and Bayesian optimization loops.

## SOPs for Use
### Aggregating Anonymized Data
```rust
use vr_platform_ml::aggregation::{AggregationRequest, add_noise};
let anonymized_data = add_noise(&raw_counts, epsilon, delta)?;
```

### Benchmarking a Model
```rust
use vr_platform_ml::benchmarking::{BenchmarkSuite, EvaluationMetrics};
let report = BenchmarkSuite::run_standard(&model, &test_set)?;
println!("F1 Score: {:.3}", report.f1);
```

## License
Proprietary. Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
