//! Stripe Billing Routes
//!
//! Three endpoints for the Guardian MVP paid API:
//! - `POST /api/v1/billing/checkout`  — Create Stripe Checkout session
//! - `POST /api/v1/billing/webhook`   — Handle Stripe webhook events
//! - `GET  /api/v1/billing/portal`    — Generate customer portal URL
//!
//! Uses raw `reqwest` to the Stripe v1 REST API (form-encoded).
//! Webhook signature verification: HMAC-SHA256 per Stripe docs.
//!
//! ## Required environment variables
//! - `STRIPE_SECRET_KEY`         — Stripe secret key (sk_live_* / sk_test_*)
//! - `STRIPE_WEBHOOK_SECRET`     — Webhook endpoint signing secret (whsec_*)
//! - `STRIPE_PRICE_RESEARCHER`   — Price ID for researcher plan ($49/mo)
//! - `STRIPE_PRICE_PROFESSIONAL` — Price ID for professional plan ($199/mo)
//! - `STRIPE_PRICE_ENTERPRISE`   — Price ID for enterprise plan ($999/mo)
//!
//! Tier: T2-S (Service boundary — λ + μ + ∂)

use axum::{
    Json, Router,
    body::Bytes,
    extract::{Query, Request},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::collections::HashMap;
use utoipa::ToSchema;

use super::common::ApiResult;

// ============================================================================
// Plan enum + price ID resolution
// ============================================================================

/// Subscription plan tiers for the Guardian MVP API.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Plan {
    /// $49/month — individual researchers and academics
    Researcher,
    /// $199/month — professional PV teams
    Professional,
    /// $999/month — enterprise / CRO deployments
    Enterprise,
}

impl Plan {
    /// Resolve the Stripe Price ID from the environment.
    fn price_id(&self) -> anyhow::Result<String> {
        let var_name = match self {
            Plan::Researcher => "STRIPE_PRICE_RESEARCHER",
            Plan::Professional => "STRIPE_PRICE_PROFESSIONAL",
            Plan::Enterprise => "STRIPE_PRICE_ENTERPRISE",
        };
        std::env::var(var_name).map_err(|_| {
            anyhow::anyhow!("Missing env var: {var_name}")
        })
    }
}

// ============================================================================
// Request / Response types
// ============================================================================

/// Request body for creating a Stripe Checkout session.
#[derive(Debug, Deserialize, ToSchema)]
pub struct CheckoutRequest {
    /// The subscription plan to purchase.
    pub plan: Plan,
    /// Your internal user ID — stored as Stripe client_reference_id.
    pub user_id: String,
    /// URL Stripe redirects to on successful payment.
    pub success_url: String,
    /// URL Stripe redirects to if the user cancels.
    pub cancel_url: String,
    /// Existing Stripe customer ID (optional — omit for new customers).
    #[serde(default)]
    pub customer_id: Option<String>,
}

/// Successful Checkout session creation response.
#[derive(Debug, Serialize, ToSchema)]
pub struct CheckoutResponse {
    /// Stripe Checkout session ID (cs_...).
    pub session_id: String,
    /// Redirect URL — send the user here to complete payment.
    pub checkout_url: String,
}

/// Query parameters for the customer portal endpoint.
#[derive(Debug, Deserialize, ToSchema)]
pub struct PortalQuery {
    /// Stripe customer ID (cus_...).
    pub customer_id: String,
    /// URL to return the customer to after they leave the portal.
    #[serde(default = "default_return_url")]
    pub return_url: String,
}

fn default_return_url() -> String {
    std::env::var("APP_BASE_URL")
        .unwrap_or_else(|_| "https://nexvigilant.com/dashboard".into())
}

/// Customer portal session response.
#[derive(Debug, Serialize, ToSchema)]
pub struct PortalResponse {
    /// Stripe billing portal URL — redirect the customer here.
    pub portal_url: String,
}

/// Webhook acknowledgement response.
#[derive(Debug, Serialize, ToSchema)]
pub struct WebhookAck {
    pub received: bool,
}

// ============================================================================
// Stripe API client helpers
// ============================================================================

/// Return the configured Stripe secret key (never log it).
fn stripe_secret_key() -> anyhow::Result<String> {
    std::env::var("STRIPE_SECRET_KEY")
        .map_err(|_| anyhow::anyhow!("STRIPE_SECRET_KEY not configured"))
}

