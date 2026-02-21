//! Authentication module — Firebase Auth REST API
//!
//! Provides auth context, sign-in/sign-up/reset pages, and route guards.

mod signin;
mod signup;
mod reset_password;

pub use signin::SignInPage;
pub use signup::SignUpPage;
pub use reset_password::ResetPasswordPage;

use leptos::prelude::*;
use wksp_types::user::{AuthState, UserProfile};

const AUTH_TOKEN_KEY: &str = "nucleus_auth_token";
const REFRESH_TOKEN_KEY: &str = "nucleus_refresh_token";
const USER_DATA_KEY: &str = "nucleus_user_data";

/// Reactive auth context available throughout the app
#[derive(Debug, Clone)]
pub struct AuthContext {
    /// Current auth state
    pub state: RwSignal<AuthState>,
    /// Current user profile (if authenticated)
    pub user: RwSignal<Option<UserProfile>>,
    /// Firebase ID token for API calls
    pub token: RwSignal<Option<String>>,
    /// Firebase refresh token (for session persistence)
    pub refresh_token: RwSignal<Option<String>>,
    /// Whether user is authenticated
    pub is_authenticated: Signal<bool>,
}

impl AuthContext {
    /// Create a new auth context and load from storage if on client
    pub fn new() -> Self {
        let state = RwSignal::new(AuthState::Loading);
        let user = RwSignal::new(None);
        let token = RwSignal::new(None);
        let refresh_token = RwSignal::new(None);
        let is_authenticated = Signal::derive(move || {
            matches!(state.get(), AuthState::Authenticated)
        });

        let ctx = Self {
            state,
            user,
            token,
            refresh_token,
            is_authenticated,
        };

        #[cfg(feature = "hydrate")]
        ctx.load_from_storage();

        ctx
    }

    /// Load tokens from local storage (Client side only)
    #[cfg(feature = "hydrate")]
    pub fn load_from_storage(&self) {
        use gloo_storage::{LocalStorage, Storage};
        
        let token: Option<String> = LocalStorage::get(AUTH_TOKEN_KEY).ok();
        let refresh: Option<String> = LocalStorage::get(REFRESH_TOKEN_KEY).ok();
        let user: Option<UserProfile> = LocalStorage::get(USER_DATA_KEY).ok();

        if let (Some(t), Some(u)) = (token, user) {
            self.token.set(Some(t));
            self.refresh_token.set(refresh);
            self.user.set(Some(u));
            self.state.set(AuthState::Authenticated);
        } else {
            self.state.set(AuthState::Unauthenticated);
        }
    }

    /// Save tokens to local storage (Client side only)
    #[cfg(feature = "hydrate")]
    pub fn save_to_storage(&self) {
        use gloo_storage::{LocalStorage, Storage};

        if let Some(t) = self.token.get() {
            let _ = LocalStorage::set(AUTH_TOKEN_KEY, t);
        }
        if let Some(r) = self.refresh_token.get() {
            let _ = LocalStorage::set(REFRESH_TOKEN_KEY, r);
        }
        if let Some(u) = self.user.get() {
            let _ = LocalStorage::set(USER_DATA_KEY, u);
        }
    }

    /// Sign out — clear all auth state and storage
    pub fn sign_out(&self) {
        self.state.set(AuthState::Unauthenticated);
        self.user.set(None);
        self.token.set(None);
        self.refresh_token.set(None);

        #[cfg(feature = "hydrate")]
        {
            use gloo_storage::{LocalStorage, Storage};
            LocalStorage::delete(AUTH_TOKEN_KEY);
            LocalStorage::delete(REFRESH_TOKEN_KEY);
            LocalStorage::delete(USER_DATA_KEY);
        }
    }
}

impl Default for AuthContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Provide auth context at app root
pub fn provide_auth_context() {
    let ctx = AuthContext::new();
    
    // Setup effect to save to storage when state changes
    #[cfg(feature = "hydrate")]
    {
        let ctx_clone = ctx.clone();
        Effect::new(move |_| {
            if matches!(ctx_clone.state.get(), AuthState::Authenticated) {
                ctx_clone.save_to_storage();
            }
        });
    }

    provide_context(ctx);
}

/// Get auth context from any component
pub fn use_auth() -> AuthContext {
    expect_context::<AuthContext>()
}

/// Server function: sign in with email/password via Firebase REST
#[server(SignIn, "/api")]
pub async fn server_sign_in(email: String, password: String) -> Result<AuthResponseData, ServerFnError> {
    use crate::firebase::auth::FirebaseAuthClient;

    let api_key = std::env::var("FIREBASE_API_KEY")
        .map_err(|_| ServerFnError::new("FIREBASE_API_KEY not set"))?;

    let client = FirebaseAuthClient::new(api_key);
    let result = client.sign_in(&email, &password).await
        .map_err(ServerFnError::new)?;

    Ok(AuthResponseData {
        id_token: result.id_token,
        refresh_token: result.refresh_token,
        email: result.email,
        local_id: result.local_id,
    })
}

/// Server function: create account via Firebase REST
#[server(SignUp, "/api")]
pub async fn server_sign_up(email: String, password: String, _name: String) -> Result<AuthResponseData, ServerFnError> {
    use crate::firebase::auth::FirebaseAuthClient;

    let api_key = std::env::var("FIREBASE_API_KEY")
        .map_err(|_| ServerFnError::new("FIREBASE_API_KEY not set"))?;

    let client = FirebaseAuthClient::new(api_key);
    let result = client.sign_up(&email, &password).await
        .map_err(ServerFnError::new)?;

    Ok(AuthResponseData {
        id_token: result.id_token,
        refresh_token: result.refresh_token,
        email: result.email,
        local_id: result.local_id,
    })
}

/// Server function: send password reset email
#[server(ResetPassword, "/api")]
pub async fn server_reset_password(email: String) -> Result<(), ServerFnError> {
    use crate::firebase::auth::FirebaseAuthClient;

    let api_key = std::env::var("FIREBASE_API_KEY")
        .map_err(|_| ServerFnError::new("FIREBASE_API_KEY not set"))?;

    let client = FirebaseAuthClient::new(api_key);
    client.send_password_reset(&email).await
        .map_err(ServerFnError::new)?;

    Ok(())
}

/// Shared auth response data (serializable across SSR boundary)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuthResponseData {
    pub id_token: String,
    pub refresh_token: String,
    pub email: String,
    pub local_id: String,
}