# AI Guidance — nexcore-zeta

Riemann Zeta function and Dirichlet L-function computation with telescope pipeline for RH confidence estimation.

## Use When
- Computing ζ(s) values, locating zeros, or verifying RH to a given height
- Running the telescope pipeline (CMV → prediction → GUE test → fingerprint → anomaly)
- Fitting scaling laws to telescope confidence vs zero count
- Hunting for Hilbert-Pólya operator candidates (Berry-Keating, xp+V, CMV truncation)
- Comparing zero spacing to GUE random matrix statistics
- Batch processing multiple height ranges in parallel

## Layer & Dependencies

**Layer:** Domain (mathematical analysis)

```
nexcore-zeta (Domain)
├── stem-complex      (Foundation: complex arithmetic)
├── stem-number-theory (Foundation: primes, Bernoulli)
├── nexcore-error     (Foundation: error types)
├── nexcore-lex-primitiva (Foundation: T1 primitives)
├── serde / serde_json (serialization)
└── rayon             (data-parallel batch)
```

**Consumers:** nexcore-mcp (13 MCP tools), nexcore-rh-proofs (proof infrastructure)

## Grounding Patterns

| Module | T1 Primitives | What It Computes |
|--------|---------------|------------------|
| `zeta` | → + N | ζ(s) dispatcher across 5 evaluation regions |
| `zeros` | σ + ∂ | Bracket search for sign changes of Z(t) |
| `riemann_siegel` | N + → | Z(t) and θ(t) on the critical line |
| `cmv` | μ + σ | Verblunsky coefficients from spectral measure |
| `pipeline` | → + σ + κ | 5-stage telescope: CMV → predict → GUE → fingerprint → anomaly |
| `prediction` | σ + ν | Zero prediction from Verblunsky model extrapolation |
| `killip_nenciu` | κ + ∂ | KS test of coefficients against GUE Beta distribution |
| `fingerprint` | κ + μ | Spectral signature: decay rate, regularity, fidelity |
| `anomaly` | ∂ + ν | Phase-model baseline + spike detection |
| `scaling` | N + → | Power-law fit: C(N) = 1 - a·N^(-b) via log-log OLS |
| `cayley` | μ + ∂ | Cayley transform H = i(I+U)(I-U)^{-1} for self-adjoint spectrum |
| `operator` | → + κ | Berry-Keating, xp+V, CMV truncation operator fits |
| `lmfdb` | π + N | Parse LMFDB JSON + 30 embedded Odlyzko zeros |
| `batch` | σ + N | Rayon-parallel telescope across height ranges |
| `statistics` | κ + ν | GUE pair correlation and spacing distribution |

## Key Entry Points

- `src/pipeline.rs` — `run_telescope()` is the main orchestrator
- `src/zeros.rs` — `find_zeros_bracket()` and `verify_rh_to_height()`
- `src/zeta.rs` — `zeta(s)` dispatcher
- `src/batch.rs` — `run_telescope_batch()` for parallel processing
- `src/scaling.rs` — `fit_scaling_law()` for confidence extrapolation

## Maintenance SOPs

- **Adding a new evaluation strategy**: Add to `src/zeta.rs` dispatcher, update region table in `lib.rs` rustdoc
- **Adding a new pipeline stage**: Add field to `TelescopeReport`, implement in `run_telescope()`, wire MCP tool in `nexcore-mcp/src/tools/zeta.rs`
- **Adding an MCP tool**: Create param struct in `nexcore-mcp/src/params/zeta.rs`, implement in `nexcore-mcp/src/tools/zeta.rs`, add dispatch arm in `unified.rs`
- **Precision limits**: f64 reliable to t < 10^6. Beyond that requires MPFR (rug) integration (not yet implemented)
- **CMV vs Jacobi**: CMV reconstruction (cmv.rs) is stable. Jacobi (inverse.rs) degrades beyond N~79 zeros. Always prefer CMV.

## MCP Tools (13)

| Tool | Purpose |
|------|---------|
| `zeta_compute` | Evaluate ζ(s) at a complex point |
| `zeta_find_zeros` | Locate zeros in height range via bracket search |
| `zeta_verify_rh` | Verify RH up to height T |
| `zeta_embedded_zeros` | Get embedded Odlyzko zeros (up to 30) |
| `zeta_lmfdb_parse` | Parse LMFDB JSON zero data |
| `zeta_telescope_run` | Run full telescope pipeline on height range |
| `zeta_batch_run` | Batch telescope across multiple ranges |
| `zeta_scaling_fit` | Fit power-law scaling to (N, confidence) data |
| `zeta_scaling_predict` | Predict confidence at given N |
| `zeta_cayley` | Cayley transform anomaly detection |
| `zeta_operator_hunt` | Hunt across all 3 operator candidates |
| `zeta_operator_candidate` | Test a single operator candidate |
| `zeta_gue_compare` | Compare spacings to GUE statistics |

## Quality Metrics

- **198 tests**, 0 failures, zero clippy warnings
- `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]`
- `#![forbid(unsafe_code)]`
- Full rustdoc on all public items
