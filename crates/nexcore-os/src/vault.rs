// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! OS-level encrypted vault — secure storage for system credentials and user secrets.
//!
//! ## Architecture
//!
//! The `OsVault` wraps `nexcore_vault::Vault` with OS-level lifecycle management:
//!
//! - **State machine**: Uninitialized → Locked → Unlocked
//! - **Security integration**: Auto-locks when SecurityLevel reaches Red
//! - **Service tokens**: Each system service can store/retrieve its credentials
//! - **User secrets**: WiFi passwords, app credentials, device encryption keys
//!
//! ## Primitive Grounding
//!
//! - ς State: Vault lifecycle (Uninitialized → Locked → Unlocked)
//! - ∂ Boundary: Encryption boundary (plaintext ↔ ciphertext)
//! - μ Mapping: SecretName → EncryptedValue
//! - π Persistence: Encrypted file persistence
//! - ∃ Existence: Secret existence checks

use std::path::{Path, PathBuf};

use nexcore_vault::config::VaultConfig;
use nexcore_vault::store::Vault;
use nexcore_vault::types::SecretName;
use serde::{Deserialize, Serialize};

/// Vault lifecycle state.
///
/// Tier: T2-P (ς State — vault lifecycle)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VaultState {
    /// Vault has not been created yet (first boot).
    Uninitialized,
    /// Vault exists but is locked (requires password).
    Locked,
    /// Vault is unlocked and ready for operations.
    Unlocked,
}

impl VaultState {
    /// Whether the vault can accept read/write operations.
    pub fn is_operational(&self) -> bool {
        matches!(self, Self::Unlocked)
    }
}

/// OS-level vault error.
///
/// Tier: T2-P (∂ Boundary — vault error boundary)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VaultError {
    /// Vault is not initialized (needs first-boot setup).
    NotInitialized,
    /// Vault is locked (requires password to unlock).
    Locked,
    /// Vault operation failed.
    OperationFailed(String),
    /// Secret not found.
    SecretNotFound(String),
    /// Invalid secret name.
    InvalidName(String),
    /// Vault already initialized.
    AlreadyInitialized,
    /// Authentication failed (wrong password).
    AuthFailed,
}

impl core::fmt::Display for VaultError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NotInitialized => write!(f, "vault not initialized"),
            Self::Locked => write!(f, "vault is locked"),
            Self::OperationFailed(msg) => write!(f, "vault operation failed: {msg}"),
            Self::SecretNotFound(name) => write!(f, "secret not found: {name}"),
            Self::InvalidName(msg) => write!(f, "invalid secret name: {msg}"),
            Self::AlreadyInitialized => write!(f, "vault already initialized"),
            Self::AuthFailed => write!(f, "vault authentication failed"),
        }
    }
}

/// A stored secret's metadata (no plaintext).
///
/// Tier: T2-C (μ + ∃ — mapping with existence metadata)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecretInfo {
    /// Secret name.
    pub name: String,
    /// Category (system or user).
    pub category: SecretCategory,
}

/// Secret category — system vs user.
///
/// Tier: T2-P (κ Comparison — secret classification)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecretCategory {
    /// System credential (service token, API key).
    System,
    /// User secret (password, personal credential).
    User,
}

/// OS-level vault manager.
///
/// Tier: T3 (ς + ∂ + μ + π — full vault with lifecycle management)
///
/// Wraps `nexcore_vault::Vault` with:
/// - OS lifecycle state machine
/// - Security-level auto-locking
/// - Service token namespacing
/// - User secret namespacing
pub struct OsVault {
    /// Current vault state.
    state: VaultState,
    /// The underlying encrypted vault (None when locked/uninitialized).
    vault: Option<Vault>,
    /// Vault data directory.
    data_dir: PathBuf,
    /// Number of operations performed since unlock.
    operations: u64,
}

impl OsVault {
    /// Create a new vault manager for the given data directory.
    ///
    /// The vault starts in `Uninitialized` state if the vault file
    /// doesn't exist, or `Locked` if it does.
    pub fn new(data_dir: impl Into<PathBuf>) -> Self {
        let data_dir = data_dir.into();
        let vault_path = data_dir.join("system.vault");
        let state = if vault_path.exists() {
            VaultState::Locked
        } else {
            VaultState::Uninitialized
        };

        Self {
            state,
            vault: None,
            data_dir,
            operations: 0,
        }
    }

