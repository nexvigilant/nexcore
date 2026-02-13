//! # Concept Grep — Deterministic Concept Expansion
//!
//! Expands a concept string into all deterministic search variants:
//! case forms, singular/plural, abbreviation prefix, truncated stems,
//! and optional section markers.
//!
//! Zero I/O. Pure function. 100% deterministic.

use std::collections::BTreeSet;
use std::fmt;

/// Result of expanding a concept into search patterns.
#[derive(Debug, Clone)]
pub struct ConceptExpansion {
    /// Original input concept
    pub original: String,
    /// All generated pattern variants (deduplicated, sorted)
    pub patterns: Vec<String>,
    /// Combined OR regex pattern
    pub regex: String,
    /// Section marker patterns (only if requested)
    pub sections: Vec<String>,
}

impl fmt::Display for ConceptExpansion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.regex)
    }
}

/// Expand a concept into all deterministic search variants.
///
/// # Rules (5 deterministic)
///
/// 1. **Case variants** — lowercase, UPPERCASE, Title Case, `camelCase`, `snake_case`, `kebab-case`
/// 2. **Singular/plural** — append/strip `s`, `es`, `ies↔y`
/// 3. **Abbreviation** — first-letter acronym for multi-word concepts
/// 4. **Truncated stem** — strip common suffixes (`-tion`, `-ing`, `-ment`, `-ity`, `-ness`, `-able`)
/// 5. **Section markers** — `## {concept}`, `# {concept}`, `### {concept}` (when `include_sections` is true)
///
/// # Arguments
///
/// * `concept` — The concept to expand (e.g., "Signal Detection")
/// * `include_sections` — Whether to include markdown section marker patterns
#[must_use]
pub fn expand_concept(concept: &str, include_sections: bool) -> ConceptExpansion {
    let concept = concept.trim();
    if concept.is_empty() {
        return ConceptExpansion {
            original: String::new(),
            patterns: Vec::new(),
            regex: String::new(),
            sections: Vec::new(),
        };
    }

    let mut patterns = BTreeSet::new();

    // Always include original
    patterns.insert(concept.to_string());

    // Rule 1: Case variants
    add_case_variants(concept, &mut patterns);

    // Rule 2: Singular/plural
    add_plural_singular(concept, &mut patterns);

    // Rule 3: Abbreviation (multi-word only)
    add_abbreviation(concept, &mut patterns);

    // Rule 4: Truncated stems
    add_stems(concept, &mut patterns);

    // Rule 5: Section markers
    let sections = if include_sections {
        generate_section_markers(concept)
    } else {
        Vec::new()
    };

    // Add section patterns to the set too
    for s in &sections {
        patterns.insert(s.clone());
    }

    let patterns: Vec<String> = patterns.into_iter().collect();

    // Build combined regex: escape each pattern, join with |
    let regex = patterns
        .iter()
        .map(|p| regex::escape(p))
        .collect::<Vec<_>>()
        .join("|");

    ConceptExpansion {
        original: concept.to_string(),
        patterns,
        regex,
        sections,
    }
}

// ============================================================================
// Rule 1: Case Variants
// ============================================================================

fn add_case_variants(concept: &str, out: &mut BTreeSet<String>) {
    let lower = concept.to_lowercase();
    let upper = concept.to_uppercase();
    let title = to_title_case(concept);
    let camel = to_camel_case(concept);
    let snake = to_snake_case(concept);
    let kebab = to_kebab_case(concept);

    out.insert(lower);
    out.insert(upper);
    out.insert(title);
    out.insert(camel);
    out.insert(snake);
    out.insert(kebab);
}

