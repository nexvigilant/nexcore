//! # Delegation Primitives
//!
//! Cross-model task routing patterns decomposed to T1 universals.
//!
//! ## Tier Classification
//!
//! - **T1 Universal**: Sequence, Mapping, Recursion, State (Rust native)
//! - **T2-Primitive**: ClassificationTree, ConfidenceScoring
//! - **T2-Composite**: PromptTemplate, ValidationPipeline
//! - **T3 Domain**: ModelSelection, ReviewProtocol
//!
//! ## Transfer Confidence
//!
//! | Tier | Confidence | Rationale |
//! |------|------------|-----------|
//! | T1   | 100%       | Universal primitives |
//! | T2-P | 85%        | Cross-domain with tuning |
//! | T2-C | 75%        | Context-dependent composition |
//! | T3   | 40%        | Domain-specific heuristics |

mod classification;
mod confidence;
mod model;
mod review;
mod routing;

pub use classification::{ClassificationBuilder, ClassificationTree, Predicate, PredicateResult};
pub use confidence::{ConfidenceScore, DelegationConfidence, ScoreDimension};
pub use model::{Model, ModelCapability, ModelStrength};
pub use review::{ReviewPhase, ReviewProtocol, ReviewResult};
pub use routing::{DelegationRouter, ErrorCost, RoutingDecision, TaskCharacteristics};
