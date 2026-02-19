//! NCBI E-utilities integration for sequence search and retrieval.
//!
//! Requires the `ncbi` feature flag. Provides `NcbiClient` for ESearch
//! (database search), EFetch (sequence download), ELink (cross-database
//! traversal), and ESummary (document metadata) against NCBI Entrez.
//!
//! ## Primitives
//!
//! μ(Mapping): query → UIDs → FASTA → BioRecord pipeline
//! ∂(Boundary): feature gate, rate limits, error types
//! →(Causality): search causes fetch causes parse

pub mod client;
pub mod error;
pub mod types;

pub use client::NcbiClient;
pub use error::NcbiError;
pub use types::{
    Database, DocSummary, EFetchParams, ELinkParams, ELinkResult, ESearchParams, ESearchResult,
    ESummaryParams,
};
