//! SQLite-backed subscription and API key store for Guardian MVP
//!
//! Manages:
//! - Subscriptions (Stripe-linked, with plan tiers and query limits)
//! - API keys (hashed, per-user, with usage tracking)
//!
//! Tier: T2-S (Service boundary — λ Location + π Persistence + ∂ Boundary)

use hmac::{Hmac, Mac};
use nexcore_codec::hex;
use nexcore_fs::dirs;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use tokio::sync::Mutex;
use utoipa::ToSchema;

/// Global store instance
static STORE: OnceLock<Arc<Mutex<GuardianStore>>> = OnceLock::new();

/// Initialize the global store. Must be called once at startup.
/// Returns an error if neither file-based nor in-memory SQLite can open.
pub fn init_store() -> nexcore_error::Result<()> {
    let db_path = std::env::var("GUARDIAN_DB_PATH").unwrap_or_else(|_| {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".claude/data/guardian.db")
            .to_string_lossy()
            .to_string()
    });

    let store = match GuardianStore::open(&db_path) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!(
                "Failed to open guardian DB at {db_path}: {e}, falling back to in-memory"
            );
            GuardianStore::open(":memory:")?
        }
    };

    // OnceLock::set returns Err if already initialized — that's fine, first writer wins
    #[allow(unused_results)]
    {
        STORE.set(Arc::new(Mutex::new(store)));
    }
    Ok(())
}

/// Get the global store, initializing in-memory if init_store() was never called.
pub fn get_store() -> Arc<Mutex<GuardianStore>> {
    match STORE.get() {
        Some(store) => store.clone(),
        None => {
            // Lazy init fallback (e.g. in tests or if init_store wasn't called)
            match GuardianStore::open(":memory:") {
                Ok(store) => {
                    let arc = Arc::new(Mutex::new(store));
                    // OnceLock::set returns Err if already set — benign race, first writer wins
                    #[allow(unused_results)]
                    {
                        STORE.set(arc.clone());
                    }
                    arc
                }
                Err(e) => {
                    tracing::error!("Cannot open in-memory SQLite: {e}");
                    // Return a disconnected store — all operations will fail gracefully via Result
                    let conn =
                        Connection::open_in_memory().unwrap_or_else(|_| std::process::abort());
                    Arc::new(Mutex::new(GuardianStore { conn }))
                }
            }
        }
    }
}

/// Guardian subscription + API key store
pub struct GuardianStore {
    conn: Connection,
}

/// Subscription record
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Subscription {
    pub user_id: String,
    pub stripe_customer_id: Option<String>,
    pub stripe_subscription_id: Option<String>,
    pub plan: String,
    pub status: String,
    pub query_limit: i64,
    pub queries_used: i64,
    pub period_start: Option<String>,
    pub period_end: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl Default for Subscription {
    fn default() -> Self {
        Self {
            user_id: String::new(),
            stripe_customer_id: None,
            stripe_subscription_id: None,
            plan: "researcher".to_string(),
            status: "inactive".to_string(),
            query_limit: 100,
            queries_used: 0,
            period_start: None,
            period_end: None,
            created_at: String::new(),
            updated_at: String::new(),
        }
    }
}

/// API key record (key itself is never stored — only hash)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ApiKeyRecord {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub key_prefix: String,
    pub created_at: String,
    pub last_used: Option<String>,
    pub revoked: bool,
}

/// Plan tier limits
fn plan_query_limit(plan: &str) -> i64 {
    match plan {
        "researcher" => 100,
        "professional" => 1000,
        "enterprise" => 10000,
        _ => 100,
    }
}

