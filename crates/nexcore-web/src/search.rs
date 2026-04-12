//! Web search via DuckDuckGo HTML (no API key required).
//!
//! Parses DuckDuckGo's HTML search results page to extract
//! titles, URLs, and snippets. Zero external API dependencies.

use nexcore_error::NexError;
use serde::{Deserialize, Serialize};

/// Search configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SearchConfig {
    /// Maximum results to return.
    pub max_results: usize,
    /// Request timeout in seconds.
    pub timeout_secs: u64,
    /// Region code (e.g. "us-en").
    pub region: String,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            max_results: 10,
            timeout_secs: 15,
            region: "us-en".into(),
        }
    }
}

/// A single search result.
#[derive(Debug, Clone, Serialize)]
pub struct SearchResult {
    /// Result title.
    pub title: String,
    /// Result URL.
    pub url: String,
    /// Snippet / description.
    pub snippet: String,
}

/// Search the web using DuckDuckGo HTML.
///
/// No API key required — parses the HTML results page directly.
pub async fn web_search(query: &str, config: &SearchConfig) -> Result<Vec<SearchResult>, NexError> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(config.timeout_secs))
        .user_agent("NexVigilant-Web/0.1 (+https://nexvigilant.com)")
        .build()
        .map_err(|e| NexError::msg(format!("HTTP client: {e}")))?;

    let search_url = format!(
        "https://html.duckduckgo.com/html/?q={}&kl={}",
        urlencoded(query),
        config.region
    );

    let response = client
        .get(&search_url)
        .send()
        .await
        .map_err(|e| NexError::msg(format!("search request failed: {e}")))?;

    if !response.status().is_success() {
        return Err(NexError::msg(format!(
            "search returned status {}",
            response.status()
        )));
    }

    let body = response
        .text()
        .await
        .map_err(|e| NexError::msg(format!("reading search body: {e}")))?;

    let results = parse_ddg_results(&body, config.max_results);
    Ok(results)
}

/// URL-encode a query string.
fn urlencoded(s: &str) -> String {
    url::form_urlencoded::Serializer::new(String::new())
        .append_pair("", s)
        .finish()
        .trim_start_matches('=')
        .to_string()
}

/// Parse DuckDuckGo HTML results page.
fn parse_ddg_results(html: &str, max: usize) -> Vec<SearchResult> {
    use scraper::{Html, Selector};

    let doc = Html::parse_document(html);

    // DDG result blocks use class "result" or "results_links"
    let result_sel = Selector::parse(".result__body, .result").ok();
    let title_sel = Selector::parse(".result__a, .result__title a").ok();
    let snippet_sel = Selector::parse(".result__snippet").ok();

    let (Some(r_sel), Some(t_sel), Some(s_sel)) = (result_sel, title_sel, snippet_sel) else {
        return vec![];
    };

    let mut results = Vec::new();

    for element in doc.select(&r_sel) {
        if results.len() >= max {
            break;
        }

        let title_el = element.select(&t_sel).next();
        let snippet_el = element.select(&s_sel).next();

        let title = title_el
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        let url = title_el
            .and_then(|el| el.value().attr("href"))
            .map(|href| {
                // DDG wraps URLs in redirect — extract actual URL
                if href.contains("uddg=") {
                    href.split("uddg=")
                        .nth(1)
                        .and_then(|u| urlencoding_decode(u))
                        .unwrap_or_else(|| href.to_string())
                } else {
                    href.to_string()
                }
            })
            .unwrap_or_default();

        let snippet = snippet_el
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        if !title.is_empty() && !url.is_empty() {
            results.push(SearchResult {
                title,
                url,
                snippet,
            });
        }
    }

    results
}

/// Decode a percent-encoded URL component.
fn urlencoding_decode(s: &str) -> Option<String> {
    url::form_urlencoded::parse(s.as_bytes())
        .next()
        .map(|(k, v)| {
            if v.is_empty() {
                k.to_string()
            } else {
                format!("{k}={v}")
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urlencoded_spaces() {
        let result = urlencoded("hello world");
        assert!(result.contains("hello"));
        assert!(result.contains("world"));
        assert!(!result.contains(' '));
    }

    #[test]
    fn parse_empty_html() {
        let results = parse_ddg_results("<html><body></body></html>", 10);
        assert!(results.is_empty());
    }

    #[test]
    fn default_config() {
        let c = SearchConfig::default();
        assert_eq!(c.max_results, 10);
        assert_eq!(c.timeout_secs, 15);
    }
}
