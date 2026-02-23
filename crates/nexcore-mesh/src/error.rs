//! Mesh error types.
//!
//! ## Tier: T2-P | Dominant: ∂ (Boundary)
//!
//! Errors are boundaries that delimit valid from invalid states.

use nexcore_error::Error;

/// Errors that can occur in mesh networking operations.
///
/// Tier: T2-P | Dominant: ∂ (Boundary) + ∅ (Void)
#[derive(Debug, Error, Clone, PartialEq)]
pub enum MeshError {
    /// Node not found in the mesh
    #[error("node not found: {0}")]
    NodeNotFound(String),

    /// Route not available to destination
    #[error("no route to destination: {0}")]
    NoRoute(String),

    /// Message TTL expired during relay
    #[error("TTL expired after {0} hops")]
    TtlExpired(u8),

    /// Circuit breaker is open for this neighbor
    #[error("circuit breaker open for neighbor: {0}")]
    CircuitBreakerOpen(String),

    /// Neighbor registry is full (capacity limit)
    #[error("neighbor capacity exceeded: max {0}")]
    NeighborCapacityExceeded(usize),

    /// Duplicate node ID in the mesh
    #[error("duplicate node ID: {0}")]
    DuplicateNode(String),

    /// Message too large for relay
    #[error("message payload too large: {size} bytes (max: {max})")]
    PayloadTooLarge {
        /// Actual size
        size: usize,
        /// Maximum allowed
        max: usize,
    },

    /// Channel communication failure
    #[error("channel send failed: {0}")]
    ChannelError(String),

    /// Invalid mesh configuration
    #[error("invalid configuration: {0}")]
    InvalidConfig(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_node_not_found() {
        let err = MeshError::NodeNotFound("node-42".to_string());
        assert!(err.to_string().contains("node-42"));
    }

    #[test]
    fn error_display_ttl_expired() {
        let err = MeshError::TtlExpired(16);
        assert!(err.to_string().contains("16"));
    }

    #[test]
    fn error_display_no_route() {
        let err = MeshError::NoRoute("dest-99".to_string());
        assert!(err.to_string().contains("dest-99"));
    }

    #[test]
    fn error_display_circuit_breaker_open() {
        let err = MeshError::CircuitBreakerOpen("neighbor-1".to_string());
        assert!(err.to_string().contains("neighbor-1"));
    }

    #[test]
    fn error_display_neighbor_capacity() {
        let err = MeshError::NeighborCapacityExceeded(32);
        assert!(err.to_string().contains("32"));
    }

    #[test]
    fn error_display_duplicate_node() {
        let err = MeshError::DuplicateNode("dup-id".to_string());
        assert!(err.to_string().contains("dup-id"));
    }

    #[test]
    fn error_display_payload_too_large() {
        let err = MeshError::PayloadTooLarge {
            size: 2048,
            max: 1024,
        };
        let msg = err.to_string();
        assert!(msg.contains("2048"));
        assert!(msg.contains("1024"));
    }

    #[test]
    fn error_display_channel_error() {
        let err = MeshError::ChannelError("receiver dropped".to_string());
        assert!(err.to_string().contains("receiver dropped"));
    }

    #[test]
    fn error_display_invalid_config() {
        let err = MeshError::InvalidConfig("TTL must be > 0".to_string());
        assert!(err.to_string().contains("TTL must be > 0"));
    }

    #[test]
    fn error_clone_eq() {
        let a = MeshError::TtlExpired(10);
        let b = a.clone();
        assert_eq!(a, b);
    }
}
