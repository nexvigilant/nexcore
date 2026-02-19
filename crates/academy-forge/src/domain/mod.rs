//! Domain-specific extraction plugins.
//!
//! Domain plugins extend the generic crate extraction with domain-specific
//! knowledge (axioms, harm types, conservation laws, etc.).

pub mod vigilance;

use std::path::Path;

use crate::error::ForgeError;
use crate::ir::DomainAnalysis;

/// Extract domain-specific analysis for a named domain.
///
/// Currently supported domains:
/// - `"vigilance"` — Theory of Vigilance (ToV)
pub fn extract_domain(
    domain_name: &str,
    _workspace_root: &Path,
) -> Result<DomainAnalysis, ForgeError> {
    match domain_name {
        "vigilance" | "tov" => Ok(vigilance::extract_vigilance_domain()),
        _ => Err(ForgeError::UnknownDomain(domain_name.to_string())),
    }
}
