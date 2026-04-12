//! Link crawler — follow links to specified depth.
//!
//! Breadth-first crawl with configurable depth, domain filtering,
//! and rate limiting. Returns a site map with extracted text per page.

use crate::extract::extract_links;
use crate::fetch::{FetchConfig, FetchResult, fetch_page};
use nexcore_error::NexError;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Crawl configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CrawlConfig {
    /// Maximum crawl depth (0 = just the seed URL).
    pub max_depth: usize,
    /// Maximum total pages to fetch.
    pub max_pages: usize,
    /// Only follow links to the same domain.
    pub same_domain_only: bool,
    /// Delay between requests in milliseconds.
    pub delay_ms: u64,
    /// Fetch configuration.
    pub fetch: FetchConfig,
}

impl Default for CrawlConfig {
    fn default() -> Self {
        Self {
            max_depth: 1,
            max_pages: 10,
            same_domain_only: true,
            delay_ms: 500,
            fetch: FetchConfig::default(),
        }
    }
}

/// A single crawled page.
#[derive(Debug, Clone, Serialize)]
pub struct CrawledPage {
    /// URL of the page.
    pub url: String,
    /// Crawl depth (0 = seed).
    pub depth: usize,
    /// Page title.
    pub title: Option<String>,
    /// Extracted text (truncated to 2000 chars).
    pub text: String,
    /// Number of outgoing links found.
    pub link_count: usize,
    /// HTTP status.
    pub status: u16,
}

/// Crawl result.
#[derive(Debug, Clone, Serialize)]
pub struct CrawlResult {
    /// Seed URL.
    pub seed: String,
    /// All crawled pages.
    pub pages: Vec<CrawledPage>,
    /// Total pages crawled.
    pub total_pages: usize,
    /// Maximum depth reached.
    pub max_depth_reached: usize,
    /// Total elapsed time in milliseconds.
    pub elapsed_ms: u64,
}

/// Crawl a website starting from a seed URL.
///
/// Breadth-first traversal. Respects `same_domain_only`, `max_depth`,
/// and `max_pages` limits.
pub async fn crawl(seed_url: &str, config: &CrawlConfig) -> Result<CrawlResult, NexError> {
    let start = std::time::Instant::now();
    let seed_domain = url::Url::parse(seed_url)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_string()));

    let mut visited: HashSet<String> = HashSet::new();
    let mut pages: Vec<CrawledPage> = Vec::new();

    // BFS queue: (url, depth)
    let mut queue: Vec<(String, usize)> = vec![(seed_url.to_string(), 0)];

    while let Some((url, depth)) = queue.first().cloned() {
        queue.remove(0);

        if visited.contains(&url) || pages.len() >= config.max_pages {
            continue;
        }
        visited.insert(url.clone());

        // Rate limit
        if !pages.is_empty() && config.delay_ms > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(config.delay_ms)).await;
        }

        match fetch_page(&url, &config.fetch).await {
            Ok(result) => {
                let links = if result.content_type.contains("html") {
                    extract_links(&result.text, &url)
                } else {
                    vec![]
                };
                let link_count = links.len();

                // Truncate text for storage
                let text = if result.text.len() > 2000 {
                    format!("{}...", &result.text[..2000])
                } else {
                    result.text
                };

                pages.push(CrawledPage {
                    url: result.url,
                    depth,
                    title: result.title,
                    text,
                    link_count,
                    status: result.status,
                });

                // Enqueue child links if within depth
                if depth < config.max_depth {
                    for link in links {
                        if visited.contains(&link.url) {
                            continue;
                        }
                        if config.same_domain_only {
                            let link_domain = url::Url::parse(&link.url)
                                .ok()
                                .and_then(|u| u.host_str().map(|h| h.to_string()));
                            if link_domain != seed_domain {
                                continue;
                            }
                        }
                        queue.push((link.url, depth + 1));
                    }
                }
            }
            Err(_) => {
                // Skip failed pages, don't halt the crawl
                pages.push(CrawledPage {
                    url,
                    depth,
                    title: None,
                    text: String::new(),
                    link_count: 0,
                    status: 0,
                });
            }
        }
    }

    let max_depth_reached = pages.iter().map(|p| p.depth).max().unwrap_or(0);

    Ok(CrawlResult {
        seed: seed_url.to_string(),
        pages: pages.clone(),
        total_pages: pages.len(),
        max_depth_reached,
        elapsed_ms: start.elapsed().as_millis() as u64,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let c = CrawlConfig::default();
        assert_eq!(c.max_depth, 1);
        assert_eq!(c.max_pages, 10);
        assert!(c.same_domain_only);
    }
}
