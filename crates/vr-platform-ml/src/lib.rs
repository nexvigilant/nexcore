//! # vr-platform-ml — PRPaaS Platform ML Engine
//!
//! Cross-tenant machine learning infrastructure for the Pharmaceutical Research
//! Platform as a Service. Provides anonymized data aggregation with differential
//! privacy, model training orchestration, multi-model inference routing,
//! standardized benchmarking, and active learning optimization.
//!
//! ## Modules
//!
//! - [`aggregation`] — Anonymized data collection with differential privacy
//! - [`training`] — Model training lifecycle and promotion gates
//! - [`serving`] — Multi-model inference routing and cost estimation
//! - [`benchmarking`] — Model evaluation with F1, MCC, and composite ranking
//! - [`active_learning`] — Uncertainty sampling and Bayesian optimization

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

pub mod active_learning;
pub mod aggregation;
pub mod benchmarking;
pub mod serving;
pub mod training;
