//! Browser state management for Chrome DevTools integration
//!
//! Uses Guardian pattern: global lazy-init with `OnceLock<Mutex<T>>`
//! Extended with event broadcast channel for Vigil integration.

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::page::Page;
use futures::StreamExt;
use parking_lot::Mutex;
use tokio::sync::broadcast;
use tracing::{debug, info, warn};

use crate::events::BrowserEvent;
use crate::handlers;

/// Broadcast channel capacity for browser events
const EVENT_CHANNEL_CAPACITY: usize = 1000;

/// Global browser context (Guardian pattern)
static BROWSER_CONTEXT: OnceLock<Arc<Mutex<BrowserState>>> = OnceLock::new();

/// Global event broadcast sender
static EVENT_SENDER: OnceLock<broadcast::Sender<BrowserEvent>> = OnceLock::new();

/// Browser configuration
///
/// Tier: T3 (Domain-specific browser settings)
#[derive(Debug, Clone)]
pub struct BrowserSettings {
    /// Headless mode (true = headless, false = headed)
    pub headless: bool,
    /// Window width
    pub width: u32,
    /// Window height
    pub height: u32,
    /// Disable GPU (required for some headless environments)
    pub disable_gpu: bool,
    /// Chrome executable path (None = auto-detect)
    pub chrome_path: Option<String>,
}

impl Default for BrowserSettings {
    fn default() -> Self {
        Self {
            headless: true,
            width: 1280,
            height: 720,
            disable_gpu: true,
            chrome_path: None,
        }
    }
}

/// Internal browser state
///
/// Tier: T3 (Domain-specific browser state)
pub struct BrowserState {
    /// The browser instance wrapped in Arc for sharing
    browser: Option<Arc<Browser>>,
    /// Handler for browser events (keeps browser alive)
    event_handle: Option<tokio::task::JoinHandle<()>>,
    /// Open pages indexed by ID
    pages: HashMap<String, Arc<Page>>,
    /// Currently selected page ID
    current_page_id: Option<String>,
    /// Next page ID counter
    next_page_id: u32,
    /// Configuration
    pub settings: BrowserSettings,
}

impl BrowserState {
    fn new() -> Self {
        Self {
            browser: None,
            event_handle: None,
            pages: HashMap::new(),
            current_page_id: None,
            next_page_id: 1,
            settings: BrowserSettings::default(),
        }
    }

    /// Generate next page ID
    fn gen_page_id(&mut self) -> String {
        let id = format!("page_{}", self.next_page_id);
        self.next_page_id += 1;
        id
    }
}

/// Get or initialize the global browser context
#[must_use]
pub fn get_context() -> Arc<Mutex<BrowserState>> {
    BROWSER_CONTEXT
        .get_or_init(|| Arc::new(Mutex::new(BrowserState::new())))
        .clone()
}

/// Get the event sender (initializes if needed)
fn get_event_sender() -> broadcast::Sender<BrowserEvent> {
    EVENT_SENDER
        .get_or_init(|| {
            let (tx, _) = broadcast::channel(EVENT_CHANNEL_CAPACITY);
            tx
        })
        .clone()
}

/// Subscribe to browser events
///
/// Returns a receiver that will receive all browser events.
/// Multiple subscribers are supported via broadcast channel.
#[must_use]
pub fn subscribe_events() -> broadcast::Receiver<BrowserEvent> {
    get_event_sender().subscribe()
}

/// Launch browser if not already running
pub async fn ensure_browser() -> Result<(), BrowserError> {
    let ctx = get_context();
    let needs_launch = {
        let state = ctx.lock();
        state.browser.is_none()
    };

    if needs_launch {
        launch_browser().await?;
    }
    Ok(())
}

