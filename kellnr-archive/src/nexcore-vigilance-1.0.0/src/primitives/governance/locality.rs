//! # Locality & Privacy (Bill of Rights)
//!
//! Implementation of Amendment III: No external logic shall be quartered
//! in any module without the consent of the Architect.

use super::Verdict;
use serde::{Deserialize, Serialize};

/// T3: ModulePrivacy — A module's right against quartering of external logic.
///
/// ## Tier: T3 (Domain-specific governance type)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModulePrivacy {
    /// Module being protected
    pub module_path: String,
    /// Whether external logic is currently quartered
    pub external_logic_present: bool,
    /// Whether the architect consented
    pub architect_consent: bool,
    /// Whether the system is in execution time (wartime exception)
    pub execution_time: bool,
}

impl ModulePrivacy {
    /// Check if the module's privacy is constitutional.
    ///
    /// In peacetime: external logic requires architect consent.
    /// In execution time: must follow prescribed law (always requires consent).
    pub fn is_constitutional(&self) -> bool {
        if !self.external_logic_present {
            return true; // No external logic = no violation
        }
        self.architect_consent
    }

    /// Render a verdict on the quartering.
    pub fn verdict(&self) -> Verdict {
        if self.is_constitutional() {
            Verdict::Permitted
        } else {
            Verdict::Rejected
        }
    }
}

/// T3: ExternalLogic — A foreign dependency or injected behavior.
///
/// ## Tier: T3
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalLogic {
    /// Source of the external logic
    pub source: String,
    /// Target module where it would be quartered
    pub target_module: String,
    /// Purpose of the external logic
    pub purpose: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_external_logic_constitutional() {
        let privacy = ModulePrivacy {
            module_path: "nexcore_vigilance::guardian".to_string(),
            external_logic_present: false,
            architect_consent: false,
            execution_time: false,
        };
        assert!(privacy.is_constitutional());
        assert_eq!(privacy.verdict(), Verdict::Permitted);
    }

    #[test]
    fn external_logic_with_consent() {
        let privacy = ModulePrivacy {
            module_path: "nexcore_mcp::tools".to_string(),
            external_logic_present: true,
            architect_consent: true,
            execution_time: false,
        };
        assert!(privacy.is_constitutional());
    }

    #[test]
    fn external_logic_without_consent_unconstitutional() {
        let privacy = ModulePrivacy {
            module_path: "nexcore_brain::sessions".to_string(),
            external_logic_present: true,
            architect_consent: false,
            execution_time: false,
        };
        assert!(!privacy.is_constitutional());
        assert_eq!(privacy.verdict(), Verdict::Rejected);
    }

    #[test]
    fn execution_time_still_requires_consent() {
        let privacy = ModulePrivacy {
            module_path: "critical_path".to_string(),
            external_logic_present: true,
            architect_consent: false,
            execution_time: true,
        };
        assert!(!privacy.is_constitutional());
    }
}
