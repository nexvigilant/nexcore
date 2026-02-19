//! Google Slides API HTTP client for Google Vids editing.
//!
//! Wraps `reqwest` with automatic token refresh on 401 and
//! targets the Slides API `presentations` endpoints.
//!
//! Tier: T2-C (μ Mapping + σ Sequence + ∂ Boundary)

use serde_json::json;
use tracing::{debug, warn};

use crate::auth::AuthManager;
use crate::types::{BatchUpdateResponse, Page, Presentation};

/// Base URL for Google Slides API v1.
const SLIDES_BASE: &str = "https://slides.googleapis.com/v1/presentations";

/// Errors from the Vids/Slides API client.
#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("auth error: {0}")]
    Auth(#[from] crate::auth::AuthError),
    #[error("HTTP request failed: {0}")]
    Http(String),
    #[error("failed to parse response: {0}")]
    Parse(String),
    #[error("Slides API error {status}: {body}")]
    Api { status: u16, body: String },
}

/// Google Slides/Vids API client with automatic token management.
#[derive(Clone)]
pub struct VidsClient {
    auth: AuthManager,
    http: reqwest::Client,
    /// Quota project for `x-goog-user-project` header (required for ADC).
    quota_project: Option<String>,
}

impl VidsClient {
    /// Create a new client, loading credentials.
    pub async fn new() -> Result<Self, ClientError> {
        let auth = AuthManager::new().await?;
        let quota_project = auth.quota_project().map(String::from);
        Ok(Self {
            auth,
            http: reqwest::Client::new(),
            quota_project,
        })
    }

    /// Get full presentation metadata including all slides and their elements.
    pub async fn get_presentation(
        &self,
        presentation_id: &str,
    ) -> Result<Presentation, ClientError> {
        let url = format!("{SLIDES_BASE}/{presentation_id}");
        self.get_json(&url).await
    }

    /// Get a specific page/scene with all its elements.
    pub async fn get_page(
        &self,
        presentation_id: &str,
        page_id: &str,
    ) -> Result<Page, ClientError> {
        let url = format!("{SLIDES_BASE}/{presentation_id}/pages/{page_id}");
        self.get_json(&url).await
    }

    /// Execute a batchUpdate with a list of requests.
    pub async fn batch_update(
        &self,
        presentation_id: &str,
        requests: Vec<serde_json::Value>,
    ) -> Result<BatchUpdateResponse, ClientError> {
        let url = format!("{SLIDES_BASE}/{presentation_id}:batchUpdate");
        let body = json!({ "requests": requests });
        self.post_json(&url, &body).await
    }

    // -----------------------------------------------------------------------
    // High-level convenience methods
    // -----------------------------------------------------------------------

    /// Delete all text from a shape and insert new text.
    /// This is the primary text-setting operation.
    pub async fn set_text(
        &self,
        presentation_id: &str,
        object_id: &str,
        text: &str,
    ) -> Result<BatchUpdateResponse, ClientError> {
        let requests = vec![
            // First: delete all existing text
            json!({
                "deleteText": {
                    "objectId": object_id,
                    "textRange": {
                        "type": "ALL"
                    }
                }
            }),
            // Then: insert new text
            json!({
                "insertText": {
                    "objectId": object_id,
                    "insertionIndex": 0,
                    "text": text
                }
            }),
        ];
        self.batch_update(presentation_id, requests).await
    }

    /// Find and replace text across the entire presentation.
    pub async fn replace_all_text(
        &self,
        presentation_id: &str,
        find: &str,
        replace_with: &str,
        match_case: bool,
    ) -> Result<BatchUpdateResponse, ClientError> {
        let requests = vec![json!({
            "replaceAllText": {
                "containsText": {
                    "text": find,
                    "matchCase": match_case
                },
                "replaceText": replace_with
            }
        })];
        self.batch_update(presentation_id, requests).await
    }

    /// Create a new blank slide at the given index.
    pub async fn create_slide(
        &self,
        presentation_id: &str,
        insertion_index: Option<u32>,
    ) -> Result<BatchUpdateResponse, ClientError> {
        let mut req = json!({
            "createSlide": {}
        });
        if let Some(idx) = insertion_index {
            req["createSlide"]["insertionIndex"] = json!(idx);
        }
        self.batch_update(presentation_id, vec![req]).await
    }

    /// Delete an object (page, shape, etc.) by ID.
    pub async fn delete_object(
        &self,
        presentation_id: &str,
        object_id: &str,
    ) -> Result<BatchUpdateResponse, ClientError> {
        let requests = vec![json!({
            "deleteObject": {
                "objectId": object_id
            }
        })];
        self.batch_update(presentation_id, requests).await
    }

