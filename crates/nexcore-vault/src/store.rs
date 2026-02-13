//! Vault store — high-level secret management operations.
//!
//! ## Primitive Foundation
//!
//! | Primitive | Manifestation |
//! |-----------|---------------|
//! | T1: Mapping (μ) | SecretName → EncryptedValue |
//! | T1: State (ς) | Vault lifecycle (create → open → operate → save) |
//! | T1: Exists (∃) | Secret presence check |
//! | T3: Vault | Full encrypted secret store |

use std::collections::BTreeMap;

use crate::cipher;
use crate::config::VaultConfig;
use crate::error::{Result, VaultError};
use crate::kdf;
use crate::persistence;
use crate::types::{PlaintextExport, SecretName, VaultEntry, VaultFile};

/// Tier: T3 — The encrypted secret vault.
pub struct Vault {
    config: VaultConfig,
    file: VaultFile,
    key: [u8; 32],
}

impl Vault {
    /// Create a new vault with the given password.
    ///
    /// # Errors
    /// Returns `VaultError::AlreadyExists` if the vault file already exists.
    pub fn create(config: VaultConfig, password: &str) -> Result<Self> {
        if config.vault_path.exists() {
            return Err(VaultError::AlreadyExists(config.vault_path.clone()));
        }

        let salt = kdf::generate_salt()?;
        let salt_bytes = salt.to_bytes()?;
        let key = kdf::derive_key(password.as_bytes(), &salt_bytes, config.pbkdf2_iterations)?;
        let file = VaultFile::new(salt);

        let vault = Self { config, file, key };
        vault.save()?;
        tracing::info!("Created new vault at {}", vault.config.vault_path.display());
        Ok(vault)
    }

    /// Open an existing vault with the given password.
    ///
    /// # Errors
    /// Returns `VaultError::VaultNotFound` if the vault file doesn't exist.
    /// Returns `VaultError::AuthFailed` if the password is wrong.
    pub fn open(config: VaultConfig, password: &str) -> Result<Self> {
        let file = persistence::load_vault(&config.vault_path)?
            .ok_or_else(|| VaultError::VaultNotFound(config.vault_path.clone()))?;

        let salt_bytes = file.salt.to_bytes()?;
        let key = kdf::derive_key(password.as_bytes(), &salt_bytes, config.pbkdf2_iterations)?;

        // Verify password by trying to decrypt the first entry
        if let Some((_name, entry)) = file.entries.iter().next() {
            cipher::decrypt(&key, &entry.nonce, &entry.ciphertext)?;
        }

        tracing::info!("Opened vault ({} entries)", file.entries.len());
        Ok(Self { config, file, key })
    }

    /// Set (or overwrite) a secret.
    ///
    /// # Errors
    /// Returns errors from encryption or I/O.
    pub fn set(&mut self, name: &SecretName, value: &str) -> Result<()> {
        let entry = self.encrypt_entry(name, value)?;
        self.file.entries.insert(name.as_str().to_string(), entry);
        self.save()?;
        tracing::debug!("Set secret: {name}");
        Ok(())
    }

    /// Get a secret value by name.
    ///
    /// # Errors
    /// Returns `VaultError::NotFound` if the secret doesn't exist.
    pub fn get(&self, name: &SecretName) -> Result<String> {
        let entry = self
            .file
            .entries
            .get(name.as_str())
            .ok_or_else(|| VaultError::NotFound(name.to_string()))?;

        let plaintext = cipher::decrypt(&self.key, &entry.nonce, &entry.ciphertext)?;
        String::from_utf8(plaintext)
            .map_err(|e| VaultError::Crypto(format!("not valid UTF-8: {e}")))
    }

    /// Delete a secret by name.
    ///
    /// # Errors
    /// Returns `VaultError::NotFound` if the secret doesn't exist.
    pub fn delete(&mut self, name: &SecretName) -> Result<()> {
        if self.file.entries.remove(name.as_str()).is_none() {
            return Err(VaultError::NotFound(name.to_string()));
        }
        self.save()?;
        tracing::debug!("Deleted secret: {name}");
        Ok(())
    }

    /// Check if a secret exists.
    #[must_use]
    pub fn has(&self, name: &SecretName) -> bool {
        self.file.entries.contains_key(name.as_str())
    }

    /// List all secret names (sorted, since BTreeMap).
    #[must_use]
    pub fn list(&self) -> Vec<String> {
        self.file.entries.keys().cloned().collect()
    }

    /// Get the number of secrets.
    #[must_use]
    pub fn len(&self) -> usize {
        self.file.entries.len()
    }

