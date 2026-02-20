//! OpenFDA food endpoint functions.
//!
//! Wraps the two food endpoints:
//! - `/food/enforcement.json` — food recalls
//! - `/food/event.json` — CAERS adverse event reports

use crate::client::{OpenFdaClient, QueryParams};
use crate::types::common::OpenFdaResponse;
use crate::types::food::{FoodEvent, FoodRecall};

/// OpenFDA endpoint path for food enforcement/recalls.
pub const FOOD_RECALL_ENDPOINT: &str = "/food/enforcement.json";
/// OpenFDA endpoint path for CAERS adverse events.
pub const FOOD_EVENT_ENDPOINT: &str = "/food/event.json";

// =============================================================================
// Endpoint Functions
// =============================================================================

/// Fetch food enforcement/recall records.
///
/// # Errors
///
/// Returns error if the API is unreachable and no cache exists.
pub async fn fetch_food_recalls(
    client: &OpenFdaClient,
    params: &QueryParams,
) -> Result<OpenFdaResponse<FoodRecall>, anyhow::Error> {
    client
        .fetch::<FoodRecall>(FOOD_RECALL_ENDPOINT, params)
        .await
}

/// Fetch CAERS (food/cosmetic adverse event) reports.
///
/// # Errors
///
/// Returns error if the API is unreachable and no cache exists.
pub async fn fetch_food_events(
    client: &OpenFdaClient,
    params: &QueryParams,
) -> Result<OpenFdaResponse<FoodEvent>, anyhow::Error> {
    client
        .fetch::<FoodEvent>(FOOD_EVENT_ENDPOINT, params)
        .await
}

// =============================================================================
// Query Builders
// =============================================================================

/// Build a food recall search string by reason keyword.
#[must_use]
pub fn food_recall_search_by_reason(reason: &str) -> String {
    format!("reason_for_recall:\"{}\"", reason)
}

/// Build a food recall search string by firm name.
#[must_use]
pub fn food_recall_search_by_firm(firm: &str) -> String {
    format!("recalling_firm:\"{}\"", firm)
}

/// Build a CAERS event search string by product brand name.
#[must_use]
pub fn food_event_search_by_product(product_name: &str) -> String {
    format!("products.name_brand:\"{}\"", product_name)
}

/// Build a CAERS event search string filtering to events with a specific outcome.
///
/// Common outcomes: "Hospitalization", "ER visit", "Death", "Disability".
#[must_use]
pub fn food_event_search_by_outcome(outcome: &str) -> String {
    format!("outcomes.outcome:\"{}\"", outcome)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn food_recall_search_reason() {
        let q = food_recall_search_by_reason("allergen");
        assert!(q.contains("reason_for_recall"));
        assert!(q.contains("allergen"));
    }

    #[test]
    fn food_event_search_product() {
        let q = food_event_search_by_product("energy drink");
        assert!(q.contains("name_brand"));
        assert!(q.contains("energy drink"));
    }
}