/// Launch a new browser instance
pub async fn launch_browser() -> Result<(), BrowserError> {
    let ctx = get_context();
    let settings = {
        let state = ctx.lock();
        state.settings.clone()
    };

    // Build browser config
    let mut builder = BrowserConfig::builder();

    if !settings.headless {
        builder = builder.with_head();
    }

    builder = builder.window_size(settings.width, settings.height);

    if settings.disable_gpu {
        builder = builder.arg("--disable-gpu");
    }

    // Common args for stability
    builder = builder
        .arg("--no-sandbox")
        .arg("--disable-dev-shm-usage")
        .arg("--disable-extensions");

    if let Some(ref path) = settings.chrome_path {
        builder = builder.chrome_executable(path);
    }

    let config = builder.build().map_err(BrowserError::Launch)?;

    let (browser, mut handler) = Browser::launch(config)
        .await
        .map_err(|e| BrowserError::Launch(e.to_string()))?;

    let tx = get_event_sender();

    // Spawn handler task with event routing
    let handle = tokio::spawn(async move {
        while (handler.next().await).is_some() {
            // Handler messages processed - keeps browser connection alive
        }
        // Browser disconnected
        handlers::handle_browser_disconnected("Browser process exited".to_string(), &tx);
    });

    // Store browser wrapped in Arc
    let mut state = ctx.lock();
    state.browser = Some(Arc::new(browser));
    state.event_handle = Some(handle);
    state.pages.clear();
    state.current_page_id = None;

    info!("Browser launched successfully");
    Ok(())
}

/// Close the browser
pub async fn close_browser() -> Result<(), BrowserError> {
    let ctx = get_context();
    let mut state = ctx.lock();

    // Drop the Arc<Browser> - browser closes when last reference drops
    state.browser = None;
    state.event_handle = None;
    state.pages.clear();
    state.current_page_id = None;

    info!("Browser closed");
    Ok(())
}

/// Create a new page and navigate to URL
pub async fn new_page(url: &str) -> Result<PageInfo, BrowserError> {
    ensure_browser().await?;

    let ctx = get_context();
    let browser = {
        let state = ctx.lock();
        state.browser.clone().ok_or(BrowserError::NotConnected)?
    };

    let page = browser
        .new_page(url)
        .await
        .map_err(|e| BrowserError::Navigation(e.to_string()))?;

    let page_id = {
        let mut state = ctx.lock();
        let id = state.gen_page_id();
        state.pages.insert(id.clone(), Arc::new(page));
        state.current_page_id = Some(id.clone());
        id
    };

    // Notify page loaded
    let tx = get_event_sender();
    handlers::handle_page_loaded(url.to_string(), None, None, &page_id, &tx);

    debug!("Created page {page_id} at {url}");

    Ok(PageInfo {
        id: page_id,
        url: url.to_string(),
        title: None,
    })
}

/// Navigate current page to URL
pub async fn navigate(url: &str) -> Result<PageInfo, BrowserError> {
    let ctx = get_context();
    let (page, page_id) = {
        let state = ctx.lock();
        let page_id = state
            .current_page_id
            .clone()
            .ok_or(BrowserError::NoPageSelected)?;
        let page = state
            .pages
            .get(&page_id)
            .cloned()
            .ok_or_else(|| BrowserError::PageNotFound(page_id.clone()))?;
        (page, page_id)
    };

    page.goto(url)
        .await
        .map_err(|e| BrowserError::Navigation(e.to_string()))?;

    let title = page.get_title().await.ok().flatten();
    let current_url = page
        .url()
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| url.to_string());

    // Notify page loaded
    let tx = get_event_sender();
    handlers::handle_page_loaded(current_url.clone(), title.clone(), None, &page_id, &tx);

    Ok(PageInfo {
        id: page_id,
        url: current_url,
        title,
    })
}

/// Select a page by ID
pub fn select_page(page_id: &str) -> Result<(), BrowserError> {
    let ctx = get_context();
    let mut state = ctx.lock();

    if !state.pages.contains_key(page_id) {
        return Err(BrowserError::PageNotFound(page_id.to_string()));
    }

    state.current_page_id = Some(page_id.to_string());
    Ok(())
}

/// Get current page (Arc-wrapped for sharing)
pub fn get_current_page() -> Result<Arc<Page>, BrowserError> {
    let ctx = get_context();
    let state = ctx.lock();

    let page_id = state
        .current_page_id
        .as_ref()
        .ok_or(BrowserError::NoPageSelected)?;
    state
        .pages
        .get(page_id)
        .cloned()
        .ok_or_else(|| BrowserError::PageNotFound(page_id.clone()))
}