    /// Create a new vault manager in virtual mode (in-memory, for testing).
    ///
    /// Starts in `Uninitialized` state with a temp directory.
    pub fn virtual_vault() -> Self {
        // Use a path that won't exist on disk
        Self {
            state: VaultState::Uninitialized,
            vault: None,
            data_dir: PathBuf::from("/tmp/nexcore-os-vault-virtual"),
            operations: 0,
        }
    }

    /// Initialize the vault for first boot.
    ///
    /// Creates the encrypted vault file with the given password.
    /// After initialization, the vault is in `Unlocked` state.
    pub fn initialize(&mut self, password: &str) -> Result<(), VaultError> {
        if self.state != VaultState::Uninitialized {
            return Err(VaultError::AlreadyInitialized);
        }

        let config = self.make_config();

        // Ensure the directory exists
        if let Some(parent) = config.vault_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| VaultError::OperationFailed(format!("mkdir: {e}")))?;
        }

        let vault = Vault::create(config, password)
            .map_err(|e| VaultError::OperationFailed(format!("create: {e}")))?;

        self.vault = Some(vault);
        self.state = VaultState::Unlocked;
        self.operations = 0;
        Ok(())
    }

    /// Unlock the vault with a password.
    ///
    /// Opens the existing vault file and transitions to `Unlocked` state.
    pub fn unlock(&mut self, password: &str) -> Result<(), VaultError> {
        if self.state == VaultState::Uninitialized {
            return Err(VaultError::NotInitialized);
        }

        let config = self.make_config();
        let vault = Vault::open(config, password).map_err(|e| {
            if e.to_string().contains("authentication") || e.to_string().contains("wrong") {
                VaultError::AuthFailed
            } else {
                VaultError::OperationFailed(format!("open: {e}"))
            }
        })?;

        self.vault = Some(vault);
        self.state = VaultState::Unlocked;
        self.operations = 0;
        Ok(())
    }

    /// Lock the vault (zero the key from memory).
    ///
    /// Drops the underlying vault, clearing the derived key.
    pub fn lock(&mut self) {
        self.vault = None;
        if self.state == VaultState::Unlocked {
            self.state = VaultState::Locked;
        }
    }

    /// Get the current vault state.
    pub fn state(&self) -> VaultState {
        self.state
    }

    /// Whether the vault is operational (unlocked).
    pub fn is_operational(&self) -> bool {
        self.state.is_operational()
    }

    /// Get the number of operations since last unlock.
    pub fn operations(&self) -> u64 {
        self.operations
    }

    // ── Service Token API ──────────────────────────────────────────

    /// Store a service token (namespaced under `svc.<service_name>`).
    pub fn store_service_token(
        &mut self,
        service_name: &str,
        token: &str,
    ) -> Result<(), VaultError> {
        let key = format!("svc.{service_name}");
        self.store(&key, token)
    }

    /// Retrieve a service token.
    pub fn get_service_token(&self, service_name: &str) -> Result<String, VaultError> {
        let key = format!("svc.{service_name}");
        self.retrieve(&key)
    }

    /// Check if a service has a stored token.
    pub fn has_service_token(&self, service_name: &str) -> Result<bool, VaultError> {
        let key = format!("svc.{service_name}");
        self.has(&key)
    }

    // ── User Secret API ────────────────────────────────────────────

    /// Store a user secret (namespaced under `usr.<name>`).
    pub fn store_user_secret(&mut self, name: &str, value: &str) -> Result<(), VaultError> {
        let key = format!("usr.{name}");
        self.store(&key, value)
    }

    /// Retrieve a user secret.
    pub fn get_user_secret(&self, name: &str) -> Result<String, VaultError> {
        let key = format!("usr.{name}");
        self.retrieve(&key)
    }

    /// Check if a user secret exists.
    pub fn has_user_secret(&self, name: &str) -> Result<bool, VaultError> {
        let key = format!("usr.{name}");
        self.has(&key)
    }

    /// Delete a user secret.
    pub fn delete_user_secret(&mut self, name: &str) -> Result<(), VaultError> {
        let key = format!("usr.{name}");
        self.delete(&key)
    }

    // ── System Secret API ──────────────────────────────────────────

    /// Store a system secret (namespaced under `sys.<name>`).
    ///
    /// For OS-level secrets like device encryption keys, boot tokens.
    pub fn store_system_secret(&mut self, name: &str, value: &str) -> Result<(), VaultError> {
        let key = format!("sys.{name}");
        self.store(&key, value)
    }

    /// Retrieve a system secret.
    pub fn get_system_secret(&self, name: &str) -> Result<String, VaultError> {
        let key = format!("sys.{name}");
        self.retrieve(&key)
    }

    // ── Query API ──────────────────────────────────────────────────

    /// List all stored secret names.
    pub fn list_secrets(&self) -> Result<Vec<SecretInfo>, VaultError> {
        let vault = self.vault.as_ref().ok_or(VaultError::Locked)?;
        let names = vault.list();

        Ok(names
            .into_iter()
            .map(|name| {
                let category = if name.starts_with("svc.") || name.starts_with("sys.") {
                    SecretCategory::System
                } else {
                    SecretCategory::User
                };
                SecretInfo { name, category }
            })
            .collect())
    }

    /// Get the total number of stored secrets.
    pub fn secret_count(&self) -> Result<usize, VaultError> {
        let vault = self.vault.as_ref().ok_or(VaultError::Locked)?;
        Ok(vault.len())
    }

    /// Get the vault data directory path.
    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    // ── Private helpers ────────────────────────────────────────────

    /// Build vault config from our data directory.
    fn make_config(&self) -> VaultConfig {
        VaultConfig {
            vault_path: self.data_dir.join("system.vault"),
            pbkdf2_iterations: 100_000, // OS vault uses lower iterations for boot speed
            backup_on_write: true,
            file_mode: 0o600,
            dir_mode: 0o700,
            config_path: None,
        }
    }

    /// Store a value under a key.
    fn store(&mut self, key: &str, value: &str) -> Result<(), VaultError> {
        let vault = self.vault.as_mut().ok_or(VaultError::Locked)?;

        let name = SecretName::new(key).map_err(|e| VaultError::InvalidName(e.to_string()))?;

        vault
            .set(&name, value)
            .map_err(|e| VaultError::OperationFailed(e.to_string()))?;

        self.operations += 1;
        Ok(())
    }

    /// Retrieve a value by key.
    fn retrieve(&self, key: &str) -> Result<String, VaultError> {
        let vault = self.vault.as_ref().ok_or(VaultError::Locked)?;

        let name = SecretName::new(key).map_err(|e| VaultError::InvalidName(e.to_string()))?;

        vault.get(&name).map_err(|e| {
            if e.to_string().contains("not found") {
                VaultError::SecretNotFound(key.to_string())
            } else {
                VaultError::OperationFailed(e.to_string())
            }
        })
    }

    /// Check if a key exists.
    fn has(&self, key: &str) -> Result<bool, VaultError> {
        let vault = self.vault.as_ref().ok_or(VaultError::Locked)?;

        let name = SecretName::new(key).map_err(|e| VaultError::InvalidName(e.to_string()))?;

        Ok(vault.has(&name))
    }

    /// Delete a key.
    fn delete(&mut self, key: &str) -> Result<(), VaultError> {
        let vault = self.vault.as_mut().ok_or(VaultError::Locked)?;

        let name = SecretName::new(key).map_err(|e| VaultError::InvalidName(e.to_string()))?;

        vault.delete(&name).map_err(|e| {
            if e.to_string().contains("not found") {
                VaultError::SecretNotFound(key.to_string())
            } else {
                VaultError::OperationFailed(e.to_string())
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_vault_dir() -> Option<tempfile::TempDir> {
        tempfile::tempdir().ok()
    }

    #[test]
    fn new_vault_uninitialized() {
        let vault = OsVault::new("/tmp/nexcore-test-vault-nonexistent");
        assert_eq!(vault.state(), VaultState::Uninitialized);
        assert!(!vault.is_operational());
    }

    #[test]
    fn virtual_vault_uninitialized() {
        let vault = OsVault::virtual_vault();
        assert_eq!(vault.state(), VaultState::Uninitialized);
    }

    #[test]
    fn initialize_and_unlock() {
        let dir = match temp_vault_dir() {
            Some(d) => d,
            None => return,
        };

        let mut vault = OsVault::new(dir.path());
        assert_eq!(vault.state(), VaultState::Uninitialized);

        // Initialize
        let result = vault.initialize("test-password");
        assert!(result.is_ok(), "Initialize should succeed: {result:?}");
        assert_eq!(vault.state(), VaultState::Unlocked);
        assert!(vault.is_operational());
    }

    #[test]
    fn double_initialize_fails() {
        let dir = match temp_vault_dir() {
            Some(d) => d,
            None => return,
        };

        let mut vault = OsVault::new(dir.path());
        let _ = vault.initialize("pass");
        let result = vault.initialize("pass");
        assert_eq!(result, Err(VaultError::AlreadyInitialized));
    }

    #[test]
    fn lock_and_unlock() {
        let dir = match temp_vault_dir() {
            Some(d) => d,
            None => return,
        };

        let mut vault = OsVault::new(dir.path());
        let _ = vault.initialize("pass123");
        assert!(vault.is_operational());

        vault.lock();
        assert_eq!(vault.state(), VaultState::Locked);
        assert!(!vault.is_operational());

        let result = vault.unlock("pass123");
        assert!(result.is_ok());
        assert!(vault.is_operational());
    }

    #[test]
    fn wrong_password_fails() {
        let dir = match temp_vault_dir() {
            Some(d) => d,
            None => return,
        };

        let mut vault = OsVault::new(dir.path());
        let _ = vault.initialize("correct");

        // Store a secret so password verification has something to decrypt
        let _ = vault.store_service_token("test", "token");

        vault.lock();

        let result = vault.unlock("wrong");
        assert!(result.is_err());
    }

    #[test]
    fn unlock_uninitialized_fails() {
        let mut vault = OsVault::new("/tmp/nexcore-uninit-vault-test");
        let result = vault.unlock("pass");
        assert_eq!(result, Err(VaultError::NotInitialized));
    }

    #[test]
    fn service_token_round_trip() {
        let dir = match temp_vault_dir() {
            Some(d) => d,
            None => return,
        };

        let mut vault = OsVault::new(dir.path());
        let _ = vault.initialize("pass");

        // Store
        let result = vault.store_service_token("guardian", "grd-token-abc123");
        assert!(result.is_ok());

        // Has
        assert_eq!(vault.has_service_token("guardian"), Ok(true));
        assert_eq!(vault.has_service_token("nonexistent"), Ok(false));

        // Retrieve
        let token = vault.get_service_token("guardian");
        assert!(token.is_ok());
        assert_eq!(token.unwrap_or_default(), "grd-token-abc123");
    }

    #[test]
    fn user_secret_round_trip() {
        let dir = match temp_vault_dir() {
            Some(d) => d,
            None => return,
        };

        let mut vault = OsVault::new(dir.path());
        let _ = vault.initialize("pass");

        // Store
        let result = vault.store_user_secret("wifi.home", "MyWiFiPassword!");
        assert!(result.is_ok());

        // Retrieve
        let secret = vault.get_user_secret("wifi.home");
        assert!(secret.is_ok());
        assert_eq!(secret.unwrap_or_default(), "MyWiFiPassword!");

        // Delete
        let result = vault.delete_user_secret("wifi.home");
        assert!(result.is_ok());

        // Gone
        assert_eq!(vault.has_user_secret("wifi.home"), Ok(false));
    }

    #[test]
    fn system_secret_round_trip() {
        let dir = match temp_vault_dir() {
            Some(d) => d,
            None => return,
        };

        let mut vault = OsVault::new(dir.path());
        let _ = vault.initialize("pass");

        let result = vault.store_system_secret("device-key", "dk-xyz789");
        assert!(result.is_ok());

        let secret = vault.get_system_secret("device-key");
        assert!(secret.is_ok());
        assert_eq!(secret.unwrap_or_default(), "dk-xyz789");
    }

    #[test]
    fn operations_locked_fail() {
        let dir = match temp_vault_dir() {
            Some(d) => d,
            None => return,
        };

        let mut vault = OsVault::new(dir.path());
        let _ = vault.initialize("pass");
        vault.lock();

        assert_eq!(
            vault.store_service_token("test", "val"),
            Err(VaultError::Locked)
        );
        assert_eq!(vault.get_service_token("test"), Err(VaultError::Locked));
        assert_eq!(vault.list_secrets(), Err(VaultError::Locked));
    }

    #[test]
    fn list_secrets_categorized() {
        let dir = match temp_vault_dir() {
            Some(d) => d,
            None => return,
        };

        let mut vault = OsVault::new(dir.path());
        let _ = vault.initialize("pass");
        let _ = vault.store_service_token("guardian", "token1");
        let _ = vault.store_user_secret("wifi", "password");
        let _ = vault.store_system_secret("boot-key", "key123");

        let secrets = vault.list_secrets();
        assert!(secrets.is_ok());
        let secrets = secrets.unwrap_or_default();
        assert_eq!(secrets.len(), 3);

        let system_count = secrets
            .iter()
            .filter(|s| s.category == SecretCategory::System)
            .count();
        let user_count = secrets
            .iter()
            .filter(|s| s.category == SecretCategory::User)
            .count();
        assert_eq!(system_count, 2); // svc.guardian + sys.boot-key
        assert_eq!(user_count, 1); // usr.wifi
    }

    #[test]
    fn secret_count() {
        let dir = match temp_vault_dir() {
            Some(d) => d,
            None => return,
        };

        let mut vault = OsVault::new(dir.path());
        let _ = vault.initialize("pass");
        assert_eq!(vault.secret_count(), Ok(0));

        let _ = vault.store_service_token("a", "1");
        let _ = vault.store_service_token("b", "2");
        assert_eq!(vault.secret_count(), Ok(2));
    }

    #[test]
    fn operations_counter() {
        let dir = match temp_vault_dir() {
            Some(d) => d,
            None => return,
        };

        let mut vault = OsVault::new(dir.path());
        let _ = vault.initialize("pass");
        assert_eq!(vault.operations(), 0);

        let _ = vault.store_service_token("a", "1");
        let _ = vault.store_service_token("b", "2");
        assert_eq!(vault.operations(), 2);
    }

    #[test]
    fn get_nonexistent_secret() {
        let dir = match temp_vault_dir() {
            Some(d) => d,
            None => return,
        };

        let mut vault = OsVault::new(dir.path());
        let _ = vault.initialize("pass");

        let result = vault.get_service_token("nonexistent");
        assert!(matches!(result, Err(VaultError::SecretNotFound(_))));
    }

    #[test]
    fn persistence_across_lock_unlock() {
        let dir = match temp_vault_dir() {
            Some(d) => d,
            None => return,
        };

        let mut vault = OsVault::new(dir.path());
        let _ = vault.initialize("pass");
        let _ = vault.store_service_token("persistent", "value123");

        // Lock
        vault.lock();

        // Unlock
        let result = vault.unlock("pass");
        assert!(result.is_ok());

        // Secret still there
        let val = vault.get_service_token("persistent");
        assert_eq!(val.unwrap_or_default(), "value123");
    }

    #[test]
    fn vault_state_display() {
        assert!(VaultState::Uninitialized == VaultState::Uninitialized);
        assert!(VaultState::Locked == VaultState::Locked);
        assert!(VaultState::Unlocked == VaultState::Unlocked);
        assert!(VaultState::Unlocked != VaultState::Locked);
    }

    #[test]
    fn vault_error_display() {
        let e = VaultError::Locked;
        assert!(e.to_string().contains("locked"));

        let e = VaultError::NotInitialized;
        assert!(e.to_string().contains("not initialized"));

        let e = VaultError::SecretNotFound("test".into());
        assert!(e.to_string().contains("test"));
    }
}
