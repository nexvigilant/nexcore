//! EPUB reader — extract content from existing `.epub` files.
//!
//! Parses an EPUB archive and returns structured book data:
//! metadata, table of contents, and chapter content as plain text or HTML.

use std::io::{Cursor, Read as _};
use std::path::Path;

use quick_xml::Reader;
use quick_xml::events::Event;
use zip::ZipArchive;

use crate::error::{PublishingError, Result};
use crate::metadata::BookMetadata;

/// A parsed EPUB book ready for reading.
#[derive(Debug)]
pub struct EpubReader {
    /// Book metadata from the OPF package document.
    pub metadata: BookMetadata,
    /// Chapters in spine order.
    pub chapters: Vec<ReadChapter>,
    /// Cover image bytes (if present).
    pub cover: Option<Vec<u8>>,
    /// Total word count across all chapters.
    pub total_words: usize,
}

/// A chapter extracted from an EPUB.
#[derive(Debug)]
pub struct ReadChapter {
    /// Chapter title (from first heading or filename).
    pub title: String,
    /// Chapter content as HTML.
    pub html: String,
    /// Chapter content as plain text (HTML stripped).
    pub text: String,
    /// Word count.
    pub word_count: usize,
    /// Original href in the EPUB.
    pub href: String,
    /// Order in spine.
    pub order: usize,
}

impl EpubReader {
    /// Open and parse an EPUB file from disk.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let data = std::fs::read(path.as_ref())?;
        Self::from_bytes(&data)
    }

    /// Parse an EPUB from raw bytes.
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        let cursor = Cursor::new(data);
        let mut archive = ZipArchive::new(cursor)
            .map_err(|e| PublishingError::Zip(format!("Invalid EPUB archive: {e}")))?;

        // Find OPF path from container.xml
        let opf_path = find_opf_path(&mut archive)?;
        let opf_dir = if let Some(pos) = opf_path.rfind('/') {
            &opf_path[..=pos]
        } else {
            ""
        };

        // Parse OPF
        let opf_xml = read_zip_entry(&mut archive, &opf_path)?;
        let (metadata, manifest, spine, cover_id) = parse_opf(&opf_xml)?;

        // Read cover image
        let cover = if let Some(ref cid) = cover_id {
            if let Some(item) = manifest.iter().find(|m| m.id == *cid) {
                let cover_path = format!("{}{}", opf_dir, item.href);
                read_zip_bytes(&mut archive, &cover_path).ok()
            } else {
                None
            }
        } else {
            None
        };

        // Read chapters in spine order
        let mut chapters = Vec::new();
        for (order, item_id) in spine.iter().enumerate() {
            let Some(item) = manifest.iter().find(|m| m.id == *item_id) else {
                continue;
            };
            if !item.media_type.contains("xhtml") && !item.media_type.contains("html") {
                continue;
            }

            let chapter_path = format!("{}{}", opf_dir, item.href);
            let chapter_html = match read_zip_entry(&mut archive, &chapter_path) {
                Ok(html) => html,
                Err(_) => continue,
            };

            let (title, body_html) = extract_chapter_content(&chapter_html);
            let text = strip_html(&body_html);
            let word_count = text.split_whitespace().count();

            chapters.push(ReadChapter {
                title: title.unwrap_or_else(|| {
                    item.href
                        .rsplit('/')
                        .next()
                        .unwrap_or(&item.href)
                        .replace(".xhtml", "")
                        .replace(".html", "")
                        .replace('_', " ")
                }),
                html: body_html,
                text,
                word_count,
                href: item.href.clone(),
                order,
            });
        }

        let total_words = chapters.iter().map(|c| c.word_count).sum();

        Ok(Self {
            metadata,
            chapters,
            cover,
            total_words,
        })
    }

    /// Get a chapter by index.
    pub fn chapter(&self, index: usize) -> Option<&ReadChapter> {
        self.chapters.get(index)
    }

    /// Print a plain-text rendering of the entire book to a string.
    pub fn to_plain_text(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!(
            "{}\nby {}\n",
            self.metadata.title,
            self.metadata.primary_author()
        ));
        output.push_str(&"=".repeat(60));
        output.push('\n');

        for chapter in &self.chapters {
            output.push_str(&format!("\n\n--- {} ---\n\n", chapter.title));
            output.push_str(&chapter.text);
        }

        output.push_str(&format!(
            "\n\n[{} chapters, {} words]\n",
            self.chapters.len(),
            self.total_words
        ));
        output
    }
}

// ─── Internal helpers ───

struct ManifestItem {
    id: String,
    href: String,
    media_type: String,
}

