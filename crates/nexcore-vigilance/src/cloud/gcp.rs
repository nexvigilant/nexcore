//! # GCP Cloud Integrations

use nexcore_chrono::DateTime;
use nexcore_error::Result;
use serde::{Deserialize, Serialize};
use tokio::process::Command;

/// GCP billing account information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingAccount {
    pub name: String,
    pub display_name: String,
    pub open: bool,
    pub master_billing_account: Option<String>,
}

/// Billing information for a GCP project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectBillingInfo {
    pub project: String,
    pub billing_account_name: Option<String>,
    pub billing_enabled: bool,
    pub billing_account_id: Option<String>,
}

/// Cost summary for a billing period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostSummary {
    pub period: String,
    pub total_cost: f64,
    pub currency: String,
    pub last_updated: DateTime,
}

/// GCP cost tracking client.
pub struct CostTracker {
    project_id: String,
}

impl CostTracker {
    pub fn new(project_id: &str) -> Self {
        Self {
            project_id: project_id.to_string(),
        }
    }

    pub fn project_id(&self) -> &str {
        &self.project_id
    }

    pub async fn get_billing_accounts(&self) -> Result<Vec<BillingAccount>> {
        let output = Command::new("gcloud")
            .args(["billing", "accounts", "list", "--format=json"])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            nexcore_error::bail!("gcloud billing accounts list failed: {}", stderr);
        }

        let accounts = serde_json::from_slice(&output.stdout)?;
        Ok(accounts)
    }

    /// Get billing information for the tracked project.
    pub async fn get_project_billing_info(&self) -> Result<ProjectBillingInfo> {
        let output = Command::new("gcloud")
            .args([
                "billing",
                "projects",
                "describe",
                &self.project_id,
                "--format=json",
            ])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            nexcore_error::bail!("gcloud billing projects describe failed: {}", stderr);
        }

        let info: serde_json::Value = serde_json::from_slice(&output.stdout)?;
        let billing_account_name = info["billingAccountName"].as_str().map(|s| s.to_string());
        let billing_enabled = info["billingEnabled"].as_bool().unwrap_or(false);
        let billing_account_id = billing_account_name
            .as_ref()
            .and_then(|name| name.split('/').next_back())
            .map(|s| s.to_string());

        Ok(ProjectBillingInfo {
            project: self.project_id.clone(),
            billing_account_name,
            billing_enabled,
            billing_account_id,
        })
    }

    /// Get current month costs from BigQuery billing export.
    pub async fn get_current_month_costs(&self, dataset: &str) -> Result<CostSummary> {
        let query = format!(
            "SELECT SUM(cost) as total_cost, currency FROM `{}.{}.gcp_billing_export_*` \
             WHERE _TABLE_SUFFIX BETWEEN FORMAT_DATE('%Y%m01', CURRENT_DATE()) \
             AND FORMAT_DATE('%Y%m%d', CURRENT_DATE()) GROUP BY currency",
            self.project_id, dataset
        );

        let output = Command::new("bq")
            .args(["query", "--use_legacy_sql=false", "--format=json", &query])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            nexcore_error::bail!("bq query failed: {}", stderr);
        }

        let results: serde_json::Value = serde_json::from_slice(&output.stdout)?;
        let row = results
            .get(0)
            .ok_or_else(|| nexcore_error::nexerror!("No cost data returned"))?;

        Ok(CostSummary {
            period: "current_month".into(),
            total_cost: row["total_cost"]
                .as_str()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0.0),
            currency: row["currency"].as_str().unwrap_or("USD").to_string(),
            last_updated: DateTime::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_tracker_creation() {
        let tracker = CostTracker::new("my-project-123");
        assert_eq!(tracker.project_id(), "my-project-123");
    }

    #[test]
    fn test_billing_account_serialization() {
        let account = BillingAccount {
            name: "billingAccounts/012345".to_string(),
            display_name: "My Billing Account".to_string(),
            open: true,
            master_billing_account: None,
        };
        let json = serde_json::to_string(&account).unwrap_or_default();
        assert!(json.contains("My Billing Account"));
    }
}
