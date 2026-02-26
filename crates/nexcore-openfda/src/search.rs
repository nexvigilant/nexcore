//! Unified fan-out search across all openFDA endpoints.
//!
//! `fan_out_search` issues concurrent requests to all major endpoints and
//! aggregates the results into a single `FanOutResults` struct.  Each field
//! is an individual `Result` so partial failures don't invalidate the whole
//! response.

use crate::client::{OpenFdaClient, QueryParams};
use crate::endpoints::{
    device::{fetch_device_510k, fetch_device_events, fetch_device_recalls},
    drug::{fetch_drug_events, fetch_drug_labels, fetch_drug_ndc, fetch_drug_recalls},
    food::{fetch_food_events, fetch_food_recalls},
    other::fetch_substances,
};
use crate::types::{
    common::OpenFdaResponse,
    device::{Device510k, DeviceEvent, DeviceRecall},
    drug::{DrugEvent, DrugLabel, DrugNdc, DrugRecall},
    food::{FoodEvent, FoodRecall},
    substance::Substance,
};

/// Results from a fan-out search spanning all major openFDA endpoints.
///
/// Each field contains either the results or the reason the endpoint failed.
/// Partial failures are isolated — a single unavailable endpoint does not
/// prevent results from others from being returned.
#[derive(Debug)]
pub struct FanOutResults {
    /// Drug adverse events (FAERS).
    pub drug_events: nexcore_error::Result<OpenFdaResponse<DrugEvent>>,
    /// Drug structured product labels.
    pub drug_labels: nexcore_error::Result<OpenFdaResponse<DrugLabel>>,
    /// Drug recalls.
    pub drug_recalls: nexcore_error::Result<OpenFdaResponse<DrugRecall>>,
    /// NDC directory entries.
    pub drug_ndc: nexcore_error::Result<OpenFdaResponse<DrugNdc>>,
    /// Medical device adverse events.
    pub device_events: nexcore_error::Result<OpenFdaResponse<DeviceEvent>>,
    /// Device recalls.
    pub device_recalls: nexcore_error::Result<OpenFdaResponse<DeviceRecall>>,
    /// Device 510(k) clearances.
    pub device_510k: nexcore_error::Result<OpenFdaResponse<Device510k>>,
    /// Food recalls.
    pub food_recalls: nexcore_error::Result<OpenFdaResponse<FoodRecall>>,
    /// CAERS food adverse events.
    pub food_events: nexcore_error::Result<OpenFdaResponse<FoodEvent>>,
    /// Substance (UNII) records.
    pub substances: nexcore_error::Result<OpenFdaResponse<Substance>>,
}

impl FanOutResults {
    /// Number of endpoints that returned at least one result.
    ///
    /// An endpoint that succeeded (`Ok`) but returned zero records is NOT counted —
    /// use `successful_calls()` if you need to count reachable-but-empty endpoints.
    #[must_use]
    pub fn successful_endpoints(&self) -> usize {
        let mut count = 0usize;
        if self
            .drug_events
            .as_ref()
            .is_ok_and(|r| !r.results.is_empty())
        {
            count += 1;
        }
        if self
            .drug_labels
            .as_ref()
            .is_ok_and(|r| !r.results.is_empty())
        {
            count += 1;
        }
        if self
            .drug_recalls
            .as_ref()
            .is_ok_and(|r| !r.results.is_empty())
        {
            count += 1;
        }
        if self.drug_ndc.as_ref().is_ok_and(|r| !r.results.is_empty()) {
            count += 1;
        }
        if self
            .device_events
            .as_ref()
            .is_ok_and(|r| !r.results.is_empty())
        {
            count += 1;
        }
        if self
            .device_recalls
            .as_ref()
            .is_ok_and(|r| !r.results.is_empty())
        {
            count += 1;
        }
        if self
            .device_510k
            .as_ref()
            .is_ok_and(|r| !r.results.is_empty())
        {
            count += 1;
        }
        if self
            .food_recalls
            .as_ref()
            .is_ok_and(|r| !r.results.is_empty())
        {
            count += 1;
        }
        if self
            .food_events
            .as_ref()
            .is_ok_and(|r| !r.results.is_empty())
        {
            count += 1;
        }
        if self
            .substances
            .as_ref()
            .is_ok_and(|r| !r.results.is_empty())
        {
            count += 1;
        }
        count
    }

