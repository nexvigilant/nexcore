//! High-performance batch processing for signal detection.
//!
//! This module provides optimized batch operations using:
//! - **Data parallelism** via Rayon for multi-core utilization
//! - **Structure-of-Arrays (SoA)** layout for cache-friendly access
//! - **Vectorizable loops** that LLVM can auto-vectorize
//!
//! # Performance
//!
//! For 100,000+ drug-event pairs, batch processing provides:
//! - 4-8x speedup from parallelism on multi-core CPUs
//! - 2-3x speedup from cache efficiency (SoA layout)
//! - Total: 8-24x faster than sequential single-call processing

pub mod parallel;

pub use parallel::{
    BatchContingencyTables, BatchResult, CompleteSignalResult, batch_chi_square_p_values,
    batch_chi_square_p_values_sequential, batch_complete_parallel,
    batch_ebgm_custom_priors_parallel, batch_ebgm_parallel, batch_ic_parallel, batch_prr_parallel,
    batch_prr_vectorized, batch_ror_parallel, build_contingency_tables,
    build_contingency_tables_parallel,
};
