//! Domain validator registry

use super::domain::{DomainValidator, GenericDomainValidator, SkillDomainValidator};
use nexcore_fs::glob::Pattern;
use std::path::Path;

/// Domain definition with detection patterns
#[derive(Debug, Clone)]
pub struct DomainDefinition {
    /// Domain name
    pub name: &'static str,
    /// Description
    pub description: &'static str,
    /// Glob patterns for detection (lower priority = checked first)
    pub patterns: &'static [&'static str],
    /// Priority (lower = higher priority)
    pub priority: u8,
}

/// Built-in domain definitions
pub static DOMAINS: &[DomainDefinition] = &[
    DomainDefinition {
        name: "skill",
        description: "Claude Code skill validation",
        patterns: &["*/SKILL.md", "skills/*", "**/SKILL.md"],
        priority: 1,
    },
    DomainDefinition {
        name: "agent",
        description: "Agent-enabled skill validation",
        patterns: &["*/agent/", "*/agent/persona.md"],
        priority: 2,
    },
    DomainDefinition {
        name: "config",
        description: "Claude Code configuration validation",
        patterns: &[
            "~/.claude.json",
            "~/.claude/settings.json",
            "**/claude.json",
        ],
        priority: 3,
    },
    DomainDefinition {
        name: "architecture",
        description: "System architecture validation",
        patterns: &["**/ARCHITECTURE.md", "**/architecture/**"],
        priority: 4,
    },
    DomainDefinition {
        name: "pv_terminology",
        description: "Pharmacovigilance terminology validation",
        patterns: &["**/pv/**", "**/pharmacovigilance/**"],
        priority: 5,
    },
    DomainDefinition {
        name: "construct",
        description: "Generic construct validation (fallback)",
        patterns: &["*"],
        priority: 100,
    },
];

/// Registry for domain validators
pub struct DomainRegistry;

impl DomainRegistry {
    /// Detects the domain for a target path
    ///
    /// Checks patterns in priority order (lower priority number = checked first).
    /// Returns "construct" as fallback if no patterns match.
    #[must_use]
    pub fn detect_domain(target: &Path) -> &'static str {
        let target_str = target.to_string_lossy();
        let target_name = target
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        // Sort by priority
        let mut domains: Vec<_> = DOMAINS.iter().collect();
        domains.sort_by_key(|d| d.priority);

        for domain in domains {
            for pattern in domain.patterns {
                // Handle special patterns
                if *pattern == "*" {
                    continue; // Skip wildcard (fallback)
                }

                // Check if pattern matches
                if let Ok(glob) = Pattern::new(pattern) {
                    if glob.matches(&target_str) || glob.matches(&target_name) {
                        return domain.name;
                    }
                }

                // Check for exact file existence
                if target.is_dir() {
                    let check_path = pattern.trim_start_matches("*/");
                    if target.join(check_path).exists() {
                        return domain.name;
                    }
                }
            }
        }

        // Fallback
        "construct"
    }

    /// Gets a domain validator for the given domain name
    #[must_use]
    pub fn get_validator(domain: &str) -> Box<dyn DomainValidator> {
        match domain {
            "skill" => Box::new(SkillDomainValidator),
            _ => Box::new(GenericDomainValidator::new(domain)),
        }
    }

    /// Gets a domain validator by auto-detecting from the target path
    #[must_use]
    pub fn get_validator_for_path(target: &Path) -> Box<dyn DomainValidator> {
        let domain = Self::detect_domain(target);
        Self::get_validator(domain)
    }

    /// Lists all registered domains
    #[must_use]
    pub fn list_domains() -> Vec<&'static DomainDefinition> {
        let mut domains: Vec<_> = DOMAINS.iter().collect();
        domains.sort_by_key(|d| d.priority);
        domains
    }

    /// Gets a domain definition by name
    #[must_use]
    pub fn get_domain(name: &str) -> Option<&'static DomainDefinition> {
        DOMAINS.iter().find(|d| d.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_detect_skill_domain() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("SKILL.md"), "# Test").unwrap();

        let domain = DomainRegistry::detect_domain(dir.path());
        assert_eq!(domain, "skill");
    }

    #[test]
    fn test_detect_fallback() {
        let dir = tempdir().unwrap();
        // Empty directory - should fallback to construct
        let domain = DomainRegistry::detect_domain(dir.path());
        assert_eq!(domain, "construct");
    }

    #[test]
    fn test_get_validator() {
        let validator = DomainRegistry::get_validator("skill");
        assert_eq!(validator.domain(), "skill");
    }

    #[test]
    fn test_list_domains() {
        let domains = DomainRegistry::list_domains();
        assert!(!domains.is_empty());
        // First should be highest priority (lowest number)
        assert_eq!(domains[0].name, "skill");
    }
}
