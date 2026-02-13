//! Vocabulary Intelligence for nexcore.
//!
//! A high-performance vocabulary analysis library implementing the Three Tiers
//! framework for term classification, affix analysis, and lexicon management.
//!
//! # Three Tiers Framework
//!
//! Based on Isabel Beck's vocabulary research:
//! - **Tier 1**: Basic, high-frequency words (the, is, run)
//! - **Tier 2**: Cross-domain academic words (validate, analyze, configure)
//! - **Tier 3**: Domain-specific terminology (FAERS, MedDRA, pharmacovigilance)
//!
//! # Example
//!
//! ```
//! use nexcore_vigilance::vocab::{classify, VocabTier, VocabDomain};
//!
//! // Classify terms into tiers
//! let result = classify("validate");
//! assert_eq!(result.tier, VocabTier::CrossDomain);
//!
//! // Detect domain-specific terms
//! let result = classify("faers");
//! assert_eq!(result.tier, VocabTier::DomainSpecific);
//! assert_eq!(result.domain, Some(VocabDomain::Pharmacovigilance));
//! ```
//!
//! # Modules
//!
//! - [`tier`]: Vocabulary tier definitions and word lists
//! - [`domain`]: Domain-specific glossaries (PV, AI/ML, DevOps)
//! - [`classifier`]: Term classification logic
//! - [`lexicon`]: Lexicon CRUD and export
//! - [`affix`]: Morphological analysis (prefixes/suffixes)

pub mod affix;
pub mod brand_semantics;
pub mod classifier;
pub mod domain;
pub mod error;
pub mod lexicon;
pub mod tier;
pub mod types;

// Re-export core types for convenience
pub use affix::{AffixAnalysis, AffixInfo};
pub use classifier::{Classification, ClassifyReason, TermFeatures, classify, classify_batch};
pub use domain::{VocabDomain, detect_domain, get_definition};
pub use error::{VocabError, VocabResult};
pub use lexicon::{Lexicon, LexiconEntry};
pub use tier::{TIER_1_WORDS, TIER_2_WORDS, TechnicalPattern, VocabTier};
pub use types::{Collocation, Compound, CompoundType, Idiom, PartOfSpeech, Token};

/// Prelude module for convenient imports.
pub mod prelude {
    pub use super::affix::{AffixAnalysis, AffixInfo};
    pub use super::classifier::{Classification, classify, classify_batch};
    pub use super::domain::{VocabDomain, detect_domain};
    pub use super::error::{VocabError, VocabResult};
    pub use super::lexicon::{Lexicon, LexiconEntry};
    pub use super::tier::VocabTier;
    pub use super::types::{Compound, PartOfSpeech, Token};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_tiers() {
        // Tier 1: Basic
        assert_eq!(classify("the").tier, VocabTier::Basic);

        // Tier 2: Academic
        assert_eq!(classify("validate").tier, VocabTier::CrossDomain);

        // Tier 3: Domain-specific
        let result = classify("faers");
        assert_eq!(result.tier, VocabTier::DomainSpecific);
        assert_eq!(result.domain, Some(VocabDomain::Pharmacovigilance));
    }

    #[test]
    fn test_affix_analysis() {
        let analysis = AffixAnalysis::analyze("unvalidated");
        assert!(analysis.has_affixes());
        assert!(analysis.prefix.is_some());
    }

    #[test]
    fn test_lexicon_workflow() {
        let mut lexicon = Lexicon::for_domain(VocabDomain::Pharmacovigilance);

        let entry = LexiconEntry::new(
            "icsr",
            "Individual Case Safety Report",
            VocabTier::DomainSpecific,
        )
        .with_domain(VocabDomain::Pharmacovigilance)
        .with_example("Submit the ICSR within 15 days.");

        assert!(lexicon.add(entry).is_ok());
        assert!(lexicon.get("icsr").is_some());
    }
}
