//! # Access Primitives
//!
//! Cross-domain access control patterns.
//!
//! ## Contents
//!
//! | Type | Tier | Confidence | Domains |
//! |------|------|------------|---------|
//! | [`AllowList`] | T2-P | 0.83 | Security, PV exclusions, rate limit exemptions |

use std::collections::HashSet;
use std::fmt;
use std::hash::Hash;

// ============================================================================
// T2-P: AllowList<T>
// ============================================================================

/// Tier: T2-P — A set of entities exempt from enforcement actions.
///
/// Grounds to: `Set<T>` (T1) + exemption semantics.
///
/// ## Primitive Test
///
/// - **No domain-internal deps**: uses only set, entity, membership, exempt, boolean
/// - **External grounding**: security whitelists, PV exclusions, rate limit exemptions
/// - **Not a synonym for Set**: carries enforcement-exemption semantics beyond raw membership
///
/// ## Transfer Confidence: 0.83
///
/// | Dimension | Score | Notes |
/// |-----------|-------|-------|
/// | Structural | 0.85 | `Set<T>` + `contains()` is universal |
/// | Functional | 0.85 | Exemption logic identical across domains |
/// | Contextual | 0.75 | Naming varies: whitelist, safelist, allowlist |
///
/// ## Cross-Domain Applications
///
/// - **Security**: trusted host/user lists (never ban)
/// - **PV**: known/expected adverse reaction exclusions
/// - **API**: rate limit exemptions (internal services)
/// - **Governance**: privileged entity lists
/// - **Content moderation**: approved content lists
/// - **Email**: safe sender lists (spam bypass)
#[derive(Debug, Clone)]
pub struct AllowList<T: Eq + Hash> {
    entries: HashSet<T>,
}

impl<T: Eq + Hash> AllowList<T> {
    /// Create an empty allow list.
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: HashSet::new(),
        }
    }

    /// Create from an iterator of entries.
    pub fn from_iter(iter: impl IntoIterator<Item = T>) -> Self {
        Self {
            entries: iter.into_iter().collect(),
        }
    }

    /// Check if an entity is allowed (exempt from enforcement).
    #[must_use]
    pub fn is_allowed(&self, entity: &T) -> bool {
        self.entries.contains(entity)
    }

    /// Check if an entity should be enforced (NOT in the allow list).
    #[must_use]
    pub fn should_enforce(&self, entity: &T) -> bool {
        !self.is_allowed(entity)
    }

    /// Add an entity to the allow list.
    /// Returns `true` if the entity was newly added.
    pub fn add(&mut self, entity: T) -> bool {
        self.entries.insert(entity)
    }

    /// Remove an entity from the allow list.
    /// Returns `true` if the entity was present.
    pub fn remove(&mut self, entity: &T) -> bool {
        self.entries.remove(entity)
    }

    /// Number of entries in the allow list.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the allow list is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Iterate over entries.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.entries.iter()
    }
}

impl<T: Eq + Hash> Default for AllowList<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Eq + Hash + fmt::Debug> fmt::Display for AllowList<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AllowList({} entries)", self.entries.len())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_allow_list() {
        let list: AllowList<String> = AllowList::new();
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
        assert!(!list.is_allowed(&"test".to_string()));
        assert!(list.should_enforce(&"test".to_string()));
    }

    #[test]
    fn add_and_check() {
        let mut list = AllowList::new();
        assert!(list.add("admin".to_string()));
        assert!(list.is_allowed(&"admin".to_string()));
        assert!(!list.should_enforce(&"admin".to_string()));
    }

    #[test]
    fn add_duplicate() {
        let mut list = AllowList::new();
        assert!(list.add(42));
        assert!(!list.add(42)); // duplicate returns false
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn remove() {
        let mut list = AllowList::new();
        list.add("x".to_string());
        assert!(list.remove(&"x".to_string()));
        assert!(!list.is_allowed(&"x".to_string()));
        assert!(!list.remove(&"x".to_string())); // already removed
    }

    #[test]
    fn from_iter() {
        let list = AllowList::from_iter(vec![1, 2, 3]);
        assert_eq!(list.len(), 3);
        assert!(list.is_allowed(&1));
        assert!(list.is_allowed(&2));
        assert!(list.is_allowed(&3));
        assert!(!list.is_allowed(&4));
    }

    #[test]
    fn enforce_logic() {
        let list = AllowList::from_iter(vec!["safe"]);
        assert!(!list.should_enforce(&"safe"));
        assert!(list.should_enforce(&"dangerous"));
    }

    #[test]
    fn default_is_empty() {
        let list: AllowList<u32> = AllowList::default();
        assert!(list.is_empty());
    }

    #[test]
    fn display() {
        let list = AllowList::from_iter(vec![1, 2, 3]);
        assert_eq!(format!("{list}"), "AllowList(3 entries)");
    }

    #[test]
    fn iter_entries() {
        let list = AllowList::from_iter(vec![10, 20]);
        let collected: Vec<&i32> = list.iter().collect();
        assert_eq!(collected.len(), 2);
    }
}
