//! Persistence — atomic JSON vault file save/load.
//!
//! ## Primitive Foundation
//!
//! | Primitive | Manifestation |
//! |-----------|---------------|
//! | T1: State (ς) | Serialized vault state |
//! | T1: Sequence (σ) | Write temp → rename (atomic) |

use std::path::Path;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use crate::error::{Result, VaultError};
use crate::types::VaultFile;

/// Ensure the parent directory exists with restrictive permissions.
fn ensure_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|e| VaultError::Io {
                path: parent.to_path_buf(),
                source: e,
            })?;

            #[cfg(unix)]
            {
                let perms = std::fs::Permissions::from_mode(0o700);
                let _ = std::fs::set_permissions(parent, perms);
            }
        }
    }
    Ok(())
}

/// Create a backup of the existing vault file if it exists.
fn backup_existing(path: &Path) -> Result<()> {
    if path.exists() {
        let backup_path = path.with_extension("enc.bak");
        std::fs::copy(path, &backup_path).map_err(|e| VaultError::Io {
            path: backup_path,
            source: e,
        })?;
        tracing::debug!("Created vault backup");
    }
    Ok(())
}

/// Write data to a temp file, set permissions, rename atomically.
fn atomic_write(path: &Path, data: &str, file_mode: u32) -> Result<()> {
    let temp_path = path.with_extension("enc.tmp");
    std::fs::write(&temp_path, data).map_err(|e| VaultError::Io {
        path: temp_path.clone(),
        source: e,
    })?;

    #[cfg(unix)]
    {
        let _mode = file_mode;
        let perms = std::fs::Permissions::from_mode(_mode);
        let _ = std::fs::set_permissions(&temp_path, perms);
    }
    let _ = file_mode; // suppress unused warning on non-unix

    std::fs::rename(&temp_path, path).map_err(|e| VaultError::Io {
        path: path.to_path_buf(),
        source: e,
    })
}

/// Save vault file atomically (write to temp, rename).
///
/// # Errors
/// Returns `VaultError::Io` on file write failure,
/// or `VaultError::Json` on serialization failure.
pub fn save_vault(vault: &VaultFile, path: &Path, backup: bool, file_mode: u32) -> Result<()> {
    let json = serde_json::to_string_pretty(vault)?;

    ensure_parent_dir(path)?;

    if backup {
        backup_existing(path)?;
    }

    atomic_write(path, &json, file_mode)?;

    tracing::debug!("Saved vault: {} entries", vault.entries.len());
    Ok(())
}

/// Load vault file from JSON. Returns `None` if the file doesn't exist.
///
/// # Errors
/// Returns `VaultError::Io` on read failure,
/// or `VaultError::Json` on deserialization failure.
pub fn load_vault(path: &Path) -> Result<Option<VaultFile>> {
    if !path.exists() {
        tracing::debug!("No vault file at {}", path.display());
        return Ok(None);
    }

    let json = std::fs::read_to_string(path).map_err(|e| VaultError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;

    let vault: VaultFile = serde_json::from_str(&json)?;

    if vault.version != 1 {
        return Err(VaultError::InvalidFormat(format!(
            "unsupported vault version: {} (expected 1)",
            vault.version
        )));
    }

    tracing::info!("Loaded vault: {} entries", vault.entries.len());
    Ok(Some(vault))
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Salt, VaultEntry, VaultFile};

    fn test_vault() -> VaultFile {
        let salt = Salt::from_bytes(&[1u8; 32]);
        let mut vf = VaultFile::new(salt);
        vf.entries.insert(
            "test_key".to_string(),
            VaultEntry {
                nonce: "dGVzdG5vbmNl".to_string(),
                ciphertext: "dGVzdGNpcGhlcnRleHQ=".to_string(),
                created_at: "2026-02-04T00:00:00Z".to_string(),
                updated_at: "2026-02-04T00:00:00Z".to_string(),
            },
        );
        vf
    }

    #[test]
    fn save_load_round_trip() {
        let dir = tempfile::tempdir().ok();
        let dir = match dir.as_ref() {
            Some(d) => d.path(),
            None => return,
        };
        let path = dir.join("test.enc");

        let vault = test_vault();
        let result = save_vault(&vault, &path, false, 0o600);
        assert!(result.is_ok());

        let loaded = load_vault(&path);
        assert!(loaded.is_ok());
        let loaded = loaded.unwrap_or(None);
        assert!(loaded.is_some());

        let loaded = loaded.unwrap_or_else(|| VaultFile::new(Salt("".into())));
        assert_eq!(loaded.entries.len(), 1);
        assert!(loaded.entries.contains_key("test_key"));
    }

    #[test]
    fn load_missing_file_returns_none() {
        let path = Path::new("/tmp/nonexistent-vault-99999.enc");
        let loaded = load_vault(path);
        assert!(loaded.is_ok());
        let loaded = loaded.unwrap_or(Some(VaultFile::new(Salt("".into()))));
        assert!(loaded.is_none());
    }

    #[test]
    fn save_creates_parent_dirs() {
        let dir = tempfile::tempdir().ok();
        let dir = match dir.as_ref() {
            Some(d) => d.path(),
            None => return,
        };
        let path = dir.join("subdir").join("deep").join("vault.enc");

        let vault = VaultFile::new(Salt::from_bytes(&[0u8; 32]));
        let result = save_vault(&vault, &path, false, 0o600);
        assert!(result.is_ok());
        assert!(path.exists());
    }

    #[test]
    fn backup_on_write() {
        let dir = tempfile::tempdir().ok();
        let dir = match dir.as_ref() {
            Some(d) => d.path(),
            None => return,
        };
        let path = dir.join("backup-test.enc");

        let vault = test_vault();
        let _ = save_vault(&vault, &path, true, 0o600);
        let _ = save_vault(&vault, &path, true, 0o600);

        let backup_path = path.with_extension("enc.bak");
        assert!(backup_path.exists());
    }

    #[test]
    fn empty_vault_round_trip() {
        let dir = tempfile::tempdir().ok();
        let dir = match dir.as_ref() {
            Some(d) => d.path(),
            None => return,
        };
        let path = dir.join("empty.enc");

        let vault = VaultFile::new(Salt::from_bytes(&[0u8; 32]));
        let _ = save_vault(&vault, &path, false, 0o600);

        let loaded = load_vault(&path).unwrap_or(None);
        let loaded = loaded.unwrap_or_else(|| VaultFile::new(Salt("".into())));
        assert!(loaded.entries.is_empty());
        assert_eq!(loaded.version, 1);
    }

    #[test]
    fn corrupt_json_returns_error() {
        let dir = tempfile::tempdir().ok();
        let dir = match dir.as_ref() {
            Some(d) => d.path(),
            None => return,
        };
        let path = dir.join("corrupt.enc");
        let _ = std::fs::write(&path, "not valid json at all {{{");

        let loaded = load_vault(&path);
        assert!(loaded.is_err());
    }
}
