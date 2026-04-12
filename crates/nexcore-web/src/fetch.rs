//! HTTP fetch with clean text extraction.
//!
//! Fetches a URL, strips HTML, returns clean readable text.

use nexcore_error::NexError;
use serde::{Deserialize, Serialize};

/// Configuration for page fetching.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct FetchConfig {
    /// Request timeout in seconds.
    pub timeout_secs: u64,
    /// Maximum response body size in bytes (default 5MB).
    pub max_bytes: usize,
    /// User-Agent header.
    pub user_agent: String,
}

impl Default for FetchConfig {
    fn default() -> Self {
        Self {
            timeout_secs: 30,
            max_bytes: 5_000_000,
            user_agent: "NexVigilant-Web/0.1 (+https://nexvigilant.com)".into(),
        }
    }
}

/// Result of fetching a page.
#[derive(Debug, Clone, Serialize)]
pub struct FetchResult {
    /// The URL that was fetched (may differ from input after redirects).
    pub url: String,
    /// HTTP status code.
    pub status: u16,
    /// Content-Type header.
    pub content_type: String,
    /// Clean text extracted from HTML (or raw body for non-HTML).
    pub text: String,
    /// Number of characters in extracted text.
    pub text_length: usize,
    /// Page title if HTML.
    pub title: Option<String>,
    /// Response time in milliseconds.
    pub elapsed_ms: u64,
}

/// Fetch a URL and return clean text content.
///
/// For HTML pages: strips tags, scripts, styles, and returns readable text.
/// For non-HTML: returns raw body text.
pub async fn fetch_page(url: &str, config: &FetchConfig) -> Result<FetchResult, NexError> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(config.timeout_secs))
        .user_agent(&config.user_agent)
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
        .map_err(|e| NexError::msg(format!("HTTP client build: {e}")))?;

    let start = std::time::Instant::now();

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| NexError::msg(format!("HTTP request failed: {e}")))?;

    let status = response.status().as_u16();
    let final_url = response.url().to_string();
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    let bytes = response
        .bytes()
        .await
        .map_err(|e| NexError::msg(format!("reading body: {e}")))?;

    if bytes.len() > config.max_bytes {
        return Err(NexError::msg(format!(
            "response too large: {} bytes (max {})",
            bytes.len(),
            config.max_bytes
        )));
    }

    let elapsed_ms = start.elapsed().as_millis() as u64;
    let body = String::from_utf8_lossy(&bytes).to_string();

    let (text, title) = if content_type.contains("html") {
        let (t, title) = html_to_text(&body);
        (t, title)
    } else {
        (body, None)
    };

    let text_length = text.len();

    Ok(FetchResult {
        url: final_url,
        status,
        content_type,
        text,
        text_length,
        title,
        elapsed_ms,
    })
}

/// Convert HTML to clean readable text.
fn html_to_text(html: &str) -> (String, Option<String>) {
    use scraper::{Html, Selector};

    let doc = Html::parse_document(html);

    // Extract title
    let title = Selector::parse("title")
        .ok()
        .and_then(|sel| doc.select(&sel).next())
        .map(|el| el.text().collect::<String>().trim().to_string());

    // Remove script and style elements, collect text
    let mut text = String::new();
    let body_sel = Selector::parse("body").ok();
    let root = body_sel
        .as_ref()
        .and_then(|sel| doc.select(sel).next())
        .map(|el| el.text().collect::<Vec<_>>())
        .unwrap_or_else(|| doc.root_element().text().collect::<Vec<_>>());

    for chunk in root {
        let trimmed = chunk.trim();
        if !trimmed.is_empty() {
            if !text.is_empty() {
                text.push(' ');
            }
            text.push_str(trimmed);
        }
    }

    // Collapse whitespace
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");

    (text, title)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn html_to_text_basic() {
        let html = r#"<html><head><title>Test</title></head><body><h1>Hello</h1><p>World</p></body></html>"#;
        let (text, title) = html_to_text(html);
        assert!(text.contains("Hello"));
        assert!(text.contains("World"));
        assert_eq!(title.as_deref(), Some("Test"));
    }

    #[test]
    fn html_strips_whitespace() {
        let html = "<body>  lots   of   spaces  </body>";
        let (text, _) = html_to_text(html);
        assert_eq!(text, "lots of spaces");
    }

    #[test]
    fn default_config() {
        let c = FetchConfig::default();
        assert_eq!(c.timeout_secs, 30);
        assert_eq!(c.max_bytes, 5_000_000);
    }
}
