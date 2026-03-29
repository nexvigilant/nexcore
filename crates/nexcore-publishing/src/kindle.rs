//! Kindle/KDP compliance checking.
//!
//! Validates a book project against Amazon Kindle Direct Publishing
//! requirements and best practices.

use crate::chapter::Chapter;
use crate::cover::CoverSpec;
use crate::metadata::BookMetadata;

/// KDP compliance report.
#[derive(Debug)]
pub struct KdpComplianceReport {
    /// Overall pass/fail.
    pub compliant: bool,
    /// Individual checks.
    pub checks: Vec<KdpCheck>,
    /// Recommendations (non-blocking).
    pub recommendations: Vec<String>,
}

/// A single KDP compliance check.
#[derive(Debug)]
pub struct KdpCheck {
    /// Check category.
    pub category: KdpCategory,
    /// Check name.
    pub name: String,
    /// Pass/fail.
    pub passed: bool,
    /// Detail message.
    pub message: String,
}

/// KDP check categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KdpCategory {
    Metadata,
    Content,
    Cover,
    Formatting,
}

impl std::fmt::Display for KdpCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Metadata => write!(f, "Metadata"),
            Self::Content => write!(f, "Content"),
            Self::Cover => write!(f, "Cover"),
            Self::Formatting => write!(f, "Formatting"),
        }
    }
}

/// Run full KDP compliance check.
pub fn check_kdp_compliance(
    metadata: &BookMetadata,
    chapters: &[Chapter],
    cover: Option<&CoverSpec>,
) -> KdpComplianceReport {
    let mut checks = Vec::new();
    let mut recommendations = Vec::new();

    // ─── Metadata checks ───

    checks.push(KdpCheck {
        category: KdpCategory::Metadata,
        name: "title_present".into(),
        passed: !metadata.title.trim().is_empty(),
        message: "Title is required".into(),
    });

    checks.push(KdpCheck {
        category: KdpCategory::Metadata,
        name: "title_length".into(),
        passed: metadata.title.len() <= 200,
        message: format!("Title length: {} chars (max 200)", metadata.title.len()),
    });

    checks.push(KdpCheck {
        category: KdpCategory::Metadata,
        name: "author_present".into(),
        passed: !metadata.authors.is_empty(),
        message: "At least one author is required".into(),
    });

    checks.push(KdpCheck {
        category: KdpCategory::Metadata,
        name: "language_set".into(),
        passed: !metadata.language.trim().is_empty(),
        message: "Language code is required".into(),
    });

    if metadata.description.is_none() {
        recommendations.push("Add a book description for better discoverability on Amazon".into());
    } else if let Some(ref desc) = metadata.description {
        checks.push(KdpCheck {
            category: KdpCategory::Metadata,
            name: "description_length".into(),
            passed: desc.len() <= 4000,
            message: format!("Description: {} chars (max 4000)", desc.len()),
        });
    }

    if metadata.subjects.is_empty() {
        recommendations
            .push("Add subject keywords (up to 7) for Amazon search optimization".into());
    }

    if metadata.bisac_codes.is_empty() {
        recommendations.push("Add BISAC category codes for proper Amazon categorization".into());
    }

    // ─── Content checks ───

    checks.push(KdpCheck {
        category: KdpCategory::Content,
        name: "has_chapters".into(),
        passed: !chapters.is_empty(),
        message: format!("{} chapter(s) found", chapters.len()),
    });

    let total_words: usize = chapters.iter().map(|c| c.word_count()).sum();
    checks.push(KdpCheck {
        category: KdpCategory::Content,
        name: "minimum_content".into(),
        passed: total_words >= 2500,
        message: format!("{total_words} words (KDP minimum ~2500 for books)"),
    });

    // Check for empty chapters
    let empty_chapters: Vec<_> = chapters.iter().filter(|c| c.word_count() < 10).collect();
    if !empty_chapters.is_empty() {
        checks.push(KdpCheck {
            category: KdpCategory::Content,
            name: "no_empty_chapters".into(),
            passed: false,
            message: format!(
                "{} chapter(s) have fewer than 10 words",
                empty_chapters.len()
            ),
        });
    }

    // Check for has_toc (KDP requires navigable TOC)
    checks.push(KdpCheck {
        category: KdpCategory::Content,
        name: "has_toc".into(),
        passed: chapters.len() > 1, // Single-chapter books technically don't need TOC
        message: "Table of contents will be generated from chapter headings".into(),
    });

    // ─── Cover checks ───

    if let Some(cover) = cover {
        let cover_result = cover.validate_kindle();
        for check in cover_result.checks {
            checks.push(KdpCheck {
                category: KdpCategory::Cover,
                name: check.name,
                passed: check.passed,
                message: check.message,
            });
        }
    } else {
        recommendations.push(
            "No cover image provided. KDP strongly recommends a cover (1600x2560 px ideal)".into(),
        );
    }

    // ─── Formatting checks ───

    // Check for excessively long paragraphs (readability)
    let long_paras = chapters
        .iter()
        .flat_map(|c| c.content.split("</p>"))
        .filter(|p| p.split_whitespace().count() > 500)
        .count();
    if long_paras > 0 {
        recommendations.push(format!(
            "{long_paras} paragraph(s) exceed 500 words — consider breaking up for readability"
        ));
    }

    let compliant = checks.iter().all(|c| c.passed);

    KdpComplianceReport {
        compliant,
        checks,
        recommendations,
    }
}

