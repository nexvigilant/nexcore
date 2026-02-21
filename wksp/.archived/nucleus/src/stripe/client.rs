//! Stripe REST API client
//!
//! Mirrors the firebase/auth.rs pattern — struct with reqwest::Client,
//! gated behind `#[cfg(feature = "ssr")]`.

use serde::Deserialize;

/// Stripe Checkout Session (subset of fields we need)
#[derive(Debug, Deserialize)]
pub struct CheckoutSession {
    pub id: String,
    pub url: Option<String>,
}

/// Stripe session status for verification
#[derive(Debug, Deserialize)]
pub struct SessionStatus {
    pub id: String,
    pub status: Option<String>,
    pub payment_status: Option<String>,
    pub customer_email: Option<String>,
}

/// Stripe API error response
#[derive(Debug, Deserialize)]
pub struct StripeError {
    pub error: StripeErrorDetail,
}

#[derive(Debug, Deserialize)]
pub struct StripeErrorDetail {
    #[serde(rename = "type")]
    pub error_type: String,
    pub message: String,
}

/// Stripe API client (SSR only — uses reqwest)
#[cfg(feature = "ssr")]
pub struct StripeClient {
    secret_key: String,
    http: reqwest::Client,
}

#[cfg(feature = "ssr")]
impl StripeClient {
    /// Create a new Stripe client with the given secret key
    pub fn new(secret_key: String) -> Self {
        Self {
            secret_key,
            http: reqwest::Client::new(),
        }
    }

    /// Create a Checkout Session for a subscription
    ///
    /// Uses form-encoded POST (Stripe requires `application/x-www-form-urlencoded`).
    pub async fn create_checkout_session(
        &self,
        price_id: &str,
        success_url: &str,
        cancel_url: &str,
    ) -> Result<CheckoutSession, String> {
        let params = [
            ("mode", "subscription"),
            ("success_url", success_url),
            ("cancel_url", cancel_url),
            ("line_items[0][price]", price_id),
            ("line_items[0][quantity]", "1"),
        ];

        let resp = self
            .http
            .post("https://api.stripe.com/v1/checkout/sessions")
            .basic_auth(&self.secret_key, Option::<&str>::None)
            .form(&params)
            .send()
            .await
            .map_err(|e| format!("Stripe network error: {e}"))?;

        if resp.status().is_success() {
            resp.json::<CheckoutSession>()
                .await
                .map_err(|e| format!("Stripe parse error: {e}"))
        } else {
            let err = resp
                .json::<StripeError>()
                .await
                .map_err(|e| format!("Stripe error parse error: {e}"))?;
            Err(err.error.message)
        }
    }

    /// Retrieve a Checkout Session by ID (for verification)
    pub async fn retrieve_session(&self, session_id: &str) -> Result<SessionStatus, String> {
        let url = format!("https://api.stripe.com/v1/checkout/sessions/{session_id}");

        let resp = self
            .http
            .get(&url)
            .basic_auth(&self.secret_key, Option::<&str>::None)
            .send()
            .await
            .map_err(|e| format!("Stripe network error: {e}"))?;

        if resp.status().is_success() {
            resp.json::<SessionStatus>()
                .await
                .map_err(|e| format!("Stripe parse error: {e}"))
        } else {
            let err = resp
                .json::<StripeError>()
                .await
                .map_err(|e| format!("Stripe error parse error: {e}"))?;
            Err(err.error.message)
        }
    }
}