impl GuardianStore {
    /// Open (or create) the store at the given path
    pub fn open(db_path: &str) -> nexcore_error::Result<Self> {
        let conn = Connection::open(db_path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS subscriptions (
                user_id TEXT PRIMARY KEY,
                stripe_customer_id TEXT,
                stripe_subscription_id TEXT,
                plan TEXT NOT NULL DEFAULT 'researcher',
                status TEXT NOT NULL DEFAULT 'active',
                query_limit INTEGER NOT NULL DEFAULT 100,
                queries_used INTEGER NOT NULL DEFAULT 0,
                period_start TEXT,
                period_end TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS api_keys (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                name TEXT NOT NULL DEFAULT 'default',
                key_hash TEXT NOT NULL,
                key_prefix TEXT NOT NULL,
                created_at TEXT NOT NULL,
                last_used TEXT,
                revoked INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY (user_id) REFERENCES subscriptions(user_id)
            );

            CREATE INDEX IF NOT EXISTS idx_api_keys_hash ON api_keys(key_hash);
            CREATE INDEX IF NOT EXISTS idx_api_keys_user ON api_keys(user_id);

            CREATE TABLE IF NOT EXISTS query_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id TEXT NOT NULL,
                endpoint TEXT NOT NULL,
                request_body TEXT,
                response_body TEXT,
                created_at TEXT NOT NULL,
                FOREIGN KEY (user_id) REFERENCES subscriptions(user_id)
            );

            CREATE INDEX IF NOT EXISTS idx_query_history_user ON query_history(user_id);
            CREATE INDEX IF NOT EXISTS idx_query_history_time ON query_history(created_at);",
        )?;

        tracing::info!("Guardian store initialized at {db_path}");
        Ok(Self { conn })
    }

    // ── Subscriptions ──────────────────────────

    /// Create or update a subscription (upsert)
    pub fn upsert_subscription(
        &self,
        user_id: &str,
        stripe_customer_id: Option<&str>,
        stripe_subscription_id: Option<&str>,
        plan: &str,
        status: &str,
    ) -> nexcore_error::Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        let limit = plan_query_limit(plan);

