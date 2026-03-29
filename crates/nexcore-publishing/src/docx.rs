//! DOCX file reader.
//!
//! Reads `.docx` files (Office Open XML) by extracting the ZIP archive
//! and parsing `word/document.xml` for paragraph content and heading styles.
//! No external DOCX library — pure Rust using `zip` + `quick-xml`.

use std::io::{Cursor, Read as _};
use std::path::Path;

use quick_xml::Reader;
use quick_xml::events::Event;
use zip::ZipArchive;

use crate::chapter::{HeadingLevel, xml_escape};
use crate::error::{PublishingError, Result};
use crate::metadata::BookMetadata;

/// A parsed DOCX document.
#[derive(Debug)]
pub struct DocxDocument {
    /// Extracted paragraphs with optional heading level.
    pub paragraphs: Vec<(Option<HeadingLevel>, String)>,
    /// Core properties extracted from `docProps/core.xml`.
    pub properties: DocxProperties,
}

/// Properties from `docProps/core.xml`.
#[derive(Debug, Default)]
pub struct DocxProperties {
    pub title: Option<String>,
    pub creator: Option<String>,
    pub description: Option<String>,
    pub subject: Option<String>,
    pub language: Option<String>,
}

impl DocxDocument {
    /// Read and parse a `.docx` file from disk.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let data = std::fs::read(path).map_err(|e| {
            PublishingError::DocxRead(format!("Cannot read {}: {e}", path.display()))
        })?;
        Self::from_bytes(&data)
    }

    /// Parse a `.docx` from raw bytes.
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        let cursor = Cursor::new(data);
        let mut archive = ZipArchive::new(cursor)
            .map_err(|e| PublishingError::Zip(format!("Invalid DOCX/ZIP archive: {e}")))?;

        // Parse document.xml for content
        let paragraphs = parse_document_xml(&mut archive)?;

        // Parse core.xml for properties
        let properties = parse_core_properties(&mut archive).unwrap_or_default();

        Ok(Self {
            paragraphs,
            properties,
        })
    }

    /// Convert DOCX properties to `BookMetadata`, using defaults for missing fields.
    pub fn to_metadata(&self) -> BookMetadata {
        let title = self
            .properties
            .title
            .clone()
            .unwrap_or_else(|| "Untitled".into());
        let author = self
            .properties
            .creator
            .clone()
            .unwrap_or_else(|| "Unknown Author".into());
        let language = self
            .properties
            .language
            .clone()
            .unwrap_or_else(|| "en".into());

        let mut meta = BookMetadata::new(title, author, language);
        meta.description = self.properties.description.clone();
        if let Some(ref subj) = self.properties.subject {
            meta.subjects = subj.split(',').map(|s| s.trim().to_string()).collect();
        }
        meta
    }

    /// Total paragraph count.
    pub fn paragraph_count(&self) -> usize {
        self.paragraphs.len()
    }

    /// Total word count across all paragraphs.
    pub fn word_count(&self) -> usize {
        self.paragraphs
            .iter()
            .map(|(_, text)| text.split_whitespace().count())
            .sum()
    }
}