fn find_opf_path(archive: &mut ZipArchive<Cursor<&[u8]>>) -> Result<String> {
    let container = read_zip_entry(archive, "META-INF/container.xml")?;
    let mut reader = Reader::from_str(&container);
    reader.config_mut().trim_text(true);

    loop {
        match reader.read_event() {
            Ok(Event::Empty(ref e)) | Ok(Event::Start(ref e)) => {
                let qname = e.name();
                let local = local_name(qname.as_ref());
                if local == b"rootfile" {
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"full-path" {
                            if let Ok(val) = std::str::from_utf8(&attr.value) {
                                return Ok(val.to_string());
                            }
                        }
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(PublishingError::Xml(format!("container.xml parse: {e}"))),
            _ => {}
        }
    }

    Err(PublishingError::Xml("No rootfile in container.xml".into()))
}

fn parse_opf(xml: &str) -> Result<(BookMetadata, Vec<ManifestItem>, Vec<String>, Option<String>)> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut title = String::new();
    let mut creator = String::new();
    let mut language = String::new();
    let mut publisher = String::new();
    let mut description = String::new();
    let mut manifest = Vec::new();
    let mut spine = Vec::new();
    let mut cover_id: Option<String> = None;
    let mut current_tag: Option<String> = None;
    let mut in_metadata = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let qname = e.name();
                let local = local_name(qname.as_ref());
                let local_str = String::from_utf8_lossy(local).to_string();

                if local == b"metadata" {
                    in_metadata = true;
                }
                if in_metadata {
                    current_tag = Some(local_str);
                }
            }
            Ok(Event::End(ref e)) => {
                let qname = e.name();
                let local = local_name(qname.as_ref());
                if local == b"metadata" {
                    in_metadata = false;
                }
                current_tag = None;
            }
            Ok(Event::Text(ref e)) => {
                if let (Some(tag), Ok(text)) = (&current_tag, e.unescape()) {
                    let text = text.trim().to_string();
                    match tag.as_str() {
                        "title" if title.is_empty() => title = text,
                        "creator" if creator.is_empty() => creator = text,
                        "language" if language.is_empty() => language = text,
                        "publisher" if publisher.is_empty() => publisher = text,
                        "description" if description.is_empty() => description = text,
                        _ => {}
                    }
                }
            }
            Ok(Event::Empty(ref e)) => {
                let qname = e.name();
                let local = local_name(qname.as_ref());

                if local == b"item" {
                    let mut id = String::new();
                    let mut href = String::new();
                    let mut media_type = String::new();
                    for attr in e.attributes().flatten() {
                        match attr.key.as_ref() {
                            b"id" => id = String::from_utf8_lossy(&attr.value).to_string(),
                            b"href" => href = String::from_utf8_lossy(&attr.value).to_string(),
                            b"media-type" => {
                                media_type = String::from_utf8_lossy(&attr.value).to_string()
                            }
                            _ => {}
                        }
                    }
                    manifest.push(ManifestItem {
                        id,
                        href,
                        media_type,
                    });
                }

                if local == b"itemref" {
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"idref" {
                            spine.push(String::from_utf8_lossy(&attr.value).to_string());
                        }
                    }
                }

                if local == b"meta" {
                    let mut name = String::new();
                    let mut content = String::new();
                    for attr in e.attributes().flatten() {
                        match attr.key.as_ref() {
                            b"name" => name = String::from_utf8_lossy(&attr.value).to_string(),
                            b"content" => {
                                content = String::from_utf8_lossy(&attr.value).to_string()
                            }
                            _ => {}
                        }
                    }
                    if name == "cover" {
                        cover_id = Some(content);
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(PublishingError::Xml(format!("OPF parse: {e}"))),
            _ => {}
        }
    }

    if title.is_empty() {
        title = "Untitled".into();
    }
    if creator.is_empty() {
        creator = "Unknown Author".into();
    }
    if language.is_empty() {
        language = "en".into();
    }

    let metadata = BookMetadata::new(title, creator, language);
    // We'd set publisher/description on the metadata but BookMetadata::new only takes 3 args
    // The caller can enrich after

    Ok((metadata, manifest, spine, cover_id))
}

