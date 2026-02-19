//! # Signal Detection Pipeline
//!
//! Unified crate containing all signal detection pipeline stages.
//! Depends on `signal-core` for types and traits.
//!
//! ## Modules
//!
//! | Module | Stage | Purpose |
//! |--------|-------|---------|
//! | `ingest` | 1 | Raw data ingestion (JSON, CSV) |
//! | `normalize` | 2 | Drug/event name standardization |
//! | `validate` | 3 | Quality checks on detection results |
//! | `detect` | 4 | Contingency table builder + detection |
//! | `threshold` | 5 | Evans criteria threshold engine |
//! | `stats` | 6 | PRR, ROR, IC, EBGM algorithms |
//! | `store` | 7 | Persistence (in-memory, JSON file) |
//! | `alert` | 8 | Alert lifecycle management |
//! | `report` | 9 | Report generation (JSON, table) |
//! | `orchestrate` | * | Full pipeline coordinator |

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

pub mod alert;
pub mod api;
pub mod core;
pub mod detect;
pub mod grounding;
pub mod ingest;
pub mod normalize;
pub mod orchestrate;
pub mod relay;
pub mod report;
pub mod spatial_bridge;
pub mod stats;
pub mod store;
pub mod threshold;
pub mod validate;

// Re-export commonly used items
pub use orchestrate::Pipeline;
pub use relay::{core_detection_chain, pv_pipeline_chain};
pub use stats::{SignalMetrics, compute_all};