/// Parse `word/document.xml` from the DOCX archive.
fn parse_document_xml(
    archive: &mut ZipArchive<Cursor<&[u8]>>,
) -> Result<Vec<(Option<HeadingLevel>, String)>> {
    let mut doc_xml = String::new();
    {
        let mut file = archive
            .by_name("word/document.xml")
            .map_err(|e| PublishingError::DocxRead(format!("Missing word/document.xml: {e}")))?;
        file.read_to_string(&mut doc_xml)
            .map_err(|e| PublishingError::DocxRead(format!("Cannot read document.xml: {e}")))?;
    }

    // Also try to load styles for heading detection
    let styles_map = load_styles(archive);

    let mut paragraphs = Vec::new();
    let mut reader = Reader::from_str(&doc_xml);
    reader.config_mut().trim_text(true);

    let mut in_paragraph = false;
    let mut in_run = false;
    let mut current_text = String::new();
    let mut current_style: Option<String> = None;
    let mut in_ppr = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let qname = e.name();
                let local = local_name(qname.as_ref());
                match local {
                    b"p" => {
                        in_paragraph = true;
                        current_text.clear();
                        current_style = None;
                    }
                    b"pPr" if in_paragraph => {
                        in_ppr = true;
                    }
                    b"pStyle" if in_ppr => {
                        for attr in e.attributes().flatten() {
                            if local_name(attr.key.as_ref()) == b"val" {
                                if let Ok(val) = std::str::from_utf8(&attr.value) {
                                    current_style = Some(val.to_string());
                                }
                            }
                        }
                    }
                    b"r" if in_paragraph => {
                        in_run = true;
                    }
                    _ => {}
                }
            }
            Ok(Event::End(ref e)) => {
                let qname = e.name();
                let local = local_name(qname.as_ref());
                match local {
                    b"p" => {
                        in_paragraph = false;
                        let text = current_text.trim().to_string();
                        if !text.is_empty() {
                            let heading = current_style
                                .as_deref()
                                .and_then(|s| resolve_heading_level(s, &styles_map));
                            paragraphs.push((heading, text));
                        }
                    }
                    b"pPr" => {
                        in_ppr = false;
                    }
                    b"r" => {
                        in_run = false;
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(ref e)) if in_run && in_paragraph => {
                if let Ok(text) = e.unescape() {
                    if !current_text.is_empty() && !current_text.ends_with(' ') {
                        current_text.push(' ');
                    }
                    current_text.push_str(&text);
                }
            }
            Ok(Event::Empty(ref e)) => {
                let qname = e.name();
                let local = local_name(qname.as_ref());
                if local == b"pStyle" && in_ppr {
                    for attr in e.attributes().flatten() {
                        if local_name(attr.key.as_ref()) == b"val" {
                            if let Ok(val) = std::str::from_utf8(&attr.value) {
                                current_style = Some(val.to_string());
                            }
                        }
                    }
                }
                // Handle <w:br/> and <w:tab/> as spaces
                if local == b"br" || local == b"tab" {
                    if in_paragraph {
                        current_text.push(' ');
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(PublishingError::Xml(format!(
                    "XML parse error in document.xml: {e}"
                )));
            }
            _ => {}
        }
    }

    Ok(paragraphs)
}

/// Load `word/styles.xml` and build a map from style ID to base heading level.
fn load_styles(
    archive: &mut ZipArchive<Cursor<&[u8]>>,
) -> std::collections::HashMap<String, HeadingLevel> {
    let mut map = std::collections::HashMap::new();
    let mut styles_xml = String::new();

    let Ok(mut file) = archive.by_name("word/styles.xml") else {
        return map;
    };
    if file.read_to_string(&mut styles_xml).is_err() {
        return map;
    }

    let mut reader = Reader::from_str(&styles_xml);
    reader.config_mut().trim_text(true);

    let mut current_style_id: Option<String> = None;
    let mut in_style = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let qname = e.name();
                let local = local_name(qname.as_ref());
                if local == b"style" {
                    in_style = true;
                    current_style_id = None;
                    for attr in e.attributes().flatten() {
                        if local_name(attr.key.as_ref()) == b"styleId" {
                            if let Ok(val) = std::str::from_utf8(&attr.value) {
                                current_style_id = Some(val.to_string());
                            }
                        }
                    }
                }
                if local == b"name" && in_style {
                    for attr in e.attributes().flatten() {
                        if local_name(attr.key.as_ref()) == b"val" {
                            if let Ok(val) = std::str::from_utf8(&attr.value) {
                                if let (Some(id), Some(level)) =
                                    (&current_style_id, HeadingLevel::from_style(val))
                                {
                                    map.insert(id.clone(), level);
                                }
                            }
                        }
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let qname = e.name();
                if local_name(qname.as_ref()) == b"style" {
                    in_style = false;
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
    }

    map
}

/// Resolve a style ID to a heading level, checking both the style map and the raw name.
fn resolve_heading_level(
    style: &str,
    styles_map: &std::collections::HashMap<String, HeadingLevel>,
) -> Option<HeadingLevel> {
    // Check styles.xml mapping first
    if let Some(level) = styles_map.get(style) {
        return Some(*level);
    }
    // Fall back to pattern matching on the style ID itself
    HeadingLevel::from_style(style)
}

/// Parse `docProps/core.xml` for document properties.
fn parse_core_properties(archive: &mut ZipArchive<Cursor<&[u8]>>) -> Result<DocxProperties> {
    let mut core_xml = String::new();
    {
        let mut file = archive
            .by_name("docProps/core.xml")
            .map_err(|e| PublishingError::DocxRead(format!("Missing docProps/core.xml: {e}")))?;
        file.read_to_string(&mut core_xml)
            .map_err(|e| PublishingError::DocxRead(format!("Cannot read core.xml: {e}")))?;
    }

    let mut props = DocxProperties::default();
    let mut reader = Reader::from_str(&core_xml);
    reader.config_mut().trim_text(true);

    let mut current_tag: Option<String> = None;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let qname = e.name();
                let local = local_name(qname.as_ref());
                current_tag = Some(String::from_utf8_lossy(local).to_string());
            }
            Ok(Event::Text(ref e)) => {
                if let (Some(tag), Ok(text)) = (&current_tag, e.unescape()) {
                    let text = text.trim().to_string();
                    if !text.is_empty() {
                        match tag.as_str() {
                            "title" => props.title = Some(text),
                            "creator" => props.creator = Some(text),
                            "description" => props.description = Some(text),
                            "subject" => props.subject = Some(text),
                            "language" => props.language = Some(text),
                            _ => {}
                        }
                    }
                }
            }
            Ok(Event::End(_)) => {
                current_tag = None;
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
    }

    Ok(props)
}

/// Extract local name from a potentially namespace-prefixed XML name.
/// e.g., `w:p` → `p`, `dc:title` → `title`.
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

    #[test]
    fn test_local_name_extraction() {
        assert_eq!(local_name(b"w:p"), b"p");
        assert_eq!(local_name(b"dc:title"), b"title");
        assert_eq!(local_name(b"body"), b"body");
    }

    #[test]
    fn test_minimal_docx_roundtrip() {
        // Build a minimal valid DOCX in memory
        let docx_bytes = build_test_docx(&[
            ("Heading1", "Chapter One"),
            ("Normal", "This is paragraph one."),
            ("Normal", "This is paragraph two."),
            ("Heading1", "Chapter Two"),
            ("Normal", "Content of chapter two."),
        ]);

        let doc = DocxDocument::from_bytes(&docx_bytes).expect("Should parse minimal DOCX");

        assert!(doc.paragraph_count() >= 4);
        // First heading should be detected
        let headings: Vec<_> = doc.paragraphs.iter().filter(|(h, _)| h.is_some()).collect();
        assert!(
            headings.len() >= 2,
            "Expected at least 2 headings, got {}",
            headings.len()
        );
    }

    #[test]
    fn test_docx_properties_extraction() {
        let docx_bytes = build_test_docx_with_props(
            "My Test Book",
            "Jane Author",
            &[("Normal", "Hello world.")],
        );

        let doc = DocxDocument::from_bytes(&docx_bytes).expect("Should parse DOCX");
        assert_eq!(doc.properties.title.as_deref(), Some("My Test Book"));
        assert_eq!(doc.properties.creator.as_deref(), Some("Jane Author"));
    }

    #[test]
    fn test_to_metadata() {
        let docx_bytes = build_test_docx_with_props(
            "Pharmacovigilance Primer",
            "Matthew Campion",
            &[("Normal", "Content here.")],
        );
        let doc = DocxDocument::from_bytes(&docx_bytes).expect("Should parse DOCX");
        let meta = doc.to_metadata();
        assert_eq!(meta.title, "Pharmacovigilance Primer");
        assert_eq!(meta.primary_author(), "Matthew Campion");
    }

    /// Build a minimal DOCX (ZIP with document.xml) for testing.
    fn build_test_docx(paragraphs: &[(&str, &str)]) -> Vec<u8> {
        build_test_docx_with_props("", "", paragraphs)
    }

    fn build_test_docx_with_props(
        title: &str,
        creator: &str,
        paragraphs: &[(&str, &str)],
    ) -> Vec<u8> {
        let mut buf = Vec::new();
        let cursor = Cursor::new(&mut buf);
        let mut zip = zip::ZipWriter::new(cursor);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);

        // [Content_Types].xml
        zip.start_file("[Content_Types].xml", options).ok();
        let ct = r#"<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="xml" ContentType="application/xml"/>
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
</Types>"#;
        std::io::Write::write_all(&mut zip, ct.as_bytes()).ok();

        // _rels/.rels
        zip.start_file("_rels/.rels", options).ok();
        let rels = r#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties" Target="docProps/core.xml"/>
</Relationships>"#;
        std::io::Write::write_all(&mut zip, rels.as_bytes()).ok();

        // word/document.xml
        zip.start_file("word/document.xml", options).ok();
        let mut doc = String::from(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:body>"#,
        );
        for (style, text) in paragraphs {
            doc.push_str(&format!(
                r#"<w:p><w:pPr><w:pStyle w:val="{style}"/></w:pPr><w:r><w:t>{text}</w:t></w:r></w:p>"#,
                style = xml_escape(style),
                text = xml_escape(text),
            ));
        }
        doc.push_str("</w:body></w:document>");
        std::io::Write::write_all(&mut zip, doc.as_bytes()).ok();

        // docProps/core.xml
        if !title.is_empty() || !creator.is_empty() {
            zip.start_file("docProps/core.xml", options).ok();
            let core = format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties"
                   xmlns:dc="http://purl.org/dc/elements/1.1/">
  <dc:title>{}</dc:title>
  <dc:creator>{}</dc:creator>
</cp:coreProperties>"#,
                xml_escape(title),
                xml_escape(creator),
            );
            std::io::Write::write_all(&mut zip, core.as_bytes()).ok();
        }

        zip.finish().ok();
        buf
    }
}
