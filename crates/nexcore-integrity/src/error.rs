//! # Integrity Error Types
//!
//! Error hierarchy for the nexcore-integrity crate.
//!
//! ## Primitive Grounding
//! - ∂ Boundary: error conditions define boundaries of valid input
//! - ∃ Existence: insufficient data checks

/// Tier: T2-C (domain composite)
///
/// Errors that can occur during integrity analysis.
#[derive(Debug, nexcore_error::Error)]
pub enum IntegrityError {
    /// Text has insufficient tokens for reliable analysis.
    #[error("insufficient text: {token_count} tokens (minimum {minimum} required)")]
    InsufficientText {
        /// Actual token count
        token_count: usize,
        /// Minimum required
        minimum: usize,
    },

    /// Invalid Bloom taxonomy level (must be 1-7).
    #[error("invalid Bloom level: {level} (must be 1-7)")]
    InvalidBloomLevel {
        /// Provided level
        level: u8,
    },

    /// Invalid PV domain identifier.
    #[error("invalid domain ID: {id} (expected D02-D12)")]
    InvalidDomainId {
        /// Provided domain ID
        id: String,
    },

    /// Invalid classification threshold.
    #[error("invalid threshold: {value} (must be 0.0-1.0)")]
    InvalidThreshold {
        /// Provided threshold value
        value: f64,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insufficient_text_display() {
        let err = IntegrityError::InsufficientText {
            token_count: 10,
            minimum: 50,
        };
        let msg = format!("{err}");
        assert!(msg.contains("10"));
        assert!(msg.contains("50"));
    }

    #[test]
    fn test_invalid_bloom_level_display() {
        let err = IntegrityError::InvalidBloomLevel { level: 9 };
        let msg = format!("{err}");
        assert!(msg.contains("9"));
    }

    #[test]
    fn test_invalid_domain_display() {
        let err = IntegrityError::InvalidDomainId {
            id: "D99".to_string(),
        };
        let msg = format!("{err}");
        assert!(msg.contains("D99"));
    }

    #[test]
    fn test_invalid_threshold_display() {
        let err = IntegrityError::InvalidThreshold { value: 1.5 };
        let msg = format!("{err}");
        assert!(msg.contains("1.5"));
    }
}
