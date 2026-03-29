//! EPUB validation.
//!
//! Structural validation of generated EPUB archives without requiring
//! external tools like `epubcheck`.

use std::io::Cursor;

use zip::ZipArchive;

use crate::error::{PublishingError, Result};

/// EPUB validation result.
#[derive(Debug)]
pub struct EpubValidation {
    /// Overall valid/invalid.
    pub valid: bool,
    /// Individual checks.
    pub checks: Vec<ValidationCheck>,
}

/// A single validation check.
#[derive(Debug)]
pub struct ValidationCheck {
    pub name: String,
    pub passed: bool,
    pub message: String,
}

/// Validate an EPUB file on disk.
pub fn validate_epub_file(path: impl AsRef<std::path::Path>) -> Result<EpubValidation> {
    let data = std::fs::read(path.as_ref())?;
    validate_epub_bytes(&data)
}

/// Validate EPUB from raw bytes.
pub fn validate_epub_bytes(data: &[u8]) -> Result<EpubValidation> {
    let mut checks = Vec::new();

    // 1. Valid ZIP archive
    let cursor = Cursor::new(data);
    let archive = match ZipArchive::new(cursor) {
        Ok(a) => {
            checks.push(ValidationCheck {
                name: "valid_zip".into(),
                passed: true,
                message: format!("{} files in archive", a.len()),
            });
            a
        }
        Err(e) => {
            checks.push(ValidationCheck {
                name: "valid_zip".into(),
                passed: false,
                message: format!("Invalid ZIP: {e}"),
            });
            return Ok(EpubValidation {
                valid: false,
                checks,
            });
        }
    };

    let names: Vec<String> = (0..archive.len())
        .filter_map(|i| archive.name_for_index(i).map(|n| n.to_string()))
        .collect();

    // 2. mimetype is first entry
    let mimetype_first = names.first().map_or(false, |n| n == "mimetype");
    checks.push(ValidationCheck {
        name: "mimetype_first".into(),
        passed: mimetype_first,
        message: if mimetype_first {
            "mimetype is first entry".into()
        } else {
            format!(
                "mimetype should be first entry, got: {}",
                names.first().unwrap_or(&"<empty>".to_string())
            )
        },
    });

    // 3. mimetype content is correct
    if mimetype_first {
        // Read mimetype content
        let cursor2 = Cursor::new(data);
        if let Ok(mut archive2) = ZipArchive::new(cursor2) {
            if let Ok(mut file) = archive2.by_name("mimetype") {
                let mut content = String::new();
                if std::io::Read::read_to_string(&mut file, &mut content).is_ok() {
                    let correct = content.trim() == "application/epub+zip";
                    checks.push(ValidationCheck {
                        name: "mimetype_content".into(),
                        passed: correct,
                        message: if correct {
                            "Correct mimetype".into()
                        } else {
                            format!("Expected 'application/epub+zip', got '{}'", content.trim())
                        },
                    });
                }
            }
        }
    }

    // 4. META-INF/container.xml exists
    let has_container = names.iter().any(|n| n == "META-INF/container.xml");
    checks.push(ValidationCheck {
        name: "container_xml".into(),
        passed: has_container,
        message: if has_container {
            "META-INF/container.xml present".into()
        } else {
            "Missing META-INF/container.xml".into()
        },
    });

    // 5. OPF package document exists
    let has_opf = names.iter().any(|n| n.ends_with(".opf"));
    checks.push(ValidationCheck {
        name: "opf_present".into(),
        passed: has_opf,
        message: if has_opf {
            "OPF package document found".into()
        } else {
            "Missing OPF package document".into()
        },
    });

    // 6. Navigation document exists (EPUB 3)
    let has_nav = names
        .iter()
        .any(|n| n.contains("nav.xhtml") || n.contains("nav.html"));
    checks.push(ValidationCheck {
        name: "nav_document".into(),
        passed: has_nav,
        message: if has_nav {
            "Navigation document present".into()
        } else {
            "Missing navigation document (nav.xhtml)".into()
        },
    });

    // 7. At least one content document
    let content_docs: Vec<_> = names
        .iter()
        .filter(|n| n.ends_with(".xhtml") || n.ends_with(".html"))
        .collect();
    let has_content = !content_docs.is_empty();
    checks.push(ValidationCheck {
        name: "content_documents".into(),
        passed: has_content,
        message: format!("{} content document(s)", content_docs.len()),
    });

    // 8. CSS stylesheet present
    let has_css = names.iter().any(|n| n.ends_with(".css"));
    checks.push(ValidationCheck {
        name: "stylesheet".into(),
        passed: has_css,
        message: if has_css {
            "CSS stylesheet present".into()
        } else {
            "No CSS stylesheet found (recommended)".into()
        },
    });

    // 9. No absolute paths in archive
    let abs_paths: Vec<_> = names.iter().filter(|n| n.starts_with('/')).collect();
    checks.push(ValidationCheck {
        name: "no_absolute_paths".into(),
        passed: abs_paths.is_empty(),
        message: if abs_paths.is_empty() {
            "No absolute paths".into()
        } else {
            format!("{} absolute path(s) found", abs_paths.len())
        },
    });

    // 10. File size sanity
    let size_mb = data.len() as f64 / (1024.0 * 1024.0);
    let size_ok = data.len() < 650 * 1024 * 1024; // 650 MB EPUB limit
    checks.push(ValidationCheck {
        name: "file_size".into(),
        passed: size_ok,
        message: format!("{size_mb:.2} MB (max 650 MB)"),
    });

    let valid = checks.iter().all(|c| c.passed);
    Ok(EpubValidation { valid, checks })
}

impl EpubValidation {
    /// Format as a human-readable report.
    pub fn to_report(&self) -> String {
        let mut report = String::new();
        let passed = self.checks.iter().filter(|c| c.passed).count();
        report.push_str(&format!(
            "EPUB Validation: {} ({}/{} checks)\n\n",
            if self.valid { "VALID" } else { "INVALID" },
            passed,
            self.checks.len(),
        ));
        for check in &self.checks {
            let icon = if check.passed { "+" } else { "-" };
            report.push_str(&format!("  [{icon}] {}: {}\n", check.name, check.message));
        }
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chapter::Chapter;
    use crate::epub::{EpubOptions, generate_epub};
    use crate::metadata::BookMetadata;

    #[test]
    fn test_validate_generated_epub() {
        let meta = BookMetadata::new("Validation Test", "Author", "en");
        let chapters = vec![Chapter::new("Ch1", "<p>Hello</p>", 0)];
        let bytes =
            generate_epub(&meta, &chapters, None, &EpubOptions::default()).expect("Generate EPUB");

        let result = validate_epub_bytes(&bytes).expect("Validate");
        assert!(result.valid, "Report:\n{}", result.to_report());
    }

    #[test]
    fn test_invalid_zip_fails() {
        let result = validate_epub_bytes(b"not a zip file").expect("Should return result");
        assert!(!result.valid);
        assert!(
            result
                .checks
                .iter()
                .any(|c| c.name == "valid_zip" && !c.passed)
        );
    }

    #[test]
    fn test_report_output() {
        let meta = BookMetadata::new("Test", "Author", "en");
        let chapters = vec![Chapter::new("Ch1", "<p>Text</p>", 0)];
        let bytes =
            generate_epub(&meta, &chapters, None, &EpubOptions::default()).expect("Generate");
        let result = validate_epub_bytes(&bytes).expect("Validate");
        let report = result.to_report();
        assert!(report.contains("EPUB Validation"));
        assert!(report.contains("[+]"));
    }
}
