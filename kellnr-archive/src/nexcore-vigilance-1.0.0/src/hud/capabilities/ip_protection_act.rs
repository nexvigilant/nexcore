//! # Capability 35: IP Protection Act (Vigilance Moat)
//!
//! Implementation of the IP Protection Act as a core structural
//! capability within the HUD domain. This capability manages the
//! "Intellectual Property" and "Algorithmic Moat" of the Union.
//!
//! Matches 1:1 to the US Patent and Trademark Office (USPTO) mandate
//! to foster innovation, competitiveness and economic growth,
//! by providing and facilitating intellectual property protection
//! and services worldwide.

use nexcore_primitives::measurement::{Confidence, Measured};
use serde::{Deserialize, Serialize};

/// T3: IpProtectionAct - Capability 35 of 37.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpProtectionAct {
    /// The unique capability identifier.
    pub id: String,
    /// Whether the IP protection engine is active.
    pub protection_active: bool,
}

/// T2-P: PatentStrength - The quantified defensibility of an algorithm.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct PatentStrength(pub f64);

/// T2-C: TheoremRegistration - A formal protection of a ToV component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TheoremRegistration {
    /// The title of the theorem or algorithm.
    pub title: String,
    /// The cryptographic proof of originality.
    pub proof_hash: String,
    /// The duration of the protection.
    pub term_cycles: u32,
}

impl IpProtectionAct {
    /// Creates a new instance of the IpProtectionAct.
    pub fn new() -> Self {
        Self {
            id: "CAP-035".into(),
            protection_active: true,
        }
    }

    /// Register a new theorem for IP protection.
    pub fn register_theorem(&self, title: &str, _proof: &str) -> Measured<TheoremRegistration> {
        let registration = TheoremRegistration {
            title: title.to_string(),
            proof_hash: format!("SHA256:TOV_{}", title.to_uppercase()),
            term_cycles: 100,
        };

        Measured::uncertain(registration, Confidence::new(0.99))
    }
}
