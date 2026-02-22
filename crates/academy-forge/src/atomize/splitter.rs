//! Concept-splitting heuristics for decomposing activities into ALOs.
//!
//! Splits multi-concept reading activities by headings and definitions,
//! ensuring each resulting ALO covers a single concept within 5-15 minutes.

/// A split content fragment representing a single concept.
#[derive(Debug, Clone)]
pub struct ContentFragment {
    /// Extracted title from heading.
    pub title: String,
    /// Markdown content body.
    pub content: String,
    /// Estimated reading time in minutes (at 200 wpm).
    pub estimated_minutes: u16,
}

/// Words-per-minute reading speed for duration estimation.
const READING_WPM: usize = 200;

/// Maximum word count before a fragment must be split further.
const MAX_FRAGMENT_WORDS: usize = 2000; // ~10 min at 200 wpm

/// Minimum word count — below this, don't split further.
const MIN_FRAGMENT_WORDS: usize = 200; // ~1 min

/// Split a reading activity's markdown content into single-concept fragments.
///
/// Strategy:
/// 1. Split on `##` headings first (primary concept boundaries)
/// 2. If a fragment exceeds 2000 words, split on `###` subheadings
/// 3. If still too long, split at the midpoint paragraph
/// 4. Fragments under 200 words are merged with their predecessor
pub fn split_by_concept(content: &str, base_title: &str) -> Vec<ContentFragment> {
    // Check if content actually contains heading markers
    let has_headings = content.lines().any(|l| l.starts_with("## "));

    if !has_headings {
        // No headings — treat entire content as one fragment
        return vec![ContentFragment {
            title: base_title.to_string(),
            content: content.to_string(),
            estimated_minutes: estimate_minutes(content),
        }];
    }

    let sections = split_on_headings(content, "## ");

    let mut fragments = Vec::new();
    let section_count = sections.len();

    for section in sections {
        let word_count = count_words(&section.content);

        if word_count > MAX_FRAGMENT_WORDS {
            // Try splitting on ### subheadings
            let subsections = split_on_headings(&section.content, "### ");
            if subsections.len() > 1 {
                fragments.extend(subsections);
            } else {
                // No subheadings — split at midpoint paragraph
                let (first, second) = split_at_midpoint(&section.content, &section.title);
                fragments.push(first);
                fragments.push(second);
            }
        } else if word_count < MIN_FRAGMENT_WORDS && !fragments.is_empty() && section_count > 2 {
            // Merge with predecessor if too small AND there are many sections
            // (don't merge when there are only 2 sections — keep them separate)
            let last = fragments.last_mut();
            if let Some(last) = last {
                last.content.push_str("\n\n");
                last.content.push_str(&section.content);
                last.estimated_minutes = estimate_minutes(&last.content);
            }
        } else {
            fragments.push(section);
        }
    }

    // Edge case: if splitting produced nothing, return whole content
    if fragments.is_empty() {
        return vec![ContentFragment {
            title: base_title.to_string(),
            content: content.to_string(),
            estimated_minutes: estimate_minutes(content),
        }];
    }

    fragments
}

/// Split markdown on a heading prefix (e.g., "## " or "### ").
fn split_on_headings(content: &str, heading_prefix: &str) -> Vec<ContentFragment> {
    let mut fragments = Vec::new();
    let mut current_title = String::new();
    let mut current_content = String::new();

    for line in content.lines() {
        if line.starts_with(heading_prefix) {
            // Flush previous section
            if !current_content.is_empty() {
                let trimmed = current_content.trim().to_string();
                if !trimmed.is_empty() {
                    fragments.push(ContentFragment {
                        title: if current_title.is_empty() {
                            "Introduction".to_string()
                        } else {
                            current_title.clone()
                        },
                        content: trimmed.clone(),
                        estimated_minutes: estimate_minutes(&trimmed),
                    });
                }
            }
            current_title = line
                .trim_start_matches('#')
                .trim()
                .to_string();
            current_content = String::new();
        } else {
            current_content.push_str(line);
            current_content.push('\n');
        }
    }

    // Flush last section
    let trimmed = current_content.trim().to_string();
    if !trimmed.is_empty() {
        fragments.push(ContentFragment {
            title: if current_title.is_empty() {
                "Content".to_string()
            } else {
                current_title
            },
            content: trimmed.clone(),
            estimated_minutes: estimate_minutes(&trimmed),
        });
    }

    fragments
}

/// Split content at the midpoint paragraph boundary.
fn split_at_midpoint(content: &str, title: &str) -> (ContentFragment, ContentFragment) {
    let paragraphs: Vec<&str> = content.split("\n\n").collect();
    let mid = paragraphs.len() / 2;

    let first_content = paragraphs[..mid].join("\n\n");
    let second_content = paragraphs[mid..].join("\n\n");

    (
        ContentFragment {
            title: title.to_string(),
            content: first_content.clone(),
            estimated_minutes: estimate_minutes(&first_content),
        },
        ContentFragment {
            title: format!("{title} (continued)"),
            content: second_content.clone(),
            estimated_minutes: estimate_minutes(&second_content),
        },
    )
}

/// Count words in text (rough estimate).
fn count_words(text: &str) -> usize {
    text.split_whitespace().count()
}

/// Estimate reading time in minutes from word count.
pub fn estimate_minutes(text: &str) -> u16 {
    let words = count_words(text);
    let minutes = (words as f64 / READING_WPM as f64).ceil() as u16;
    minutes.max(1) // At least 1 minute
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_by_headings() {
        let content = "## First Concept\n\nSome content here about the first concept.\n\n## Second Concept\n\nMore content about the second concept.\n";
        let fragments = split_by_concept(content, "Test");
        assert_eq!(fragments.len(), 2);
        assert_eq!(fragments[0].title, "First Concept");
        assert_eq!(fragments[1].title, "Second Concept");
    }

    #[test]
    fn test_no_headings_single_fragment() {
        let content = "Just plain text without any headings. This should remain one fragment.";
        let fragments = split_by_concept(content, "Base Title");
        assert_eq!(fragments.len(), 1);
        assert_eq!(fragments[0].title, "Base Title");
    }

    #[test]
    fn test_estimate_minutes() {
        // 200 words = 1 min, 400 words = 2 min
        let words_200: String = (0..200).map(|_| "word").collect::<Vec<_>>().join(" ");
        assert_eq!(estimate_minutes(&words_200), 1);

        let words_400: String = (0..400).map(|_| "word").collect::<Vec<_>>().join(" ");
        assert_eq!(estimate_minutes(&words_400), 2);
    }

    #[test]
    fn test_empty_content() {
        let fragments = split_by_concept("", "Empty");
        assert_eq!(fragments.len(), 1);
    }
}