    /// Number of endpoints that completed without error (including those returning zero records).
    #[must_use]
    pub fn successful_calls(&self) -> usize {
        let checks: [bool; 10] = [
            self.drug_events.is_ok(),
            self.drug_labels.is_ok(),
            self.drug_recalls.is_ok(),
            self.drug_ndc.is_ok(),
            self.device_events.is_ok(),
            self.device_recalls.is_ok(),
            self.device_510k.is_ok(),
            self.food_recalls.is_ok(),
            self.food_events.is_ok(),
            self.substances.is_ok(),
        ];
        checks.iter().filter(|&&v| v).count()
    }

    /// Total records returned across all successful endpoints.
    #[must_use]
    pub fn total_results(&self) -> usize {
        let mut total = 0usize;
        if let Ok(ref r) = self.drug_events {
            total += r.results.len();
        }
        if let Ok(ref r) = self.drug_labels {
            total += r.results.len();
        }
        if let Ok(ref r) = self.drug_recalls {
            total += r.results.len();
        }
        if let Ok(ref r) = self.drug_ndc {
            total += r.results.len();
        }
        if let Ok(ref r) = self.device_events {
            total += r.results.len();
        }
        if let Ok(ref r) = self.device_recalls {
            total += r.results.len();
        }
        if let Ok(ref r) = self.device_510k {
            total += r.results.len();
        }
        if let Ok(ref r) = self.food_recalls {
            total += r.results.len();
        }
        if let Ok(ref r) = self.food_events {
            total += r.results.len();
        }
        if let Ok(ref r) = self.substances {
            total += r.results.len();
        }
        total
    }
}