fn to_title_case(s: &str) -> String {
    s.split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => {
                    let upper: String = c.to_uppercase().collect();
                    let rest: String = chars.collect::<String>().to_lowercase();
                    format!("{upper}{rest}")
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn to_camel_case(s: &str) -> String {
    let words: Vec<&str> = s
        .split(|c: char| c.is_whitespace() || c == '-' || c == '_')
        .filter(|w| !w.is_empty())
        .collect();
    if words.is_empty() {
        return String::new();
    }
    let mut result = words[0].to_lowercase();
    for word in &words[1..] {
        let mut chars = word.chars();
        if let Some(c) = chars.next() {
            result.extend(c.to_uppercase());
            result.push_str(&chars.collect::<String>().to_lowercase());
        }
    }
    result
}

fn to_snake_case(s: &str) -> String {
    s.split(|c: char| c.is_whitespace() || c == '-')
        .filter(|w| !w.is_empty())
        .map(|w| w.to_lowercase())
        .collect::<Vec<_>>()
        .join("_")
}

fn to_kebab_case(s: &str) -> String {
    s.split(|c: char| c.is_whitespace() || c == '_')
        .filter(|w| !w.is_empty())
        .map(|w| w.to_lowercase())
        .collect::<Vec<_>>()
        .join("-")
}

// ============================================================================
// Rule 2: Singular/Plural
// ============================================================================

fn add_plural_singular(concept: &str, out: &mut BTreeSet<String>) {
    let lower = concept.to_lowercase();
    let words: Vec<&str> = lower.split_whitespace().collect();
    if words.is_empty() {
        return;
    }

    // Operate on last word (the noun)
    let last = words[words.len() - 1];
    let prefix: String = if words.len() > 1 {
        let head: Vec<&str> = words[..words.len() - 1].to_vec();
        format!("{} ", head.join(" "))
    } else {
        String::new()
    };

    // Generate both singular and plural forms
    for form in pluralize(last) {
        out.insert(format!("{prefix}{form}"));
    }
}

/// Returns both singular and plural forms of a word.
fn pluralize(word: &str) -> Vec<String> {
    let mut forms = vec![word.to_string()];

    if let Some(stem) = word.strip_suffix("ies") {
        // frequencies -> frequency
        forms.push(format!("{stem}y"));
    } else if let Some(stem) = word.strip_suffix('y') {
        if !word.ends_with("ey")
            && !word.ends_with("ay")
            && !word.ends_with("oy")
            && !word.ends_with("uy")
            && word.len() > 2
        {
            // frequency -> frequencies
            forms.push(format!("{stem}ies"));
        } else {
            forms.push(format!("{word}s"));
        }
    } else if word.ends_with("ses")
        || word.ends_with("xes")
        || word.ends_with("zes")
        || word.ends_with("ches")
        || word.ends_with("shes")
    {
        // processes -> process
        if let Some(stem) = word.strip_suffix("es") {
            forms.push(stem.to_string());
        }
    } else if let Some(stem) = word.strip_suffix('s') {
        if !word.ends_with("ss") {
            // signals -> signal
            forms.push(stem.to_string());
        } else {
            forms.push(format!("{word}es"));
        }
    } else if word.ends_with("ch")
        || word.ends_with("sh")
        || word.ends_with('x')
        || word.ends_with('z')
    {
        // match -> matches
        forms.push(format!("{word}es"));
    } else {
        // signal -> signals
        forms.push(format!("{word}s"));
    }

    forms
}

// ============================================================================
// Rule 3: Abbreviation
// ============================================================================

fn add_abbreviation(concept: &str, out: &mut BTreeSet<String>) {
    let words: Vec<&str> = concept
        .split(|c: char| c.is_whitespace() || c == '-' || c == '_')
        .filter(|w| !w.is_empty())
        .collect();

    if words.len() < 2 {
        // Also add prefix abbreviation for single long words (3+ chars)
        // Char-aware slicing: safe for multi-byte Unicode (e.g., "Üntersuchung")
        let lower = concept.to_lowercase();
        if lower.chars().count() >= 6 {
            out.insert(lower.chars().take(3).collect());
        }
        return;
    }

    // First-letter acronym (uppercase)
    let acronym: String = words
        .iter()
        .filter_map(|w| w.chars().next())
        .map(|c| c.to_ascii_uppercase())
        .collect();

    if acronym.len() >= 2 {
        out.insert(acronym);
    }
}

// ============================================================================
// Rule 4: Truncated Stems
// ============================================================================

// Ordered shortest-first within groups so shorter suffixes that yield
// better stems get matched before longer ones (e.g., "ion" before "tion"
// gives "detect" from "detection" instead of "detec").
const SUFFIXES: &[&str] = &[
    "ment", "ness", "able", "ible", "ance", "ence", "ical", "ious", "eous", "ling", "ity", "ing",
    "ful", "ous", "ive", "ism", "ist", "ure", "ant", "ent", "ary", "ory", "ion", "al", "ly", "er",
    "ed",
];

fn add_stems(concept: &str, out: &mut BTreeSet<String>) {
    let lower = concept.to_lowercase();
    let words: Vec<&str> = lower.split_whitespace().collect();

    for word in &words {
        for suffix in SUFFIXES {
            if let Some(stem) = word.strip_suffix(suffix) {
                if stem.chars().count() >= 3 {
                    out.insert(stem.to_string());
                }
                break; // Only strip the first matched suffix
            }
        }
    }
}

// ============================================================================
// Rule 5: Section Markers
// ============================================================================

fn generate_section_markers(concept: &str) -> Vec<String> {
    let title = to_title_case(concept);
    let lower = concept.to_lowercase();
    vec![
        format!("# {title}"),
        format!("## {title}"),
        format!("### {title}"),
        format!("# {lower}"),
        format!("## {lower}"),
        format!("### {lower}"),
    ]
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- Rule 1: Case Variants ---

    #[test]
    fn test_case_variants_signal_detection() {
        let exp = expand_concept("Signal Detection", false);
        assert!(exp.patterns.contains(&"signal detection".to_string()));
        assert!(exp.patterns.contains(&"SIGNAL DETECTION".to_string()));
        assert!(exp.patterns.contains(&"Signal Detection".to_string()));
        assert!(exp.patterns.contains(&"signalDetection".to_string()));
        assert!(exp.patterns.contains(&"signal_detection".to_string()));
        assert!(exp.patterns.contains(&"signal-detection".to_string()));
    }

    #[test]
    fn test_case_variants_count() {
        let exp = expand_concept("Hello World", false);
        // At least 6 case forms should exist
        let case_forms = [
            "hello world",
            "HELLO WORLD",
            "Hello World",
            "helloWorld",
            "hello_world",
            "hello-world",
        ];
        for form in &case_forms {
            assert!(
                exp.patterns.contains(&form.to_string()),
                "Missing case form: {form}"
            );
        }
    }

    // --- Rule 2: Singular/Plural ---

    #[test]
    fn test_plural_signals() {
        let exp = expand_concept("signal", false);
        assert!(exp.patterns.contains(&"signals".to_string()));
    }

    #[test]
    fn test_singular_from_plural() {
        let exp = expand_concept("signals", false);
        assert!(exp.patterns.contains(&"signal".to_string()));
    }

    #[test]
    fn test_ies_plural() {
        let exp = expand_concept("frequency", false);
        assert!(
            exp.patterns.contains(&"frequencies".to_string()),
            "patterns: {:?}",
            exp.patterns
        );
    }

    #[test]
    fn test_ies_singular() {
        let exp = expand_concept("frequencies", false);
        assert!(
            exp.patterns.contains(&"frequency".to_string()),
            "patterns: {:?}",
            exp.patterns
        );
    }

    #[test]
    fn test_plural_multiword() {
        let exp = expand_concept("conservation laws", false);
        assert!(exp.patterns.contains(&"conservation law".to_string()));
    }

    // --- Rule 3: Abbreviation ---

    #[test]
    fn test_abbreviation_multiword() {
        let exp = expand_concept("Signal Detection", false);
        assert!(
            exp.patterns.contains(&"SD".to_string()),
            "patterns: {:?}",
            exp.patterns
        );
    }

    #[test]
    fn test_abbreviation_three_words() {
        let exp = expand_concept("adverse drug reaction", false);
        assert!(exp.patterns.contains(&"ADR".to_string()));
    }

    #[test]
    fn test_abbreviation_long_single_word() {
        let exp = expand_concept("pharmacovigilance", false);
        assert!(
            exp.patterns.contains(&"pha".to_string()),
            "patterns: {:?}",
            exp.patterns
        );
    }

    // --- Rule 4: Truncated Stems ---

    #[test]
    fn test_stem_detection() {
        let exp = expand_concept("detection", false);
        assert!(
            exp.patterns.contains(&"detect".to_string()),
            "patterns: {:?}",
            exp.patterns
        );
    }

    #[test]
    fn test_stem_processing() {
        let exp = expand_concept("processing", false);
        assert!(
            exp.patterns.contains(&"process".to_string()),
            "patterns: {:?}",
            exp.patterns
        );
    }

    #[test]
    fn test_stem_management() {
        let exp = expand_concept("management", false);
        assert!(
            exp.patterns.contains(&"manage".to_string()),
            "patterns: {:?}",
            exp.patterns
        );
    }

    #[test]
    fn test_stem_in_multiword() {
        let exp = expand_concept("Signal Detection", false);
        assert!(
            exp.patterns.contains(&"detect".to_string()),
            "patterns: {:?}",
            exp.patterns
        );
    }

    // --- Rule 5: Section Markers ---

    #[test]
    fn test_section_markers() {
        let exp = expand_concept("Signal Detection", true);
        assert!(exp.sections.contains(&"## Signal Detection".to_string()));
        assert!(exp.sections.contains(&"# Signal Detection".to_string()));
        assert!(exp.sections.contains(&"### Signal Detection".to_string()));
        assert!(exp.sections.contains(&"## signal detection".to_string()));
    }

    #[test]
    fn test_no_sections_by_default() {
        let exp = expand_concept("Signal Detection", false);
        assert!(exp.sections.is_empty());
    }

    // --- Regex ---

    #[test]
    fn test_regex_compiles() {
        let exp = expand_concept("Signal Detection", true);
        let re = regex::Regex::new(&exp.regex);
        assert!(re.is_ok(), "Regex failed to compile: {:?}", re.err());
    }

    #[test]
    fn test_regex_matches_original() {
        let exp = expand_concept("Signal Detection", false);
        let re = regex::Regex::new(&exp.regex).unwrap_or_else(|_| unreachable!());
        assert!(re.is_match("Signal Detection"));
        assert!(re.is_match("signal detection"));
        assert!(re.is_match("SIGNAL DETECTION"));
        assert!(re.is_match("signal_detection"));
    }

    // --- Edge Cases ---

    #[test]
    fn test_empty_input() {
        let exp = expand_concept("", false);
        assert!(exp.patterns.is_empty());
        assert!(exp.regex.is_empty());
    }

    #[test]
    fn test_single_char() {
        let exp = expand_concept("x", false);
        assert!(exp.patterns.contains(&"x".to_string()));
        assert!(exp.patterns.contains(&"X".to_string()));
    }

    #[test]
    fn test_hyphenated_term() {
        let exp = expand_concept("dose-response", false);
        assert!(exp.patterns.contains(&"dose_response".to_string()));
        assert!(exp.patterns.contains(&"doseResponse".to_string()));
        assert!(exp.patterns.contains(&"DR".to_string()));
    }

    #[test]
    fn test_acronym_input() {
        let exp = expand_concept("PRR", false);
        assert!(exp.patterns.contains(&"PRR".to_string()));
        assert!(exp.patterns.contains(&"prr".to_string()));
    }

    #[test]
    fn test_display_trait() {
        let exp = expand_concept("test", false);
        let display = format!("{exp}");
        assert!(!display.is_empty());
    }

    #[test]
    fn test_whitespace_trimmed() {
        let exp = expand_concept("  signal  ", false);
        assert_eq!(exp.original, "signal");
    }

    #[test]
    fn test_deterministic_output() {
        let a = expand_concept("Signal Detection", true);
        let b = expand_concept("Signal Detection", true);
        assert_eq!(a.patterns, b.patterns);
        assert_eq!(a.regex, b.regex);
        assert_eq!(a.sections, b.sections);
    }

    #[test]
    fn test_patterns_are_sorted_and_unique() {
        let exp = expand_concept("Signal Detection", true);
        let mut sorted = exp.patterns.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(exp.patterns, sorted);
    }
}
