//! Connection management for the registry SQLite database.
//!
//! Provides a lightweight connection wrapper that handles initialization,
//! WAL mode, and provides the default database path.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use rusqlite::Connection;

use crate::error::Result;
use crate::schema;

/// Default database filename.
const DB_FILENAME: &str = "skills.db";

/// Default data directory relative to home.
const DATA_DIR: &str = ".claude/data";

/// Resolve the default database path: `~/.claude/data/skills.db`
#[must_use]
pub fn default_db_path() -> PathBuf {
    let home = home_dir();
    home.join(DATA_DIR).join(DB_FILENAME)
}

/// Get the home directory path.
fn home_dir() -> PathBuf {
    std::env::var("HOME").map_or_else(|_| PathBuf::from("/tmp"), PathBuf::from)
}

/// A managed database connection with automatic schema initialization.
///
/// Thread-safe via internal `Mutex<Connection>`. For the registry use case
/// (single CLI process, sequential MCP calls), this is sufficient.
#[derive(Debug, Clone)]
pub struct RegistryPool {
    inner: Arc<Mutex<Connection>>,
    path: PathBuf,
}

impl RegistryPool {
    /// Open (or create) a database at the given path and initialize schema.
    ///
    /// # Errors
    ///
    /// Returns an error if the database cannot be opened or schema fails.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    crate::error::RegistryError::Migration(format!("Cannot create dir: {e}"))
                })?;
            }
        }

        let conn = Connection::open(&path)?;
        schema::initialize(&conn)?;

        tracing::info!(path = %path.display(), "Registry database opened");

        Ok(Self {
            inner: Arc::new(Mutex::new(conn)),
            path,
        })
    }

    /// Open an in-memory database (useful for testing).
    ///
    /// # Errors
    ///
    /// Returns an error if the schema cannot be initialized.
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        schema::initialize(&conn)?;

        Ok(Self {
            inner: Arc::new(Mutex::new(conn)),
            path: PathBuf::from(":memory:"),
        })
    }

    /// Open the default database at `~/.claude/data/skills.db`.
    ///
    /// # Errors
    ///
    /// Returns an error if the database cannot be opened.
    pub fn open_default() -> Result<Self> {
        Self::open(default_db_path())
    }

    /// Execute a closure with exclusive access to the connection.
    ///
    /// # Errors
    ///
    /// Returns an error if the lock is poisoned or the closure fails.
    pub fn with_conn<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&Connection) -> Result<T>,
    {
        let conn = self
            .inner
            .lock()
            .map_err(|e| crate::error::RegistryError::Migration(format!("Lock poisoned: {e}")))?;
        f(&conn)
    }

    /// Get the database file path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_in_memory() {
        let pool = RegistryPool::open_in_memory();
        assert!(pool.is_ok());
    }

    #[test]
    fn test_with_conn() {
        let pool = RegistryPool::open_in_memory().map_err(|e| format!("{e}"));
        let pool = pool.ok();
        assert!(pool.is_some());
        let pool = pool.unwrap_or_else(|| unreachable!());
        let result = pool.with_conn(|conn| {
            let count: i64 =
                conn.query_row("SELECT COUNT(*) FROM active_skills", [], |row| row.get(0))?;
            Ok(count)
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_default_db_path() {
        let path = default_db_path();
        assert!(path.to_string_lossy().contains("skills.db"));
        assert!(path.to_string_lossy().contains(".claude/data"));
    }
}
