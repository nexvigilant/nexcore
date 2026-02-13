//! Domain-specific vocabulary definitions.
//!
//! Provides predefined glossaries for specialized domains including:
//! - Pharmacovigilance
//! - AI/ML
//! - NexVigilant
//! - DevOps

use phf::{phf_map, phf_set};
use serde::{Deserialize, Serialize};

/// Supported vocabulary domains.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VocabDomain {
    /// Pharmacovigilance and drug safety.
    Pharmacovigilance,
    /// Artificial intelligence and machine learning.
    AiMl,
    /// NexVigilant platform terminology.
    NexVigilant,
    /// DevOps and infrastructure.
    DevOps,
}

impl VocabDomain {
    /// All known domains.
    pub const ALL: &'static [Self] = &[
        Self::Pharmacovigilance,
        Self::AiMl,
        Self::NexVigilant,
        Self::DevOps,
    ];

    /// Parse domain from string. O(1)
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "pharmacovigilance" | "pv" => Some(Self::Pharmacovigilance),
            "ai_ml" | "aiml" | "ai" | "ml" => Some(Self::AiMl),
            "nexvigilant" | "nv" => Some(Self::NexVigilant),
            "devops" | "ops" => Some(Self::DevOps),
            _ => None,
        }
    }

    /// Get the glossary for this domain. O(1)
    #[must_use]
    pub fn glossary(&self) -> &'static phf::Set<&'static str> {
        match self {
            Self::Pharmacovigilance => &PHARMACOVIGILANCE_GLOSSARY,
            Self::AiMl => &AI_ML_GLOSSARY,
            Self::NexVigilant => &NEXVIGILANT_GLOSSARY,
            Self::DevOps => &DEVOPS_GLOSSARY,
        }
    }

    /// Get the definitions map for this domain. O(1)
    #[must_use]
    pub fn definitions(&self) -> &'static phf::Map<&'static str, &'static str> {
        match self {
            Self::Pharmacovigilance => &PHARMACOVIGILANCE_DEFINITIONS,
            Self::AiMl => &AI_ML_DEFINITIONS,
            Self::NexVigilant => &NEXVIGILANT_DEFINITIONS,
            Self::DevOps => &DEVOPS_DEFINITIONS,
        }
    }

    /// Get the domain indicators. O(1)
    #[must_use]
    pub fn indicators(&self) -> &'static [&'static str] {
        match self {
            Self::Pharmacovigilance => PHARMACOVIGILANCE_INDICATORS,
            Self::AiMl => AI_ML_INDICATORS,
            Self::NexVigilant => NEXVIGILANT_INDICATORS,
            Self::DevOps => DEVOPS_INDICATORS,
        }
    }

    /// Display name. O(1)
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::Pharmacovigilance => "Pharmacovigilance",
            Self::AiMl => "AI/ML",
            Self::NexVigilant => "NexVigilant",
            Self::DevOps => "DevOps",
        }
    }

    /// Check if glossary contains a term. O(1)
    #[must_use]
    pub fn contains(&self, term: &str) -> bool {
        self.glossary().contains(term)
    }
}

impl std::fmt::Display for VocabDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

// ============================================================================
// PHARMACOVIGILANCE
// ============================================================================

/// Pharmacovigilance domain glossary (Tier 3 terms).
pub static PHARMACOVIGILANCE_GLOSSARY: phf::Set<&'static str> = phf_set! {
    "icsr", "faers", "meddra", "whoart", "medwatch", "vaers", "cioms",
    "pharmacovigilance", "prr", "ror", "ebgm", "disproportionality",
    "psur", "pbrer", "rems", "rmp", "labeling",
};

/// Pharmacovigilance definitions.
pub static PHARMACOVIGILANCE_DEFINITIONS: phf::Map<&'static str, &'static str> = phf_map! {
    "pharmacovigilance" => "The science and activities relating to the detection, assessment, understanding and prevention of adverse effects or any other drug-related problem.",
    "icsr" => "Individual Case Safety Report - standardized format for reporting suspected adverse drug reactions.",
    "faers" => "FDA Adverse Event Reporting System - US database of adverse event reports.",
    "meddra" => "Medical Dictionary for Regulatory Activities - standardized medical terminology.",
    "prr" => "Proportional Reporting Ratio - statistical measure for signal detection.",
    "ror" => "Reporting Odds Ratio - statistical measure comparing event rates.",
    "ebgm" => "Empirical Bayesian Geometric Mean - Bayesian disproportionality measure.",
    "psur" => "Periodic Safety Update Report - regular safety summary submitted to regulators.",
    "pbrer" => "Periodic Benefit-Risk Evaluation Report - comprehensive benefit-risk assessment.",
    "rems" => "Risk Evaluation and Mitigation Strategies - FDA drug safety program.",
    "rmp" => "Risk Management Plan - structured safety monitoring plan.",
    "cioms" => "Council for International Organizations of Medical Sciences - develops pharmacovigilance standards.",
    "labeling" => "Product information including package insert, prescribing information.",
    "disproportionality" => "Statistical method comparing observed vs expected adverse event frequencies.",
    "medwatch" => "FDA safety reporting program for medical products.",
    "vaers" => "Vaccine Adverse Event Reporting System - US vaccine safety database.",
    "whoart" => "WHO Adverse Reaction Terminology - legacy medical terminology system.",
};

