//! # NexVigilant Core — Web Autonomy
//!
//! Autonomous internet tools for AI agents. Fetch pages, parse HTML,
//! extract content, follow links, search the web — all accessible
//! via MCP and REST.
//!
//! ## Capabilities
//!
//! | Tool | What |
//! |------|------|
//! | `fetch` | HTTP GET with clean text extraction |
//! | `extract` | CSS selector-based content extraction |
//! | `links` | Extract and classify all links from a page |
//! | `search` | Web search via DuckDuckGo HTML |
//! | `crawl` | Follow links to specified depth |
//! | `metadata` | Extract page title, description, OpenGraph |

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

pub mod crawl;
pub mod extract;
pub mod fetch;
pub mod metadata;
pub mod search;

/// Convenience prelude.
pub mod prelude {
    pub use crate::crawl::{CrawlConfig, CrawlResult, crawl};
    pub use crate::extract::{ExtractResult, extract_css, extract_text};
    pub use crate::fetch::{FetchConfig, FetchResult, fetch_page};
    pub use crate::metadata::{PageMetadata, extract_metadata};
    pub use crate::search::{SearchConfig, SearchResult, web_search};
}
