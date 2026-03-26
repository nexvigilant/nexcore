//! Claude Code documentation tools — fetch and search official docs.
//!
//! Consolidated from `nexcore-docs-mcp` satellite server.
//! 4 tools: docs_claude_list_pages, docs_claude_get_page, docs_claude_search, docs_claude_index.
//!
//! Uses OnceLock-based lazy state for HTTP client and in-memory cache.
//!
//! Tier: T3 (π Persistence + μ Mapping + σ Sequence + ∂ Boundary)

use std::sync::OnceLock;
use std::time::Duration;

use crate::params::{DocsClaudeGetPageParams, DocsClaudeListPagesParams, DocsClaudeSearchParams};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};

// ============================================================================
// Lazy state
// ============================================================================

static HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
static PAGES: OnceLock<Vec<DocPage>> = OnceLock::new();

fn http_client() -> &'static reqwest::Client {
    HTTP_CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("nexcore-mcp-docs/0.1.0")
            .build()
            .unwrap_or_default()
    })
}

fn pages() -> &'static [DocPage] {
    PAGES.get_or_init(init_pages)
}

// ============================================================================
// Doc page definitions
// ============================================================================

struct DocPage {
    title: &'static str,
    path: &'static str,
    url: &'static str,
    description: &'static str,
}

fn init_pages() -> Vec<DocPage> {
    vec![
        DocPage {
            title: "Overview",
            path: "overview",
            url: "https://code.claude.com/docs/en/overview",
            description: "Learn about Claude Code",
        },
        DocPage {
            title: "Quickstart",
            path: "quickstart",
            url: "https://code.claude.com/docs/en/quickstart",
            description: "Getting started guide",
        },
        DocPage {
            title: "Setup",
            path: "setup",
            url: "https://code.claude.com/docs/en/setup",
            description: "Install and authenticate",
        },
        DocPage {
            title: "How It Works",
            path: "how-claude-code-works",
            url: "https://code.claude.com/docs/en/how-claude-code-works",
            description: "Agentic loop and tools",
        },
        DocPage {
            title: "Common Workflows",
            path: "common-workflows",
            url: "https://code.claude.com/docs/en/common-workflows",
            description: "Step-by-step guides",
        },
        DocPage {
            title: "Best Practices",
            path: "best-practices",
            url: "https://code.claude.com/docs/en/best-practices",
            description: "Tips and patterns",
        },
        DocPage {
            title: "CLI Reference",
            path: "cli-reference",
            url: "https://code.claude.com/docs/en/cli-reference",
            description: "Commands and flags",
        },
        DocPage {
            title: "Settings",
            path: "settings",
            url: "https://code.claude.com/docs/en/settings",
            description: "Configuration options",
        },
        DocPage {
            title: "MCP",
            path: "mcp",
            url: "https://code.claude.com/docs/en/mcp",
            description: "Model Context Protocol",
        },
        DocPage {
            title: "Skills",
            path: "skills",
            url: "https://code.claude.com/docs/en/skills",
            description: "Extend Claude's capabilities",
        },
        DocPage {
            title: "Hooks Guide",
            path: "hooks-guide",
            url: "https://code.claude.com/docs/en/hooks-guide",
            description: "Customize behavior",
        },
        DocPage {
            title: "Hooks Reference",
            path: "hooks",
            url: "https://code.claude.com/docs/en/hooks",
            description: "Hooks documentation",
        },
        DocPage {
            title: "Subagents",
            path: "sub-agents",
            url: "https://code.claude.com/docs/en/sub-agents",
            description: "Specialized AI agents",
        },
        DocPage {
            title: "Plugins",
            path: "plugins",
            url: "https://code.claude.com/docs/en/plugins",
            description: "Create plugins",
        },
        DocPage {
            title: "VS Code",
            path: "vs-code",
            url: "https://code.claude.com/docs/en/vs-code",
            description: "VS Code extension",
        },
        DocPage {
            title: "JetBrains",
            path: "jetbrains",
            url: "https://code.claude.com/docs/en/jetbrains",
            description: "JetBrains IDEs",
        },
        DocPage {
            title: "GitHub Actions",
            path: "github-actions",
            url: "https://code.claude.com/docs/en/github-actions",
            description: "CI/CD integration",
        },
        DocPage {
            title: "Security",
            path: "security",
            url: "https://code.claude.com/docs/en/security",
            description: "Security safeguards",
        },
        DocPage {
            title: "Troubleshooting",
            path: "troubleshooting",
            url: "https://code.claude.com/docs/en/troubleshooting",
            description: "Common issues",
        },
    ]
}

