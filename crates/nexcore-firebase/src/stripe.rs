//! Stripe REST API client
//!
//! Provides checkout session creation and retrieval
//! via the Stripe REST API.

use nexcore_error::NexError;
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

/// Stripe API client (requires `client` feature — uses reqwest)
#[cfg(feature = "client")]
pub struct StripeClient {
    secret_key: String,
    http: reqwest::Client,
}

#[cfg(feature = "client")]
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
    ///
    /// # Errors
    /// Returns `NexError` on network failure, parse failure, or Stripe API error.
    pub async fn create_checkout_session(
        &self,
        price_id: &str,
        success_url: &str,
        cancel_url: &str,
    ) -> Result<CheckoutSession, NexError> {
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
            .map_err(|e| NexError::new(format!("Stripe network error: {e}")))?;

        if resp.status().is_success() {
            resp.json::<CheckoutSession>()
                .await
                .map_err(|e| NexError::new(format!("Stripe parse error: {e}")))
        } else {
            let err = resp
                .json::<StripeError>()
                .await
                .map_err(|e| NexError::new(format!("Stripe error parse error: {e}")))?;
            Err(NexError::new(err.error.message))
        }
    }

    /// Retrieve a Checkout Session by ID (for verification)
    ///
    /// # Errors
    /// Returns `NexError` on network failure, parse failure, or Stripe API error.
    pub async fn retrieve_session(&self, session_id: &str) -> Result<SessionStatus, NexError> {
        let url = format!("https://api.stripe.com/v1/checkout/sessions/{session_id}");

        let resp = self
            .http
            .get(&url)
            .basic_auth(&self.secret_key, Option::<&str>::None)
            .send()
            .await
            .map_err(|e| NexError::new(format!("Stripe network error: {e}")))?;

        if resp.status().is_success() {
            resp.json::<SessionStatus>()
                .await
                .map_err(|e| NexError::new(format!("Stripe parse error: {e}")))
        } else {
            let err = resp
                .json::<StripeError>()
                .await
                .map_err(|e| NexError::new(format!("Stripe error parse error: {e}")))?;
            Err(NexError::new(err.error.message))
        }
    }
}
