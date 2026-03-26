//! Firebase Auth REST API client
//!
//! Implements sign-in, sign-up, password reset, and token refresh
//! via the Firebase Identity Toolkit REST API.

use nexcore_error::NexError;
use serde::{Deserialize, Serialize};

/// Firebase Auth REST API base URL
const AUTH_BASE: &str = "https://identitytoolkit.googleapis.com/v1";

/// Sign-in request payload
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignInRequest {
    pub email: String,
    pub password: String,
    pub return_secure_token: bool,
}

/// Sign-up request payload
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignUpRequest {
    pub email: String,
    pub password: String,
    pub return_secure_token: bool,
}

/// Password reset request payload
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PasswordResetRequest {
    pub request_type: String,
    pub email: String,
}

/// Token refresh request payload
#[derive(Debug, Serialize)]
pub struct RefreshTokenRequest {
    pub grant_type: String,
    pub refresh_token: String,
}

/// Firebase Auth response for sign-in / sign-up
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthResponse {
    pub id_token: String,
    pub email: String,
    pub refresh_token: String,
    pub expires_in: String,
    pub local_id: String,
    pub registered: Option<bool>,
}

/// Firebase Auth error response
#[derive(Debug, Deserialize)]
pub struct AuthErrorResponse {
    pub error: AuthErrorDetail,
}

#[derive(Debug, Deserialize)]
pub struct AuthErrorDetail {
    pub code: u16,
    pub message: String,
}

/// Token refresh response
#[derive(Debug, Deserialize)]
pub struct RefreshResponse {
    pub id_token: String,
    pub refresh_token: String,
    pub expires_in: String,
    pub token_type: String,
    pub user_id: String,
}

/// Firebase Auth client (requires `client` feature — uses reqwest)
#[cfg(feature = "client")]
pub struct FirebaseAuthClient {
    api_key: String,
    http: reqwest::Client,
}

#[cfg(feature = "client")]
impl FirebaseAuthClient {
    /// Create a new auth client with the given Firebase API key
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            http: reqwest::Client::new(),
        }
    }

    /// Sign in with email and password
    ///
    /// # Errors
    /// Returns `NexError` on network failure, parse failure, or Firebase auth error.
    pub async fn sign_in(&self, email: &str, password: &str) -> Result<AuthResponse, NexError> {
        let url = format!(
            "{AUTH_BASE}/accounts:signInWithPassword?key={}",
            self.api_key
        );
        let body = SignInRequest {
            email: email.to_string(),
            password: password.to_string(),
            return_secure_token: true,
        };

        let resp = self
            .http
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| NexError::new(format!("Network error: {e}")))?;

        if resp.status().is_success() {
            resp.json::<AuthResponse>()
                .await
                .map_err(|e| NexError::new(format!("Parse error: {e}")))
        } else {
            let err = resp
                .json::<AuthErrorResponse>()
                .await
                .map_err(|e| NexError::new(format!("Error parse error: {e}")))?;
            Err(NexError::new(err.error.message))
        }
    }

    /// Create a new account with email and password
    ///
    /// # Errors
    /// Returns `NexError` on network failure, parse failure, or Firebase auth error.
    pub async fn sign_up(&self, email: &str, password: &str) -> Result<AuthResponse, NexError> {
        let url = format!("{AUTH_BASE}/accounts:signUp?key={}", self.api_key);
        let body = SignUpRequest {
            email: email.to_string(),
            password: password.to_string(),
            return_secure_token: true,
        };

        let resp = self
            .http
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| NexError::new(format!("Network error: {e}")))?;

        if resp.status().is_success() {
            resp.json::<AuthResponse>()
                .await
                .map_err(|e| NexError::new(format!("Parse error: {e}")))
        } else {
            let err = resp
                .json::<AuthErrorResponse>()
                .await
                .map_err(|e| NexError::new(format!("Error parse error: {e}")))?;
            Err(NexError::new(err.error.message))
        }
    }

    /// Send password reset email
    ///
    /// # Errors
    /// Returns `NexError` on network failure, parse failure, or Firebase auth error.
    pub async fn send_password_reset(&self, email: &str) -> Result<(), NexError> {
        let url = format!("{AUTH_BASE}/accounts:sendOobCode?key={}", self.api_key);
        let body = PasswordResetRequest {
            request_type: "PASSWORD_RESET".to_string(),
            email: email.to_string(),
        };

        let resp = self
            .http
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| NexError::new(format!("Network error: {e}")))?;

        if resp.status().is_success() {
            Ok(())
        } else {
            let err = resp
                .json::<AuthErrorResponse>()
                .await
                .map_err(|e| NexError::new(format!("Error parse error: {e}")))?;
            Err(NexError::new(err.error.message))
        }
    }

    /// Refresh an expired ID token
    ///
    /// # Errors
    /// Returns `NexError` on network failure, parse failure, or Firebase auth error.
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<RefreshResponse, NexError> {
        let url = format!(
            "https://securetoken.googleapis.com/v1/token?key={}",
            self.api_key
        );
        let body = RefreshTokenRequest {
            grant_type: "refresh_token".to_string(),
            refresh_token: refresh_token.to_string(),
        };

        let resp = self
            .http
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| NexError::new(format!("Network error: {e}")))?;

        if resp.status().is_success() {
            resp.json::<RefreshResponse>()
                .await
                .map_err(|e| NexError::new(format!("Parse error: {e}")))
        } else {
            let err = resp
                .json::<AuthErrorResponse>()
                .await
                .map_err(|e| NexError::new(format!("Error parse error: {e}")))?;
            Err(NexError::new(err.error.message))
        }
    }
}
