//! OpenFDA "other" endpoint functions.
//!
//! Covers:
//! - `/other/substance.json` — FDA substance registry (UNII)

use crate::client::{OpenFdaClient, QueryParams};
use crate::types::common::OpenFdaResponse;
use crate::types::substance::Substance;

/// OpenFDA endpoint path for substance records.
pub const SUBSTANCE_ENDPOINT: &str = "/other/substance.json";

// =============================================================================
// Endpoint Function
// =============================================================================

/// Fetch substance records from the FDA Substance Registration System.
///
/// # Errors
///
/// Returns error if the API is unreachable and no cache exists.
pub async fn fetch_substances(
    client: &OpenFdaClient,
    params: &QueryParams,
) -> Result<OpenFdaResponse<Substance>, nexcore_error::NexError> {
    client.fetch::<Substance>(SUBSTANCE_ENDPOINT, params).await
}

// =============================================================================
// Query Builders
// =============================================================================

/// Build a substance search string by substance name.
#[must_use]
pub fn substance_search_by_name(name: &str) -> String {
    format!("substance_name:\"{}\"", name)
}

/// Build a substance search string by UNII code.
#[must_use]
pub fn substance_search_by_unii(unii: &str) -> String {
    format!("unii:\"{}\"", unii)
}

/// Build a substance search string by CAS registry number.
#[must_use]
pub fn substance_search_by_cas(cas: &str) -> String {
    format!("cas:\"{}\"", cas)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn substance_search_name_contains_term() {
        let q = substance_search_by_name("aspirin");
        assert!(q.contains("substance_name"));
        assert!(q.contains("aspirin"));
    }

    #[test]
    fn substance_search_unii() {
        let q = substance_search_by_unii("R16CO5Y76E");
        assert!(q.contains("unii"));
        assert!(q.contains("R16CO5Y76E"));
    }

    #[test]
    fn substance_search_cas() {
        let q = substance_search_by_cas("50-78-2");
        assert!(q.contains("cas"));
        assert!(q.contains("50-78-2"));
    }
}
