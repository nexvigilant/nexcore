//! CSS selector-based content extraction.
//!
//! Extract specific elements from HTML using CSS selectors.

use nexcore_error::NexError;
use serde::Serialize;

/// Result of CSS extraction.
#[derive(Debug, Clone, Serialize)]
pub struct ExtractResult {
    /// CSS selector used.
    pub selector: String,
    /// Matched elements as text.
    pub matches: Vec<String>,
    /// Number of matches.
    pub count: usize,
}

/// Extract text content from HTML using a CSS selector.
pub fn extract_css(html: &str, selector: &str) -> Result<ExtractResult, NexError> {
    use scraper::{Html, Selector};

    let doc = Html::parse_document(html);
    let sel = Selector::parse(selector)
        .map_err(|e| NexError::msg(format!("invalid CSS selector '{selector}': {e:?}")))?;

    let matches: Vec<String> = doc
        .select(&sel)
        .map(|el| el.text().collect::<String>().trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let count = matches.len();
    Ok(ExtractResult {
        selector: selector.to_string(),
        matches,
        count,
    })
}

/// Extract clean text from HTML body, stripping all tags.
pub fn extract_text(html: &str) -> String {
    use scraper::{Html, Selector};

    let doc = Html::parse_document(html);
    let body = Selector::parse("body").ok();
    let chunks: Vec<&str> = body
        .as_ref()
        .and_then(|sel| doc.select(sel).next())
        .map(|el| el.text().collect::<Vec<_>>())
        .unwrap_or_else(|| doc.root_element().text().collect::<Vec<_>>());

    chunks
        .iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Extract all links (href attributes) from HTML.
pub fn extract_links(html: &str, base_url: &str) -> Vec<Link> {
    use scraper::{Html, Selector};

    let doc = Html::parse_document(html);
    let sel = match Selector::parse("a[href]") {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    let base = url::Url::parse(base_url).ok();

    doc.select(&sel)
        .filter_map(|el| {
            let href = el.value().attr("href")?;
            let text = el.text().collect::<String>().trim().to_string();

            // Resolve relative URLs
            let resolved = if let Some(ref b) = base {
                b.join(href).ok().map(|u| u.to_string())
            } else {
                Some(href.to_string())
            };

            resolved.map(|url| Link {
                url,
                text: if text.is_empty() { None } else { Some(text) },
                is_external: href.starts_with("http"),
            })
        })
        .collect()
}

/// A link extracted from a page.
#[derive(Debug, Clone, Serialize)]
pub struct Link {
    /// Resolved URL.
    pub url: String,
    /// Link text (anchor text).
    pub text: Option<String>,
    /// Whether the link points to an external domain.
    pub is_external: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_css_headings() {
        let html = "<html><body><h1>Title</h1><h2>Sub</h2><p>Text</p></body></html>";
        let result = extract_css(html, "h1, h2").unwrap();
        assert_eq!(result.count, 2);
        assert_eq!(result.matches[0], "Title");
        assert_eq!(result.matches[1], "Sub");
    }

    #[test]
    fn extract_text_strips_tags() {
        let html = "<body><p>Hello <b>world</b></p></body>";
        let text = extract_text(html);
        assert!(text.contains("Hello"));
        assert!(text.contains("world"));
    }

    #[test]
    fn extract_links_resolves_relative() {
        let html = r#"<body><a href="/page">Link</a><a href="https://other.com">Ext</a></body>"#;
        let links = extract_links(html, "https://example.com");
        assert_eq!(links.len(), 2);
        assert_eq!(links[0].url, "https://example.com/page");
        assert!(links[1].is_external);
    }

    #[test]
    fn invalid_selector_returns_error() {
        let result = extract_css("<body></body>", "###invalid");
        assert!(result.is_err());
    }
}