// ============================================================================
// Tool implementations
// ============================================================================

/// List all available Claude Code documentation pages.
pub fn docs_claude_list_pages(
    params: DocsClaudeListPagesParams,
) -> Result<CallToolResult, McpError> {
    let page_list: Vec<String> = pages()
        .iter()
        .filter(|p| {
            params.category.as_ref().is_none_or(|cat| {
                let c = cat.to_lowercase();
                p.path.contains(&c) || p.title.to_lowercase().contains(&c)
            })
        })
        .map(|p| format!("- **{}** (`{}`): {}", p.title, p.path, p.description))
        .collect();

    let result = if page_list.is_empty() {
        "No pages found.".to_string()
    } else {
        format!(
            "# Claude Code Docs\n\n{}\n\n*Use docs_claude_get_page with path.*",
            page_list.join("\n")
        )
    };
    Ok(CallToolResult::success(vec![Content::text(result)]))
}

/// Get content of a specific documentation page.
pub async fn docs_claude_get_page(
    params: DocsClaudeGetPageParams,
) -> Result<CallToolResult, McpError> {
    let q = params.page.to_lowercase();
    let page = pages().iter().find(|p| p.path == q || p.path.contains(&q));

    match page {
        Some(p) => match fetch_page(p.url).await {
            Ok(c) => Ok(CallToolResult::success(vec![Content::text(format!(
                "# {}\n\n{}",
                p.title, c
            ))])),
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Error: {e}"
            ))])),
        },
        None => Ok(CallToolResult::success(vec![Content::text(
            "Page not found. Use docs_claude_list_pages.",
        )])),
    }
}

/// Search documentation for a topic.
pub fn docs_claude_search(params: DocsClaudeSearchParams) -> Result<CallToolResult, McpError> {
    let q = params.query.to_lowercase();
    let limit = params.limit.unwrap_or(5);

    let mut scored: Vec<(&DocPage, i32)> = pages()
        .iter()
        .filter_map(|p| {
            let mut s = 0i32;
            if p.title.to_lowercase().contains(&q) {
                s += 50;
            }
            if p.path.contains(&q) {
                s += 30;
            }
            if p.description.to_lowercase().contains(&q) {
                s += 20;
            }
            if s > 0 { Some((p, s)) } else { None }
        })
        .collect();
    scored.sort_by(|a, b| b.1.cmp(&a.1));

    let results: Vec<String> = scored
        .into_iter()
        .take(limit)
        .map(|(p, _)| format!("- **{}** (`{}`): {}", p.title, p.path, p.description))
        .collect();

    let result = if results.is_empty() {
        format!("No results for '{}'.", params.query)
    } else {
        format!("# Results for '{}'\n\n{}", params.query, results.join("\n"))
    };
    Ok(CallToolResult::success(vec![Content::text(result)]))
}

/// Get the documentation index.
pub async fn docs_claude_index() -> Result<CallToolResult, McpError> {
    match fetch_page("https://code.claude.com/docs/llms.txt").await {
        Ok(c) => Ok(CallToolResult::success(vec![Content::text(format!(
            "# Index\n\n{c}"
        ))])),
        Err(_) => {
            let list: Vec<String> = pages()
                .iter()
                .map(|p| format!("- {} ({})", p.title, p.path))
                .collect();
            Ok(CallToolResult::success(vec![Content::text(format!(
                "# Index (cached)\n\n{}",
                list.join("\n")
            ))]))
        }
    }
}

// ============================================================================
// HTTP helpers
// ============================================================================

async fn fetch_page(url: &str) -> Result<String, nexcore_error::NexError> {
    let md_url = format!("{url}.md");
    let response = http_client()
        .get(&md_url)
        .send()
        .await
        .map_err(|e| nexcore_error::NexError::new(format!("Fetch error: {e}")))?;
    if !response.status().is_success() {
        return Err(nexcore_error::NexError::new(format!(
            "HTTP {}",
            response.status()
        )));
    }
    response
        .text()
        .await
        .map_err(|e| nexcore_error::NexError::new(format!("Read error: {e}")))
}
