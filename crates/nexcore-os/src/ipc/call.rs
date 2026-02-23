// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Typed service requests and responses for structured IPC.
//!
//! ## Primitive Grounding
//!
//! - → Causality: Request causally mandates a Response
//! - μ Mapping: Request routing based on method names
//! - N Quantity: Payload data structures

use super::identity::{CallerIdentity, CapabilityToken};
use serde::{Deserialize, Serialize};

/// A structured request sent to a service.
///
/// Tier: T2-C (→ Causality + μ Mapping)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRequest {
    /// The method or operation being requested.
    pub method: String,
    /// JSON payload with arguments for the operation.
    pub payload: serde_json::Value,
}

impl ServiceRequest {
    /// Create a new request.
    pub fn new(method: impl Into<String>, payload: serde_json::Value) -> Self {
        Self {
            method: method.into(),
            payload,
        }
    }
}

/// A structured response from a service.
///
/// Tier: T2-C (→ Causality + ς State)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceResponse {
    /// Whether the operation succeeded.
    pub success: bool,
    /// The resulting payload (data on success, error details on failure).
    pub payload: serde_json::Value,
}

impl ServiceResponse {
    /// Create a success response.
    pub fn success(payload: serde_json::Value) -> Self {
        Self {
            success: true,
            payload,
        }
    }

    /// Create a failure response.
    pub fn error(reason: impl Into<String>) -> Self {
        Self {
            success: false,
            payload: serde_json::json!({ "error": reason.into() }),
        }
    }
}

/// A fully specified IPC call wrapper with identity and capabilities.
///
/// Tier: T3 (→ + μ + ∂ + ς)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceCall {
    /// Unique identifier for this call (used to correlate response).
    pub id: String,
    /// Who is making the request.
    pub caller: CallerIdentity,
    /// Security capability token (optional).
    pub token: Option<CapabilityToken>,
    /// The service intended to handle this call.
    pub target_service: String,
    /// The actual request data.
    pub request: ServiceRequest,
}

impl ServiceCall {
    /// Create a new service call.
    pub fn new(
        caller: CallerIdentity,
        target_service: impl Into<String>,
        request: ServiceRequest,
    ) -> Self {
        Self {
            id: nexcore_id::NexId::v4().to_string(),
            caller,
            token: None,
            target_service: target_service.into(),
            request,
        }
    }

    /// Attach a capability token.
    pub fn with_token(mut self, token: CapabilityToken) -> Self {
        self.token = Some(token);
        self
    }
}
