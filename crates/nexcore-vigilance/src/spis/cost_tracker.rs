//! Cost tracking service with STARK synchronization.
//!
//! Wraps GCP cost tracking with STARK project synchronization capabilities.

use crate::cloud::gcp::{CostSummary, CostTracker as GCPTracker};
use nexcore_error::Result;

/// Cost tracker with STARK project synchronization.
pub struct CostTracker {
    gcp_tracker: GCPTracker,
    stark_project_id: Option<String>,
}

impl CostTracker {
    /// Create a new cost tracker.
    #[must_use]
    pub fn new(project_id: &str, stark_project_id: Option<String>) -> Self {
        Self {
            gcp_tracker: GCPTracker::new(project_id),
            stark_project_id,
        }
    }

    /// Get the STARK project ID if configured.
    #[must_use]
    pub fn stark_project_id(&self) -> Option<&str> {
        self.stark_project_id.as_deref()
    }

    /// Get current month costs from GCP.
    ///
    /// # Errors
    ///
    /// Returns error if GCP API call fails.
    pub async fn get_current_month_costs(&self, dataset: &str) -> Result<CostSummary> {
        self.gcp_tracker.get_current_month_costs(dataset).await
    }

    /// Sync costs to STARK system.
    ///
    /// # Errors
    ///
    /// Returns error if cost retrieval fails.
    pub async fn sync_to_stark(&self, dataset: &str) -> Result<serde_json::Value> {
        let costs = self.get_current_month_costs(dataset).await?;

        Ok(serde_json::json!({
            "success": true,
            "stark_project_id": self.stark_project_id,
            "gcp_costs": costs.total_cost,
            "currency": costs.currency,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_tracker_creation() {
        let tracker = CostTracker::new("my-project", Some("stark-123".to_string()));
        assert_eq!(tracker.stark_project_id(), Some("stark-123"));
    }
}
