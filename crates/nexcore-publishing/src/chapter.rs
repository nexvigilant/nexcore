//! Chapter extraction, ordering, and content representation.

use serde::{Deserialize, Serialize};

/// A single chapter or section of the book.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    /// Chapter title (extracted from heading or assigned).
    pub title: String,
    /// Chapter content as cleaned XHTML body fragment.
    pub content: String,
    /// Zero-based order in the book.
    pub order: usize,
    /// Heading level that triggered this chapter split (1 = H1, 2 = H2).
    pub level: HeadingLevel,
    /// Optional filename override for the EPUB content document.
    pub filename: Option<String>,
}

/// Heading levels used for chapter detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum HeadingLevel {
    /// H1 — top-level chapter break.
    H1 = 1,
    /// H2 — section within a chapter.
    H2 = 2,
    /// H3 — subsection.
    H3 = 3,
}

impl HeadingLevel {
    /// Parse from a DOCX paragraph style name.
    pub fn from_style(style: &str) -> Option<Self> {
        let lower = style.to_lowercase();
        if lower.contains("heading1") || lower == "heading 1" || lower == "title" {
            Some(Self::H1)
        } else if lower.contains("heading2") || lower == "heading 2" || lower == "subtitle" {
            Some(Self::H2)
        } else if lower.contains("heading3") || lower == "heading 3" {
            Some(Self::H3)
        } else {
            None
        }
    }

    /// Returns the XHTML tag name.
    pub fn tag(&self) -> &'static str {
        match self {
            Self::H1 => "h1",
            Self::H2 => "h2",
            Self::H3 => "h3",
        }
    }
}

impl Chapter {
    /// Create a new chapter.
    pub fn new(title: impl Into<String>, content: impl Into<String>, order: usize) -> Self {
        Self {
            title: title.into(),
            content: content.into(),
            order,
            level: HeadingLevel::H1,
            filename: None,
        }
    }

    /// Generate the EPUB content document filename.
    pub fn epub_filename(&self) -> String {
        if let Some(ref name) = self.filename {
            return name.clone();
        }
        format!("chapter_{:03}.xhtml", self.order + 1)
    }

    /// Wrap chapter content in a complete XHTML document.
    pub fn to_xhtml(&self, css_path: &str) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" xml:lang="en">
<head>
  <meta charset="UTF-8" />
  <title>{title}</title>
  <link rel="stylesheet" type="text/css" href="{css}" />
</head>
<body>
  <{tag}>{title}</{tag}>
{content}
</body>
</html>"#,
            title = xml_escape(&self.title),
            css = css_path,
            tag = self.level.tag(),
            content = self.content,
        )
    }

    /// Word count estimate for the chapter content.
    pub fn word_count(&self) -> usize {
        self.content.split_whitespace().count()
    }
}

/// Split raw paragraphs into chapters based on heading detection.
///
/// Each heading at `split_level` or above starts a new chapter.
/// Content before the first heading goes into a "Front Matter" chapter.
pub fn split_into_chapters(
    paragraphs: &[(Option<HeadingLevel>, String)],
    split_level: HeadingLevel,
) -> Vec<Chapter> {
    let mut chapters = Vec::new();
    let mut current_title = String::from("Front Matter");
    let mut current_content = String::new();
    let mut current_level = HeadingLevel::H1;
    let mut has_content = false;

    for (heading, text) in paragraphs {
        if let Some(level) = heading {
            if *level <= split_level {
                // Save previous chapter if it has content
                if has_content {
                    chapters.push(Chapter {
                        title: current_title.clone(),
                        content: current_content.trim().to_string(),
                        order: chapters.len(),
                        level: current_level,
                        filename: None,
                    });
                }
                current_title = text.clone();
                current_content = String::new();
                current_level = *level;
                has_content = false;
                continue;
            }
            // Lower heading — include as subheading in current chapter
            current_content.push_str(&format!(
                "  <{tag}>{text}</{tag}>\n",
                tag = level.tag(),
                text = xml_escape(text)
            ));
            has_content = true;
        } else if !text.trim().is_empty() {
            current_content.push_str(&format!("  <p>{}</p>\n", xml_escape(text)));
            has_content = true;
        }
    }

    // Final chapter
    if has_content {
        chapters.push(Chapter {
            title: current_title,
            content: current_content.trim().to_string(),
            order: chapters.len(),
            level: current_level,
            filename: None,
        });
    }

    chapters
}

/// Minimal XML escaping for text content.
pub(crate) fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heading_level_from_style() {
        assert_eq!(HeadingLevel::from_style("Heading1"), Some(HeadingLevel::H1));
        assert_eq!(
            HeadingLevel::from_style("heading 2"),
            Some(HeadingLevel::H2)
        );
        assert_eq!(HeadingLevel::from_style("Title"), Some(HeadingLevel::H1));
        assert_eq!(HeadingLevel::from_style("Normal"), None);
    }

    #[test]
    fn test_chapter_filename() {
        let ch = Chapter::new("Intro", "content", 0);
        assert_eq!(ch.epub_filename(), "chapter_001.xhtml");

        let mut ch2 = Chapter::new("Custom", "content", 5);
        ch2.filename = Some("dedication.xhtml".into());
        assert_eq!(ch2.epub_filename(), "dedication.xhtml");
    }

    #[test]
    fn test_split_into_chapters() {
        let paragraphs = vec![
            (Some(HeadingLevel::H1), "Chapter One".into()),
            (None, "First paragraph.".into()),
            (None, "Second paragraph.".into()),
            (Some(HeadingLevel::H1), "Chapter Two".into()),
            (None, "Third paragraph.".into()),
        ];
        let chapters = split_into_chapters(&paragraphs, HeadingLevel::H1);
        assert_eq!(chapters.len(), 2);
        assert_eq!(chapters[0].title, "Chapter One");
        assert_eq!(chapters[1].title, "Chapter Two");
        assert!(chapters[0].content.contains("First paragraph"));
        assert!(chapters[1].content.contains("Third paragraph"));
    }

    #[test]
    fn test_front_matter_chapter() {
        let paragraphs = vec![
            (None, "Some text before any heading.".into()),
            (Some(HeadingLevel::H1), "Real Chapter".into()),
            (None, "Chapter content.".into()),
        ];
        let chapters = split_into_chapters(&paragraphs, HeadingLevel::H1);
        assert_eq!(chapters.len(), 2);
        assert_eq!(chapters[0].title, "Front Matter");
        assert_eq!(chapters[1].title, "Real Chapter");
    }

    #[test]
    fn test_xml_escape() {
        assert_eq!(xml_escape("A & B"), "A &amp; B");
        assert_eq!(xml_escape("<tag>"), "&lt;tag&gt;");
    }

    #[test]
    fn test_word_count() {
        let ch = Chapter::new("Test", "one two three four five", 0);
        assert_eq!(ch.word_count(), 5);
    }
}
