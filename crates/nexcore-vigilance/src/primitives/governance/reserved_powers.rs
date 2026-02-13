//! # Reserved Powers (Bill of Rights)
//!
//! Implementation of Amendment X: Powers not delegated to NexVigilant
//! by the Constitution, nor prohibited to the Domains, are reserved
//! to the Domains respectively, or to the Architects.

use serde::{Deserialize, Serialize};

/// T3: ReservedPower — A power retained by a domain or architect
/// because it was not delegated to the central system.
///
/// ## Tier: T3 (Domain-specific governance type)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReservedPower {
    /// Description of the power
    pub power: String,
    /// Who holds this power
    pub held_by: PowerHolder,
    /// Whether NexVigilant has claimed this power
    pub claimed_by_central: bool,
    /// Whether the constitution explicitly delegates this power
    pub constitutionally_delegated: bool,
    /// Whether the constitution prohibits domains from exercising it
    pub prohibited_to_domains: bool,
}

impl ReservedPower {
    /// A power is validly reserved if not delegated centrally
    /// and not prohibited to domains.
    pub fn is_validly_reserved(&self) -> bool {
        !self.constitutionally_delegated && !self.prohibited_to_domains
    }

    /// Check if central has unconstitutionally claimed a reserved power.
    pub fn is_overreach(&self) -> bool {
        self.claimed_by_central && self.is_validly_reserved()
    }

    /// Determine who rightfully holds this power.
    pub fn rightful_holder(&self) -> PowerHolder {
        if self.constitutionally_delegated {
            PowerHolder::NexVigilant
        } else if self.prohibited_to_domains {
            PowerHolder::Architect
        } else {
            self.held_by.clone()
        }
    }
}

/// T3: Who holds a reserved power.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PowerHolder {
    /// A specific domain (crate, module, subsystem)
    Domain(String),
    /// The architect (human designer)
    Architect,
    /// The central system (NexVigilant)
    NexVigilant,
}

/// T3: DomainReservation — A domain's assertion of reserved powers.
///
/// ## Tier: T3
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainReservation {
    /// Domain asserting its powers
    pub domain: String,
    /// Powers the domain claims
    pub claimed_powers: Vec<String>,
    /// Powers that are constitutionally valid
    pub valid_powers: Vec<String>,
}

impl DomainReservation {
    /// Check if all claimed powers are valid.
    pub fn all_claims_valid(&self) -> bool {
        self.claimed_powers.len() == self.valid_powers.len()
    }

    /// Number of invalid claims.
    pub fn invalid_claim_count(&self) -> usize {
        self.claimed_powers
            .len()
            .saturating_sub(self.valid_powers.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validly_reserved_to_domain() {
        let power = ReservedPower {
            power: "Internal signal threshold tuning".to_string(),
            held_by: PowerHolder::Domain("nexcore-guardian-engine".to_string()),
            claimed_by_central: false,
            constitutionally_delegated: false,
            prohibited_to_domains: false,
        };
        assert!(power.is_validly_reserved());
        assert!(!power.is_overreach());
        assert_eq!(
            power.rightful_holder(),
            PowerHolder::Domain("nexcore-guardian-engine".to_string())
        );
    }

    #[test]
    fn central_overreach_detected() {
        let power = ReservedPower {
            power: "Domain-specific naming convention".to_string(),
            held_by: PowerHolder::Domain("nexcore-pvos".to_string()),
            claimed_by_central: true,
            constitutionally_delegated: false,
            prohibited_to_domains: false,
        };
        assert!(power.is_overreach());
    }

    #[test]
    fn constitutionally_delegated_not_reserved() {
        let power = ReservedPower {
            power: "Patient safety enforcement".to_string(),
            held_by: PowerHolder::NexVigilant,
            claimed_by_central: true,
            constitutionally_delegated: true,
            prohibited_to_domains: false,
        };
        assert!(!power.is_validly_reserved());
        assert!(!power.is_overreach());
        assert_eq!(power.rightful_holder(), PowerHolder::NexVigilant);
    }

    #[test]
    fn prohibited_to_domains_goes_to_architect() {
        let power = ReservedPower {
            power: "Modify the Constitution".to_string(),
            held_by: PowerHolder::Architect,
            claimed_by_central: false,
            constitutionally_delegated: false,
            prohibited_to_domains: true,
        };
        assert!(!power.is_validly_reserved());
        assert_eq!(power.rightful_holder(), PowerHolder::Architect);
    }

    #[test]
    fn domain_reservation_all_valid() {
        let reservation = DomainReservation {
            domain: "nexcore-brain".to_string(),
            claimed_powers: vec![
                "session management".to_string(),
                "artifact storage".to_string(),
            ],
            valid_powers: vec![
                "session management".to_string(),
                "artifact storage".to_string(),
            ],
        };
        assert!(reservation.all_claims_valid());
        assert_eq!(reservation.invalid_claim_count(), 0);
    }

    #[test]
    fn domain_reservation_some_invalid() {
        let reservation = DomainReservation {
            domain: "rogue-crate".to_string(),
            claimed_powers: vec![
                "local config".to_string(),
                "override safety".to_string(),
                "bypass validation".to_string(),
            ],
            valid_powers: vec!["local config".to_string()],
        };
        assert!(!reservation.all_claims_valid());
        assert_eq!(reservation.invalid_claim_count(), 2);
    }
}
