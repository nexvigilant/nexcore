//! # Homeostasis Memory Primitives
//!
//! Documents which T1 Lex Primitiva are operational in this crate and how
//! each manifests in the homeostasis memory domain.
//!
//! ## Operational Primitives (5)
//!
//! | Symbol | Primitive | Manifestation |
//! |--------|-----------|---------------|
//! | pi | Persistence | Incidents and playbooks persist across control loops |
//! | mu | Mapping | Signatures map patterns to response playbooks |
//! | kappa | Comparison | Similarity scoring drives pattern recognition |
//! | sigma | Sequence | Playbook steps execute in defined order |
//! | arrow | Causality | Incidents record cause-effect chains |

use nexcore_lex_primitiva::primitiva::LexPrimitiva;
use serde::Serialize;

/// A single primitive entry in the crate manifest.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PrimitiveEntry {
    /// The T1 Lex Primitiva symbol.
    pub primitive: LexPrimitiva,
    /// How this primitive manifests in homeostasis memory.
    pub manifestation: &'static str,
    /// Which types primarily exhibit this primitive.
    #[serde(skip)]
    pub primary_types: &'static [&'static str],
}

/// The complete primitive manifest for this crate.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CratePrimitiveManifest {
    /// Crate name.
    pub crate_name: &'static str,
    /// The operational primitives.
    pub primitives: Vec<PrimitiveEntry>,
}

impl CratePrimitiveManifest {
    /// Returns the number of operational primitives.
    #[must_use]
    pub fn count(&self) -> usize {
        self.primitives.len()
    }

    /// Returns true if the given primitive is operational in this crate.
    #[must_use]
    pub fn contains(&self, primitive: LexPrimitiva) -> bool {
        self.primitives.iter().any(|e| e.primitive == primitive)
    }

    /// Returns the manifestation description for a given primitive, if present.
    #[must_use]
    pub fn manifestation_of(&self, primitive: LexPrimitiva) -> Option<&'static str> {
        self.primitives
            .iter()
            .find(|e| e.primitive == primitive)
            .map(|e| e.manifestation)
    }
}

/// Returns the T1 primitive manifest for `nexcore-homeostasis-memory`.
///
/// Documents the 5 operational primitives and how each manifests.
#[must_use]
pub fn manifest() -> CratePrimitiveManifest {
    CratePrimitiveManifest {
        crate_name: "nexcore-homeostasis-memory",
        primitives: vec![
            PrimitiveEntry {
                primitive: LexPrimitiva::Persistence,
                manifestation: "Incidents and playbooks persist across control loops, \
                    enabling the system to remember past threats and responses",
                primary_types: &["Incident", "Playbook", "MemoryStore"],
            },
            PrimitiveEntry {
                primitive: LexPrimitiva::Mapping,
                manifestation: "Signatures map incident patterns to recommended playbooks; \
                    the core pattern-to-response transformation",
                primary_types: &["IncidentSignature", "PlaybookMatch", "MemoryStore"],
            },
            PrimitiveEntry {
                primitive: LexPrimitiva::Comparison,
                manifestation: "Similarity scoring between incident signatures enables \
                    pattern recognition and historical matching",
                primary_types: &[
                    "IncidentSignature",
                    "SimilarIncident",
                    "PlaybookMatch",
                    "MemoryStats",
                ],
            },
            PrimitiveEntry {
                primitive: LexPrimitiva::Sequence,
                manifestation: "Playbook steps execute in defined order; incidents form \
                    a temporal sequence from detection to resolution",
                primary_types: &["PlaybookStep", "Playbook", "Incident"],
            },
            PrimitiveEntry {
                primitive: LexPrimitiva::Causality,
                manifestation: "Incidents record cause-effect chains: trigger sensors \
                    cause detection, detection causes response, response causes resolution",
                primary_types: &["Incident", "PlaybookStep"],
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;

    #[test]
    fn manifest_has_5_primitives() {
        let m = manifest();
        assert_eq!(m.count(), 5);
    }

    #[test]
    fn manifest_crate_name() {
        let m = manifest();
        assert_eq!(m.crate_name, "nexcore-homeostasis-memory");
    }

    #[test]
    fn contains_persistence() {
        let m = manifest();
        assert!(m.contains(LexPrimitiva::Persistence));
    }

    #[test]
    fn contains_mapping() {
        let m = manifest();
        assert!(m.contains(LexPrimitiva::Mapping));
    }

    #[test]
    fn contains_comparison() {
        let m = manifest();
        assert!(m.contains(LexPrimitiva::Comparison));
    }

    #[test]
    fn contains_sequence() {
        let m = manifest();
        assert!(m.contains(LexPrimitiva::Sequence));
    }

    #[test]
    fn contains_causality() {
        let m = manifest();
        assert!(m.contains(LexPrimitiva::Causality));
    }

    #[test]
    fn does_not_contain_void() {
        let m = manifest();
        assert!(!m.contains(LexPrimitiva::Void));
    }

    #[test]
    fn does_not_contain_recursion() {
        let m = manifest();
        assert!(!m.contains(LexPrimitiva::Recursion));
    }

    #[test]
    fn manifestation_of_persistence() {
        let m = manifest();
        let desc = m.manifestation_of(LexPrimitiva::Persistence);
        assert!(desc.is_some());
        let desc = desc.unwrap_or("");
        assert!(desc.contains("persist"));
    }

    #[test]
    fn manifestation_of_missing_primitive() {
        let m = manifest();
        let desc = m.manifestation_of(LexPrimitiva::Void);
        assert!(desc.is_none());
    }

    #[test]
    fn all_entries_have_primary_types() {
        let m = manifest();
        for entry in &m.primitives {
            assert!(
                !entry.primary_types.is_empty(),
                "{:?} has no primary types",
                entry.primitive,
            );
        }
    }

    #[test]
    fn all_entries_have_nonempty_manifestation() {
        let m = manifest();
        for entry in &m.primitives {
            assert!(
                !entry.manifestation.is_empty(),
                "{:?} has empty manifestation",
                entry.primitive,
            );
        }
    }

    #[test]
    fn no_duplicate_primitives() {
        let m = manifest();
        let mut seen = std::collections::HashSet::new();
        for entry in &m.primitives {
            assert!(
                seen.insert(entry.primitive),
                "{:?} appears twice",
                entry.primitive,
            );
        }
    }

    #[test]
    fn serde_round_trip() {
        let m = manifest();
        let json = serde_json::to_string(&m);
        assert!(json.is_ok());
    }
}
