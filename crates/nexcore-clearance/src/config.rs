//! # Clearance Configuration
//!
//! Runtime configuration mapping levels to policies.
//!
//! ## Primitive Grounding
//! - **Tier**: T2-C
//! - **Dominant**: ς State (ς + μ + N + ∂)

use crate::access::AccessMode;
use crate::level::ClassificationLevel;
use crate::policy::ClearancePolicy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Runtime clearance configuration.
///
/// Maps each classification level to its enforcement policy.
///
/// ## Tier: T2-C
/// ## Dominant: ς State
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClearanceConfig {
    /// JSON schema version.
    pub version: u32,
    /// Default classification for untagged assets.
    pub default_level: ClassificationLevel,
    /// Default access mode when no policy is found.
    pub default_mode: AccessMode,
    /// Per-level policies.
    pub level_policies: HashMap<ClassificationLevel, ClearancePolicy>,
    /// Optional mode override (forces a specific mode regardless of level).
    pub mode_override: Option<AccessMode>,
}

impl ClearanceConfig {
    /// Create a config with default policies for all levels.
    #[must_use]
    pub fn with_defaults() -> Self {
        let mut policies = HashMap::new();
        policies.insert(
            ClassificationLevel::Public,
            ClearancePolicy::default_for(ClassificationLevel::Public),
        );
        policies.insert(
            ClassificationLevel::Internal,
            ClearancePolicy::default_for(ClassificationLevel::Internal),
        );
        policies.insert(
            ClassificationLevel::Confidential,
            ClearancePolicy::default_for(ClassificationLevel::Confidential),
        );
        policies.insert(
            ClassificationLevel::Secret,
            ClearancePolicy::default_for(ClassificationLevel::Secret),
        );
        policies.insert(
            ClassificationLevel::TopSecret,
            ClearancePolicy::default_for(ClassificationLevel::TopSecret),
        );

        Self {
            version: 1,
            default_level: ClassificationLevel::Internal,
            default_mode: AccessMode::Guarded,
            level_policies: policies,
            mode_override: None,
        }
    }

    /// Look up the policy for a given level.
    #[must_use]
    pub fn policy_for(&self, level: ClassificationLevel) -> ClearancePolicy {
        self.level_policies
            .get(&level)
            .cloned()
            .unwrap_or_else(|| ClearancePolicy::default_for(level))
    }

    /// Effective access mode for a given level, considering override.
    #[must_use]
    pub fn effective_mode(&self, level: ClassificationLevel) -> AccessMode {
        if let Some(override_mode) = self.mode_override {
            return override_mode;
        }
        self.policy_for(level).access_mode
    }

    /// Set a mode override that applies to all levels.
    pub fn override_mode(&mut self, mode: AccessMode) {
        self.mode_override = Some(mode);
    }

    /// Clear the mode override.
    pub fn clear_override(&mut self) {
        self.mode_override = None;
    }
}

impl Default for ClearanceConfig {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_all_levels() {
        let config = ClearanceConfig::with_defaults();
        assert!(
            config
                .level_policies
                .contains_key(&ClassificationLevel::Public)
        );
        assert!(
            config
                .level_policies
                .contains_key(&ClassificationLevel::Internal)
        );
        assert!(
            config
                .level_policies
                .contains_key(&ClassificationLevel::Confidential)
        );
        assert!(
            config
                .level_policies
                .contains_key(&ClassificationLevel::Secret)
        );
        assert!(
            config
                .level_policies
                .contains_key(&ClassificationLevel::TopSecret)
        );
    }

    #[test]
    fn policy_for_returns_correct() {
        let config = ClearanceConfig::with_defaults();
        let policy = config.policy_for(ClassificationLevel::Secret);
        assert_eq!(policy.access_mode, AccessMode::Enforced);
    }

    #[test]
    fn policy_for_missing_returns_default() {
        let config = ClearanceConfig {
            version: 1,
            default_level: ClassificationLevel::Public,
            default_mode: AccessMode::Unrestricted,
            level_policies: HashMap::new(),
            mode_override: None,
        };
        let policy = config.policy_for(ClassificationLevel::Secret);
        assert_eq!(policy.access_mode, AccessMode::Enforced);
    }

    #[test]
    fn effective_mode_normal() {
        let config = ClearanceConfig::with_defaults();
        assert_eq!(
            config.effective_mode(ClassificationLevel::Public),
            AccessMode::Unrestricted,
        );
        assert_eq!(
            config.effective_mode(ClassificationLevel::TopSecret),
            AccessMode::Lockdown,
        );
    }

    #[test]
    fn effective_mode_with_override() {
        let mut config = ClearanceConfig::with_defaults();
        config.override_mode(AccessMode::Lockdown);
        assert_eq!(
            config.effective_mode(ClassificationLevel::Public),
            AccessMode::Lockdown,
        );
    }

    #[test]
    fn clear_override() {
        let mut config = ClearanceConfig::with_defaults();
        config.override_mode(AccessMode::Lockdown);
        config.clear_override();
        assert_eq!(
            config.effective_mode(ClassificationLevel::Public),
            AccessMode::Unrestricted,
        );
    }

    #[test]
    fn default_level_is_internal() {
        let config = ClearanceConfig::with_defaults();
        assert_eq!(config.default_level, ClassificationLevel::Internal);
    }

    #[test]
    fn default_mode_is_guarded() {
        let config = ClearanceConfig::with_defaults();
        assert_eq!(config.default_mode, AccessMode::Guarded);
    }

    #[test]
    fn serde_roundtrip() {
        let config = ClearanceConfig::with_defaults();
        let json = serde_json::to_string(&config).unwrap_or_default();
        let parsed: Result<ClearanceConfig, _> = serde_json::from_str(&json);
        assert!(parsed.is_ok());
    }

    #[test]
    fn version_is_one() {
        let config = ClearanceConfig::with_defaults();
        assert_eq!(config.version, 1);
    }
}