    /// Check if the vault is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.file.entries.is_empty()
    }

    /// Export all secrets as plaintext.
    ///
    /// # Errors
    /// Returns errors from decryption.
    pub fn export(&self) -> Result<PlaintextExport> {
        let mut secrets = BTreeMap::new();
        for (name, entry) in &self.file.entries {
            let plaintext = cipher::decrypt(&self.key, &entry.nonce, &entry.ciphertext)?;
            let value = String::from_utf8(plaintext)
                .map_err(|e| VaultError::Crypto(format!("'{name}' not UTF-8: {e}")))?;
            secrets.insert(name.clone(), value);
        }
        Ok(PlaintextExport {
            version: 1,
            secrets,
        })
    }

    /// Import secrets from a plaintext export.
    ///
    /// # Errors
    /// Returns errors from encryption or I/O.
    pub fn import(&mut self, export: &PlaintextExport) -> Result<usize> {
        let mut count = 0;
        for (name, value) in &export.secrets {
            let secret_name = SecretName::new(name.clone())?;
            self.set(&secret_name, value)?;
            count += 1;
        }
        Ok(count)
    }

    /// Change the vault password. Re-encrypts all secrets.
    ///
    /// # Errors
    /// Returns errors from key derivation, encryption, or I/O.
    pub fn change_password(&mut self, new_password: &str) -> Result<()> {
        let export = self.export()?;
        let (new_salt, new_key) = self.derive_new_credentials(new_password)?;

        self.file.salt = new_salt;
        self.key = new_key;
        self.file.entries.clear();

        self.reencrypt_all(&export)?;
        self.save()?;
        tracing::info!(
            "Changed password, re-encrypted {} secrets",
            export.secrets.len()
        );
        Ok(())
    }

    // ── Private helpers ──────────────────────────────────────────

    /// Save the current vault state to disk.
    fn save(&self) -> Result<()> {
        persistence::save_vault(
            &self.file,
            &self.config.vault_path,
            self.config.backup_on_write,
            self.config.file_mode,
        )
    }

    /// Encrypt a single entry, preserving created_at if it exists.
    fn encrypt_entry(&self, name: &SecretName, value: &str) -> Result<VaultEntry> {
        let (nonce, ciphertext) = cipher::encrypt(&self.key, value.as_bytes())?;
        let now = chrono::Utc::now().to_rfc3339();

        let existing_created = self
            .file
            .entries
            .get(name.as_str())
            .map(|e| e.created_at.clone());

        Ok(VaultEntry {
            nonce,
            ciphertext,
            created_at: existing_created.unwrap_or_else(|| now.clone()),
            updated_at: now,
        })
    }

    /// Generate new salt + derived key for password change.
    fn derive_new_credentials(&self, password: &str) -> Result<(crate::types::Salt, [u8; 32])> {
        let new_salt = kdf::generate_salt()?;
        let salt_bytes = new_salt.to_bytes()?;
        let new_key = kdf::derive_key(
            password.as_bytes(),
            &salt_bytes,
            self.config.pbkdf2_iterations,
        )?;
        Ok((new_salt, new_key))
    }

    /// Re-encrypt all secrets from a plaintext export.
    fn reencrypt_all(&mut self, export: &PlaintextExport) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        for (name, value) in &export.secrets {
            let (nonce, ciphertext) = cipher::encrypt(&self.key, value.as_bytes())?;
            self.file.entries.insert(
                name.clone(),
                VaultEntry {
                    nonce,
                    ciphertext,
                    created_at: now.clone(),
                    updated_at: now.clone(),
                },
            );
        }
        Ok(())
    }
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn test_config(dir: &std::path::Path) -> VaultConfig {
        VaultConfig {
            vault_path: dir.join("test.enc"),
            pbkdf2_iterations: 1000, // Low for test speed
            backup_on_write: false,
            file_mode: 0o600,
            dir_mode: 0o700,
            config_path: None,
        }
    }

    #[test]
    fn create_and_open_vault() {
        let dir = tempfile::tempdir().ok();
        let dir = match dir.as_ref() {
            Some(d) => d.path(),
            None => return,
        };
        let config = test_config(dir);

        let vault = Vault::create(config.clone(), "test-pass");
        assert!(vault.is_ok());

        let vault = Vault::open(config, "test-pass");
        assert!(vault.is_ok());
    }

    #[test]
    fn create_already_exists() {
        let dir = tempfile::tempdir().ok();
        let dir = match dir.as_ref() {
            Some(d) => d.path(),
            None => return,
        };
        let config = test_config(dir);
        let _ = Vault::create(config.clone(), "pass");
        let result = Vault::create(config, "pass");
        assert!(result.is_err());
    }

    #[test]
    fn open_nonexistent_fails() {
        let config = VaultConfig {
            vault_path: PathBuf::from("/tmp/nonexistent-vault-open-test.enc"),
            pbkdf2_iterations: 1000,
            backup_on_write: false,
            file_mode: 0o600,
            dir_mode: 0o700,
            config_path: None,
        };
        let result = Vault::open(config, "pass");
        assert!(result.is_err());
    }

    #[test]
    fn set_get_round_trip() {
        let dir = tempfile::tempdir().ok();
        let dir = match dir.as_ref() {
            Some(d) => d.path(),
            None => return,
        };
        let config = test_config(dir);
        let vault = Vault::create(config, "pass");
        let mut vault = match vault {
            Ok(v) => v,
            Err(_) => return,
        };

        let name = SecretName::new("api_key").unwrap_or(SecretName("x".into()));
        let set_result = vault.set(&name, "sk-abc123");
        assert!(set_result.is_ok());

        let got = vault.get(&name);
        assert!(got.is_ok());
        assert_eq!(got.unwrap_or_default(), "sk-abc123");
    }

    #[test]
    fn get_nonexistent_returns_not_found() {
        let dir = tempfile::tempdir().ok();
        let dir = match dir.as_ref() {
            Some(d) => d.path(),
            None => return,
        };
        let config = test_config(dir);
        let vault = Vault::create(config, "pass");
        let vault = match vault {
            Ok(v) => v,
            Err(_) => return,
        };

        let name = SecretName::new("missing").unwrap_or(SecretName("x".into()));
        assert!(vault.get(&name).is_err());
    }

    #[test]
    fn delete_secret() {
        let dir = tempfile::tempdir().ok();
        let dir = match dir.as_ref() {
            Some(d) => d.path(),
            None => return,
        };
        let config = test_config(dir);
        let mut vault = match Vault::create(config, "pass") {
            Ok(v) => v,
            Err(_) => return,
        };

        let name = SecretName::new("to_delete").unwrap_or(SecretName("x".into()));
        let _ = vault.set(&name, "value");
        assert!(vault.has(&name));
        assert!(vault.delete(&name).is_ok());
        assert!(!vault.has(&name));
    }

    #[test]
    fn delete_nonexistent_fails() {
        let dir = tempfile::tempdir().ok();
        let dir = match dir.as_ref() {
            Some(d) => d.path(),
            None => return,
        };
        let config = test_config(dir);
        let mut vault = match Vault::create(config, "pass") {
            Ok(v) => v,
            Err(_) => return,
        };

        let name = SecretName::new("missing").unwrap_or(SecretName("x".into()));
        assert!(vault.delete(&name).is_err());
    }

    #[test]
    fn list_sorted() {
        let dir = tempfile::tempdir().ok();
        let dir = match dir.as_ref() {
            Some(d) => d.path(),
            None => return,
        };
        let config = test_config(dir);
        let mut vault = match Vault::create(config, "pass") {
            Ok(v) => v,
            Err(_) => return,
        };

        for n in &["zebra", "apple", "mango"] {
            let name = SecretName::new(*n).unwrap_or(SecretName("x".into()));
            let _ = vault.set(&name, "val");
        }

        assert_eq!(vault.list(), vec!["apple", "mango", "zebra"]);
    }

    #[test]
    fn has_correctness() {
        let dir = tempfile::tempdir().ok();
        let dir = match dir.as_ref() {
            Some(d) => d.path(),
            None => return,
        };
        let config = test_config(dir);
        let mut vault = match Vault::create(config, "pass") {
            Ok(v) => v,
            Err(_) => return,
        };

        let name = SecretName::new("exists").unwrap_or(SecretName("x".into()));
        assert!(!vault.has(&name));
        let _ = vault.set(&name, "val");
        assert!(vault.has(&name));
    }

    #[test]
    fn overwrite_existing_secret() {
        let dir = tempfile::tempdir().ok();
        let dir = match dir.as_ref() {
            Some(d) => d.path(),
            None => return,
        };
        let config = test_config(dir);
        let mut vault = match Vault::create(config, "pass") {
            Ok(v) => v,
            Err(_) => return,
        };

        let name = SecretName::new("key").unwrap_or(SecretName("x".into()));
        let _ = vault.set(&name, "v1");
        let _ = vault.set(&name, "v2");
        assert_eq!(vault.get(&name).unwrap_or_default(), "v2");
        assert_eq!(vault.len(), 1);
    }

    #[test]
    fn export_import_round_trip() {
        let dir = tempfile::tempdir().ok();
        let dir = match dir.as_ref() {
            Some(d) => d.path(),
            None => return,
        };

        let cfg1 = VaultConfig {
            vault_path: dir.join("v1.enc"),
            pbkdf2_iterations: 1000,
            backup_on_write: false,
            file_mode: 0o600,
            dir_mode: 0o700,
            config_path: None,
        };

        let mut v1 = match Vault::create(cfg1, "pass1") {
            Ok(v) => v,
            Err(_) => return,
        };

        let n1 = SecretName::new("key1").unwrap_or(SecretName("x".into()));
        let n2 = SecretName::new("key2").unwrap_or(SecretName("x".into()));
        let _ = v1.set(&n1, "val1");
        let _ = v1.set(&n2, "val2");

        let export = match v1.export() {
            Ok(e) => e,
            Err(_) => return,
        };

        let cfg2 = VaultConfig {
            vault_path: dir.join("v2.enc"),
            pbkdf2_iterations: 1000,
            backup_on_write: false,
            file_mode: 0o600,
            dir_mode: 0o700,
            config_path: None,
        };
        let mut v2 = match Vault::create(cfg2, "pass2") {
            Ok(v) => v,
            Err(_) => return,
        };

        let count = v2.import(&export);
        assert_eq!(count.unwrap_or(0), 2);
        assert_eq!(v2.get(&n1).unwrap_or_default(), "val1");
        assert_eq!(v2.get(&n2).unwrap_or_default(), "val2");
    }

    #[test]
    fn wrong_password_fails() {
        let dir = tempfile::tempdir().ok();
        let dir = match dir.as_ref() {
            Some(d) => d.path(),
            None => return,
        };
        let config = test_config(dir);
        let mut vault = match Vault::create(config.clone(), "correct") {
            Ok(v) => v,
            Err(_) => return,
        };

        let name = SecretName::new("test").unwrap_or(SecretName("x".into()));
        let _ = vault.set(&name, "value");

        assert!(Vault::open(config, "wrong").is_err());
    }

    #[test]
    fn change_password_works() {
        let dir = tempfile::tempdir().ok();
        let dir = match dir.as_ref() {
            Some(d) => d.path(),
            None => return,
        };
        let config = test_config(dir);
        let mut vault = match Vault::create(config.clone(), "old-pass") {
            Ok(v) => v,
            Err(_) => return,
        };

        let name = SecretName::new("secret").unwrap_or(SecretName("x".into()));
        let _ = vault.set(&name, "hidden");
        assert!(vault.change_password("new-pass").is_ok());

        assert!(Vault::open(config.clone(), "old-pass").is_err());
        let new_v = Vault::open(config, "new-pass");
        assert!(new_v.is_ok());
        let new_v = match new_v {
            Ok(v) => v,
            Err(_) => return,
        };
        assert_eq!(new_v.get(&name).unwrap_or_default(), "hidden");
    }

    #[test]
    fn len_and_is_empty() {
        let dir = tempfile::tempdir().ok();
        let dir = match dir.as_ref() {
            Some(d) => d.path(),
            None => return,
        };
        let config = test_config(dir);
        let mut vault = match Vault::create(config, "pass") {
            Ok(v) => v,
            Err(_) => return,
        };

        assert!(vault.is_empty());
        assert_eq!(vault.len(), 0);

        let name = SecretName::new("k").unwrap_or(SecretName("x".into()));
        let _ = vault.set(&name, "v");
        assert!(!vault.is_empty());
        assert_eq!(vault.len(), 1);
    }

    #[test]
    fn persistence_across_open() {
        let dir = tempfile::tempdir().ok();
        let dir = match dir.as_ref() {
            Some(d) => d.path(),
            None => return,
        };
        let config = test_config(dir);

        {
            let mut vault = match Vault::create(config.clone(), "pass") {
                Ok(v) => v,
                Err(_) => return,
            };
            let n = SecretName::new("persist").unwrap_or(SecretName("x".into()));
            let _ = vault.set(&n, "val");
        }

        {
            let vault = match Vault::open(config, "pass") {
                Ok(v) => v,
                Err(_) => return,
            };
            let n = SecretName::new("persist").unwrap_or(SecretName("x".into()));
            assert_eq!(vault.get(&n).unwrap_or_default(), "val");
        }
    }
}
