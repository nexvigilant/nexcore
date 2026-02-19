//! Google Apps Script API client.

use anyhow::{Result, anyhow};
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// Google Apps Script API client.
///
/// Requires an OAuth2 access token with `script.projects` scope.
pub struct AppsScriptAPI {
    client: Client,
    script_id: String,
    access_token: String,
}

#[derive(Debug, Serialize)]
struct ScriptRequest {
    function: String,
    parameters: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct ScriptResponse {
    response: Option<ScriptResult>,
    error: Option<ScriptError>,
}

#[derive(Debug, Deserialize)]
struct ScriptResult {
    result: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct ScriptError {
    details: Vec<ErrorDetail>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ErrorDetail {
    error_type: String,
    error_message: String,
}

impl AppsScriptAPI {
    /// Create a new Apps Script API client.
    ///
    /// # Arguments
    ///
    /// * `script_id` - The deployment ID of the Google Apps Script project
    /// * `access_token` - OAuth2 access token with `script.projects` scope
    pub fn new(script_id: &str, access_token: &str) -> Self {
        Self {
            client: Client::new(),
            script_id: script_id.to_string(),
            access_token: access_token.to_string(),
        }
    }

    /// Get the script ID.
    pub fn script_id(&self) -> &str {
        &self.script_id
    }

    /// Execute a function in the Google Apps Script.
    ///
    /// # Arguments
    ///
    /// * `function_name` - Name of the function to execute
    /// * `parameters` - Arguments to pass to the function
    ///
    /// # Returns
    ///
    /// The JSON result from the script execution.
    pub async fn execute_function(
        &self,
        function_name: &str,
        parameters: Vec<serde_json::Value>,
    ) -> Result<serde_json::Value> {
        let url = format!(
            "https://script.googleapis.com/v1/scripts/{}:run",
            self.script_id
        );

        let request = ScriptRequest {
            function: function_name.to_string(),
            parameters,
        };

        let resp = self
            .client
            .post(&url)
            .bearer_auth(&self.access_token)
            .json(&request)
            .send()
            .await?
            .json::<ScriptResponse>()
            .await?;

        if let Some(error) = resp.error {
            let detail = error
                .details
                .first()
                .ok_or_else(|| anyhow!("Unknown script error"))?;
            return Err(anyhow!("{}: {}", detail.error_type, detail.error_message));
        }

        Ok(resp
            .response
            .map(|r| r.result)
            .unwrap_or(serde_json::Value::Null))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_creation() {
        let api = AppsScriptAPI::new("script-123", "token-abc");
        assert_eq!(api.script_id(), "script-123");
    }
}
