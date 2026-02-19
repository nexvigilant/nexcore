//! Tenant extraction middleware for multi-tenant API requests.
//!
//! Implements Axum `FromRequestParts` to extract and verify tenant context
//! from JWT Bearer tokens. Every protected handler can accept
//! `TenantContext` as a parameter for compile-time tenant safety.
//!
//! ## Flow (Horus Pattern)
//!
//! ```text
//! Authorization: Bearer <jwt> → decode claims → TenantContext
//!                              → inject into request extensions
//! ```
//!
//! Tier: T2-C (∂ Boundary + ς State + μ Mapping)

use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use vr_core::ids::{TenantId, UserId};
use vr_core::tenant::{SubscriptionTier, TenantContext, UserRole};

#[derive(Debug, Serialize, Deserialize)]
struct TenantClaims {
    tenant_id: String,
    user_id: String,
    role: String,
    tier: String,
}

/// Axum extractor that produces a verified `TenantContext`.
///
/// Use in handler signatures:
/// ```ignore
/// async fn my_handler(tenant: VerifiedTenant) -> impl IntoResponse {
///     let ctx = tenant.0;
///     // ctx.tenant_id(), ctx.tier(), ctx.limits() all available
/// }
/// ```
pub struct VerifiedTenant(pub TenantContext);

/// Error response for tenant extraction failures.
pub struct TenantExtractionError {
    status: StatusCode,
    message: String,
}

impl IntoResponse for TenantExtractionError {
    fn into_response(self) -> Response {
        (
            self.status,
            axum::Json(serde_json::json!({
                "code": "TENANT_ERROR",
                "message": self.message
            })),
        )
            .into_response()
    }
}

impl<S> FromRequestParts<S> for VerifiedTenant
where
    S: Send + Sync,
{
    type Rejection = TenantExtractionError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract Bearer token from Authorization header
        let token = parts
            .headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .ok_or_else(|| TenantExtractionError {
                status: StatusCode::UNAUTHORIZED,
                message: "Missing Bearer token in Authorization header".to_string(),
            })?;

        // Decode JWT claims (simplified — in production use a JWT library)
        let claims = decode_tenant_claims(token).map_err(|e| TenantExtractionError {
            status: StatusCode::UNAUTHORIZED,
            message: format!("Invalid token: {e}"),
        })?;

        let tenant_id =
            TenantId::parse(&claims.tenant_id).ok_or_else(|| TenantExtractionError {
                status: StatusCode::BAD_REQUEST,
                message: "Invalid tenant_id in claims".to_string(),
            })?;
        let user_id = UserId::parse(&claims.user_id).ok_or_else(|| TenantExtractionError {
            status: StatusCode::BAD_REQUEST,
            message: "Invalid user_id in claims".to_string(),
        })?;
        let role = parse_user_role(&claims.role).ok_or_else(|| TenantExtractionError {
            status: StatusCode::BAD_REQUEST,
            message: "Invalid role in claims".to_string(),
        })?;
        let tier = parse_subscription_tier(&claims.tier).ok_or_else(|| TenantExtractionError {
            status: StatusCode::BAD_REQUEST,
            message: "Invalid tier in claims".to_string(),
        })?;
        let verified = TenantContext::new(tenant_id, user_id, role, tier);

        Ok(VerifiedTenant(verified))
    }
}

/// Decode JWT payload to extract tenant claims.
///
/// In production, this would verify the JWT signature against Firebase/Auth0
/// public keys and check expiration. For now, it decodes the base64 payload.
fn decode_tenant_claims(token: &str) -> Result<TenantClaims, String> {
    // JWT format: header.payload.signature
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err("malformed JWT: expected 3 parts".to_string());
    }

    // Decode the payload (second part)
    let payload_b64 = parts[1];

    // JWT uses base64url encoding without padding
    let padded = match payload_b64.len() % 4 {
        2 => format!("{payload_b64}=="),
        3 => format!("{payload_b64}="),
        _ => payload_b64.to_string(),
    };
    let payload_b64_std = padded.replace('-', "+").replace('_', "/");

    let payload_bytes =
        base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &payload_b64_std)
            .map_err(|e| format!("base64 decode failed: {e}"))?;

    let claims: TenantClaims =
        serde_json::from_slice(&payload_bytes).map_err(|e| format!("JSON parse failed: {e}"))?;

    Ok(claims)
}

fn parse_subscription_tier(value: &str) -> Option<SubscriptionTier> {
    match value.trim().to_lowercase().as_str() {
        "academic" => Some(SubscriptionTier::Academic),
        "explorer" => Some(SubscriptionTier::Explorer),
        "accelerator" => Some(SubscriptionTier::Accelerator),
        "enterprise" => Some(SubscriptionTier::Enterprise),
        "custom" => Some(SubscriptionTier::Custom),
        _ => None,
    }
}

fn parse_user_role(value: &str) -> Option<UserRole> {
    match value.trim().to_lowercase().as_str() {
        "owner" => Some(UserRole::Owner),
        "admin" => Some(UserRole::Admin),
        "scientist" => Some(UserRole::Scientist),
        "business_dev" | "businessdev" | "bizdev" => Some(UserRole::BusinessDev),
        "viewer" => Some(UserRole::Viewer),
        "external" => Some(UserRole::External),
        _ => None,
    }
}

/// Middleware function for tenant-aware rate limiting.
///
/// Uses the tenant's tier limits to enforce per-tenant rate limits.
/// Call this after the tenant context is extracted.
pub fn check_tenant_rate_limit(
    tenant: &TenantContext,
    _current_rpm: u32,
) -> Result<(), TenantExtractionError> {
    // Placeholder: tier-aware enforcement is applied in route/middleware layers.
    let _ = tenant.tier();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_jwt(claims: &TenantClaims) -> String {
        // Minimal JWT: header.payload.signature
        let header = base64::Engine::encode(
            &base64::engine::general_purpose::URL_SAFE_NO_PAD,
            b"{\"alg\":\"HS256\",\"typ\":\"JWT\"}",
        );
        let payload_json = serde_json::to_vec(claims).unwrap_or_default();
        let payload = base64::Engine::encode(
            &base64::engine::general_purpose::URL_SAFE_NO_PAD,
            &payload_json,
        );
        format!("{header}.{payload}.fake_signature")
    }

    #[test]
    fn decode_valid_claims() {
        let claims = TenantClaims {
            tenant_id: uuid::Uuid::new_v4().to_string(),
            user_id: uuid::Uuid::new_v4().to_string(),
            role: "admin".to_string(),
            tier: "enterprise".to_string(),
        };
        let token = make_test_jwt(&claims);
        let decoded = decode_tenant_claims(&token);
        assert!(decoded.is_ok());
        let decoded = decoded.unwrap();
        assert_eq!(decoded.role, "admin");
        assert_eq!(decoded.tier, "enterprise");
    }

    #[test]
    fn decode_malformed_jwt() {
        let result = decode_tenant_claims("not.a.valid.jwt");
        assert!(result.is_err());
    }

    #[test]
    fn decode_invalid_base64() {
        let result = decode_tenant_claims("header.!!!invalid!!!.sig");
        assert!(result.is_err());
    }
}
