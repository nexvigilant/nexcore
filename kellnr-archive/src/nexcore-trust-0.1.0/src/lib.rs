#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! # NexVigilant Core — trust
//!
//! Bayesian trust algorithm with asymmetric evidence weighting and temporal decay.
//!
//! ## Mathematical Model
//!
//! Trust is modeled as a Beta distribution `Beta(alpha, beta)`:
//!
//! - **Score** = `alpha / (alpha + beta)` (expected value)
//! - **Positive evidence**: `alpha += weight`
//! - **Negative evidence**: `beta += weight * asymmetry_factor`
//! - **Temporal decay**: parameters decay toward prior via `exp(-lambda * dt)`
//! - **Uncertainty**: `Var = alpha * beta / ((alpha + beta)^2 * (alpha + beta + 1))`
//!
//! ## Modules
//!
//! - [`engine`]: Core single-dimension Bayesian trust engine
//! - [`confidence`]: Beta CDF, credible intervals, exceedance probability
//! - [`dimension`]: Multi-dimensional trust (Ability/Benevolence/Integrity)
//! - [`volatility`]: Trust velocity and change detection
//! - [`history`]: Audit trail ring buffer
//! - [`network`]: Source-attributed trust aggregation and transitivity
//! - [`safety`]: Patient safety integration (ICH E2A, WHO-UMC, Naranjo)
//! - [`policy`]: Trust-based decision engine (Allow/Deny/Escalate)

pub mod confidence;
pub mod dimension;
pub mod engine;
pub mod evidence;
pub mod grounding;
pub mod history;
pub mod level;
pub mod network;
pub mod policy;
pub mod safety;
pub mod spatial_bridge;
pub mod volatility;

pub use confidence::{CredibleInterval, beta_cdf, beta_quantile, credible_interval, prob_exceeds};
pub use dimension::{DimensionWeights, MultiTrustEngine, MultiTrustSnapshot, TrustDimension};
pub use engine::{TrustConfig, TrustEngine, TrustSnapshot};
pub use evidence::Evidence;
pub use history::{AuditEntry, TrustHistory};
pub use level::{LevelThresholds, TrustLevel};
pub use network::{SourcedTrust, TrustSource, chain_trust, transitive_trust};
pub use policy::{
    DecisionFactor, DecisionRationale, PolicyConfig, TrustDecision, decide, decide_simple,
};
pub use safety::{
    CausalityAssessment, HarmSeverity, harm_adjusted_weight, naranjo_to_causality,
    record_harm_evidence, safety_config, safety_engine, serious_harm_floor, who_umc_to_causality,
};
pub use volatility::{TrustDirection, TrustVelocity};
