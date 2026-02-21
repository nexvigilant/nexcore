//! Shared intelligence content models and in-app dataset.

#[derive(Debug, Clone, Copy)]
pub struct IntelligenceArticle {
    pub slug: &'static str,
    pub title: &'static str,
    pub excerpt: &'static str,
    pub category: &'static str,
    pub date: &'static str,
    pub read_time: &'static str,
    pub author: &'static str,
    pub body: &'static [&'static str],
}

#[derive(Debug, Clone, Copy)]
pub struct IntelligenceSeries {
    pub slug: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub parts: u8,
    pub tag: &'static str,
    pub audience: &'static str,
    pub episodes: &'static [&'static str],
}

pub const ARTICLE_CATEGORIES: [&str; 4] = ["All", "Safety Signals", "Industry", "Education"];

pub const ARTICLES: [IntelligenceArticle; 4] = [
    IntelligenceArticle {
        slug: "understanding-prr-signal-detection",
        title: "Understanding PRR Signal Detection",
        excerpt: "A deep dive into Proportional Reporting Ratios and their role in pharmacovigilance.",
        category: "Education",
        date: "February 2026",
        read_time: "7 min",
        author: "NexVigilant Intelligence Team",
        body: &[
            "Proportional Reporting Ratio (PRR) is one of the foundational disproportionality methods used to detect potential safety signals.",
            "In practice, PRR should be interpreted with confidence intervals and minimum case thresholds to avoid noise from sparse data.",
            "Effective signal triage combines PRR with clinical plausibility, temporal patterns, and corroborating evidence from other methods.",
        ],
    },
    IntelligenceArticle {
        slug: "q4-2025-safety-signal-summary",
        title: "Q4 2025 Safety Signal Summary",
        excerpt: "Key safety signals identified across major therapeutic areas.",
        category: "Safety Signals",
        date: "January 2026",
        read_time: "6 min",
        author: "NexVigilant Intelligence Team",
        body: &[
            "Q4 analyses showed concentrated signal clusters in cardiovascular and immune-mediated event families.",
            "The most actionable findings were linked to products with increasing reporting velocity and elevated disproportionality metrics.",
            "Priority recommendations include focused case review, label impact assessment, and targeted regulator-facing summaries.",
        ],
    },
    IntelligenceArticle {
        slug: "future-of-pv-technology",
        title: "The Future of PV Technology",
        excerpt: "How AI and automation are transforming drug safety monitoring.",
        category: "Industry",
        date: "January 2026",
        read_time: "8 min",
        author: "NexVigilant Intelligence Team",
        body: &[
            "Automation is shifting pharmacovigilance operations from throughput-only models toward decision-support and quality controls.",
            "The strongest near-term use cases are case triage assistance, duplicate detection, and literature monitoring acceleration.",
            "Governance remains central: explainability, validation evidence, and change control determine regulatory readiness.",
        ],
    },
    IntelligenceArticle {
        slug: "causality-assessment-playbook",
        title: "Causality Assessment Playbook",
        excerpt: "Practical guidance for combining structured causality frameworks with signal analytics.",
        category: "Education",
        date: "December 2025",
        read_time: "9 min",
        author: "NexVigilant Intelligence Team",
        body: &[
            "Causality assessments should integrate structured frameworks with quantitative signal context rather than relying on either in isolation.",
            "A robust playbook aligns evidence quality, temporal association, dechallenge/rechallenge observations, and biological plausibility.",
            "Standardized review templates improve consistency across reviewers and accelerate escalation decisions.",
        ],
    },
];

pub const SERIES: [IntelligenceSeries; 2] = [
    IntelligenceSeries {
        slug: "foundations",
        title: "Foundations of Pharmacovigilance",
        description: "From adverse event reporting to signal detection, a practical multi-part foundation.",
        parts: 5,
        tag: "Core",
        audience: "Early-career PV and cross-functional teams",
        episodes: &[
            "Part 1: Case Processing Fundamentals",
            "Part 2: Signal Detection Methods",
            "Part 3: Causality Frameworks",
            "Part 4: Benefit-Risk Communication",
            "Part 5: Inspection Readiness",
        ],
    },
    IntelligenceSeries {
        slug: "career-transitions",
        title: "Career Transitions in Drug Safety",
        description: "A focused series for professionals moving into pharmacovigilance and safety science roles.",
        parts: 3,
        tag: "Career",
        audience: "Clinical and regulatory professionals transitioning into PV",
        episodes: &[
            "Part 1: Role Mapping and Skill Gap Analysis",
            "Part 2: Portfolio and Interview Strategy",
            "Part 3: 90-Day Transition Plan",
        ],
    },
];

pub fn article_by_slug(slug: &str) -> Option<&'static IntelligenceArticle> {
    ARTICLES.iter().find(|a| a.slug == slug)
}

pub fn series_by_slug(slug: &str) -> Option<&'static IntelligenceSeries> {
    SERIES.iter().find(|s| s.slug == slug)
}
