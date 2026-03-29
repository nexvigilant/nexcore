//! Cover image validation for EPUB and Kindle publishing.

use std::path::Path;

use crate::error::{PublishingError, Result};

/// Cover image specification and validation.
#[derive(Debug, Clone)]
pub struct CoverSpec {
    /// Path to the cover image file.
    pub path: std::path::PathBuf,
    /// Image width in pixels (read from file or provided).
    pub width: u32,
    /// Image height in pixels.
    pub height: u32,
    /// Image format.
    pub format: ImageFormat,
    /// File size in bytes.
    pub file_size: u64,
}

/// Supported cover image formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    Jpeg,
    Png,
    Gif,
    Webp,
}

impl ImageFormat {
    /// Detect format from file extension.
    pub fn from_extension(path: &Path) -> Option<Self> {
        let ext = path.extension()?.to_str()?.to_lowercase();
        match ext.as_str() {
            "jpg" | "jpeg" => Some(Self::Jpeg),
            "png" => Some(Self::Png),
            "gif" => Some(Self::Gif),
            "webp" => Some(Self::Webp),
            _ => None,
        }
    }

    /// MIME type for the EPUB manifest.
    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::Jpeg => "image/jpeg",
            Self::Png => "image/png",
            Self::Gif => "image/gif",
            Self::Webp => "image/webp",
        }
    }

    /// File extension for the EPUB content.
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Jpeg => "jpg",
            Self::Png => "png",
            Self::Gif => "gif",
            Self::Webp => "webp",
        }
    }
}

/// KDP (Kindle Direct Publishing) cover requirements.
pub struct KdpCoverRequirements;

impl KdpCoverRequirements {
    /// Minimum width in pixels.
    pub const MIN_WIDTH: u32 = 625;
    /// Maximum width in pixels.
    pub const MAX_WIDTH: u32 = 10000;
    /// Minimum height in pixels.
    pub const MIN_HEIGHT: u32 = 1000;
    /// Maximum height in pixels.
    pub const MAX_HEIGHT: u32 = 10000;
    /// Ideal aspect ratio (height:width) = 1.6:1.
    pub const IDEAL_RATIO: f64 = 1.6;
    /// Ratio tolerance.
    pub const RATIO_TOLERANCE: f64 = 0.2;
    /// Maximum file size: 50 MB.
    pub const MAX_FILE_SIZE: u64 = 50 * 1024 * 1024;
    /// Recommended minimum DPI.
    pub const MIN_DPI: u32 = 300;
}

/// EPUB cover requirements (less strict than Kindle).
pub struct EpubCoverRequirements;

impl EpubCoverRequirements {
    /// Minimum recommended width.
    pub const MIN_WIDTH: u32 = 600;
    /// Minimum recommended height.
    pub const MIN_HEIGHT: u32 = 800;
    /// Maximum file size: 20 MB (practical limit for ereaders).
    pub const MAX_FILE_SIZE: u64 = 20 * 1024 * 1024;
}

/// Validation result for cover images.
#[derive(Debug, Clone)]
pub struct CoverValidation {
    /// Overall pass/fail.
    pub valid: bool,
    /// Individual check results.
    pub checks: Vec<CoverCheck>,
}

/// A single validation check on the cover image.
#[derive(Debug, Clone)]
pub struct CoverCheck {
    /// Check name.
    pub name: String,
    /// Pass or fail.
    pub passed: bool,
    /// Description of the result.
    pub message: String,
}

