//! Rust code parsing utilities using syn.

pub mod performance;
pub mod purity;
mod rust;

pub use performance::{
    AllocationSite, AsyncIssue, CloneClassification, CloneSite, ComplexityAnnotation,
    MemoryGrowthSite, analyze_complexity, detect_allocations, detect_async_issues, detect_clones,
    detect_iterator_issues, detect_lock_issues, detect_memory_growth, detect_string_issues,
};
pub use rust::{PanicViolation, check_panic_patterns, extract_constructs};