/// POST form-encoded data to the Stripe v1 API.
async fn stripe_post(
    path: &str,
    params: &[(&str, &str)],
) -> anyhow::Result<serde_json::Value> {
    let key = stripe_secret_key()?;
    let client = reqwest::Client::new();
    let url = format!("https://api.stripe.com/v1/{path}");

    let resp = client
        .post(&url)
        .basic_auth(&key, Some(""))
        .form(params)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Stripe HTTP error: {e}"))?;

    let status = resp.status();
    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| anyhow::anyhow!("Stripe response parse error: {e}"))?;

    if !status.is_success() {
        let msg = body
            .pointer("/error/message")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown Stripe error");
        return Err(anyhow::anyhow!("Stripe API {status}: {msg}"));
    }

    Ok(body)
}

// ============================================================================
// Handlers
// ============================================================================

/// Create a Stripe Checkout session for plan selection.
///
/// Redirects the user to Stripe's hosted checkout page.
/// On success, Stripe fires `checkout.session.completed` webhook.
#[utoipa::path(
    post,
    path = "/api/v1/billing/checkout",
    tag = "billing",
    request_body = CheckoutRequest,
    responses(
        (status = 200, description = "Checkout session created", body = CheckoutResponse),
        (status = 400, description = "Invalid plan or missing price ID"),
        (status = 500, description = "Stripe API error"),
    )
)]
pub async fn create_checkout(
    Json(req): Json<CheckoutRequest>,
) -> ApiResult<Json<CheckoutResponse>> {
    let price_id = req.plan.price_id().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "code": "MISSING_PRICE_ID", "message": e.to_string() })),
        )
            .into_response()
    })?;

    let mut params: Vec<(&str, String)> = vec![
        ("mode", "subscription".into()),
        ("line_items[0][price]", price_id),
        ("line_items[0][quantity]", "1".into()),
        ("success_url", req.success_url.clone()),
        ("cancel_url", req.cancel_url.clone()),
        ("client_reference_id", req.user_id.clone()),
    ];

    if let Some(ref cid) = req.customer_id {
        params.push(("customer", cid.clone()));
    }

    // Convert to &str slices for the helper
    let param_refs: Vec<(&str, &str)> = params
        .iter()
        .map(|(k, v)| (*k, v.as_str()))
        .collect();

    let body = stripe_post("checkout/sessions", &param_refs)
        .await
        .map_err(|e| {
            tracing::error!("Stripe checkout error: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "code": "STRIPE_ERROR",
                    "message": "Failed to create checkout session"
                })),
            )
                .into_response()
        })?;

    let session_id = body["id"]
        .as_str()
        .ok_or_else(|| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "code": "STRIPE_PARSE_ERROR",
                    "message": "Missing session id in Stripe response"
                })),
            )
                .into_response()
        })?
        .to_string();

    let checkout_url = body["url"]
        .as_str()
        .ok_or_else(|| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "code": "STRIPE_PARSE_ERROR",
                    "message": "Missing url in Stripe response"
                })),
            )
                .into_response()
        })?
        .to_string();

    Ok(Json(CheckoutResponse {
        session_id,
        checkout_url,
    }))
}

