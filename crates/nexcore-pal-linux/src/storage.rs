// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Linux storage implementation via std::fs.
//!
//! Tier: T3 (π Persistence + μ Mapping + ∂ Boundary — Linux-specific)

use nexcore_pal::Storage;
use nexcore_pal::error::StorageError;
use std::fs;
use std::path::Path;

/// Linux storage backed by the filesystem.
///
/// Tier: T3 (Linux-specific storage implementation)
pub struct LinuxStorage {
    /// Root path for all storage operations.
    root: String,
}

impl LinuxStorage {
    /// Create a new storage subsystem rooted at the given path.
    pub fn new(root: &str) -> Self {
        Self {
            root: root.to_string(),
        }
    }

    /// Get the storage root path.
    pub fn root(&self) -> &str {
        &self.root
    }

    /// Resolve a relative path against the root.
    fn resolve(&self, path: &str) -> String {
        if path.starts_with('/') {
            path.to_string()
        } else {
            format!("{}/{}", self.root, path)
        }
    }
}

impl Storage for LinuxStorage {
    fn read(&self, path: &str) -> Result<Vec<u8>, StorageError> {
        let resolved = self.resolve(path);
        fs::read(&resolved).map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => StorageError::NotFound,
            std::io::ErrorKind::PermissionDenied => StorageError::PermissionDenied,
            _ => StorageError::IoError,
        })
    }

    fn write(&mut self, path: &str, data: &[u8]) -> Result<(), StorageError> {
        let resolved = self.resolve(path);

        // Ensure parent directory exists
        if let Some(parent) = Path::new(&resolved).parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).map_err(|e| match e.kind() {
                    std::io::ErrorKind::PermissionDenied => StorageError::PermissionDenied,
                    _ => StorageError::IoError,
                })?;
            }
        }

        fs::write(&resolved, data).map_err(|e| match e.kind() {
            std::io::ErrorKind::PermissionDenied => StorageError::PermissionDenied,
            _ => StorageError::IoError,
        })
    }

    fn delete(&mut self, path: &str) -> Result<(), StorageError> {
        let resolved = self.resolve(path);
        fs::remove_file(&resolved).map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => StorageError::NotFound,
            std::io::ErrorKind::PermissionDenied => StorageError::PermissionDenied,
            _ => StorageError::IoError,
        })
    }

    fn exists(&self, path: &str) -> bool {
        let resolved = self.resolve(path);
        Path::new(&resolved).exists()
    }

    fn available_bytes(&self) -> Result<u64, StorageError> {
        // Use statvfs via nix crate
        #[cfg(target_os = "linux")]
        {
            use nix::sys::statvfs::statvfs;
            let stat = statvfs(self.root.as_str()).map_err(|_| StorageError::IoError)?;
            // Use `as u64` for portability: identity on x86_64 (already u64),
            // widening on armv7 (u32 → u64).
            #[allow(
                clippy::unnecessary_cast,
                reason = "Cross-arch portability for statvfs numeric types"
            )]
            let avail = stat.blocks_available() as u64 * stat.fragment_size() as u64;
            Ok(avail)
        }
        #[cfg(not(target_os = "linux"))]
        {
            Err(StorageError::IoError)
        }
    }

    fn total_bytes(&self) -> Result<u64, StorageError> {
        #[cfg(target_os = "linux")]
        {
            use nix::sys::statvfs::statvfs;
            let stat = statvfs(self.root.as_str()).map_err(|_| StorageError::IoError)?;
            // Use `as u64` for portability: identity on x86_64 (already u64),
            // widening on armv7 (u32 → u64).
            #[allow(
                clippy::unnecessary_cast,
                reason = "Cross-arch portability for statvfs numeric types"
            )]
            let total = stat.blocks() as u64 * stat.fragment_size() as u64;
            Ok(total)
        }
        #[cfg(not(target_os = "linux"))]
        {
            Err(StorageError::IoError)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn storage_write_read_delete() {
        let dir = tempfile::tempdir().unwrap_or_else(|_| {
            tempfile::TempDir::new().unwrap_or_else(|_| {
                // Fallback — shouldn't happen in tests
                tempfile::tempdir().ok().unwrap()
            })
        });
        let root = dir.path().to_string_lossy().to_string();
        let mut storage = LinuxStorage::new(&root);

        // Write
        let data = b"hello nexcore os";
        let result = storage.write("test.txt", data);
        assert!(result.is_ok());

        // Exists
        assert!(storage.exists("test.txt"));

        // Read
        let read_result = storage.read("test.txt");
        assert!(read_result.is_ok());
        if let Ok(read_data) = read_result {
            assert_eq!(read_data, data);
        }

        // Delete
        let del_result = storage.delete("test.txt");
        assert!(del_result.is_ok());
        assert!(!storage.exists("test.txt"));
    }

    #[test]
    fn read_nonexistent() {
        let storage = LinuxStorage::new("/tmp");
        let result = storage.read("nonexistent_file_nexcore_test_12345.txt");
        assert!(result.is_err());
    }

    #[test]
    fn nested_write() {
        let dir = tempfile::tempdir().unwrap_or_else(|_| tempfile::tempdir().ok().unwrap());
        let root = dir.path().to_string_lossy().to_string();
        let mut storage = LinuxStorage::new(&root);

        let result = storage.write("sub/dir/file.txt", b"nested");
        assert!(result.is_ok());
        assert!(storage.exists("sub/dir/file.txt"));
    }

    #[test]
    fn absolute_path_resolution() {
        let storage = LinuxStorage::new("/tmp");
        assert_eq!(storage.resolve("/absolute/path"), "/absolute/path");
        assert_eq!(storage.resolve("relative"), "/tmp/relative");
    }

    #[test]
    fn available_bytes_works() {
        let storage = LinuxStorage::new("/tmp");
        let result = storage.available_bytes();
        // On Linux this should succeed for /tmp
        #[cfg(target_os = "linux")]
        assert!(result.is_ok());
    }
}