/// Pharmacovigilance domain indicators.
pub static PHARMACOVIGILANCE_INDICATORS: &[&str] = &[
    "adverse",
    "safety",
    "report",
    "signal",
    "event",
    "drug",
    "reaction",
    "causality",
];

// ============================================================================
// AI/ML
// ============================================================================

/// AI/ML domain glossary (Tier 3 terms).
pub static AI_ML_GLOSSARY: phf::Set<&'static str> = phf_set! {
    "embedding", "embeddings", "tokenization", "tokenizer", "transformer",
    "attention", "inference", "training", "neural", "nlp", "llm",
    "rag", "retrieval", "generation", "prompt", "token",
    "vector", "semantic", "latent", "encoder", "decoder", "bert", "gpt",
};

/// AI/ML definitions.
pub static AI_ML_DEFINITIONS: phf::Map<&'static str, &'static str> = phf_map! {
    "embedding" => "Dense vector representation of text/data in continuous space.",
    "tokenization" => "Process of breaking text into tokens (words, subwords, characters).",
    "transformer" => "Neural network architecture using self-attention mechanisms.",
    "inference" => "Using a trained model to make predictions on new data.",
    "rag" => "Retrieval-Augmented Generation - combining search with generation.",
    "llm" => "Large Language Model - neural network trained on vast text corpora.",
    "prompt" => "Input text provided to guide model behavior.",
    "attention" => "Mechanism allowing models to focus on relevant input parts.",
    "encoder" => "Network component that compresses input into latent representation.",
    "decoder" => "Network component that generates output from latent representation.",
    "bert" => "Bidirectional Encoder Representations from Transformers - masked language model.",
    "gpt" => "Generative Pre-trained Transformer - autoregressive language model.",
    "semantic" => "Relating to meaning rather than surface form.",
    "latent" => "Hidden representation learned by a model.",
    "nlp" => "Natural Language Processing - AI for understanding human language.",
};

/// AI/ML domain indicators.
pub static AI_ML_INDICATORS: &[&str] = &[
    "model",
    "train",
    "predict",
    "neural",
    "embed",
    "vector",
    "token",
    "inference",
];

// ============================================================================
// NEXVIGILANT
// ============================================================================

/// NexVigilant platform glossary (Tier 3 terms).
pub static NEXVIGILANT_GLOSSARY: phf::Set<&'static str> = phf_set! {
    "guardian", "academy", "nucleus", "neural", "community", "careers",
    "insights", "core", "macv", "cccp", "convergence", "vigil",
    "nexvigilant",
};

/// NexVigilant definitions.
pub static NEXVIGILANT_DEFINITIONS: phf::Map<&'static str, &'static str> = phf_map! {
    "guardian" => "NexVigilant module for regulatory monitoring, signal detection, and pharmaceutical intelligence.",
    "academy" => "NexVigilant module for professional development, competency-based learning, and educational pathways.",
    "nucleus" => "NexVigilant core module providing foundational services and shared infrastructure.",
    "neural" => "NexVigilant module for AI/ML capabilities, agent orchestration, and intelligent automation.",
    "community" => "NexVigilant module for professional networking, discussion forums, and peer collaboration.",
    "careers" => "NexVigilant module for job matching, career development, and professional transitions.",
    "insights" => "NexVigilant module for analytics, reporting, and business intelligence.",
    "core" => "Central NexVigilant infrastructure providing shared services across all modules.",
    "macv" => "Multi-Agent Collaborative Verification protocol for AI-assisted validation workflows.",
    "cccp" => "Campion Consulting Competency Protocol - framework for systematic skill development.",
    "convergence" => "Discovery of mechanistic connections between distinct drug classes or therapeutic areas.",
    "vigil" => "NexVigilant safety monitoring and alerting subsystem.",
    "nexvigilant" => "Unified pharmacovigilance and professional development platform.",
};

/// NexVigilant domain indicators.
pub static NEXVIGILANT_INDICATORS: &[&str] = &[
    "guardian", "academy", "nucleus", "pathway", "tier", "protocol", "vigilant",
];

// ============================================================================
// DEVOPS
// ============================================================================

