//! Browser executor for Vigil
//!
//! Executes browser automation actions via nexcore-browser.

use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use tracing::{debug, info, warn};

use nexcore_browser::state::{close_browser, close_page, is_browser_running, navigate, new_page};

use crate::executors::Executor;
use crate::models::{ExecutorResult, ExecutorType};

/// Browser executor for Vigil DecisionEngine
///
/// Executes browser automation actions:
/// - navigate: Navigate to URL
/// - new_page: Create new page at URL
/// - close_page: Close a page by ID
/// - close_browser: Close browser
pub struct BrowserExecutor;

impl BrowserExecutor {
    /// Create a new browser executor
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Execute navigate action
    async fn execute_navigate(&self, params: &Value) -> anyhow::Result<ExecutorResult> {
        let url = params
            .get("url")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("Missing 'url' parameter"))?;

        info!(url = %url, "Navigating to URL");

        match navigate(url).await {
            Ok(page_info) => {
                let msg = ["Navigated to ", &page_info.url].concat();
                Ok(ExecutorResult {
                    executor: ExecutorType::Browser,
                    success: true,
                    output: Some(msg),
                    error: None,
                    metadata: {
                        let mut m = HashMap::new();
                        m.insert("page_id".to_string(), Value::String(page_info.id));
                        m.insert("url".to_string(), Value::String(page_info.url));
                        if let Some(title) = page_info.title {
                            m.insert("title".to_string(), Value::String(title));
                        }
                        m
                    },
                })
            }
            Err(e) => Ok(ExecutorResult {
                executor: ExecutorType::Browser,
                success: false,
                output: None,
                error: Some(e.to_string()),
                metadata: HashMap::new(),
            }),
        }
    }

    /// Execute new_page action
    async fn execute_new_page(&self, params: &Value) -> anyhow::Result<ExecutorResult> {
        let url = params
            .get("url")
            .and_then(Value::as_str)
            .unwrap_or("about:blank");

        info!(url = %url, "Creating new page");

        match new_page(url).await {
            Ok(page_info) => {
                let msg = ["Created page ", &page_info.id, " at ", &page_info.url].concat();
                Ok(ExecutorResult {
                    executor: ExecutorType::Browser,
                    success: true,
                    output: Some(msg),
                    error: None,
                    metadata: {
                        let mut m = HashMap::new();
                        m.insert("page_id".to_string(), Value::String(page_info.id));
                        m.insert("url".to_string(), Value::String(page_info.url));
                        m
                    },
                })
            }
            Err(e) => Ok(ExecutorResult {
                executor: ExecutorType::Browser,
                success: false,
                output: None,
                error: Some(e.to_string()),
                metadata: HashMap::new(),
            }),
        }
    }

    /// Execute close_page action
    async fn execute_close_page(&self, params: &Value) -> anyhow::Result<ExecutorResult> {
        let page_id = params
            .get("page_id")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("Missing 'page_id' parameter"))?;

        info!(page_id = %page_id, "Closing page");

        match close_page(page_id).await {
            Ok(()) => {
                let msg = ["Closed page ", page_id].concat();
                Ok(ExecutorResult {
                    executor: ExecutorType::Browser,
                    success: true,
                    output: Some(msg),
                    error: None,
                    metadata: HashMap::new(),
                })
            }
            Err(e) => Ok(ExecutorResult {
                executor: ExecutorType::Browser,
                success: false,
                output: None,
                error: Some(e.to_string()),
                metadata: HashMap::new(),
            }),
        }
    }

    /// Execute close_browser action
    async fn execute_close_browser(&self) -> anyhow::Result<ExecutorResult> {
        info!("Closing browser");

        match close_browser().await {
            Ok(()) => Ok(ExecutorResult {
                executor: ExecutorType::Browser,
                success: true,
                output: Some("Browser closed".to_string()),
                error: None,
                metadata: HashMap::new(),
            }),
            Err(e) => Ok(ExecutorResult {
                executor: ExecutorType::Browser,
                success: false,
                output: None,
                error: Some(e.to_string()),
                metadata: HashMap::new(),
            }),
        }
    }
}

impl Default for BrowserExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Executor for BrowserExecutor {
    fn executor_type(&self) -> ExecutorType {
        ExecutorType::Browser
    }

    async fn execute(&self, action: &str, params: Value) -> anyhow::Result<ExecutorResult> {
        debug!(action = %action, "BrowserExecutor executing action");

        match action {
            "navigate" => self.execute_navigate(&params).await,
            "new_page" => self.execute_new_page(&params).await,
            "close_page" => self.execute_close_page(&params).await,
            "close_browser" => self.execute_close_browser().await,
            unknown => {
                warn!(action = %unknown, "Unknown browser action");
                let err_msg = ["Unknown action: ", unknown].concat();
                Ok(ExecutorResult {
                    executor: ExecutorType::Browser,
                    success: false,
                    output: None,
                    error: Some(err_msg),
                    metadata: HashMap::new(),
                })
            }
        }
    }

    async fn health_check(&self) -> bool {
        is_browser_running()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_type() {
        let executor = BrowserExecutor::new();
        assert_eq!(executor.executor_type(), ExecutorType::Browser);
    }

    #[tokio::test]
    async fn test_health_check_no_browser() {
        let executor = BrowserExecutor::new();
        // Without launching browser, health check should return false
        assert!(!executor.health_check().await);
    }
}