/// Handle incoming Stripe webhook events.
///
/// Verifies the `Stripe-Signature` header using HMAC-SHA256 before processing.
/// Events handled:
/// - `checkout.session.completed`       → activate subscription
/// - `customer.subscription.updated`   → update plan tier
/// - `customer.subscription.deleted`   → deactivate account
/// - `invoice.payment_failed`          → flag account, notify
#[utoipa::path(
    post,
    path = "/api/v1/billing/webhook",
    tag = "billing",
    responses(
        (status = 200, description = "Event received", body = WebhookAck),
        (status = 400, description = "Invalid signature or payload"),
    )
)]
pub async fn handle_webhook(headers: HeaderMap, body: Bytes) -> Response {
    // Verify Stripe-Signature header
    let sig_header = match headers
        .get("stripe-signature")
        .and_then(|v| v.to_str().ok())
    {
        Some(s) => s.to_string(),
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "code": "MISSING_SIGNATURE",
                    "message": "stripe-signature header required"
                })),
            )
                .into_response();
        }
    };

    let webhook_secret = match std::env::var("STRIPE_WEBHOOK_SECRET") {
        Ok(s) => s,
        Err(_) => {
            tracing::error!("STRIPE_WEBHOOK_SECRET not configured");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    if let Err(e) = verify_stripe_signature(&sig_header, &body, &webhook_secret) {
        tracing::warn!("Stripe webhook signature verification failed: {e}");
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "code": "INVALID_SIGNATURE",
                "message": "Webhook signature verification failed"
            })),
        )
            .into_response();
    }

    // Parse the event
    let event: serde_json::Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "code": "PARSE_ERROR",
                    "message": e.to_string()
                })),
            )
                .into_response();
        }
    };

    let event_type = event["type"].as_str().unwrap_or("unknown");
    let event_id = event["id"].as_str().unwrap_or("unknown");

    tracing::info!(event_type, event_id, "Stripe webhook received");

    match event_type {
        "checkout.session.completed" => {
            handle_checkout_completed(&event);
        }
        "customer.subscription.updated" => {
            handle_subscription_updated(&event);
        }
        "customer.subscription.deleted" => {
            handle_subscription_deleted(&event);
        }
        "invoice.payment_failed" => {
            handle_payment_failed(&event);
        }
        other => {
            tracing::debug!(event_type = other, "Unhandled Stripe event type");
        }
    }

    (StatusCode::OK, Json(WebhookAck { received: true })).into_response()
}

/// Generate a Stripe customer portal session URL.
///
/// Returns a short-lived URL the customer uses to manage their subscription,
/// update payment methods, and view billing history.
#[utoipa::path(
    get,
    path = "/api/v1/billing/portal",
    tag = "billing",
    params(
        ("customer_id" = String, Query, description = "Stripe customer ID (cus_...)"),
        ("return_url" = Option<String>, Query, description = "Return URL after leaving portal"),
    ),
    responses(
        (status = 200, description = "Portal URL generated", body = PortalResponse),
        (status = 500, description = "Stripe API error"),
    )
)]
pub async fn customer_portal(
    Query(q): Query<PortalQuery>,
) -> ApiResult<Json<PortalResponse>> {
    let params = [
        ("customer", q.customer_id.as_str()),
        ("return_url", q.return_url.as_str()),
    ];

    let body = stripe_post("billing_portal/sessions", &params)
        .await
        .map_err(|e| {
            tracing::error!("Stripe portal error: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "code": "STRIPE_ERROR",
                    "message": "Failed to create portal session"
                })),
            )
                .into_response()
        })?;

    let portal_url = body["url"]
        .as_str()
        .ok_or_else(|| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "code": "STRIPE_PARSE_ERROR",
                    "message": "Missing url in Stripe portal response"
                })),
            )
                .into_response()
        })?
        .to_string();

    Ok(Json(PortalResponse { portal_url }))
}

// ============================================================================
// Stripe webhook verification
// ============================================================================

/// Verify Stripe webhook signature using HMAC-SHA256.
///
/// Per Stripe docs: the signed payload is `{timestamp}.{body}`.
/// The `Stripe-Signature` header format: `t=<ts>,v1=<sig1>,v1=<sig2>,...`
fn verify_stripe_signature(
    sig_header: &str,
    payload: &[u8],
    secret: &str,
) -> anyhow::Result<()> {
    let parts: HashMap<&str, Vec<&str>> = sig_header
        .split(',')
        .filter_map(|item| {
            let mut kv = item.splitn(2, '=');
            let k = kv.next()?;
            let v = kv.next()?;
            Some((k, v))
        })
        .fold(HashMap::new(), |mut acc, (k, v)| {
            acc.entry(k).or_default().push(v);
            acc
        });

    let timestamp = parts
        .get("t")
        .and_then(|v| v.first())
        .ok_or_else(|| anyhow::anyhow!("Missing timestamp in Stripe-Signature"))?;

    let signatures: Vec<&str> = parts
        .get("v1")
        .cloned()
        .unwrap_or_default();

    if signatures.is_empty() {
        return Err(anyhow::anyhow!("No v1 signatures in Stripe-Signature"));
    }

    // signed_payload = timestamp + "." + raw_body
    let mut signed_payload = timestamp.as_bytes().to_vec();
    signed_payload.push(b'.');
    signed_payload.extend_from_slice(payload);

    let mut mac =
        Hmac::<Sha256>::new_from_slice(secret.as_bytes())
            .map_err(|e| anyhow::anyhow!("HMAC init error: {e}"))?;
    mac.update(&signed_payload);
    let computed = hex::encode(mac.finalize().into_bytes());

    // Constant-time check against any provided v1 signature
    let matched = signatures.iter().any(|sig| {
        if sig.len() != computed.len() {
            return false;
        }
        let mut diff = 0u8;
        for (a, b) in sig.bytes().zip(computed.bytes()) {
            diff |= a ^ b;
        }
        diff == 0
    });

    if matched {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Stripe signature mismatch"))
    }
}