/// Get page by ID (Arc-wrapped for sharing)
pub fn get_page(page_id: &str) -> Result<Arc<Page>, BrowserError> {
    let ctx = get_context();
    let state = ctx.lock();
    state
        .pages
        .get(page_id)
        .cloned()
        .ok_or_else(|| BrowserError::PageNotFound(page_id.to_string()))
}

/// Close a page by ID
pub async fn close_page(page_id: &str) -> Result<(), BrowserError> {
    let ctx = get_context();
    let page = {
        let mut state = ctx.lock();
        let page = state
            .pages
            .remove(page_id)
            .ok_or_else(|| BrowserError::PageNotFound(page_id.to_string()))?;

        // If closing current page, select another
        if state.current_page_id.as_deref() == Some(page_id) {
            state.current_page_id = state.pages.keys().next().cloned();
        }
        page
    };

    // Page::close() takes self by value. Use Arc::try_unwrap to get ownership.
    match Arc::try_unwrap(page) {
        Ok(owned_page) => {
            owned_page
                .close()
                .await
                .map_err(|e| BrowserError::PageClose(e.to_string()))?;
        }
        Err(_arc) => {
            // Other references exist; page will close when last Arc drops
            warn!("Page {page_id} has other references, will close when dropped");
        }
    }
    Ok(())
}

/// List all open pages
pub async fn list_pages() -> Result<Vec<PageInfo>, BrowserError> {
    let ctx = get_context();
    let pages_snapshot: Vec<(String, Arc<Page>)> = {
        let state = ctx.lock();
        state
            .pages
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    };

    let mut pages = Vec::new();
    for (id, page) in pages_snapshot {
        let url = page.url().await.ok().flatten().unwrap_or_default();
        let title = page.get_title().await.ok().flatten();
        pages.push(PageInfo { id, url, title });
    }

    Ok(pages)
}

/// Get current page ID
#[must_use]
pub fn current_page_id() -> Option<String> {
    let ctx = get_context();
    let state = ctx.lock();
    state.current_page_id.clone()
}

/// Update browser settings
pub fn update_settings(settings: BrowserSettings) {
    let ctx = get_context();
    let mut state = ctx.lock();
    state.settings = settings;
}

/// Check if browser is running
#[must_use]
pub fn is_browser_running() -> bool {
    let ctx = get_context();
    let state = ctx.lock();
    state.browser.is_some()
}

/// Get page count
#[must_use]
pub fn page_count() -> usize {
    let ctx = get_context();
    let state = ctx.lock();
    state.pages.len()
}

/// Page information
///
/// Tier: T3 (Domain-specific page info)
#[derive(Debug, Clone, serde::Serialize)]
pub struct PageInfo {
    /// Unique page identifier.
    pub id: String,
    /// Current page URL.
    pub url: String,
    /// Page title (if available).
    pub title: Option<String>,
}

/// Browser errors
///
/// Tier: T3 (Domain-specific browser error)
#[derive(Debug, thiserror::Error)]
pub enum BrowserError {
    /// Browser process failed to start.
    #[error("Failed to launch browser: {0}")]
    Launch(String),

    /// No browser instance is running.
    #[error("Browser not connected")]
    NotConnected,

    /// Operation requires a selected page but none is active.
    #[error("No page selected")]
    NoPageSelected,

    /// Referenced page ID does not exist.
    #[error("Page not found: {0}")]
    PageNotFound(String),

    /// Page navigation failed.
    #[error("Navigation failed: {0}")]
    Navigation(String),

    /// Page close operation failed.
    #[error("Failed to close page: {0}")]
    PageClose(String),

    /// DOM element not found by selector.
    #[error("Element not found: {0}")]
    ElementNotFound(String),

    /// JavaScript evaluation returned an error.
    #[error("JavaScript evaluation failed: {0}")]
    JsEval(String),

    /// Screenshot capture failed.
    #[error("Screenshot failed: {0}")]
    Screenshot(String),

    /// Performance trace recording/stopping failed.
    #[error("Performance trace failed: {0}")]
    Trace(String),

    /// Network interception or request failed.
    #[error("Network operation failed: {0}")]
    Network(String),

    /// Keyboard/mouse input failed.
    #[error("Input operation failed: {0}")]
    Input(String),
}