    /// Create a text box on a specific page with given text.
    pub async fn create_text_box(
        &self,
        presentation_id: &str,
        page_id: &str,
        text: &str,
        x_emu: i64,
        y_emu: i64,
        width_emu: i64,
        height_emu: i64,
    ) -> Result<BatchUpdateResponse, ClientError> {
        // Generate a unique object ID
        let new_id = format!("textbox_{}", chrono::Utc::now().timestamp_millis());

        let requests = vec![
            // Create the shape
            json!({
                "createShape": {
                    "objectId": new_id,
                    "shapeType": "TEXT_BOX",
                    "elementProperties": {
                        "pageObjectId": page_id,
                        "size": {
                            "width": { "magnitude": width_emu, "unit": "EMU" },
                            "height": { "magnitude": height_emu, "unit": "EMU" }
                        },
                        "transform": {
                            "scaleX": 1.0,
                            "scaleY": 1.0,
                            "translateX": x_emu,
                            "translateY": y_emu,
                            "unit": "EMU"
                        }
                    }
                }
            }),
            // Insert text into the new shape
            json!({
                "insertText": {
                    "objectId": new_id,
                    "insertionIndex": 0,
                    "text": text
                }
            }),
        ];
        self.batch_update(presentation_id, requests).await
    }

    // -----------------------------------------------------------------------
    // Internal HTTP helpers with auto-retry on 401
    // -----------------------------------------------------------------------

    /// Attach the `x-goog-user-project` quota header if a quota project is configured.
    fn apply_quota_header(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        match &self.quota_project {
            Some(project) => builder.header("x-goog-user-project", project),
            None => builder,
        }
    }

    async fn get_json<T: serde::de::DeserializeOwned>(&self, url: &str) -> Result<T, ClientError> {
        let token = self.auth.get_token().await?;
        let req = self.http.get(url).bearer_auth(&token);
        let resp = self
            .apply_quota_header(req)
            .send()
            .await
            .map_err(|e| ClientError::Http(e.to_string()))?;

        if resp.status().as_u16() == 401 {
            warn!("Got 401, refreshing token and retrying");
            let token = self.auth.get_token().await?;
            let req = self.http.get(url).bearer_auth(&token);
            let resp = self
                .apply_quota_header(req)
                .send()
                .await
                .map_err(|e| ClientError::Http(e.to_string()))?;
            return self.parse_response(resp).await;
        }

        self.parse_response(resp).await
    }

    async fn post_json<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
        body: &serde_json::Value,
    ) -> Result<T, ClientError> {
        let token = self.auth.get_token().await?;
        let req = self.http.post(url).bearer_auth(&token).json(body);
        let resp = self
            .apply_quota_header(req)
            .send()
            .await
            .map_err(|e| ClientError::Http(e.to_string()))?;

        if resp.status().as_u16() == 401 {
            warn!("Got 401 on POST, refreshing token and retrying");
            let token = self.auth.get_token().await?;
            let req = self.http.post(url).bearer_auth(&token).json(body);
            let resp = self
                .apply_quota_header(req)
                .send()
                .await
                .map_err(|e| ClientError::Http(e.to_string()))?;
            return self.parse_response(resp).await;
        }

        self.parse_response(resp).await
    }

    async fn parse_response<T: serde::de::DeserializeOwned>(
        &self,
        resp: reqwest::Response,
    ) -> Result<T, ClientError> {
        let status = resp.status().as_u16();
        if !resp.status().is_success() {
            let body = resp
                .text()
                .await
                .unwrap_or_else(|_| "<failed to read body>".into());
            return Err(ClientError::Api { status, body });
        }

        let text = resp
            .text()
            .await
            .map_err(|e| ClientError::Parse(e.to_string()))?;

        debug!(response_len = text.len(), "Parsing Slides API response");

        serde_json::from_str(&text)
            .map_err(|e| ClientError::Parse(format!("{e}: {}", &text[..text.len().min(500)])))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slides_base_url_format() {
        let id = "abc123";
        let url = format!("{SLIDES_BASE}/{id}");
        assert_eq!(url, "https://slides.googleapis.com/v1/presentations/abc123");
    }

    #[test]
    fn batch_update_url_format() {
        let id = "abc123";
        let url = format!("{SLIDES_BASE}/{id}:batchUpdate");
        assert!(url.ends_with(":batchUpdate"));
    }

    #[test]
    fn page_url_format() {
        let id = "abc123";
        let page_id = "p1";
        let url = format!("{SLIDES_BASE}/{id}/pages/{page_id}");
        assert!(url.contains("/pages/p1"));
    }

    #[test]
    fn set_text_request_shape() {
        let requests = vec![
            json!({
                "deleteText": {
                    "objectId": "shape1",
                    "textRange": { "type": "ALL" }
                }
            }),
            json!({
                "insertText": {
                    "objectId": "shape1",
                    "insertionIndex": 0,
                    "text": "Hello World With Spaces"
                }
            }),
        ];
        assert_eq!(requests.len(), 2);
        assert!(
            requests[1]["insertText"]["text"]
                .as_str()
                .is_some_and(|t| t.contains(' '))
        );
    }
}