        self.conn.execute(
            "INSERT INTO subscriptions (user_id, stripe_customer_id, stripe_subscription_id, plan, status, query_limit, queries_used, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, ?7, ?7)
             ON CONFLICT(user_id) DO UPDATE SET
                stripe_customer_id = COALESCE(?2, stripe_customer_id),
                stripe_subscription_id = COALESCE(?3, stripe_subscription_id),
                plan = ?4,
                status = ?5,
                query_limit = ?6,
                updated_at = ?7",
            params![user_id, stripe_customer_id, stripe_subscription_id, plan, status, limit, now],
        )?;
        Ok(())
    }

    /// Get subscription by user_id
    pub fn get_subscription(&self, user_id: &str) -> nexcore_error::Result<Option<Subscription>> {
        let mut stmt = self.conn.prepare(
            "SELECT user_id, stripe_customer_id, stripe_subscription_id, plan, status,
                    query_limit, queries_used, period_start, period_end, created_at, updated_at
             FROM subscriptions WHERE user_id = ?1",
        )?;

        let mut rows = stmt.query_map(params![user_id], |row| {
            Ok(Subscription {
                user_id: row.get(0)?,
                stripe_customer_id: row.get(1)?,
                stripe_subscription_id: row.get(2)?,
                plan: row.get(3)?,
                status: row.get(4)?,
                query_limit: row.get(5)?,
                queries_used: row.get(6)?,
                period_start: row.get(7)?,
                period_end: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })?;

        match rows.next() {
            Some(Ok(sub)) => Ok(Some(sub)),
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }

    /// Get subscription by Stripe customer ID
    pub fn get_subscription_by_stripe_customer(
        &self,
        stripe_customer_id: &str,
    ) -> nexcore_error::Result<Option<Subscription>> {
        let mut stmt = self.conn.prepare(
            "SELECT user_id, stripe_customer_id, stripe_subscription_id, plan, status,
                    query_limit, queries_used, period_start, period_end, created_at, updated_at
             FROM subscriptions WHERE stripe_customer_id = ?1",
        )?;

        let mut rows = stmt.query_map(params![stripe_customer_id], |row| {
            Ok(Subscription {
                user_id: row.get(0)?,
                stripe_customer_id: row.get(1)?,
                stripe_subscription_id: row.get(2)?,
                plan: row.get(3)?,
                status: row.get(4)?,
                query_limit: row.get(5)?,
                queries_used: row.get(6)?,
                period_start: row.get(7)?,
                period_end: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })?;

        match rows.next() {
            Some(Ok(sub)) => Ok(Some(sub)),
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }

    /// Update subscription status
    pub fn update_subscription_status(
        &self,
        stripe_customer_id: &str,
        status: &str,
    ) -> nexcore_error::Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE subscriptions SET status = ?1, updated_at = ?2 WHERE stripe_customer_id = ?3",
            params![status, now, stripe_customer_id],
        )?;
        Ok(())
    }

    /// Update subscription plan tier
    pub fn update_subscription_plan(
        &self,
        stripe_customer_id: &str,
        plan: &str,
    ) -> nexcore_error::Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        let limit = plan_query_limit(plan);
        self.conn.execute(
            "UPDATE subscriptions SET plan = ?1, query_limit = ?2, updated_at = ?3 WHERE stripe_customer_id = ?4",
            params![plan, limit, now, stripe_customer_id],
        )?;
        Ok(())
    }

    /// Increment usage counter. Returns (queries_used, query_limit) after increment.
    pub fn increment_usage(&self, user_id: &str) -> nexcore_error::Result<(i64, i64)> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE subscriptions SET queries_used = queries_used + 1, updated_at = ?1 WHERE user_id = ?2",
            params![now, user_id],
        )?;

        let mut stmt = self
            .conn
            .prepare("SELECT queries_used, query_limit FROM subscriptions WHERE user_id = ?1")?;
        let result = stmt.query_row(params![user_id], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
        })?;
        Ok(result)
    }

    /// Check if user has remaining queries
    pub fn check_rate_limit(&self, user_id: &str) -> nexcore_error::Result<bool> {
        let mut stmt = self.conn.prepare(
            "SELECT queries_used < query_limit FROM subscriptions WHERE user_id = ?1 AND status = 'active'",
        )?;
        let allowed = stmt
            .query_row(params![user_id], |row| row.get::<_, bool>(0))
            .unwrap_or(false);
        Ok(allowed)
    }

    /// Reset usage counters (call at period start)
    pub fn reset_usage(&self, user_id: &str) -> nexcore_error::Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE subscriptions SET queries_used = 0, period_start = ?1, updated_at = ?1 WHERE user_id = ?2",
            params![now, user_id],
        )?;
        Ok(())
    }

    // ── API Keys ──────────────────────────

    /// Create an API key. Returns (key_id, raw_key) — raw_key is shown once, then only hash stored.
    pub fn create_api_key(
        &self,
        user_id: &str,
        name: &str,
    ) -> nexcore_error::Result<(String, String)> {
        let key_id = nexcore_id::NexId::v4().to_string();
        let raw_key = format!("grd_{}", generate_random_key());
        let key_hash = hash_api_key(&raw_key);
        let key_prefix = raw_key.chars().take(12).collect::<String>();
        let now = chrono::Utc::now().to_rfc3339();

        self.conn.execute(
            "INSERT INTO api_keys (id, user_id, name, key_hash, key_prefix, created_at, revoked)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0)",
            params![key_id, user_id, name, key_hash, key_prefix, now],
        )?;

        Ok((key_id, raw_key))
    }

    /// Validate an API key. Returns user_id if valid and not revoked.
    pub fn validate_api_key(&self, raw_key: &str) -> nexcore_error::Result<Option<String>> {
        let key_hash = hash_api_key(raw_key);
        let now = chrono::Utc::now().to_rfc3339();

        let mut stmt = self.conn.prepare(
            "SELECT ak.user_id FROM api_keys ak
             JOIN subscriptions s ON ak.user_id = s.user_id
             WHERE ak.key_hash = ?1 AND ak.revoked = 0 AND s.status = 'active'",
        )?;

        let result = stmt.query_row(params![key_hash], |row| row.get::<_, String>(0));

        match result {
            Ok(user_id) => {
                // Best-effort update of last_used timestamp
                if let Err(e) = self.conn.execute(
                    "UPDATE api_keys SET last_used = ?1 WHERE key_hash = ?2",
                    params![now, key_hash],
                ) {
                    tracing::warn!("Failed to update api_key last_used: {e}");
                }
                Ok(Some(user_id))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// List API keys for a user (no raw keys — only metadata)
    pub fn list_api_keys(&self, user_id: &str) -> nexcore_error::Result<Vec<ApiKeyRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, user_id, name, key_prefix, created_at, last_used, revoked
             FROM api_keys WHERE user_id = ?1 ORDER BY created_at DESC",
        )?;

        let keys = stmt
            .query_map(params![user_id], |row| {
                Ok(ApiKeyRecord {
                    id: row.get(0)?,
                    user_id: row.get(1)?,
                    name: row.get(2)?,
                    key_prefix: row.get(3)?,
                    created_at: row.get(4)?,
                    last_used: row.get(5)?,
                    revoked: row.get::<_, i32>(6)? != 0,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(keys)
    }

    /// Revoke an API key
    pub fn revoke_api_key(&self, key_id: &str, user_id: &str) -> nexcore_error::Result<bool> {
        let rows = self.conn.execute(
            "UPDATE api_keys SET revoked = 1 WHERE id = ?1 AND user_id = ?2",
            params![key_id, user_id],
        )?;
        Ok(rows > 0)
    }

    // ── Query History ──────────────────────────

    /// Record a query in history
    pub fn record_query(
        &self,
        user_id: &str,
        endpoint: &str,
        request_body: Option<&str>,
        response_body: Option<&str>,
    ) -> nexcore_error::Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO query_history (user_id, endpoint, request_body, response_body, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![user_id, endpoint, request_body, response_body, now],
        )?;
        Ok(())
    }

    /// Get query history (paginated)
    pub fn get_history(
        &self,
        user_id: &str,
        limit: i64,
        offset: i64,
    ) -> nexcore_error::Result<Vec<serde_json::Value>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, endpoint, request_body, response_body, created_at
             FROM query_history WHERE user_id = ?1
             ORDER BY created_at DESC LIMIT ?2 OFFSET ?3",
        )?;

        let rows = stmt
            .query_map(params![user_id, limit, offset], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, i64>(0)?,
                    "endpoint": row.get::<_, String>(1)?,
                    "request": row.get::<_, Option<String>>(2)?,
                    "response": row.get::<_, Option<String>>(3)?,
                    "created_at": row.get::<_, String>(4)?,
                }))
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(rows)
    }
}

