//! GCloud Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! Google Cloud CLI (Config, Projects, Secrets, Storage, Compute, Run, Logging).

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for gcloud config get
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GcloudConfigGetParams {
    /// Property name (e.g., 'project', 'account', 'compute/region')
    pub property: String,
}

/// Parameters for gcloud config set
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GcloudConfigSetParams {
    /// Property to set (e.g., 'project', 'compute/region')
    pub property: String,
    /// Value to set
    pub value: String,
}

/// Parameters for gcloud project describe
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GcloudProjectParams {
    /// Project ID
    pub project_id: String,
}

/// Parameters with optional project
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GcloudOptionalProjectParams {
    /// Project ID (uses default if not specified)
    #[serde(default)]
    pub project: Option<String>,
}

/// Parameters for secrets access
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GcloudSecretsAccessParams {
    /// Secret name
    pub secret_name: String,
    /// Version to access (default: 'latest')
    #[serde(default = "default_latest")]
    pub version: String,
    /// Project ID
    #[serde(default)]
    pub project: Option<String>,
}

fn default_latest() -> String {
    "latest".to_string()
}

/// Parameters for GCS path operations
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GcloudStoragePathParams {
    /// GCS path (e.g., 'gs://bucket-name/prefix/')
    pub path: String,
}

/// Parameters for storage copy
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GcloudStorageCpParams {
    /// Source path (local or gs://)
    pub source: String,
    /// Destination path (local or gs://)
    pub destination: String,
    /// Copy directories recursively
    #[serde(default)]
    pub recursive: bool,
}

/// Parameters for compute instances list
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GcloudComputeInstancesParams {
    /// Project ID
    #[serde(default)]
    pub project: Option<String>,
    /// Zone filter
    #[serde(default)]
    pub zone: Option<String>,
}

/// Parameters for Cloud Run/Functions with region
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GcloudServiceListParams {
    /// Project ID
    #[serde(default)]
    pub project: Option<String>,
    /// Region filter
    #[serde(default)]
    pub region: Option<String>,
}

/// Parameters for Cloud Run/Functions describe
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GcloudServiceDescribeParams {
    /// Service/function name
    pub name: String,
    /// Region where deployed
    pub region: String,
    /// Project ID
    #[serde(default)]
    pub project: Option<String>,
}

/// Parameters for logging read
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GcloudLoggingReadParams {
    /// Log filter expression
    pub filter: String,
    /// Maximum entries to return (default: 50)
    #[serde(default = "default_log_limit")]
    pub limit: u32,
    /// Project ID
    #[serde(default)]
    pub project: Option<String>,
}

fn default_log_limit() -> u32 {
    50
}

/// Parameters for generic gcloud command
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GcloudRunCommandParams {
    /// Command to run (without 'gcloud' prefix)
    pub command: String,
    /// Timeout in seconds (default: 60)
    #[serde(default = "default_gcloud_timeout")]
    pub timeout: u64,
}

fn default_gcloud_timeout() -> u64 {
    60
}
