// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Caller identity and capability tokens for IPC authentication.
//!
//! ## Primitive Grounding
//!
//! - ∂ Boundary: Capability tokens define permission boundaries
//! - ς State: Identity of the caller

use serde::{Deserialize, Serialize};

/// Cryptographic or opaque token proving a capability.
///
/// Tier: T2-P (∂ Boundary)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CapabilityToken(String);

impl CapabilityToken {
    /// Create a new capability token.
    pub fn new(token: impl Into<String>) -> Self {
        Self(token.into())
    }

    /// Get the token string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Identity of an IPC message sender.
///
/// Tier: T2-P (ς State)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CallerIdentity {
    /// The OS kernel itself.
    System,
    /// A system service.
    Service(String),
    /// An authenticated user.
    User(String),
    /// Unauthenticated / Unknown.
    Anonymous,
}

impl std::fmt::Display for CallerIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::System => write!(f, "system"),
            Self::Service(name) => write!(f, "service:{}", name),
            Self::User(name) => write!(f, "user:{}", name),
            Self::Anonymous => write!(f, "anonymous"),
        }
    }
}
