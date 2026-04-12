//! Page metadata extraction — title, description, OpenGraph, structured data.

use serde::Serialize;

/// Extracted page metadata.
#[derive(Debug, Clone, Serialize)]
pub struct PageMetadata {
    /// Page title from <title> tag.
    pub title: Option<String>,
    /// Meta description.
    pub description: Option<String>,
    /// Canonical URL.
    pub canonical: Option<String>,
    /// OpenGraph title.
    pub og_title: Option<String>,
    /// OpenGraph description.
    pub og_description: Option<String>,
    /// OpenGraph image URL.
    pub og_image: Option<String>,
    /// OpenGraph type (article, website, etc).
    pub og_type: Option<String>,
    /// Language from html lang attribute.
    pub language: Option<String>,
    /// All meta keywords.
    pub keywords: Vec<String>,
}

/// Extract metadata from HTML.
pub fn extract_metadata(html: &str) -> PageMetadata {
    use scraper::{Html, Selector};

    let doc = Html::parse_document(html);

    let title = sel_text(&doc, "title");
    let description = meta_content(&doc, "description");
    let canonical = sel_attr(&doc, "link[rel=canonical]", "href");
    let og_title = meta_property(&doc, "og:title");
    let og_description = meta_property(&doc, "og:description");
    let og_image = meta_property(&doc, "og:image");
    let og_type = meta_property(&doc, "og:type");

    let language = Selector::parse("html")
        .ok()
        .and_then(|sel| doc.select(&sel).next())
        .and_then(|el| el.value().attr("lang"))
        .map(|s| s.to_string());

    let keywords = meta_content(&doc, "keywords")
        .map(|k| {
            k.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default();

    PageMetadata {
        title,
        description,
        canonical,
        og_title,
        og_description,
        og_image,
        og_type,
        language,
        keywords,
    }
}

fn sel_text(doc: &scraper::Html, selector: &str) -> Option<String> {
    scraper::Selector::parse(selector)
        .ok()
        .and_then(|sel| doc.select(&sel).next())
        .map(|el| el.text().collect::<String>().trim().to_string())
        .filter(|s| !s.is_empty())
}

fn sel_attr(doc: &scraper::Html, selector: &str, attr: &str) -> Option<String> {
    scraper::Selector::parse(selector)
        .ok()
        .and_then(|sel| doc.select(&sel).next())
        .and_then(|el| el.value().attr(attr))
        .map(|s| s.to_string())
}

fn meta_content(doc: &scraper::Html, name: &str) -> Option<String> {
    let selector = format!("meta[name=\"{name}\"]");
    sel_attr(doc, &selector, "content")
}

fn meta_property(doc: &scraper::Html, property: &str) -> Option<String> {
    let selector = format!("meta[property=\"{property}\"]");
    sel_attr(doc, &selector, "content")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_full_metadata() {
        let html = r#"
        <html lang="en">
        <head>
            <title>Test Page</title>
            <meta name="description" content="A test page">
            <meta name="keywords" content="test, page, rust">
            <meta property="og:title" content="OG Title">
            <meta property="og:type" content="article">
            <link rel="canonical" href="https://example.com/test">
        </head>
        <body></body>
        </html>"#;

        let m = extract_metadata(html);
        assert_eq!(m.title.as_deref(), Some("Test Page"));
        assert_eq!(m.description.as_deref(), Some("A test page"));
        assert_eq!(m.og_title.as_deref(), Some("OG Title"));
        assert_eq!(m.og_type.as_deref(), Some("article"));
        assert_eq!(m.canonical.as_deref(), Some("https://example.com/test"));
        assert_eq!(m.language.as_deref(), Some("en"));
        assert_eq!(m.keywords, vec!["test", "page", "rust"]);
    }

    #[test]
    fn empty_html_returns_none() {
        let m = extract_metadata("<html><body></body></html>");
        assert!(m.title.is_none());
        assert!(m.description.is_none());
        assert!(m.keywords.is_empty());
    }
}
