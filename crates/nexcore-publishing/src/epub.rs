//! EPUB 3.0 writer.
//!
//! Generates a valid EPUB 3.0 archive from chapters, metadata, and cover image.
//! Pure Rust — no pandoc dependency. Outputs a standards-compliant `.epub` file.

use std::io::{Cursor, Write as _};
use std::path::Path;

use zip::CompressionMethod;
use zip::write::SimpleFileOptions;

use crate::chapter::Chapter;
use crate::cover::CoverSpec;
use crate::error::{PublishingError, Result};
use crate::metadata::BookMetadata;

/// EPUB generation options.
#[derive(Debug, Clone)]
pub struct EpubOptions {
    /// Include EPUB 2 NCX navigation for backward compatibility.
    pub include_ncx: bool,
    /// CSS content for the book stylesheet.
    pub css: String,
    /// EPUB version string.
    pub epub_version: String,
}

impl Default for EpubOptions {
    fn default() -> Self {
        Self {
            include_ncx: true,
            css: default_css().to_string(),
            epub_version: "3.0".into(),
        }
    }
}

/// Write a complete EPUB file to disk.
pub fn write_epub(
    output_path: impl AsRef<Path>,
    metadata: &BookMetadata,
    chapters: &[Chapter],
    cover: Option<&CoverSpec>,
    options: &EpubOptions,
) -> Result<EpubManifest> {
    let bytes = generate_epub(metadata, chapters, cover, options)?;
    std::fs::write(output_path.as_ref(), &bytes)?;
    Ok(EpubManifest {
        file_count: chapters.len() + 4, // chapters + mimetype + container + opf + nav + css
        total_bytes: bytes.len(),
    })
}

/// Generate EPUB as in-memory bytes.
pub fn generate_epub(
    metadata: &BookMetadata,
    chapters: &[Chapter],
    cover: Option<&CoverSpec>,
    options: &EpubOptions,
) -> Result<Vec<u8>> {
    if chapters.is_empty() {
        return Err(PublishingError::EpubWrite("No chapters to publish".into()));
    }

    let mut buf = Vec::new();
    let cursor = Cursor::new(&mut buf);
    let mut zip = zip::ZipWriter::new(cursor);

    // 1. mimetype (MUST be first, uncompressed, no extra field)
    let stored = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    let deflated = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    zip.start_file("mimetype", stored)
        .map_err(|e| PublishingError::Zip(format!("Cannot write mimetype: {e}")))?;
    zip.write_all(b"application/epub+zip")
        .map_err(|e| PublishingError::Zip(format!("Write mimetype: {e}")))?;

    // 2. META-INF/container.xml
    zip.start_file("META-INF/container.xml", deflated)
        .map_err(|e| PublishingError::Zip(format!("Cannot write container.xml: {e}")))?;
    zip.write_all(CONTAINER_XML.as_bytes())
        .map_err(|e| PublishingError::Zip(format!("Write container.xml: {e}")))?;

    // 3. OEBPS/style.css
    zip.start_file("OEBPS/style.css", deflated)
        .map_err(|e| PublishingError::Zip(format!("Cannot write style.css: {e}")))?;
    zip.write_all(options.css.as_bytes())
        .map_err(|e| PublishingError::Zip(format!("Write style.css: {e}")))?;

    // 4. Cover image (if provided)
    if let Some(cover) = cover {
        let cover_filename = format!("OEBPS/{}", cover.epub_filename());
        zip.start_file(&cover_filename, stored)
            .map_err(|e| PublishingError::Zip(format!("Cannot write cover: {e}")))?;
        let cover_data = std::fs::read(&cover.path)?;
        zip.write_all(&cover_data)
            .map_err(|e| PublishingError::Zip(format!("Write cover: {e}")))?;

        // Cover XHTML page
        zip.start_file("OEBPS/cover.xhtml", deflated)
            .map_err(|e| PublishingError::Zip(format!("Cannot write cover.xhtml: {e}")))?;
        let cover_xhtml = generate_cover_xhtml(&cover.epub_filename(), &metadata.title);
        zip.write_all(cover_xhtml.as_bytes())
            .map_err(|e| PublishingError::Zip(format!("Write cover.xhtml: {e}")))?;
    }

    // 5. Chapter XHTML files
    for chapter in chapters {
        let filename = format!("OEBPS/{}", chapter.epub_filename());
        zip.start_file(&filename, deflated)
            .map_err(|e| PublishingError::Zip(format!("Cannot write chapter: {e}")))?;
        let xhtml = chapter.to_xhtml("style.css");
        zip.write_all(xhtml.as_bytes())
            .map_err(|e| PublishingError::Zip(format!("Write chapter: {e}")))?;
    }

    // 6. Navigation document (EPUB 3)
    zip.start_file("OEBPS/nav.xhtml", deflated)
        .map_err(|e| PublishingError::Zip(format!("Cannot write nav.xhtml: {e}")))?;
    let nav = generate_nav_xhtml(chapters, &metadata.title);
    zip.write_all(nav.as_bytes())
        .map_err(|e| PublishingError::Zip(format!("Write nav.xhtml: {e}")))?;

    // 7. NCX (EPUB 2 compat)
    if options.include_ncx {
        zip.start_file("OEBPS/toc.ncx", deflated)
            .map_err(|e| PublishingError::Zip(format!("Cannot write toc.ncx: {e}")))?;
        let ncx = generate_ncx(chapters, metadata);
        zip.write_all(ncx.as_bytes())
            .map_err(|e| PublishingError::Zip(format!("Write toc.ncx: {e}")))?;
    }

    // 8. OPF package document (MUST be last of OEBPS — references all above)
    zip.start_file("OEBPS/content.opf", deflated)
        .map_err(|e| PublishingError::Zip(format!("Cannot write content.opf: {e}")))?;
    let opf = generate_opf(metadata, chapters, cover, options);
    zip.write_all(opf.as_bytes())
        .map_err(|e| PublishingError::Zip(format!("Write content.opf: {e}")))?;

    zip.finish()
        .map_err(|e| PublishingError::Zip(format!("Finalize ZIP: {e}")))?;

    Ok(buf)
}

