// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # KSB Types — Knowledge, Skills, Behaviors
//!
//! Core domain types for competency-based education.
//! Migrated from `studio/backend/app/models/course.py`.
//!
//! ## Primitive Grounding
//!
//! | Component | Primitive | Meaning |
//! |-----------|-----------|---------|
//! | KSB category | Σ | Sum type: Knowledge \| Skill \| Behavior |
//! | Component ID | λ | Location/address within framework |
//! | Framework | σ | Ordered collection of components |

use serde::{Deserialize, Serialize};

/// The three dimensions of competency (ICH E17-aligned).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KsbCategory {
    /// Factual and conceptual understanding.
    Knowledge,
    /// Ability to perform tasks and procedures.
    Skill,
    /// Observable professional conduct and attitudes.
    Behavior,
}

impl KsbCategory {
    /// Display label used in academy UI.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Knowledge => "Knowledge",
            Self::Skill => "Skills",
            Self::Behavior => "Behaviors",
        }
    }

    /// Module description template.
    pub fn module_description(&self) -> String {
        format!(
            "Learn the essential {} components",
            self.label().to_lowercase()
        )
    }

    /// All three categories in standard order.
    pub const ALL: [KsbCategory; 3] = [Self::Knowledge, Self::Skill, Self::Behavior];
}

/// A single KSB component within a framework.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KsbComponent {
    /// Unique identifier (e.g., "K1", "S3", "B7").
    pub id: String,
    /// Category of this component.
    pub category: KsbCategory,
    /// Title of the competency.
    pub title: String,
    /// Detailed description.
    pub description: String,
}

/// A complete KSB framework decomposition for a topic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KsbFramework {
    /// Topic this framework covers.
    pub topic: String,
    /// Domain context (e.g., "Pharmaceutical Industry").
    pub domain: String,
    /// Knowledge components.
    pub knowledge: Vec<KsbComponent>,
    /// Skill components.
    pub skills: Vec<KsbComponent>,
    /// Behavior components.
    pub behaviors: Vec<KsbComponent>,
}

impl KsbFramework {
    /// Total number of components across all categories.
    pub fn total_components(&self) -> usize {
        self.knowledge.len() + self.skills.len() + self.behaviors.len()
    }

    /// Get all components for a given category.
    pub fn by_category(&self, cat: KsbCategory) -> &[KsbComponent] {
        match cat {
            KsbCategory::Knowledge => &self.knowledge,
            KsbCategory::Skill => &self.skills,
            KsbCategory::Behavior => &self.behaviors,
        }
    }
}

/// Compute similarity between two KSB titles using normalized longest common subsequence.
///
/// Returns a score in [0.0, 1.0]. This replaces Python's `difflib.SequenceMatcher`.
pub fn ksb_similarity(a: &str, b: &str) -> f64 {
    let a_lower = a.to_lowercase();
    let b_lower = b.to_lowercase();
    let a_bytes = a_lower.as_bytes();
    let b_bytes = b_lower.as_bytes();
    let m = a_bytes.len();
    let n = b_bytes.len();

    if m == 0 || n == 0 {
        return 0.0;
    }

    // LCS via dynamic programming (space-optimized: two rows)
    let mut prev = vec![0u32; n + 1];
    let mut curr = vec![0u32; n + 1];

    for i in 1..=m {
        for j in 1..=n {
            if a_bytes[i - 1] == b_bytes[j - 1] {
                curr[j] = prev[j - 1] + 1;
            } else {
                curr[j] = prev[j].max(curr[j - 1]);
            }
        }
        std::mem::swap(&mut prev, &mut curr);
        curr.fill(0);
    }

    let lcs_len = prev[n] as f64;
    // Normalize by average length (matches SequenceMatcher behavior)
    2.0 * lcs_len / (m + n) as f64
}

/// Match discovered KSBs against an existing library of approved KSBs.
///
/// Returns pairs of (discovered_index, library_index, similarity_score)
/// for all matches above the threshold.
pub fn match_ksbs(
    discovered: &[&str],
    library: &[&str],
    threshold: f64,
) -> Vec<(usize, usize, f64)> {
    let mut matches = Vec::new();
    for (di, d_title) in discovered.iter().enumerate() {
        let mut best_score = 0.0;
        let mut best_idx = 0;
        for (li, l_title) in library.iter().enumerate() {
            let score = ksb_similarity(d_title, l_title);
            if score > best_score {
                best_score = score;
                best_idx = li;
            }
        }
        if best_score >= threshold {
            matches.push((di, best_idx, best_score));
        }
    }
    matches
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ksb_category_labels() {
        assert_eq!(KsbCategory::Knowledge.label(), "Knowledge");
        assert_eq!(KsbCategory::Skill.label(), "Skills");
        assert_eq!(KsbCategory::Behavior.label(), "Behaviors");
    }

    #[test]
    fn similarity_identical() {
        let score = ksb_similarity("Drug Safety Assessment", "Drug Safety Assessment");
        assert!((score - 1.0).abs() < 0.001);
    }

    #[test]
    fn similarity_partial() {
        let score = ksb_similarity("Drug Safety Assessment", "Drug Safety Evaluation");
        assert!(score > 0.6); // Substantial overlap
    }

    #[test]
    fn similarity_unrelated() {
        let score = ksb_similarity("Pharmacovigilance", "Quantum Computing");
        // LCS shares some common letters (a, n, i, c, etc.), so score is moderate
        assert!(score < 0.5);
    }

    #[test]
    fn match_ksbs_finds_matches() {
        let discovered = vec!["Drug Safety", "Signal Detection", "Regulatory Compliance"];
        let library = vec![
            "Drug Safety Assessment",
            "PV Signal Detection Methods",
            "Cooking",
        ];
        let matches = match_ksbs(&discovered, &library, 0.5);
        assert_eq!(matches.len(), 2); // Drug Safety + Signal Detection match
    }

    #[test]
    fn framework_total_components() {
        let fw = KsbFramework {
            topic: "PV".into(),
            domain: "Pharma".into(),
            knowledge: vec![KsbComponent {
                id: "K1".into(),
                category: KsbCategory::Knowledge,
                title: "Test".into(),
                description: "Desc".into(),
            }],
            skills: vec![],
            behaviors: vec![],
        };
        assert_eq!(fw.total_components(), 1);
    }
}
