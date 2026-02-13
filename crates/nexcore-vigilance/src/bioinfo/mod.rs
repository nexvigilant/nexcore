//! # Bioinformatics Domain
//!
//! Bioinformatics domain types and algorithms for KEGG pathway analysis,
//! convergence detection, and entity resolution.
//!
//! ## Features
//!
//! - **Entity Types**: Gene, drug, compound, and ID source enums
//! - **Convergence Analysis**: Jaccard similarity and multi-entity hotspot detection
//! - **P-TEFb Constants**: Pre-defined gene data for P-TEFb complex research

pub mod convergence;
pub mod error;
pub mod ptefb;
pub mod types;

// Re-export commonly used items
pub use convergence::{
    ConvergenceAnalysis, ConvergenceLevel, ConvergenceResult, ConvergenceScore, EntityInfo,
    MultiEntityConvergence, PathwayHotspot, SharedPathway, compute_convergence_result,
    compute_jaccard, compute_multi_entity_convergence, interpret_convergence,
};
pub use error::{BioinfoError, BioinfoResult};
pub use ptefb::{
    AF9, AFF4, BRD4, CCNT1, CCNT2, CDK9, ELL2, ENL, HEXIM1, HEXIM2, LARP7, MEPCE, PtefbGene,
    PtefbGeneData, PtefbRole, all_ptefb_genes, all_ptefb_kegg_ids, genes_by_role, get_ptefb_gene,
};
pub use types::{
    CompoundEntry, DiseaseEntry, DrugEntry, EntityType, GeneEntry, IdSource, PathwayEntry,
    ResponseFormat,
};