/// Hash an API key using HMAC-SHA256 with a server secret
fn hash_api_key(raw_key: &str) -> String {
    use sha2::Digest;
    let secret = std::env::var("API_KEY_SECRET")
        .unwrap_or_else(|_| "guardian-dev-secret-change-in-prod".to_string());
    match Hmac::<Sha256>::new_from_slice(secret.as_bytes()) {
        Ok(mut mac) => {
            mac.update(raw_key.as_bytes());
            hex::encode(mac.finalize().into_bytes())
        }
        Err(_) => {
            // HMAC accepts any key size — unreachable in practice.
            // Deterministic fallback using plain SHA-256.
            let hash = Sha256::digest(raw_key.as_bytes());
            hex::encode(hash)
        }
    }
}

/// Generate a random API key (32 hex chars)
fn generate_random_key() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);

    // Simple LCG PRNG — sufficient for API key generation in MVP
    // Production should use ring or rand crate
    let mut state = seed;
    let mut key = String::with_capacity(32);
    for _ in 0..32 {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let nibble = ((state >> 33) & 0xF) as u8;
        key.push(char::from(if nibble < 10 {
            b'0' + nibble
        } else {
            b'a' + nibble - 10
        }));
    }
    key
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_store() -> GuardianStore {
        GuardianStore::open(":memory:").unwrap_or_else(|_| std::process::abort())
    }

    #[test]
    fn test_subscription_lifecycle() {
        let store = test_store();

        store
            .upsert_subscription(
                "user1",
                Some("cus_123"),
                Some("sub_456"),
                "researcher",
                "active",
            )
            .unwrap_or_else(|_| std::process::abort());

        let sub = store
            .get_subscription("user1")
            .unwrap_or_else(|_| std::process::abort());
        assert!(sub.is_some());
        let sub = sub.unwrap_or_default();
        assert_eq!(sub.plan, "researcher");
        assert_eq!(sub.query_limit, 100);
        assert_eq!(sub.queries_used, 0);

        let (used, limit) = store
            .increment_usage("user1")
            .unwrap_or_else(|_| std::process::abort());
        assert_eq!(used, 1);
        assert_eq!(limit, 100);

        assert!(store.check_rate_limit("user1").unwrap_or(false));

        store
            .update_subscription_plan("cus_123", "professional")
            .unwrap_or_else(|_| std::process::abort());
        let sub = store
            .get_subscription("user1")
            .unwrap_or_else(|_| std::process::abort())
            .unwrap_or_default();
        assert_eq!(sub.plan, "professional");
        assert_eq!(sub.query_limit, 1000);
    }

    #[test]
    fn test_api_key_lifecycle() {
        let store = test_store();

        store
            .upsert_subscription("user1", Some("cus_123"), None, "researcher", "active")
            .unwrap_or_else(|_| std::process::abort());

        let (key_id, raw_key) = store
            .create_api_key("user1", "test-key")
            .unwrap_or_else(|_| std::process::abort());
        assert!(raw_key.starts_with("grd_"));

        let user = store
            .validate_api_key(&raw_key)
            .unwrap_or_else(|_| std::process::abort());
        assert_eq!(user, Some("user1".to_string()));

        let user = store
            .validate_api_key("grd_invalid")
            .unwrap_or_else(|_| std::process::abort());
        assert_eq!(user, None);

        let keys = store
            .list_api_keys("user1")
            .unwrap_or_else(|_| std::process::abort());
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].id, key_id);
        assert!(!keys[0].revoked);

        assert!(store.revoke_api_key(&key_id, "user1").unwrap_or(false));

        let user = store
            .validate_api_key(&raw_key)
            .unwrap_or_else(|_| std::process::abort());
        assert_eq!(user, None);
    }

    #[test]
    fn test_query_history() {
        let store = test_store();
        store
            .upsert_subscription("user1", None, None, "researcher", "active")
            .unwrap_or_else(|_| std::process::abort());

        store
            .record_query(
                "user1",
                "/analyze",
                Some(r#"{"a":15}"#),
                Some(r#"{"prr":2.5}"#),
            )
            .unwrap_or_else(|_| std::process::abort());

        let history = store
            .get_history("user1", 10, 0)
            .unwrap_or_else(|_| std::process::abort());
        assert_eq!(history.len(), 1);
    }

    #[test]
    fn test_hash_deterministic() {
        let h1 = hash_api_key("grd_test123");
        let h2 = hash_api_key("grd_test123");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_plan_limits() {
        assert_eq!(plan_query_limit("researcher"), 100);
        assert_eq!(plan_query_limit("professional"), 1000);
        assert_eq!(plan_query_limit("enterprise"), 10000);
        assert_eq!(plan_query_limit("unknown"), 100);
    }
}