fn extract_chapter_content(xhtml: &str) -> (Option<String>, String) {
    let mut reader = Reader::from_str(xhtml);
    reader.config_mut().trim_text(false);

    let mut title: Option<String> = None;
    let mut in_body = false;
    let mut body_content = String::new();
    let mut in_heading = false;
    let mut heading_text = String::new();
    let mut depth: u32 = 0;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let qname = e.name();
                let local = local_name(qname.as_ref());
                let tag = String::from_utf8_lossy(local).to_string();

                if tag == "body" {
                    in_body = true;
                    continue;
                }
                if in_body {
                    if (tag == "h1" || tag == "h2" || tag == "h3") && title.is_none() {
                        in_heading = true;
                        heading_text.clear();
                    }
                    body_content.push_str(&format!("<{tag}>"));
                    depth += 1;
                }
            }
            Ok(Event::End(ref e)) => {
                let qname = e.name();
                let local = local_name(qname.as_ref());
                let tag = String::from_utf8_lossy(local).to_string();

                if tag == "body" {
                    in_body = false;
                    continue;
                }
                if in_body {
                    body_content.push_str(&format!("</{tag}>"));
                    if in_heading {
                        in_heading = false;
                        title = Some(heading_text.trim().to_string());
                    }
                    depth = depth.saturating_sub(1);
                }
            }
            Ok(Event::Text(ref e)) => {
                if in_body {
                    if let Ok(text) = e.unescape() {
                        body_content.push_str(&text);
                        if in_heading {
                            heading_text.push_str(&text);
                        }
                    }
                }
            }
            Ok(Event::Empty(ref e)) => {
                if in_body {
                    let qname = e.name();
                    let local = local_name(qname.as_ref());
                    let tag = String::from_utf8_lossy(local).to_string();
                    body_content.push_str(&format!("<{tag} />"));
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
    }

    (title, body_content)
}

fn strip_html(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                result.push(' ');
            }
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }
    // Collapse whitespace
    let mut prev_space = false;
    result
        .chars()
        .filter(|&c| {
            if c.is_whitespace() {
                if prev_space {
                    return false;
                }
                prev_space = true;
            } else {
                prev_space = false;
            }
            true
        })
        .collect::<String>()
        .trim()
        .to_string()
}

fn read_zip_entry(archive: &mut ZipArchive<Cursor<&[u8]>>, path: &str) -> Result<String> {
    let mut content = String::new();
    let mut file = archive
        .by_name(path)
        .map_err(|e| PublishingError::Zip(format!("Missing {path}: {e}")))?;
    file.read_to_string(&mut content)
        .map_err(|e| PublishingError::Zip(format!("Read {path}: {e}")))?;
    Ok(content)
}

fn read_zip_bytes(archive: &mut ZipArchive<Cursor<&[u8]>>, path: &str) -> Result<Vec<u8>> {
    let mut content = Vec::new();
    let mut file = archive
        .by_name(path)
        .map_err(|e| PublishingError::Zip(format!("Missing {path}: {e}")))?;
    file.read_to_end(&mut content)
        .map_err(|e| PublishingError::Zip(format!("Read {path}: {e}")))?;
    Ok(content)
}

fn local_name(name: &[u8]) -> &[u8] {
    if let Some(pos) = name.iter().position(|&b| b == b':') {
        &name[pos + 1..]
    } else {
        name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chapter::Chapter;
    use crate::epub::{EpubOptions, generate_epub};

    #[test]
    fn test_roundtrip_write_then_read() {
        // Generate an EPUB
        let meta = crate::metadata::BookMetadata::new("Roundtrip Test", "Test Author", "en");
        let chapters = vec![
            Chapter::new(
                "Chapter One",
                "<p>Hello from chapter one. This is a test.</p>",
                0,
            ),
            Chapter::new("Chapter Two", "<p>Content of chapter two here.</p>", 1),
        ];
        let epub_bytes =
            generate_epub(&meta, &chapters, None, &EpubOptions::default()).expect("generate");

        // Read it back
        let reader = EpubReader::from_bytes(&epub_bytes).expect("read");

        assert_eq!(reader.metadata.title, "Roundtrip Test");
        assert_eq!(reader.metadata.primary_author(), "Test Author");
        assert_eq!(reader.chapters.len(), 2);
        assert_eq!(reader.chapters[0].title, "Chapter One");
        assert_eq!(reader.chapters[1].title, "Chapter Two");
        assert!(reader.chapters[0].text.contains("Hello from chapter one"));
        assert!(reader.total_words > 0);
    }

    #[test]
    fn test_plain_text_output() {
        let meta = crate::metadata::BookMetadata::new("My Book", "Author", "en");
        let chapters = vec![Chapter::new("Intro", "<p>Welcome to the book.</p>", 0)];
        let epub_bytes =
            generate_epub(&meta, &chapters, None, &EpubOptions::default()).expect("generate");

        let reader = EpubReader::from_bytes(&epub_bytes).expect("read");
        let text = reader.to_plain_text();

        assert!(text.contains("My Book"));
        assert!(text.contains("by Author"));
        assert!(text.contains("Intro"));
        assert!(text.contains("Welcome to the book"));
    }

    #[test]
    fn test_strip_html() {
        assert_eq!(
            strip_html("<p>Hello <strong>world</strong></p>"),
            "Hello world"
        );
        assert_eq!(strip_html("<h1>Title</h1><p>Text</p>"), "Title Text");
    }

    #[test]
    fn test_invalid_epub_errors() {
        let result = EpubReader::from_bytes(b"not a zip");
        assert!(result.is_err());
    }
}
