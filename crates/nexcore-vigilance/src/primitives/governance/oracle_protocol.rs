//! # The Oracle Protocol
//!
//! Implementation of the "External Oracle" requirement from the Constitution.
//! This module defines how the Union requests validation from systems beyond
//! its own self-referential limits.

use nexcore_primitives::measurement::{Confidence, Measured};
use serde::{Deserialize, Serialize};

// ============================================================================
// T1: UNIVERSAL PRIMITIVES (ORACLE)
// ============================================================================

/// T1: RequestID - Unique marker for an oracle query.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RequestID(pub String);

// ============================================================================
// T2-P: QUANTITIES
// ============================================================================

/// T2-P: OracleReputation - The historical reliability of an external validator.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct OracleReputation(pub f64);

// ============================================================================
// T2-C: COMPOSITES
// ============================================================================

/// T2-C: OracleQuery - A request for external validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleQuery {
    pub id: RequestID,
    pub payload: String,
    pub expected_confidence: Confidence,
}

/// T2-C: OracleResponse - The result of an external validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleResponse<T> {
    pub query_id: RequestID,
    pub validation_result: Measured<T>,
    pub source_reputation: OracleReputation,
}

impl<T> OracleResponse<T> {
    /// Combine Oracle confidence with Reputation.
    pub fn definitive_confidence(&self) -> Confidence {
        self.validation_result
            .confidence
            .combine(Confidence::new(self.source_reputation.0))
    }
}

/// T3: OracleIntegrationLayer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleIntegrationLayer {
    pub oracle_id: String,
    pub reputation: OracleReputation,
}

impl OracleIntegrationLayer {
    /// Validate a Union claim using the Oracle.
    pub fn validate_claim<T>(&self, claim: T, id: &str) -> OracleResponse<T> {
        OracleResponse {
            query_id: RequestID(id.into()),
            validation_result: Measured::certain(claim), // Simplified for simulation
            source_reputation: self.reputation,
        }
    }
}
