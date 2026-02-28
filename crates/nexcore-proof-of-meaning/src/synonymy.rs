//! Synonymy — curated synonym resolution for regulatory PV terminology.
//!
//! Chemistry analogue: Isotopes. Different labels, same element.
//! "cardiac" and "heart" are isotopes of the OrganSystem::Cardiac atom.
//!
//! Synonym groups are curated from ICH guidelines, MedDRA, and WHO-UMC.
//! No ML needed — these are domain-expert-defined equivalences.

use std::collections::HashMap;

use crate::element::ElementClass;

/// A synonym group — a set of labels that map to the same canonical atom.
#[derive(Debug, Clone)]
pub struct SynonymGroup {
    /// The canonical label (the "most common isotope").
    pub canonical: &'static str,
    /// All labels in the group (including canonical).
    pub members: &'static [&'static str],
    /// Element class this group belongs to.
    pub class: ElementClass,
    /// Intra-group similarity: how similar are members to each other?
    /// 1.0 = true synonyms. 0.7-0.9 = near-synonyms (graded).
    pub similarity: f64,
}

/// The synonym registry — all curated groups indexed for fast lookup.
pub struct SynonymRegistry {
    /// label → (canonical, class, similarity)
    lookup: HashMap<String, (&'static str, ElementClass, f64)>,
    /// canonical → group
    groups: HashMap<&'static str, &'static SynonymGroup>,
}

// =============================================================================
// CURATED SYNONYM GROUPS
// =============================================================================

/// Organ system synonyms (MedDRA SOC level + common lay terms).
static ORGAN_SYSTEM_GROUPS: &[SynonymGroup] = &[
    SynonymGroup {
        canonical: "cardiac",
        members: &[
            "cardiac",
            "heart",
            "heart-related",
            "cardiovascular",
            "cardiological",
        ],
        class: ElementClass::OrganSystem,
        similarity: 0.90,
    },
    SynonymGroup {
        canonical: "hepatic",
        members: &["hepatic", "liver", "hepatobiliary", "hepatological"],
        class: ElementClass::OrganSystem,
        similarity: 0.90,
    },
    SynonymGroup {
        canonical: "renal",
        members: &["renal", "kidney", "nephrological", "urinary"],
        class: ElementClass::OrganSystem,
        similarity: 0.85,
    },
    SynonymGroup {
        canonical: "neurological",
        members: &[
            "neurological",
            "neural",
            "nervous",
            "cerebral",
            "brain",
            "cns",
        ],
        class: ElementClass::OrganSystem,
        similarity: 0.85,
    },
    SynonymGroup {
        canonical: "pulmonary",
        members: &["pulmonary", "respiratory", "lung", "thoracic"],
        class: ElementClass::OrganSystem,
        similarity: 0.85,
    },
    SynonymGroup {
        canonical: "gastrointestinal",
        members: &[
            "gastrointestinal",
            "gi",
            "gastric",
            "intestinal",
            "digestive",
            "stomach",
        ],
        class: ElementClass::OrganSystem,
        similarity: 0.85,
    },
    SynonymGroup {
        canonical: "dermatologic",
        members: &[
            "dermatologic",
            "dermal",
            "skin",
            "cutaneous",
            "subcutaneous",
        ],
        class: ElementClass::OrganSystem,
        similarity: 0.85,
    },
    SynonymGroup {
        canonical: "hematologic",
        members: &["hematologic", "haematological", "blood", "hematopoietic"],
        class: ElementClass::OrganSystem,
        similarity: 0.85,
    },
    SynonymGroup {
        canonical: "musculoskeletal",
        members: &[
            "musculoskeletal",
            "skeletal",
            "muscular",
            "orthopedic",
            "joint",
        ],
        class: ElementClass::OrganSystem,
        similarity: 0.80,
    },
    SynonymGroup {
        canonical: "ophthalmic",
        members: &["ophthalmic", "ocular", "eye", "visual"],
        class: ElementClass::OrganSystem,
        similarity: 0.85,
    },
];

/// Observation type synonyms (ICH E2D, E2B(R3) terminology).
static OBSERVATION_TYPE_GROUPS: &[SynonymGroup] = &[
    SynonymGroup {
        canonical: "event",
        members: &["event", "occurrence", "incident", "episode"],
        class: ElementClass::ObservationType,
        similarity: 0.90,
    },
    SynonymGroup {
        canonical: "reaction",
        members: &["reaction", "response"],
        class: ElementClass::ObservationType,
        similarity: 0.95,
    },
    SynonymGroup {
        canonical: "experience",
        members: &["experience"],
        class: ElementClass::ObservationType,
        similarity: 1.00,
    },
    SynonymGroup {
        canonical: "effect",
        members: &["effect", "side effect"],
        class: ElementClass::ObservationType,
        similarity: 0.90,
    },
];

/// Cross-group observation near-synonyms.
/// "event" ≈ "reaction" ≈ "experience" — they're different words for
/// the same concept in PV context. This is the critical group that
/// makes the Rosetta Proof work.
static OBSERVATION_NEAR_SYNONYMS: &SynonymGroup = &SynonymGroup {
    canonical: "event",
    members: &[
        "event",
        "reaction",
        "experience",
        "effect",
        "occurrence",
        "incident",
        "episode",
        "response",
    ],
    class: ElementClass::ObservationType,
    similarity: 0.80, // Near-synonym, not exact
};

/// Action synonyms (ICH E2B action taken).
static ACTION_GROUPS: &[SynonymGroup] = &[
    SynonymGroup {
        canonical: "immunization",
        members: &["immunization", "vaccination", "vaccine", "inoculation"],
        class: ElementClass::Action,
        similarity: 0.90,
    },
    SynonymGroup {
        canonical: "withdrawn",
        members: &[
            "withdrawn",
            "discontinued",
            "stopped",
            "ceased",
            "terminated",
        ],
        class: ElementClass::Action,
        similarity: 0.90,
    },
    SynonymGroup {
        canonical: "hospitalized",
        members: &["hospitalized", "hospitalised", "admitted", "inpatient"],
        class: ElementClass::Action,
        similarity: 0.90,
    },
];

/// Causality synonyms (WHO-UMC + Naranjo categories).
static CAUSALITY_GROUPS: &[SynonymGroup] = &[
    SynonymGroup {
        canonical: "related",
        members: &["related", "associated", "attributed", "linked", "connected"],
        class: ElementClass::Causality,
        similarity: 0.85,
    },
    SynonymGroup {
        canonical: "suspected",
        members: &["suspected", "possible", "potential", "putative"],
        class: ElementClass::Causality,
        similarity: 0.80,
    },
    SynonymGroup {
        canonical: "unrelated",
        members: &["unrelated", "unassociated", "coincidental", "incidental"],
        class: ElementClass::Causality,
        similarity: 0.85,
    },
];

/// Severity synonyms (CTCAE + ICH E2A).
static SEVERITY_GROUPS: &[SynonymGroup] = &[
    SynonymGroup {
        canonical: "serious",
        members: &["serious", "significant", "important"],
        class: ElementClass::Severity,
        similarity: 0.80,
    },
    SynonymGroup {
        canonical: "severe",
        members: &["severe", "intense", "marked"],
        class: ElementClass::Severity,
        similarity: 0.85,
    },
    SynonymGroup {
        canonical: "fatal",
        members: &["fatal", "lethal", "death", "died", "deceased"],
        class: ElementClass::Severity,
        similarity: 0.90,
    },
];

/// Modifier synonyms.
static MODIFIER_GROUPS: &[SynonymGroup] = &[
    SynonymGroup {
        canonical: "adverse",
        members: &["adverse", "untoward", "unwanted", "undesirable", "harmful"],
        class: ElementClass::Modifier,
        similarity: 0.85,
    },
    SynonymGroup {
        canonical: "unexpected",
        members: &[
            "unexpected",
            "unlisted",
            "unanticipated",
            "unforeseen",
            "novel",
        ],
        class: ElementClass::Modifier,
        similarity: 0.85,
    },
    SynonymGroup {
        canonical: "drug",
        members: &[
            "drug",
            "medication",
            "medicine",
            "pharmaceutical",
            "medicinal product",
            "therapeutic",
        ],
        class: ElementClass::Modifier,
        similarity: 0.85,
    },
];

/// Temporality synonyms.
static TEMPORALITY_GROUPS: &[SynonymGroup] = &[
    SynonymGroup {
        canonical: "following",
        members: &["following", "after", "post", "subsequent", "upon"],
        class: ElementClass::Temporality,
        similarity: 0.90,
    },
    SynonymGroup {
        canonical: "acute",
        members: &["acute", "sudden", "abrupt", "rapid-onset"],
        class: ElementClass::Temporality,
        similarity: 0.85,
    },
    SynonymGroup {
        canonical: "chronic",
        members: &["chronic", "long-term", "prolonged", "persistent", "ongoing"],
        class: ElementClass::Temporality,
        similarity: 0.85,
    },
];

/// Outcome synonyms.
static OUTCOME_GROUPS: &[SynonymGroup] = &[
    SynonymGroup {
        canonical: "recovered",
        members: &["recovered", "resolved", "remitted", "abated"],
        class: ElementClass::Outcome,
        similarity: 0.90,
    },
    SynonymGroup {
        canonical: "recovering",
        members: &["recovering", "resolving", "improving", "ameliorating"],
        class: ElementClass::Outcome,
        similarity: 0.90,
    },
];

// =============================================================================
// SYNONYM REGISTRY
// =============================================================================

impl SynonymRegistry {
    /// Build the full synonym registry from all curated groups.
    pub fn pv_standard() -> Self {
        let mut lookup = HashMap::new();
        let mut groups = HashMap::new();

        let all_group_slices: &[&[SynonymGroup]] = &[
            ORGAN_SYSTEM_GROUPS,
            OBSERVATION_TYPE_GROUPS,
            ACTION_GROUPS,
            CAUSALITY_GROUPS,
            SEVERITY_GROUPS,
            MODIFIER_GROUPS,
            TEMPORALITY_GROUPS,
            OUTCOME_GROUPS,
        ];

        for slice in all_group_slices {
            for group in *slice {
                groups.insert(group.canonical, group);
                for &member in group.members {
                    lookup.insert(
                        member.to_lowercase(),
                        (group.canonical, group.class.clone(), group.similarity),
                    );
                }
            }
        }

        // Register the cross-group observation near-synonyms
        groups.insert(
            OBSERVATION_NEAR_SYNONYMS.canonical,
            OBSERVATION_NEAR_SYNONYMS,
        );

        Self { lookup, groups }
    }

