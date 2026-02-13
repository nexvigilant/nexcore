//! MCP Tool Suggester Hook
//!
//! Event: UserPromptSubmit
//!
//! Analyzes user prompts for topics that have optimized MCP tool implementations.
//! Suggests using `mcp__nexcore__*` tools instead of manual implementations.
//!
//! This promotes the "MCP-first" philosophy: if nexcore has a tool for it,
//! use the tool instead of reinventing the wheel.
//!
//! Records suggestions to mcp_efficacy.json for Phase 2 CTVP validation.
//!
//! ## Phase 3 CTVP: Canary Rollout
//!
//! Supports gradual rollout via `~/.claude/mcp_efficacy_config.toml`:
//! - `feature_flags.enabled` - Master switch
//! - `feature_flags.rollout_percentage` - Canary percentage (0-100)
//! - `keywords.*` - Custom keyword mappings

use nexcore_hooks::mcp_config::McpEfficacyConfig;
use nexcore_hooks::mcp_efficacy::with_efficacy_registry;
use nexcore_hooks::{exit_skip_prompt, exit_with_context, read_input};
use regex::Regex;

/// Check if keyword matches as a whole word (word boundary matching)
/// Falls back to substring matching if regex fails
fn matches_word(text: &str, keyword: &str) -> bool {
    // Multi-word keywords: use contains (they're specific enough)
    if keyword.contains(' ') || keyword.contains('-') {
        return text.contains(keyword);
    }
    // Single words: use word boundary regex, fallback to contains
    let escaped = regex::escape(keyword);
    Regex::new(&format!(r"\b{}\b", escaped))
        .map(|re| re.is_match(text))
        .unwrap_or_else(|_| text.contains(keyword))
}

/// MCP tool suggestion pattern
struct McpSuggestion {
    /// Keywords that trigger this suggestion (any match)
    keywords: &'static [&'static str],
    /// MCP tools to suggest
    tools: &'static [&'static str],
    /// Category description
    category: &'static str,
    /// When to use
    use_case: &'static str,
}