// ============================================================================
// Event handlers (integration stubs — wire to tenant store in Stream 1)
// ============================================================================

fn handle_checkout_completed(event: &serde_json::Value) {
    let session = &event["data"]["object"];
    let customer_id = session["customer"].as_str().unwrap_or("unknown");
    let user_id = session["client_reference_id"].as_str().unwrap_or("unknown");
    let subscription_id = session["subscription"].as_str().unwrap_or("unknown");
    tracing::info!(
        customer_id,
        user_id,
        subscription_id,
        "Stripe checkout.session.completed — activate subscription"
    );
    // TODO (Stream 1): call tenant store to activate subscription + set plan tier
}

fn handle_subscription_updated(event: &serde_json::Value) {
    let sub = &event["data"]["object"];
    let customer_id = sub["customer"].as_str().unwrap_or("unknown");
    let status = sub["status"].as_str().unwrap_or("unknown");
    tracing::info!(
        customer_id,
        status,
        "Stripe customer.subscription.updated — update plan tier"
    );
    // TODO (Stream 1): update tenant plan tier in store
}

fn handle_subscription_deleted(event: &serde_json::Value) {
    let sub = &event["data"]["object"];
    let customer_id = sub["customer"].as_str().unwrap_or("unknown");
    tracing::info!(
        customer_id,
        "Stripe customer.subscription.deleted — deactivate account"
    );
    // TODO (Stream 1): deactivate tenant account in store
}

fn handle_payment_failed(event: &serde_json::Value) {
    let invoice = &event["data"]["object"];
    let customer_id = invoice["customer"].as_str().unwrap_or("unknown");
    let amount_due = invoice["amount_due"].as_i64().unwrap_or(0);
    tracing::warn!(
        customer_id,
        amount_due,
        "Stripe invoice.payment_failed — flag account for dunning"
    );
    // TODO (Stream 1): flag tenant account, trigger notification
}

// ============================================================================
// Router
// ============================================================================

pub fn router() -> Router<crate::ApiState> {
    Router::new()
        .route("/checkout", post(create_checkout))
        .route("/webhook", post(handle_webhook))
        .route("/portal", get(customer_portal))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_stripe_signature_valid() {
        // Construct a real HMAC so verification passes
        let secret = "whsec_test_secret";
        let timestamp = "1614556800";
        let body = b"{\"type\":\"checkout.session.completed\"}";

        let mut signed_payload = timestamp.as_bytes().to_vec();
        signed_payload.push(b'.');
        signed_payload.extend_from_slice(body);

        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())
            .expect("HMAC init should not fail in test");
        mac.update(&signed_payload);
        let sig = hex::encode(mac.finalize().into_bytes());

        let header = format!("t={timestamp},v1={sig}");
        assert!(verify_stripe_signature(&header, body, secret).is_ok());
    }

    #[test]
    fn test_verify_stripe_signature_invalid() {
        let result = verify_stripe_signature(
            "t=1614556800,v1=deadbeefdeadbeef",
            b"payload",
            "wrong_secret",
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_stripe_signature_missing_timestamp() {
        let result = verify_stripe_signature("v1=abc123", b"payload", "secret");
        assert!(result.is_err());
    }

    #[test]
    fn test_plan_serialization() {
        let json = serde_json::to_string(&Plan::Professional)
            .expect("Serialization should not fail in test");
        assert_eq!(json, r#""professional""#);
    }
}