/// Search a term across all major openFDA endpoints concurrently.
///
/// The `term` is passed as-is as the openFDA `search` parameter.  Use
/// endpoint-specific query builders in [`crate::endpoints`] to construct
/// field-qualified search strings.
///
/// # Arguments
///
/// * `client` — Shared `OpenFdaClient`.
/// * `term` — openFDA search expression (e.g., `"aspirin"` or
///   `"patient.drug.openfda.generic_name:\"aspirin\""`).
/// * `limit` — Maximum records per endpoint (capped at 100 for fan-out to
///   keep latency bounded).
///
/// # Returns
///
/// A [`FanOutResults`] with individual `Result` per endpoint.  Never returns
/// an `Err` itself — individual failures are captured inside the struct.
pub async fn fan_out_search(
    client: &OpenFdaClient,
    term: &str,
    limit: Option<u32>,
) -> FanOutResults {
    let n = limit.unwrap_or(20).min(100);

    let drug_p = QueryParams::search(term, n);
    let label_p = QueryParams::search(term, n);
    let drug_recall_p = QueryParams::search(term, n);
    let ndc_p = QueryParams::search(term, n);
    let dev_event_p = QueryParams::search(term, n);
    let dev_recall_p = QueryParams::search(term, n);
    let dev_510k_p = QueryParams::search(term, n);
    let food_recall_p = QueryParams::search(term, n);
    let food_event_p = QueryParams::search(term, n);
    let sub_p = QueryParams::search(term, n);

    let (
        drug_events,
        drug_labels,
        drug_recalls,
        drug_ndc,
        device_events,
        device_recalls,
        device_510k,
        food_recalls,
        food_events,
        substances,
    ) = tokio::join!(
        fetch_drug_events(client, &drug_p),
        fetch_drug_labels(client, &label_p),
        fetch_drug_recalls(client, &drug_recall_p),
        fetch_drug_ndc(client, &ndc_p),
        fetch_device_events(client, &dev_event_p),
        fetch_device_recalls(client, &dev_recall_p),
        fetch_device_510k(client, &dev_510k_p),
        fetch_food_recalls(client, &food_recall_p),
        fetch_food_events(client, &food_event_p),
        fetch_substances(client, &sub_p),
    );

    FanOutResults {
        drug_events,
        drug_labels,
        drug_recalls,
        drug_ndc,
        device_events,
        device_recalls,
        device_510k,
        food_recalls,
        food_events,
        substances,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::common::{OpenFdaMeta, ResultsMeta};

    // Build an Ok response with `n` placeholder DrugEvent results (constructed field-by-field).
    fn ok_drug_response(n: usize) -> nexcore_error::Result<OpenFdaResponse<DrugEvent>> {
        let results: Vec<DrugEvent> = (0..n)
            .map(|i| DrugEvent {
                safetyreportid: format!("{i}"),
                serious: "1".to_string(),
                receiptdate: String::new(),
                reporttype: String::new(),
                seriousnessdeath: None,
                seriousnesshospitalization: None,
                seriousnessdisabling: None,
                seriousnesscongenitalanomali: None,
                seriousnesslifethreatening: None,
                seriousnessother: None,
                primarysource: None,
                patient: None,
            })
            .collect();
        Ok(OpenFdaResponse {
            meta: OpenFdaMeta {
                results: ResultsMeta {
                    total: n as u64,
                    limit: n as u32,
                    skip: 0,
                },
                ..Default::default()
            },
            results,
        })
    }

    fn err_response<T>() -> nexcore_error::Result<OpenFdaResponse<T>> {
        Err(nexcore_error::nexerror!("test error"))
    }

    fn empty_ok<T>() -> nexcore_error::Result<OpenFdaResponse<T>> {
        Ok(OpenFdaResponse {
            meta: OpenFdaMeta::default(),
            results: Vec::new(),
        })
    }

    // Helper: build FanOutResults with all-Err arms (uses DrugEvent for homogeneous fields).
    fn all_err_results() -> FanOutResults {
        FanOutResults {
            drug_events: err_response(),
            drug_labels: err_response(),
            drug_recalls: err_response(),
            drug_ndc: err_response(),
            device_events: err_response(),
            device_recalls: err_response(),
            device_510k: err_response(),
            food_recalls: err_response(),
            food_events: err_response(),
            substances: err_response(),
        }
    }

    #[test]
    fn successful_endpoints_zero_when_all_fail() {
        let results = all_err_results();
        assert_eq!(results.successful_endpoints(), 0);
        assert_eq!(results.successful_calls(), 0);
        assert_eq!(results.total_results(), 0);
    }

    #[test]
    fn successful_calls_counts_ok_including_empty() {
        // drug_events returns Ok with 0 records — should count for successful_calls but NOT successful_endpoints
        let results = FanOutResults {
            drug_events: empty_ok(),
            drug_labels: err_response(),
            drug_recalls: err_response(),
            drug_ndc: err_response(),
            device_events: err_response(),
            device_recalls: err_response(),
            device_510k: err_response(),
            food_recalls: err_response(),
            food_events: err_response(),
            substances: err_response(),
        };
        assert_eq!(
            results.successful_calls(),
            1,
            "empty-Ok endpoint is a successful call"
        );
        assert_eq!(
            results.successful_endpoints(),
            0,
            "empty-Ok endpoint has no results — must not count"
        );
        assert_eq!(results.total_results(), 0);
    }

    #[test]
    fn successful_endpoints_counts_only_non_empty_ok() {
        // Only drug_events has actual results (3 records)
        let results = FanOutResults {
            drug_events: ok_drug_response(3),
            drug_labels: empty_ok(),
            drug_recalls: err_response(),
            drug_ndc: err_response(),
            device_events: err_response(),
            device_recalls: err_response(),
            device_510k: err_response(),
            food_recalls: err_response(),
            food_events: err_response(),
            substances: err_response(),
        };
        assert_eq!(results.successful_endpoints(), 1);
        assert_eq!(
            results.successful_calls(),
            2,
            "drug_events + drug_labels (empty Ok)"
        );
        assert_eq!(results.total_results(), 3);
    }

    #[test]
    fn total_results_sums_across_all_ok_arms() {
        // drug_events has 5 results; drug_labels is Ok but empty (0 results).
        // total_results counts all records across all Ok arms, so expect 5.
        // successful_endpoints counts only non-empty Ok arms, so expect 1.
        let results = FanOutResults {
            drug_events: ok_drug_response(5),
            drug_labels: empty_ok(),
            drug_recalls: err_response(),
            drug_ndc: err_response(),
            device_events: err_response(),
            device_recalls: err_response(),
            device_510k: err_response(),
            food_recalls: err_response(),
            food_events: err_response(),
            substances: err_response(),
        };
        assert_eq!(results.total_results(), 5);
        assert_eq!(results.successful_endpoints(), 1);
        assert_eq!(
            results.successful_calls(),
            2,
            "drug_events (5 results) + drug_labels (empty Ok)"
        );
    }

    #[test]
    fn query_params_search_limit_respected() {
        let p = QueryParams::search("aspirin", 20);
        assert_eq!(p.limit, Some(20));
        assert_eq!(p.search.as_deref(), Some("aspirin"));
    }
}