/// Summary of what was generated.
#[derive(Debug)]
pub struct EpubManifest {
    pub file_count: usize,
    pub total_bytes: usize,
}

// ─── Static Templates ───

const CONTAINER_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>"#;

/// Generate the OPF package document.
fn generate_opf(
    metadata: &BookMetadata,
    chapters: &[Chapter],
    cover: Option<&CoverSpec>,
    options: &EpubOptions,
) -> String {
    let unique_id = metadata.unique_id();
    let mut opf = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="{version}" unique-identifier="BookId">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:opf="http://www.idpf.org/2007/opf">
    <dc:identifier id="BookId">{id}</dc:identifier>
    <dc:title>{title}</dc:title>
    <dc:language>{lang}</dc:language>
    <meta property="dcterms:modified">{date}</meta>
"#,
        version = options.epub_version,
        id = xml_esc(&unique_id),
        title = xml_esc(&metadata.title),
        lang = xml_esc(&metadata.language),
        date = metadata.date.as_deref().unwrap_or("2026-01-01T00:00:00Z"),
    );

    // Authors
    for author in &metadata.authors {
        opf.push_str(&format!(
            "    <dc:creator opf:role=\"{}\">{}</dc:creator>\n",
            author.role.marc_code(),
            xml_esc(&author.name),
        ));
    }

    if let Some(ref publisher) = metadata.publisher {
        opf.push_str(&format!(
            "    <dc:publisher>{}</dc:publisher>\n",
            xml_esc(publisher)
        ));
    }
    if let Some(ref desc) = metadata.description {
        opf.push_str(&format!(
            "    <dc:description>{}</dc:description>\n",
            xml_esc(desc)
        ));
    }
    for subj in &metadata.subjects {
        opf.push_str(&format!("    <dc:subject>{}</dc:subject>\n", xml_esc(subj)));
    }
    if let Some(ref rights) = metadata.rights {
        opf.push_str(&format!("    <dc:rights>{}</dc:rights>\n", xml_esc(rights)));
    }

    // Cover meta
    if cover.is_some() {
        opf.push_str("    <meta name=\"cover\" content=\"cover-image\"/>\n");
    }

    opf.push_str("  </metadata>\n\n  <manifest>\n");

    // Manifest items
    opf.push_str("    <item id=\"nav\" href=\"nav.xhtml\" media-type=\"application/xhtml+xml\" properties=\"nav\"/>\n");
    opf.push_str("    <item id=\"css\" href=\"style.css\" media-type=\"text/css\"/>\n");

    if options.include_ncx {
        opf.push_str(
            "    <item id=\"ncx\" href=\"toc.ncx\" media-type=\"application/x-dtbncx+xml\"/>\n",
        );
    }

    if let Some(cover) = cover {
        opf.push_str(&format!(
            "    <item id=\"cover-image\" href=\"{}\" media-type=\"{}\" properties=\"cover-image\"/>\n",
            cover.epub_filename(),
            cover.format.mime_type(),
        ));
        opf.push_str(
            "    <item id=\"cover\" href=\"cover.xhtml\" media-type=\"application/xhtml+xml\"/>\n",
        );
    }

    for chapter in chapters {
        opf.push_str(&format!(
            "    <item id=\"ch{order}\" href=\"{file}\" media-type=\"application/xhtml+xml\"/>\n",
            order = chapter.order,
            file = chapter.epub_filename(),
        ));
    }

    opf.push_str("  </manifest>\n\n  <spine");
    if options.include_ncx {
        opf.push_str(" toc=\"ncx\"");
    }
    opf.push_str(">\n");

    if cover.is_some() {
        opf.push_str("    <itemref idref=\"cover\"/>\n");
    }
    for chapter in chapters {
        opf.push_str(&format!("    <itemref idref=\"ch{}\"/>\n", chapter.order));
    }

    opf.push_str("  </spine>\n</package>\n");
    opf
}