/// MCP tool suggestions based on user intent
const MCP_SUGGESTIONS: &[McpSuggestion] = &[
    // Foundation algorithms
    McpSuggestion {
        keywords: &[
            "levenshtein",
            "edit distance",
            "string similarity",
            "fuzzy match",
            "typo",
        ],
        tools: &[
            "mcp__nexcore__foundation_levenshtein",
            "mcp__nexcore__foundation_fuzzy_search",
        ],
        category: "String Similarity",
        use_case: "10-63x faster than Python implementations",
    },
    McpSuggestion {
        keywords: &["sha256", "hash", "checksum", "integrity"],
        tools: &["mcp__nexcore__foundation_sha256"],
        category: "Cryptographic Hashing",
        use_case: "20x faster than shell sha256sum for programmatic use",
    },
    McpSuggestion {
        keywords: &["yaml", "parse yaml", "yaml to json"],
        tools: &["mcp__nexcore__foundation_yaml_parse"],
        category: "YAML Processing",
        use_case: "7x faster parsing with validation",
    },
    McpSuggestion {
        keywords: &["topological sort", "dependency order", "build order", "dag"],
        tools: &[
            "mcp__nexcore__foundation_graph_topsort",
            "mcp__nexcore__foundation_graph_levels",
        ],
        category: "Graph Algorithms",
        use_case: "Parallel execution levels and dependency ordering",
    },
    // PV Signal Detection
    McpSuggestion {
        keywords: &[
            "signal detection",
            "disproportionality",
            "prr",
            "ror",
            "ebgm",
            "ic",
        ],
        tools: &[
            "mcp__nexcore__pv_signal_complete",
            "mcp__nexcore__pv_signal_prr",
            "mcp__nexcore__pv_signal_ror",
        ],
        category: "Pharmacovigilance Signals",
        use_case: "All 5 algorithms at once with pv_signal_complete",
    },
    McpSuggestion {
        keywords: &["naranjo", "causality", "who-umc", "adverse event causality"],
        tools: &[
            "mcp__nexcore__pv_naranjo_quick",
            "mcp__nexcore__pv_who_umc_quick",
        ],
        category: "Causality Assessment",
        use_case: "5-question quick causality scoring",
    },
    McpSuggestion {
        keywords: &["chi-square", "chi square", "statistical test"],
        tools: &["mcp__nexcore__pv_chi_square"],
        category: "Statistical Testing",
        use_case: "Chi-square with Yates correction",
    },
    // FAERS
    McpSuggestion {
        keywords: &[
            "faers",
            "fda adverse",
            "drug events",
            "adverse event database",
        ],
        tools: &[
            "mcp__nexcore__faers_search",
            "mcp__nexcore__faers_drug_events",
            "mcp__nexcore__faers_signal_check",
        ],
        category: "FDA FAERS Database",
        use_case: "Search FDA adverse events, compare drugs, check signals",
    },
    // Guidelines
    McpSuggestion {
        keywords: &[
            "ich guideline",
            "cioms",
            "ema gvp",
            "regulatory guideline",
            "pv guideline",
        ],
        tools: &[
            "mcp__nexcore__guidelines_search",
            "mcp__nexcore__guidelines_get",
            "mcp__nexcore__guidelines_pv_all",
        ],
        category: "Regulatory Guidelines",
        use_case: "Search ICH/CIOMS/EMA guidelines with full-text retrieval",
    },
    // Vigilance
    McpSuggestion {
        keywords: &[
            "safety margin",
            "risk score",
            "harm type",
            "theory of vigilance",
            "tov",
        ],
        tools: &[
            "mcp__nexcore__vigilance_safety_margin",
            "mcp__nexcore__vigilance_risk_score",
            "mcp__nexcore__vigilance_harm_types",
        ],
        category: "Theory of Vigilance",
        use_case: "Guardian-AV risk scoring and ToV axioms",
    },
    // Skills
    McpSuggestion {
        keywords: &["skill", "skills list", "find skill", "skill registry"],
        tools: &[
            "mcp__nexcore__skill_list",
            "mcp__nexcore__skill_get",
            "mcp__nexcore__skill_search_by_tag",
        ],
        category: "Skill Management",
        use_case: "Search and retrieve from 300+ Diamond-compliant skills",
    },
    McpSuggestion {
        keywords: &["validate skill", "skill compliance", "diamond validation"],
        tools: &["mcp__nexcore__skill_validate"],
        category: "Skill Validation",
        use_case: "Diamond v2 compliance checking",
    },
    // GCloud
    McpSuggestion {
        keywords: &[
            "gcloud",
            "google cloud",
            "gcp secret",
            "cloud run",
            "cloud storage",
        ],
        tools: &[
            "mcp__nexcore__gcloud_secrets_list",
            "mcp__nexcore__gcloud_run_services_list",
            "mcp__nexcore__gcloud_storage_ls",
        ],
        category: "Google Cloud",
        use_case: "Type-safe GCloud operations with safety checks",
    },
    // Principles
    McpSuggestion {
        keywords: &[
            "dalio",
            "first principles",
            "kiss principle",
            "decision making",
        ],
        tools: &[
            "mcp__nexcore__principles_list",
            "mcp__nexcore__principles_get",
            "mcp__nexcore__principles_search",
        ],
        category: "Decision Principles",
        use_case: "Retrieve Dalio principles, KISS, first-principles thinking",
    },
    // Spaced repetition
    McpSuggestion {
        keywords: &["spaced repetition", "fsrs", "anki", "learning schedule"],
        tools: &["mcp__nexcore__foundation_fsrs_review"],
        category: "Learning & Review",
        use_case: "FSRS-4.5 algorithm for optimal review scheduling",
    },
];

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_skip_prompt(),
    };

    // Load config for canary rollout (Phase 3 CTVP)
    let config = McpEfficacyConfig::load();

    // Check canary rollout - skip if not in rollout cohort
    if !config.should_track(&input.session_id) {
        exit_skip_prompt();
    }

    let session_id = &input.session_id;
    let prompt = match input.get_prompt() {
        Some(p) => p,
        None => exit_skip_prompt(),
    };

    let prompt_lower = prompt.to_lowercase();

    // Find matching suggestions using word boundary matching (built-in patterns)
    let matches: Vec<_> = MCP_SUGGESTIONS
        .iter()
        .filter(|s| s.keywords.iter().any(|k| matches_word(&prompt_lower, k)))
        .collect();

    // Also check custom keywords from config
    let custom_matches: Vec<_> = config
        .get_custom_keywords()
        .values()
        .filter(|m| {
            m.keywords
                .iter()
                .any(|k| matches_word(&prompt_lower, &k.to_lowercase()))
        })
        .collect();

    if matches.is_empty() && custom_matches.is_empty() {
        exit_skip_prompt();
    }

    // Collect all suggested tools (built-in + custom)
    let mut suggested_tools: Vec<String> = matches
        .iter()
        .flat_map(|m| m.tools.iter().map(|s| (*s).to_string()))
        .collect();
    suggested_tools.extend(custom_matches.iter().flat_map(|m| m.tools.clone()));

    let mut matched_keywords: Vec<String> = matches
        .iter()
        .flat_map(|m| {
            m.keywords
                .iter()
                .filter(|k| matches_word(&prompt_lower, k))
                .map(|s| (*s).to_string())
        })
        .collect();
    matched_keywords.extend(custom_matches.iter().flat_map(|m| {
        m.keywords
            .iter()
            .filter(|k| matches_word(&prompt_lower, &k.to_lowercase()))
            .cloned()
    }));

    let category = matches
        .first()
        .map(|m| m.category)
        .or_else(|| custom_matches.first().map(|m| m.category.as_str()))
        .unwrap_or("Unknown");

    // Record suggestion for efficacy tracking (Phase 2 CTVP)
    let _ = with_efficacy_registry(|r| {
        r.record_suggestion(
            session_id,
            suggested_tools.clone(),
            matched_keywords,
            category,
        );
    });

    // Build context message
    let mut context = String::new();
    context.push_str("\n🔧 **MCP TOOL SUGGESTIONS** ─────────────────────────────\n");
    context.push_str("   NexCore has optimized tools for this task:\n\n");

    // Built-in matches
    matches.iter().for_each(|m| {
        context.push_str(&format!("   **{}**\n", m.category));
        context.push_str(&format!("   Use case: {}\n", m.use_case));
        context.push_str("   Tools:\n");
        m.tools.iter().for_each(|t| {
            context.push_str(&format!("     • `{}`\n", t));
        });
        context.push('\n');
    });

    // Custom matches from config
    custom_matches.iter().for_each(|m| {
        context.push_str(&format!("   **{}**\n", m.category));
        if !m.use_case.is_empty() {
            context.push_str(&format!("   Use case: {}\n", m.use_case));
        }
        context.push_str("   Tools:\n");
        m.tools.iter().for_each(|t| {
            context.push_str(&format!("     • `{}`\n", t));
        });
        context.push('\n');
    });

    context.push_str("   ⚡ **PREFER MCP TOOLS** over manual implementations.\n");
    context.push_str("───────────────────────────────────────────────────────\n");

    exit_with_context(&context);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_word_boundary_matches_whole_word() {
        assert!(matches_word("calculate sha256 hash", "sha256"));
        assert!(matches_word("calculate hash value", "hash"));
        assert!(matches_word("hash this file", "hash"));
    }

    #[test]
    fn test_word_boundary_rejects_partial() {
        // "hash" should NOT match "hashtag" or "rehash"
        assert!(!matches_word("add a hashtag", "hash"));
        assert!(!matches_word("rehash the data", "hash"));
        // Note: "#hash" DOES match because # is not a word char
        // This is acceptable - the main false positives are avoided
    }

    #[test]
    fn test_multi_word_keywords() {
        // Multi-word keywords use contains (specific enough)
        assert!(matches_word("calculate edit distance", "edit distance"));
        assert!(matches_word("use chi-square test", "chi-square"));
    }

    #[test]
    fn test_case_sensitivity() {
        // matches_word expects lowercase input
        assert!(matches_word("calculate sha256", "sha256"));
        assert!(matches_word("use yaml parser", "yaml"));
    }
}
