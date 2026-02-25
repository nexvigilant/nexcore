// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Antibody registry loader.
//!
//! ## Primitive Grounding: π (Persistence) + μ (Mapping)
//!
//! Loads antibodies from YAML files, transforming persistent storage
//! into runtime-ready patterns.

use crate::error::{ImmunityError, ImmunityResult};
use crate::types::AntibodyRegistry;
use std::path::Path;

/// Default path to the antibody registry.
pub const DEFAULT_REGISTRY_PATH: &str = "~/.claude/immunity/antibodies.yaml";

/// Expand tilde in path.
fn expand_tilde(path: &str) -> String {
    if path.starts_with("~/")
        && let Some(home) = std::env::var_os("HOME")
    {
        return path.replacen('~', home.to_string_lossy().as_ref(), 1);
    }
    path.to_string()
}

/// Load antibody registry from YAML file.
///
/// ## Tier: T2-P (π + μ)
///
/// # Errors
///
/// Returns error if file cannot be read or parsed.
pub fn load_registry<P: AsRef<Path>>(path: P) -> ImmunityResult<AntibodyRegistry> {
    let path_str = path.as_ref().to_string_lossy();
    let expanded = expand_tilde(&path_str);
    let expanded_path = Path::new(&expanded);

    let content = std::fs::read_to_string(expanded_path)
        .map_err(|e| ImmunityError::LoadFailed(format!("{}: {}", expanded_path.display(), e)))?;

    let registry: AntibodyRegistry = serde_yml::from_str(&content)?;

    tracing::info!(
        "Loaded {} antibodies from {}",
        registry.len(),
        expanded_path.display()
    );

    Ok(registry)
}

/// Load registry from default path.
///
/// # Errors
///
/// Returns error if default registry cannot be loaded.
pub fn load_default_registry() -> ImmunityResult<AntibodyRegistry> {
    load_registry(DEFAULT_REGISTRY_PATH)
}

/// Load registry from string content.
///
/// # Errors
///
/// Returns error if content cannot be parsed.
pub fn load_from_str(content: &str) -> ImmunityResult<AntibodyRegistry> {
    let registry: AntibodyRegistry = serde_yml::from_str(content)?;
    Ok(registry)
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL_REGISTRY: &str = r#"
version: "1.0"
antibodies: []
"#;

    const SINGLE_ANTIBODY: &str = r#"
version: "1.0"
antibodies:
  - id: TEST-001
    name: test-antibody
    threat_type: DAMP
    severity: high
    description: Test antibody for unit tests.
    detection:
      code_patterns:
        - pattern: '\.unwrap\(\)'
          language: rust
      file_contexts:
        - "*.rs"
    response:
      strategy: warn
      description: Use ? operator instead.
    confidence: 0.9
"#;

    #[test]
    fn test_load_minimal() {
        let registry = load_from_str(MINIMAL_REGISTRY);
        assert!(registry.is_ok());
        let registry = registry.ok().unwrap_or_else(|| AntibodyRegistry::new());
        assert!(registry.is_empty());
    }

    #[test]
    fn test_load_single_antibody() {
        let registry = load_from_str(SINGLE_ANTIBODY);
        assert!(registry.is_ok());
        let registry = registry.ok().unwrap_or_else(|| AntibodyRegistry::new());
        assert_eq!(registry.len(), 1);

        let ab = registry.get("TEST-001");
        assert!(ab.is_some());
        let ab = ab.unwrap_or_else(|| panic!("TEST-001 not found"));
        assert_eq!(ab.name, "test-antibody");
        assert_eq!(ab.severity, crate::types::ThreatLevel::High);
    }

    #[test]
    fn test_expand_tilde() {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/test".to_string());
        let expanded = expand_tilde("~/test/path");
        assert!(expanded.starts_with(&home));
        assert!(expanded.ends_with("/test/path"));
    }

    #[test]
    fn test_expand_tilde_no_tilde() {
        let path = "/absolute/path";
        assert_eq!(expand_tilde(path), path);
    }

    #[test]
    fn test_load_invalid_yaml() {
        let result = load_from_str("not: valid: yaml: {{");
        assert!(result.is_err());
    }
}
