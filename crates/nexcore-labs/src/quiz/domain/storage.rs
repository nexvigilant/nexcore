//! Storage domain types.
//!
//! Defines file storage items and metadata.

use nexcore_chrono::DateTime;
use nexcore_id::NexId;
use serde::{Deserialize, Serialize};

/// A stored file item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageItem {
    /// Unique file identifier.
    pub id: NexId,

    /// Upload timestamp.
    pub uploaded_at: DateTime,

    /// MIME type of the file.
    pub mime_type: String,

    /// MD5 hash of file content (16 bytes).
    pub hash: Option<Vec<u8>>,

    /// Owner user ID (None if orphaned).
    pub user_id: Option<NexId>,

    /// File size in bytes.
    pub size: i64,

    /// Storage path/key.
    pub storage_path: Option<String>,

    /// Soft deletion timestamp.
    pub deleted_at: Option<DateTime>,

    /// Alt text for accessibility.
    pub alt_text: Option<String>,

    /// Original filename.
    pub filename: Option<String>,

    /// ThumbHash for image previews.
    pub thumbhash: Option<String>,

    /// Storage server identifier.
    pub server: Option<String>,

    /// Whether this was imported (e.g., from Kahoot).
    pub imported: bool,
}

impl StorageItem {
    /// Create a new storage item.
    pub fn new(user_id: NexId, mime_type: String, size: i64, storage_path: String) -> Self {
        Self {
            id: NexId::v4(),
            uploaded_at: DateTime::now(),
            mime_type,
            hash: None,
            user_id: Some(user_id),
            size,
            storage_path: Some(storage_path),
            deleted_at: None,
            alt_text: None,
            filename: None,
            thumbhash: None,
            server: None,
            imported: false,
        }
    }

    /// Check if the item has been soft-deleted.
    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }

    /// Soft-delete the item.
    pub fn delete(&mut self) {
        self.deleted_at = Some(DateTime::now());
    }

    /// Check if this is an image.
    pub fn is_image(&self) -> bool {
        self.mime_type.starts_with("image/")
    }

    /// Check if this is a video.
    pub fn is_video(&self) -> bool {
        self.mime_type.starts_with("video/")
    }

    /// Get hash as hex string.
    pub fn hash_hex(&self) -> Option<String> {
        self.hash.as_ref().map(|h| bytes_to_hex(h))
    }
}

/// Public storage item response (for API).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicStorageItem {
    /// File ID.
    pub id: NexId,

    /// Upload timestamp.
    pub uploaded_at: DateTime,

    /// MIME type.
    pub mime_type: String,

    /// Hash as hex string.
    pub hash: Option<String>,

    /// File size in bytes.
    pub size: i64,

    /// Deletion timestamp.
    pub deleted_at: Option<DateTime>,

    /// Alt text.
    pub alt_text: Option<String>,

    /// Original filename.
    pub filename: Option<String>,

    /// ThumbHash.
    pub thumbhash: Option<String>,

    /// Server identifier.
    pub server: Option<String>,

    /// Whether imported.
    pub imported: bool,
}

impl From<&StorageItem> for PublicStorageItem {
    fn from(item: &StorageItem) -> Self {
        Self {
            id: item.id,
            uploaded_at: item.uploaded_at,
            mime_type: item.mime_type.clone(),
            hash: item.hash_hex(),
            size: item.size,
            deleted_at: item.deleted_at,
            alt_text: item.alt_text.clone(),
            filename: item.filename.clone(),
            thumbhash: item.thumbhash.clone(),
            server: item.server.clone(),
            imported: item.imported,
        }
    }
}

/// Input for updating a storage item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateStorageItem {
    /// New filename.
    pub filename: Option<String>,

    /// New alt text.
    pub alt_text: Option<String>,
}

/// Allowed MIME types for upload.
pub const ALLOWED_MIME_TYPES: &[&str] = &[
    "image/png",
    "image/jpeg",
    "image/gif",
    "image/webp",
    "video/mp4",
];

/// Check if a MIME type is allowed for upload.
///
/// # Arguments
///
/// * `mime_type` - The MIME type string to check
///
/// # Returns
///
/// `true` if the MIME type is in the allowed list, `false` otherwise.
pub fn is_allowed_mime_type(mime_type: &str) -> bool {
    ALLOWED_MIME_TYPES.contains(&mime_type)
}

/// Convert bytes to hex string.
fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_storage_item() {
        let user_id = NexId::v4();
        let item = StorageItem::new(user_id, "image/png".into(), 1024, "uploads/test.png".into());

        assert!(!item.is_deleted());
        assert!(item.is_image());
        assert!(!item.is_video());
        assert_eq!(item.user_id, Some(user_id));
    }

    #[test]
    fn test_soft_delete() {
        let mut item = StorageItem::new(
            NexId::v4(),
            "image/jpeg".into(),
            512,
            "uploads/img.jpg".into(),
        );

        assert!(!item.is_deleted());
        item.delete();
        assert!(item.is_deleted());
    }

    #[test]
    fn test_allowed_mime_types() {
        assert!(is_allowed_mime_type("image/png"));
        assert!(is_allowed_mime_type("video/mp4"));
        assert!(!is_allowed_mime_type("application/pdf"));
        assert!(!is_allowed_mime_type("text/plain"));
    }

    #[test]
    fn test_hash_hex() {
        let mut item = StorageItem::new(NexId::v4(), "image/png".into(), 100, "test".into());

        assert!(item.hash_hex().is_none());

        item.hash = Some(vec![0xde, 0xad, 0xbe, 0xef]);
        assert_eq!(item.hash_hex(), Some("deadbeef".into()));
    }
}