/// Generate the EPUB 3 navigation document.
fn generate_nav_xhtml(chapters: &[Chapter], title: &str) -> String {
    let mut nav = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops" xml:lang="en">
<head>
  <meta charset="UTF-8" />
  <title>{title} — Table of Contents</title>
</head>
<body>
  <nav epub:type="toc" id="toc">
    <h1>Table of Contents</h1>
    <ol>
"#,
        title = xml_esc(title),
    );

    for chapter in chapters {
        nav.push_str(&format!(
            "      <li><a href=\"{}\">{}</a></li>\n",
            chapter.epub_filename(),
            xml_esc(&chapter.title),
        ));
    }

    nav.push_str("    </ol>\n  </nav>\n</body>\n</html>\n");
    nav
}

/// Generate the NCX navigation (EPUB 2 backward compatibility).
fn generate_ncx(chapters: &[Chapter], metadata: &BookMetadata) -> String {
    let mut ncx = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/" version="2005-1">
  <head>
    <meta name="dtb:uid" content="{id}"/>
    <meta name="dtb:depth" content="1"/>
    <meta name="dtb:totalPageCount" content="0"/>
    <meta name="dtb:maxPageNumber" content="0"/>
  </head>
  <docTitle><text>{title}</text></docTitle>
  <navMap>
"#,
        id = xml_esc(&metadata.unique_id()),
        title = xml_esc(&metadata.title),
    );

    for (i, chapter) in chapters.iter().enumerate() {
        ncx.push_str(&format!(
            r#"    <navPoint id="navPoint-{i}" playOrder="{order}">
      <navLabel><text>{title}</text></navLabel>
      <content src="{file}"/>
    </navPoint>
"#,
            i = i + 1,
            order = i + 1,
            title = xml_esc(&chapter.title),
            file = chapter.epub_filename(),
        ));
    }

    ncx.push_str("  </navMap>\n</ncx>\n");
    ncx
}

/// Generate the cover page XHTML.
fn generate_cover_xhtml(cover_filename: &str, title: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" xml:lang="en">
<head>
  <meta charset="UTF-8" />
  <title>{title}</title>
  <style>
    body {{ margin: 0; padding: 0; text-align: center; }}
    img {{ max-width: 100%; max-height: 100%; }}
  </style>
</head>
<body>
  <div>
    <img src="{src}" alt="{title}" />
  </div>
</body>
</html>"#,
        title = xml_esc(title),
        src = cover_filename,
    )
}

/// Default book CSS.
fn default_css() -> &'static str {
    r#"/* NexVigilant Publishing — Default Book Stylesheet */
body {
  font-family: Georgia, "Times New Roman", serif;
  margin: 1em;
  line-height: 1.6;
  color: #1a1a1a;
}

h1 {
  font-size: 2em;
  margin-top: 2em;
  margin-bottom: 0.5em;
  page-break-before: always;
  text-align: center;
}

h2 {
  font-size: 1.5em;
  margin-top: 1.5em;
  margin-bottom: 0.4em;
}

h3 {
  font-size: 1.2em;
  margin-top: 1em;
  margin-bottom: 0.3em;
}

p {
  margin-top: 0.3em;
  margin-bottom: 0.3em;
  text-indent: 1.5em;
}

p:first-child,
h1 + p,
h2 + p,
h3 + p {
  text-indent: 0;
}