impl CoverSpec {
    /// Create a `CoverSpec` from a file path by reading dimensions from the file header.
    ///
    /// Uses a lightweight header-only read — no full image decode.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(PublishingError::Cover(format!(
                "File not found: {}",
                path.display()
            )));
        }

        let format = ImageFormat::from_extension(path).ok_or_else(|| {
            PublishingError::Cover(format!("Unsupported image format: {}", path.display()))
        })?;

        let file_size = std::fs::metadata(path)
            .map_err(|e| PublishingError::Cover(format!("Cannot read file metadata: {e}")))?
            .len();

        // Read image dimensions from file header
        let data = std::fs::read(path)?;
        let (width, height) = read_image_dimensions(&data, format).ok_or_else(|| {
            PublishingError::Cover("Could not read image dimensions from file header".into())
        })?;

        Ok(Self {
            path: path.to_path_buf(),
            width,
            height,
            format,
            file_size,
        })
    }

    /// Create from known dimensions (for testing or when dimensions are pre-computed).
    pub fn from_dimensions(
        path: impl Into<std::path::PathBuf>,
        width: u32,
        height: u32,
        format: ImageFormat,
        file_size: u64,
    ) -> Self {
        Self {
            path: path.into(),
            width,
            height,
            format,
            file_size,
        }
    }

    /// Validate against Kindle/KDP requirements.
    pub fn validate_kindle(&self) -> CoverValidation {
        let mut checks = Vec::new();

        // Dimension checks
        checks.push(CoverCheck {
            name: "min_width".into(),
            passed: self.width >= KdpCoverRequirements::MIN_WIDTH,
            message: format!(
                "Width {} px (min {})",
                self.width,
                KdpCoverRequirements::MIN_WIDTH
            ),
        });
        checks.push(CoverCheck {
            name: "max_width".into(),
            passed: self.width <= KdpCoverRequirements::MAX_WIDTH,
            message: format!(
                "Width {} px (max {})",
                self.width,
                KdpCoverRequirements::MAX_WIDTH
            ),
        });
        checks.push(CoverCheck {
            name: "min_height".into(),
            passed: self.height >= KdpCoverRequirements::MIN_HEIGHT,
            message: format!(
                "Height {} px (min {})",
                self.height,
                KdpCoverRequirements::MIN_HEIGHT
            ),
        });

        // Aspect ratio
        let ratio = f64::from(self.height) / f64::from(self.width);
        let ratio_ok = (ratio - KdpCoverRequirements::IDEAL_RATIO).abs()
            <= KdpCoverRequirements::RATIO_TOLERANCE;
        checks.push(CoverCheck {
            name: "aspect_ratio".into(),
            passed: ratio_ok,
            message: format!(
                "Aspect ratio {ratio:.2}:1 (ideal {:.1}:1 ± {:.1})",
                KdpCoverRequirements::IDEAL_RATIO,
                KdpCoverRequirements::RATIO_TOLERANCE
            ),
        });

        // File size
        checks.push(CoverCheck {
            name: "file_size".into(),
            passed: self.file_size <= KdpCoverRequirements::MAX_FILE_SIZE,
            message: format!(
                "File size {} KB (max {} MB)",
                self.file_size / 1024,
                KdpCoverRequirements::MAX_FILE_SIZE / (1024 * 1024)
            ),
        });

        // Format
        let format_ok = matches!(self.format, ImageFormat::Jpeg | ImageFormat::Png);
        checks.push(CoverCheck {
            name: "format".into(),
            passed: format_ok,
            message: format!("Format: {:?} (JPEG or PNG required for KDP)", self.format),
        });

        let valid = checks.iter().all(|c| c.passed);
        CoverValidation { valid, checks }
    }

    /// Validate against general EPUB requirements.
    pub fn validate_epub(&self) -> CoverValidation {
        let mut checks = Vec::new();

        checks.push(CoverCheck {
            name: "min_width".into(),
            passed: self.width >= EpubCoverRequirements::MIN_WIDTH,
            message: format!(
                "Width {} px (min {})",
                self.width,
                EpubCoverRequirements::MIN_WIDTH
            ),
        });
        checks.push(CoverCheck {
            name: "min_height".into(),
            passed: self.height >= EpubCoverRequirements::MIN_HEIGHT,
            message: format!(
                "Height {} px (min {})",
                self.height,
                EpubCoverRequirements::MIN_HEIGHT
            ),
        });
        checks.push(CoverCheck {
            name: "file_size".into(),
            passed: self.file_size <= EpubCoverRequirements::MAX_FILE_SIZE,
            message: format!(
                "File size {} KB (max {} MB)",
                self.file_size / 1024,
                EpubCoverRequirements::MAX_FILE_SIZE / (1024 * 1024)
            ),
        });

        let valid = checks.iter().all(|c| c.passed);
        CoverValidation { valid, checks }
    }

    /// EPUB manifest filename for the cover image.
    pub fn epub_filename(&self) -> String {
        format!("cover.{}", self.format.extension())
    }
}

/// Read image dimensions from raw file bytes without decoding the full image.
fn read_image_dimensions(data: &[u8], format: ImageFormat) -> Option<(u32, u32)> {
    match format {
        ImageFormat::Png => read_png_dimensions(data),
        ImageFormat::Jpeg => read_jpeg_dimensions(data),
        ImageFormat::Gif => read_gif_dimensions(data),
        ImageFormat::Webp => read_webp_dimensions(data),
    }
}

/// PNG: dimensions at bytes 16-23 of the IHDR chunk.
fn read_png_dimensions(data: &[u8]) -> Option<(u32, u32)> {
    // PNG signature (8 bytes) + IHDR length (4) + "IHDR" (4) + width (4) + height (4) = 24 bytes min
    if data.len() < 24 {
        return None;
    }
    // Check PNG signature
    if &data[0..8] != b"\x89PNG\r\n\x1a\n" {
        return None;
    }
    let width = u32::from_be_bytes([data[16], data[17], data[18], data[19]]);
    let height = u32::from_be_bytes([data[20], data[21], data[22], data[23]]);
    Some((width, height))
}