/// DevOps domain glossary (Tier 3 terms).
pub static DEVOPS_GLOSSARY: phf::Set<&'static str> = phf_set! {
    "ci", "cd", "kubernetes", "k8s", "docker", "container",
    "pod", "deployment", "ingress", "configmap", "secret",
    "helm", "terraform", "ansible", "jenkins", "argocd", "gitops", "iac",
};

/// DevOps definitions.
pub static DEVOPS_DEFINITIONS: phf::Map<&'static str, &'static str> = phf_map! {
    "ci" => "Continuous Integration - automated building and testing of code changes.",
    "cd" => "Continuous Deployment/Delivery - automated release of validated changes.",
    "container" => "Lightweight, standalone executable package including code and dependencies.",
    "kubernetes" => "Container orchestration platform for automated deployment and scaling.",
    "k8s" => "Abbreviation for Kubernetes (K + 8 letters + S).",
    "docker" => "Platform for building, shipping, and running containerized applications.",
    "gitops" => "Using Git as single source of truth for declarative infrastructure.",
    "iac" => "Infrastructure as Code - managing infrastructure through definition files.",
    "helm" => "Package manager for Kubernetes applications.",
    "terraform" => "Infrastructure as Code tool for cloud resource provisioning.",
    "ansible" => "Automation platform for configuration management and deployment.",
    "pod" => "Smallest deployable unit in Kubernetes, containing one or more containers.",
    "ingress" => "Kubernetes resource for managing external access to services.",
    "configmap" => "Kubernetes resource for storing non-sensitive configuration data.",
    "argocd" => "GitOps continuous delivery tool for Kubernetes.",
};

/// DevOps domain indicators.
pub static DEVOPS_INDICATORS: &[&str] = &[
    "deploy",
    "container",
    "pipeline",
    "config",
    "scale",
    "monitor",
    "cluster",
];

// ============================================================================
// DOMAIN DETECTION
// ============================================================================

/// Detect which domain a term belongs to via glossary lookup. O(1)
///
/// Checks only glossary membership using O(1) PHF lookup per domain.
#[must_use]
pub fn detect_domain_exact(term: &str) -> Option<VocabDomain> {
    let lower = term.to_lowercase();
    VocabDomain::ALL
        .iter()
        .find(|d| d.contains(&lower))
        .copied()
}

/// Detect domain with indicator fallback. O(n)
///
/// First checks glossary (O(1)), then checks if term matches any indicator.
#[must_use]
pub fn detect_domain(term: &str) -> Option<VocabDomain> {
    // First try exact glossary match - O(1) per domain
    if let Some(domain) = detect_domain_exact(term) {
        return Some(domain);
    }

    // Then check indicator matches - O(D * I) where D=4, I=8 (constant)
    let lower = term.to_lowercase();
    for domain in VocabDomain::ALL {
        if domain.indicators().iter().any(|i| *i == lower) {
            return Some(*domain);
        }
    }

    None
}

/// Get definition for a term in any domain. O(1)
#[must_use]
pub fn get_definition(term: &str) -> Option<(&'static str, VocabDomain)> {
    let lower = term.to_lowercase();

    for domain in VocabDomain::ALL {
        if let Some(def) = domain.definitions().get(lower.as_str()) {
            return Some((def, *domain));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_parse() {
        assert_eq!(
            VocabDomain::parse("pharmacovigilance"),
            Some(VocabDomain::Pharmacovigilance)
        );
        assert_eq!(
            VocabDomain::parse("pv"),
            Some(VocabDomain::Pharmacovigilance)
        );
        assert_eq!(VocabDomain::parse("AI_ML"), Some(VocabDomain::AiMl));
        assert_eq!(VocabDomain::parse("devops"), Some(VocabDomain::DevOps));
        assert_eq!(VocabDomain::parse("unknown"), None);
    }

    #[test]
    fn test_detect_domain() {
        assert_eq!(detect_domain("faers"), Some(VocabDomain::Pharmacovigilance));
        assert_eq!(detect_domain("transformer"), Some(VocabDomain::AiMl));
        assert_eq!(detect_domain("kubernetes"), Some(VocabDomain::DevOps));
        assert_eq!(detect_domain("guardian"), Some(VocabDomain::NexVigilant));
    }

    #[test]
    fn test_get_definition() {
        let result = get_definition("prr");
        assert!(result.is_some());
        if let Some((def, domain)) = result {
            assert!(def.contains("Proportional Reporting Ratio"));
            assert_eq!(domain, VocabDomain::Pharmacovigilance);
        }
    }

    #[test]
    fn test_domain_glossaries_not_empty() {
        for domain in VocabDomain::ALL {
            assert!(
                !domain.glossary().is_empty(),
                "{} glossary is empty",
                domain
            );
        }
    }
}