    /// Look up a token's canonical form and class.
    /// Returns (canonical_label, element_class, similarity_to_canonical).
    pub fn resolve(&self, token: &str) -> Option<(&'static str, ElementClass, f64)> {
        self.lookup.get(&token.to_lowercase()).cloned()
    }

    /// Check if two tokens are synonyms (same canonical form).
    /// Returns the similarity score if they are, None if they aren't.
    pub fn synonym_similarity(&self, a: &str, b: &str) -> Option<f64> {
        let a_lower = a.to_lowercase();
        let b_lower = b.to_lowercase();

        // Exact match
        if a_lower == b_lower {
            return Some(1.0);
        }

        // Same canonical in any group?
        let a_info = self.lookup.get(&a_lower)?;
        let b_info = self.lookup.get(&b_lower)?;

        if a_info.0 == b_info.0 {
            // Same canonical → within-group synonyms
            return Some(a_info.2.min(b_info.2));
        }

        // Check cross-group observation near-synonyms
        if a_info.1 == ElementClass::ObservationType && b_info.1 == ElementClass::ObservationType {
            let a_in_near = OBSERVATION_NEAR_SYNONYMS
                .members
                .iter()
                .any(|m| m.to_lowercase() == a_lower);
            let b_in_near = OBSERVATION_NEAR_SYNONYMS
                .members
                .iter()
                .any(|m| m.to_lowercase() == b_lower);
            if a_in_near && b_in_near {
                return Some(OBSERVATION_NEAR_SYNONYMS.similarity);
            }
        }

        None
    }

    /// Get the synonym group for a canonical label.
    pub fn group(&self, canonical: &str) -> Option<&&SynonymGroup> {
        self.groups.get(canonical)
    }

    /// Total number of registered synonym mappings.
    pub fn count(&self) -> usize {
        self.lookup.len()
    }

    /// Total number of synonym groups.
    pub fn group_count(&self) -> usize {
        self.groups.len()
    }
}

impl Default for SynonymRegistry {
    fn default() -> Self {
        Self::pv_standard()
    }
}