/// Summary statistics for the compliance report.
impl KdpComplianceReport {
    /// Count of passing checks.
    pub fn passed_count(&self) -> usize {
        self.checks.iter().filter(|c| c.passed).count()
    }

    /// Count of failing checks.
    pub fn failed_count(&self) -> usize {
        self.checks.iter().filter(|c| !c.passed).count()
    }

    /// Format as a human-readable report.
    pub fn to_report(&self) -> String {
        let mut report = String::new();
        report.push_str(&format!(
            "KDP Compliance: {} ({}/{} checks passed)\n\n",
            if self.compliant { "PASS" } else { "FAIL" },
            self.passed_count(),
            self.checks.len(),
        ));

        for check in &self.checks {
            let icon = if check.passed { "+" } else { "-" };
            report.push_str(&format!(
                "  [{icon}] {cat}/{name}: {msg}\n",
                cat = check.category,
                name = check.name,
                msg = check.message,
            ));
        }

        if !self.recommendations.is_empty() {
            report.push_str("\nRecommendations:\n");
            for rec in &self.recommendations {
                report.push_str(&format!("  * {rec}\n"));
            }
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_metadata() -> BookMetadata {
        let mut meta = BookMetadata::new("Test Book", "Author Name", "en");
        meta.description = Some("A test book description.".into());
        meta.subjects = vec!["Testing".into()];
        meta
    }

    fn sample_chapters(word_count: usize) -> Vec<Chapter> {
        let content: String = (0..word_count).map(|i| format!("word{i} ")).collect();
        let wrapped = format!("<p>{content}</p>");
        vec![
            Chapter::new("Chapter 1", &wrapped, 0),
            Chapter::new("Chapter 2", &wrapped, 1),
        ]
    }

    #[test]
    fn test_compliant_book() {
        let meta = sample_metadata();
        let chapters = sample_chapters(5000);
        let report = check_kdp_compliance(&meta, &chapters, None);
        assert!(report.compliant, "Report:\n{}", report.to_report());
    }

    #[test]
    fn test_empty_title_fails() {
        let meta = BookMetadata::new("", "Author", "en");
        let chapters = sample_chapters(3000);
        let report = check_kdp_compliance(&meta, &chapters, None);
        assert!(!report.compliant);
        assert!(
            report
                .checks
                .iter()
                .any(|c| c.name == "title_present" && !c.passed),
            "Should fail title_present check"
        );
    }

    #[test]
    fn test_too_few_words_fails() {
        let meta = sample_metadata();
        let chapters = vec![Chapter::new("Ch1", "<p>Short.</p>", 0)];
        let report = check_kdp_compliance(&meta, &chapters, None);
        assert!(!report.compliant);
    }

    #[test]
    fn test_report_formatting() {
        let meta = sample_metadata();
        let chapters = sample_chapters(3000);
        let report = check_kdp_compliance(&meta, &chapters, None);
        let text = report.to_report();
        assert!(text.contains("KDP Compliance: PASS"));
        assert!(text.contains("[+]"));
    }

    #[test]
    fn test_missing_description_recommendation() {
        let meta = BookMetadata::new("Title", "Author", "en");
        let chapters = sample_chapters(3000);
        let report = check_kdp_compliance(&meta, &chapters, None);
        assert!(
            report
                .recommendations
                .iter()
                .any(|r| r.contains("description")),
            "Should recommend adding description"
        );
    }
}