blockquote {
  margin: 1em 2em;
  font-style: italic;
  border-left: 3px solid #ccc;
  padding-left: 1em;
}

em { font-style: italic; }
strong { font-weight: bold; }

.chapter-number {
  font-size: 0.9em;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: #666;
}
"#
}

/// XML escape helper (re-export from chapter for convenience).
fn xml_esc(s: &str) -> String {
    crate::chapter::xml_escape(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_epub_minimal() {
        let meta = BookMetadata::new("Test Book", "Test Author", "en");
        let chapters = vec![
            Chapter::new("Chapter 1", "<p>Hello world.</p>", 0),
            Chapter::new("Chapter 2", "<p>Goodbye world.</p>", 1),
        ];
        let options = EpubOptions::default();

        let bytes = generate_epub(&meta, &chapters, None, &options).expect("Should generate EPUB");

        // Verify it's a valid ZIP
        let cursor = Cursor::new(&bytes);
        let archive = zip::ZipArchive::new(cursor).expect("Should be valid ZIP");

        // Check required files exist
        let names: Vec<String> = (0..archive.len())
            .filter_map(|i| archive.name_for_index(i).map(|n| n.to_string()))
            .collect();

        assert!(names.contains(&"mimetype".to_string()));
        assert!(names.contains(&"META-INF/container.xml".to_string()));
        assert!(names.contains(&"OEBPS/content.opf".to_string()));
        assert!(names.contains(&"OEBPS/nav.xhtml".to_string()));
        assert!(names.contains(&"OEBPS/style.css".to_string()));
        assert!(names.contains(&"OEBPS/chapter_001.xhtml".to_string()));
        assert!(names.contains(&"OEBPS/chapter_002.xhtml".to_string()));
    }

    #[test]
    fn test_empty_chapters_error() {
        let meta = BookMetadata::new("Empty", "Author", "en");
        let result = generate_epub(&meta, &[], None, &EpubOptions::default());
        assert!(result.is_err());
    }

    #[test]
    fn test_opf_contains_metadata() {
        let mut meta = BookMetadata::new("My Novel", "Jane Doe", "en-US");
        meta.publisher = Some("NexVigilant Press".into());
        meta.description = Some("A compelling story.".into());
        meta.subjects = vec!["Fiction".into(), "Thriller".into()];

        let chapters = vec![Chapter::new("Ch1", "<p>Text</p>", 0)];
        let opf = generate_opf(&meta, &chapters, None, &EpubOptions::default());

        assert!(opf.contains("My Novel"));
        assert!(opf.contains("Jane Doe"));
        assert!(opf.contains("en-US"));
        assert!(opf.contains("NexVigilant Press"));
        assert!(opf.contains("A compelling story."));
        assert!(opf.contains("Fiction"));
        assert!(opf.contains("Thriller"));
    }

    #[test]
    fn test_nav_xhtml_lists_chapters() {
        let chapters = vec![
            Chapter::new("Introduction", "", 0),
            Chapter::new("Methods", "", 1),
            Chapter::new("Results", "", 2),
        ];
        let nav = generate_nav_xhtml(&chapters, "Research Paper");
        assert!(nav.contains("Introduction"));
        assert!(nav.contains("Methods"));
        assert!(nav.contains("Results"));
        assert!(nav.contains("chapter_001.xhtml"));
    }

    #[test]
    fn test_ncx_generation() {
        let meta = BookMetadata::new("Test", "Author", "en");
        let chapters = vec![Chapter::new("Ch1", "", 0), Chapter::new("Ch2", "", 1)];
        let ncx = generate_ncx(&chapters, &meta);
        assert!(ncx.contains("navPoint"));
        assert!(ncx.contains("Ch1"));
        assert!(ncx.contains("Ch2"));
    }

    #[test]
    fn test_default_css_is_valid() {
        let css = default_css();
        assert!(css.contains("font-family"));
        assert!(css.contains("page-break-before"));
    }

    #[test]
    fn test_mimetype_is_first_in_archive() {
        let meta = BookMetadata::new("Test", "Author", "en");
        let chapters = vec![Chapter::new("Ch1", "<p>Hello</p>", 0)];
        let bytes = generate_epub(&meta, &chapters, None, &EpubOptions::default())
            .expect("Should generate");

        let cursor = Cursor::new(&bytes);
        let archive = zip::ZipArchive::new(cursor).expect("Valid ZIP");
        // First file must be mimetype
        let first = archive.name_for_index(0).expect("has first file");
        assert_eq!(first, "mimetype");
    }
}
