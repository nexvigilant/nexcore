//! Authentication context for API key management
//!
//! Stores API key in browser localStorage (client) or environment (server).
//! Provides reactive context for auth state across the app.
//!
//! Tier: T2-C (State + Boundary + Persistence)

use leptos::prelude::*;

/// Auth context holding API configuration
/// Tier: T2-C (State + Boundary)
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub api_url: RwSignal<String>,
    pub api_key: RwSignal<String>,
    pub is_authenticated: Signal<bool>,
}

impl Default for AuthContext {
    fn default() -> Self {
        Self::new()
    }
}

impl AuthContext {
    /// Create auth context with defaults
    pub fn new() -> Self {
        let api_url = RwSignal::new(default_api_url());
        let api_key = RwSignal::new(String::new());
        let is_authenticated = Signal::derive(move || !api_key.get().is_empty());

        Self {
            api_url,
            api_key,
            is_authenticated,
        }
    }
}

fn default_api_url() -> String {
    #[cfg(feature = "ssr")]
    {
        std::env::var("NCOS_API_URL").unwrap_or_else(|_| "http://localhost:3030".to_string())
    }
    #[cfg(not(feature = "ssr"))]
    {
        String::from("http://localhost:3030")
    }
}

/// Provide auth context at app root
pub fn provide_auth_context() {
    let ctx = AuthContext::new();
    provide_context(ctx);
}

/// Get auth context from any component
pub fn use_auth() -> AuthContext {
    expect_context::<AuthContext>()
}
