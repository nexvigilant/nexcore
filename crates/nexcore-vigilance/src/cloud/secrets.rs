//! # GCP Secret Manager Client
//!
//! Wrapper for Google Cloud Secret Manager using the official SDK.
//!
//! ## Example
//!
//! ```ignore
//! use nexcore_vigilance::cloud::secrets::SecretClient;
//!
//! let client = SecretClient::new("my-project").await?;
//! let api_key = client.get("api-key-secret").await?;
//! ```

use nexcore_error::{Result, nexerror};
use tokio::process::Command;

/// GCP Secret Manager client.
///
/// Provides a simple interface to access secrets from Google Cloud Secret Manager.
/// Uses `gcloud` CLI under the hood for authentication via ADC.
pub struct SecretClient {
    project_id: String,
}

impl SecretClient {
    /// Create a new Secret Manager client.
    ///
    /// # Arguments
    ///
    /// * `project_id` - The GCP project ID containing the secrets
    #[must_use]
    pub fn new(project_id: &str) -> Self {
        Self {
            project_id: project_id.to_string(),
        }
    }

    /// Get the project ID.
    #[must_use]
    pub fn project_id(&self) -> &str {
        &self.project_id
    }

    /// Access the latest version of a secret by name.
    ///
    /// # Arguments
    ///
    /// * `secret_id` - The secret name/ID
    ///
    /// # Returns
    ///
    /// The secret value as a string
    ///
    /// # Errors
    ///
    /// Returns error if the secret doesn't exist or access is denied.
    pub async fn get(&self, secret_id: &str) -> Result<String> {
        let secret_name = format!(
            "projects/{}/secrets/{}/versions/latest",
            self.project_id, secret_id
        );

        let output = Command::new("gcloud")
            .args([
                "secrets",
                "versions",
                "access",
                "latest",
                "--secret",
                secret_id,
                "--project",
                &self.project_id,
            ])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(nexerror!(
                "Failed to access secret '{}': {}",
                secret_name,
                stderr
            ));
        }

        let secret_value = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;
        Ok(secret_value.trim().to_string())
    }

    /// Check if a secret exists.
    ///
    /// # Arguments
    ///
    /// * `secret_id` - The secret name/ID
    pub async fn exists(&self, secret_id: &str) -> Result<bool> {
        let output = Command::new("gcloud")
            .args([
                "secrets",
                "describe",
                secret_id,
                "--project",
                &self.project_id,
                "--format=json",
            ])
            .output()
            .await?;

        Ok(output.status.success())
    }

    /// List all secrets in the project.
    ///
    /// # Returns
    ///
    /// Vector of secret names
    pub async fn list(&self) -> Result<Vec<String>> {
        let output = Command::new("gcloud")
            .args([
                "secrets",
                "list",
                "--project",
                &self.project_id,
                "--format=value(name)",
            ])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(nexerror!("Failed to list secrets: {}", stderr));
        }

        let stdout = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;
        let secrets: Vec<String> = stdout
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Ok(secrets)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = SecretClient::new("my-project-123");
        assert_eq!(client.project_id(), "my-project-123");
    }
}