/// JPEG: scan for SOF0/SOF2 marker to find dimensions.
fn read_jpeg_dimensions(data: &[u8]) -> Option<(u32, u32)> {
    if data.len() < 2 || data[0] != 0xFF || data[1] != 0xD8 {
        return None;
    }
    let mut pos = 2;
    while pos + 4 < data.len() {
        if data[pos] != 0xFF {
            pos += 1;
            continue;
        }
        let marker = data[pos + 1];
        // SOF0 (0xC0) or SOF2 (0xC2) — baseline or progressive
        if marker == 0xC0 || marker == 0xC2 {
            if pos + 9 < data.len() {
                let height = u32::from(data[pos + 5]) << 8 | u32::from(data[pos + 6]);
                let width = u32::from(data[pos + 7]) << 8 | u32::from(data[pos + 8]);
                return Some((width, height));
            }
            return None;
        }
        // Skip this marker segment
        if pos + 3 < data.len() {
            let seg_len = u16::from_be_bytes([data[pos + 2], data[pos + 3]]) as usize;
            pos += 2 + seg_len;
        } else {
            break;
        }
    }
    None
}

/// GIF: dimensions at bytes 6-9 (little-endian).
fn read_gif_dimensions(data: &[u8]) -> Option<(u32, u32)> {
    if data.len() < 10 {
        return None;
    }
    if &data[0..3] != b"GIF" {
        return None;
    }
    let width = u32::from(u16::from_le_bytes([data[6], data[7]]));
    let height = u32::from(u16::from_le_bytes([data[8], data[9]]));
    Some((width, height))
}

/// WebP: dimensions from VP8/VP8L header.
fn read_webp_dimensions(data: &[u8]) -> Option<(u32, u32)> {
    if data.len() < 30 {
        return None;
    }
    if &data[0..4] != b"RIFF" || &data[8..12] != b"WEBP" {
        return None;
    }
    // VP8 lossy
    if &data[12..16] == b"VP8 " && data.len() >= 30 {
        let width = u32::from(u16::from_le_bytes([data[26], data[27]])) & 0x3FFF;
        let height = u32::from(u16::from_le_bytes([data[28], data[29]])) & 0x3FFF;
        return Some((width, height));
    }
    // VP8L lossless
    if &data[12..16] == b"VP8L" && data.len() >= 25 {
        let b1 = u32::from(data[21]);
        let b2 = u32::from(data[22]);
        let b3 = u32::from(data[23]);
        let b4 = u32::from(data[24]);
        let width = (b1 | (b2 & 0x3F) << 8) + 1;
        let height = ((b2 >> 6) | (b3 << 2) | (b4 & 0xF) << 10) + 1;
        return Some((width, height));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_image_format_detection() {
        assert_eq!(
            ImageFormat::from_extension(Path::new("cover.jpg")),
            Some(ImageFormat::Jpeg)
        );
        assert_eq!(
            ImageFormat::from_extension(Path::new("cover.jpeg")),
            Some(ImageFormat::Jpeg)
        );
        assert_eq!(
            ImageFormat::from_extension(Path::new("cover.png")),
            Some(ImageFormat::Png)
        );
        assert_eq!(
            ImageFormat::from_extension(Path::new("cover.gif")),
            Some(ImageFormat::Gif)
        );
        assert_eq!(ImageFormat::from_extension(Path::new("cover.txt")), None);
    }

    #[test]
    fn test_kindle_validation_good_cover() {
        let spec = CoverSpec::from_dimensions(
            PathBuf::from("cover.jpg"),
            1600,
            2560,
            ImageFormat::Jpeg,
            500_000,
        );
        let result = spec.validate_kindle();
        assert!(result.valid);
    }

    #[test]
    fn test_kindle_validation_too_small() {
        let spec = CoverSpec::from_dimensions(
            PathBuf::from("cover.jpg"),
            400,
            600,
            ImageFormat::Jpeg,
            100_000,
        );
        let result = spec.validate_kindle();
        assert!(!result.valid);
        assert!(
            !result
                .checks
                .iter()
                .find(|c| c.name == "min_width")
                .map_or(false, |c| c.passed)
        );
    }

    #[test]
    fn test_epub_validation() {
        let spec = CoverSpec::from_dimensions(
            PathBuf::from("cover.png"),
            800,
            1200,
            ImageFormat::Png,
            200_000,
        );
        let result = spec.validate_epub();
        assert!(result.valid);
    }

    #[test]
    fn test_mime_types() {
        assert_eq!(ImageFormat::Jpeg.mime_type(), "image/jpeg");
        assert_eq!(ImageFormat::Png.mime_type(), "image/png");
    }

    #[test]
    fn test_png_dimension_reading() {
        // Minimal valid PNG header
        let mut data = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]; // signature
        data.extend_from_slice(&[0, 0, 0, 13]); // IHDR length
        data.extend_from_slice(b"IHDR"); // chunk type
        data.extend_from_slice(&800u32.to_be_bytes()); // width
        data.extend_from_slice(&1200u32.to_be_bytes()); // height
        assert_eq!(read_png_dimensions(&data), Some((800, 1200)));
    }
}
