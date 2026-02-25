//! # Anonymization Lifecycle — Typestate Pattern
//!
//! ## Primitive Foundation
//!
//! | Primitive | Manifestation |
//! |-----------|---------------|
//! | ς State | Four states: Raw → Pseudonymized → Anonymized → Aggregated |
//! | σ Sequence | Ordered transitions (no skipping) |
//! | ∝ Irreversibility | Anonymized state has no reverse path |
//! | ∂ Boundary | State transitions are type-enforced boundaries |
//!
//! ## Tier: T2-C (ς + σ + ∝ + ∂)
//!
//! The compiler enforces that:
//! - `Raw` can only become `Pseudonymized`
//! - `Pseudonymized` can become `Anonymized` (or be reversed to `Raw` with auth)
//! - `Anonymized` can only become `Aggregated` (NO reverse path — ∝)
//! - `Aggregated` is terminal

use serde::{Deserialize, Serialize};
use std::fmt;
use std::marker::PhantomData;

use crate::audit::{RedactionAudit, RedactionEntry};
use crate::error::{GhostError, Result};

// ── State Marker Types ─────────────────────────────────────────────

/// Marker: data is in raw (unprocessed) form.
#[derive(Debug, Clone, Copy)]
pub struct Raw;

/// Marker: data has been pseudonymized (reversible with key).
#[derive(Debug, Clone, Copy)]
pub struct Pseudonymized;

/// Marker: data has been anonymized (irreversible).
#[derive(Debug, Clone, Copy)]
pub struct Anonymized;

/// Marker: data has been aggregated (terminal state).
#[derive(Debug, Clone, Copy)]
pub struct Aggregated;

// ── Authorization ──────────────────────────────────────────────────

/// Authorization token for reversal operations.
///
/// In Standard mode, a single authorization suffices.
/// In Strict mode, two distinct authorizations are required.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReversalAuthorization {
    /// Who authorized the reversal.
    pub authorizer: String,
    /// Reason for reversal (e.g., "ICH E2B submission").
    pub reason: String,
    /// ISO 8601 timestamp.
    pub timestamp: String,
}

impl ReversalAuthorization {
    /// Create a new authorization with current timestamp.
    #[must_use]
    pub fn new(authorizer: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            authorizer: authorizer.into(),
            reason: reason.into(),
            timestamp: nexcore_chrono::DateTime::now().to_rfc3339(),
        }
    }
}

// ── Lifecycle Container ────────────────────────────────────────────

/// Type-state container tracking data through the anonymization lifecycle.
///
/// `S` is the state marker (Raw, Pseudonymized, Anonymized, Aggregated).
/// `T` is the data payload type.
///
/// ## Tier: T2-C (ς + σ + ∝ + ∂)
pub struct AnonymizationLifecycle<S, T> {
    data: T,
    audit: RedactionAudit,
    _state: PhantomData<S>,
}

// Manual Debug to avoid S: Debug bound
impl<S, T: fmt::Debug> fmt::Debug for AnonymizationLifecycle<S, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AnonymizationLifecycle")
            .field("data", &self.data)
            .field("audit_entries", &self.audit.len())
            .finish()
    }
}

// ── Raw state ──────────────────────────────────────────────────────

impl<T> AnonymizationLifecycle<Raw, T> {
    /// Create a new lifecycle container with raw data.
    #[must_use]
    pub fn new(data: T) -> Self {
        Self {
            data,
            audit: RedactionAudit::new(),
            _state: PhantomData,
        }
    }

    /// Access the raw data.
    #[must_use]
    pub fn data(&self) -> &T {
        &self.data
    }

    /// Transition to Pseudonymized state.
    ///
    /// The `transform` closure applies pseudonymization to the data.
    /// An audit entry is automatically recorded.
    pub fn pseudonymize<F>(self, transform: F) -> AnonymizationLifecycle<Pseudonymized, T>
    where
        F: FnOnce(T) -> T,
    {
        let data = transform(self.data);
        let mut audit = self.audit;
        audit.append(RedactionEntry::new(
            "*",
            "pseudonymize",
            "lifecycle: Raw → Pseudonymized",
            "all",
        ));
        AnonymizationLifecycle {
            data,
            audit,
            _state: PhantomData,
        }
    }
}

// ── Pseudonymized state ────────────────────────────────────────────

impl<T> AnonymizationLifecycle<Pseudonymized, T> {
    /// Access the pseudonymized data.
    #[must_use]
    pub fn data(&self) -> &T {
        &self.data
    }

    /// Transition to Anonymized state (irreversible).
    pub fn anonymize<F>(self, transform: F) -> AnonymizationLifecycle<Anonymized, T>
    where
        F: FnOnce(T) -> T,
    {
        let data = transform(self.data);
        let mut audit = self.audit;
        audit.append(RedactionEntry::new(
            "*",
            "anonymize",
            "lifecycle: Pseudonymized → Anonymized (IRREVERSIBLE)",
            "all",
        ));
        AnonymizationLifecycle {
            data,
            audit,
            _state: PhantomData,
        }
    }

    /// Reverse to Raw state (requires authorization).
    ///
    /// # Errors
    /// Returns `GhostError::ReversalDenied` if authorization is insufficient.
    pub fn reverse<F>(
        self,
        auth: ReversalAuthorization,
        transform: F,
    ) -> Result<AnonymizationLifecycle<Raw, T>>
    where
        F: FnOnce(T) -> T,
    {
        if auth.authorizer.is_empty() {
            return Err(GhostError::ReversalDenied(
                "authorizer cannot be empty".into(),
            ));
        }
        let data = transform(self.data);
        let mut audit = self.audit;
        audit.append(RedactionEntry::new(
            "*",
            "reverse",
            format!(
                "lifecycle: Pseudonymized → Raw (auth: {}, reason: {})",
                auth.authorizer, auth.reason
            ),
            "all",
        ));
        Ok(AnonymizationLifecycle {
            data,
            audit,
            _state: PhantomData,
        })
    }

    /// Reverse with dual authorization (required in Strict mode).
    ///
    /// # Errors
    /// Returns `GhostError::ReversalDenied` if authorizations are insufficient
    /// or if both authorizations are from the same person.
    pub fn reverse_dual_auth<F>(
        self,
        auth1: ReversalAuthorization,
        auth2: ReversalAuthorization,
        transform: F,
    ) -> Result<AnonymizationLifecycle<Raw, T>>
    where
        F: FnOnce(T) -> T,
    {
        if auth1.authorizer == auth2.authorizer {
            return Err(GhostError::ReversalDenied(
                "dual-auth requires two distinct authorizers".into(),
            ));
        }
        let data = transform(self.data);
        let mut audit = self.audit;
        audit.append(RedactionEntry::new(
            "*",
            "reverse_dual",
            format!(
                "lifecycle: Pseudonymized → Raw (dual-auth: {} + {}, reason: {})",
                auth1.authorizer, auth2.authorizer, auth1.reason
            ),
            "all",
        ));
        Ok(AnonymizationLifecycle {
            data,
            audit,
            _state: PhantomData,
        })
    }
}

// ── Anonymized state (NO reverse method — compiler enforces ∝) ────

impl<T> AnonymizationLifecycle<Anonymized, T> {
    /// Access the anonymized data.
    #[must_use]
    pub fn data(&self) -> &T {
        &self.data
    }

    /// Transition to Aggregated state (terminal).
    pub fn aggregate<F>(self, transform: F) -> AnonymizationLifecycle<Aggregated, T>
    where
        F: FnOnce(T) -> T,
    {
        let data = transform(self.data);
        let mut audit = self.audit;
        audit.append(RedactionEntry::new(
            "*",
            "aggregate",
            "lifecycle: Anonymized → Aggregated (TERMINAL)",
            "all",
        ));
        AnonymizationLifecycle {
            data,
            audit,
            _state: PhantomData,
        }
    }
}

// ── Aggregated state (terminal — no transitions) ───────────────────

impl<T> AnonymizationLifecycle<Aggregated, T> {
    /// Access the aggregated data.
    #[must_use]
    pub fn data(&self) -> &T {
        &self.data
    }
}

// ── Common methods (all states) ────────────────────────────────────

impl<S, T> AnonymizationLifecycle<S, T> {
    /// Access the audit trail.
    #[must_use]
    pub fn audit(&self) -> &RedactionAudit {
        &self.audit
    }
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_to_pseudonymized() {
        let lc = AnonymizationLifecycle::<Raw, String>::new("John Doe".to_string());
        let pseudo = lc.pseudonymize(|_| "PSEUDO_TOKEN".to_string());
        assert_eq!(pseudo.data(), "PSEUDO_TOKEN");
        assert_eq!(pseudo.audit().len(), 1);
    }

    #[test]
    fn pseudonymized_to_anonymized() {
        let lc = AnonymizationLifecycle::<Raw, String>::new("data".to_string());
        let pseudo = lc.pseudonymize(|s| format!("P:{s}"));
        let anon = pseudo.anonymize(|_| "ANON".to_string());
        assert_eq!(anon.data(), "ANON");
        assert_eq!(anon.audit().len(), 2);
    }

    #[test]
    fn anonymized_to_aggregated() {
        let lc = AnonymizationLifecycle::<Raw, i32>::new(42);
        let pseudo = lc.pseudonymize(|x| x);
        let anon = pseudo.anonymize(|x| x);
        let agg = anon.aggregate(|x| x * 10);
        assert_eq!(*agg.data(), 420);
        assert_eq!(agg.audit().len(), 3);
    }

    #[test]
    fn full_lifecycle_audit_chain() {
        let lc = AnonymizationLifecycle::<Raw, String>::new("raw".to_string());
        let pseudo = lc.pseudonymize(|_| "pseudo".to_string());
        let anon = pseudo.anonymize(|_| "anon".to_string());
        let agg = anon.aggregate(|_| "agg".to_string());
        let entries = agg.audit().entries();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].action, "pseudonymize");
        assert_eq!(entries[1].action, "anonymize");
        assert_eq!(entries[2].action, "aggregate");
    }

    #[test]
    fn reversal_with_valid_auth() {
        let lc = AnonymizationLifecycle::<Raw, String>::new("original".to_string());
        let pseudo = lc.pseudonymize(|_| "pseudo".to_string());
        let auth = ReversalAuthorization::new("admin", "ICH E2B submission");
        let result = pseudo.reverse(auth, |_| "original".to_string());
        assert!(result.is_ok());
        let raw = result.unwrap_or_else(|_| AnonymizationLifecycle::new("fail".to_string()));
        assert_eq!(raw.data(), "original");
    }

    #[test]
    fn reversal_denied_empty_authorizer() {
        let lc = AnonymizationLifecycle::<Raw, String>::new("data".to_string());
        let pseudo = lc.pseudonymize(|s| s);
        let auth = ReversalAuthorization::new("", "no reason");
        let result = pseudo.reverse(auth, |s| s);
        assert!(result.is_err());
    }

    #[test]
    fn dual_auth_reversal() {
        let lc = AnonymizationLifecycle::<Raw, String>::new("data".to_string());
        let pseudo = lc.pseudonymize(|s| s);
        let auth1 = ReversalAuthorization::new("admin_a", "review");
        let auth2 = ReversalAuthorization::new("admin_b", "review");
        let result = pseudo.reverse_dual_auth(auth1, auth2, |s| s);
        assert!(result.is_ok());
    }

    #[test]
    fn dual_auth_denied_same_person() {
        let lc = AnonymizationLifecycle::<Raw, String>::new("data".to_string());
        let pseudo = lc.pseudonymize(|s| s);
        let auth1 = ReversalAuthorization::new("admin", "review");
        let auth2 = ReversalAuthorization::new("admin", "second try");
        let result = pseudo.reverse_dual_auth(auth1, auth2, |s| s);
        assert!(result.is_err());
    }

    #[test]
    fn reversal_records_in_audit() {
        let lc = AnonymizationLifecycle::<Raw, String>::new("data".to_string());
        let pseudo = lc.pseudonymize(|s| s);
        let auth = ReversalAuthorization::new("admin", "test");
        let raw = pseudo
            .reverse(auth, |s| s)
            .unwrap_or_else(|_| AnonymizationLifecycle::new("fail".to_string()));
        let entries = raw.audit().entries();
        assert!(entries.iter().any(|e| e.action == "reverse"));
    }

    #[test]
    fn raw_data_access() {
        let lc = AnonymizationLifecycle::<Raw, Vec<u8>>::new(vec![1, 2, 3]);
        assert_eq!(lc.data(), &vec![1, 2, 3]);
    }

    #[test]
    fn pseudonymized_data_access() {
        let lc = AnonymizationLifecycle::<Raw, i32>::new(100);
        let pseudo = lc.pseudonymize(|x| x + 1);
        assert_eq!(*pseudo.data(), 101);
    }

    #[test]
    fn anonymized_data_access() {
        let lc = AnonymizationLifecycle::<Raw, i32>::new(50);
        let pseudo = lc.pseudonymize(|x| x);
        let anon = pseudo.anonymize(|x| x * 2);
        assert_eq!(*anon.data(), 100);
    }

    #[test]
    fn aggregated_data_access() {
        let lc = AnonymizationLifecycle::<Raw, i32>::new(1);
        let pseudo = lc.pseudonymize(|x| x);
        let anon = pseudo.anonymize(|x| x);
        let agg = anon.aggregate(|x| x + 999);
        assert_eq!(*agg.data(), 1000);
    }

    #[test]
    fn new_lifecycle_has_empty_audit() {
        let lc = AnonymizationLifecycle::<Raw, ()>::new(());
        assert!(lc.audit().is_empty());
    }

    #[test]
    fn debug_format_works() {
        let lc = AnonymizationLifecycle::<Raw, String>::new("test".to_string());
        let debug = format!("{lc:?}");
        assert!(debug.contains("test"));
    }

    #[test]
    fn reversal_authorization_has_timestamp() {
        let auth = ReversalAuthorization::new("user", "reason");
        assert!(!auth.timestamp.is_empty());
    }

    #[test]
    fn pseudonymize_then_anonymize_then_aggregate_audit_count() {
        let lc = AnonymizationLifecycle::<Raw, ()>::new(());
        let p = lc.pseudonymize(|x| x);
        assert_eq!(p.audit().len(), 1);
        let a = p.anonymize(|x| x);
        assert_eq!(a.audit().len(), 2);
        let g = a.aggregate(|x| x);
        assert_eq!(g.audit().len(), 3);
    }

    #[test]
    fn reversal_then_re_pseudonymize() {
        let lc = AnonymizationLifecycle::<Raw, String>::new("orig".to_string());
        let pseudo = lc.pseudonymize(|_| "p1".to_string());
        let auth = ReversalAuthorization::new("admin", "fix");
        let raw = pseudo
            .reverse(auth, |_| "orig".to_string())
            .unwrap_or_else(|_| AnonymizationLifecycle::new("fail".to_string()));
        let pseudo2 = raw.pseudonymize(|_| "p2".to_string());
        assert_eq!(pseudo2.data(), "p2");
        // audit: pseudonymize + reverse + pseudonymize = 3 entries
        assert_eq!(pseudo2.audit().len(), 3);
    }

    #[test]
    fn dual_auth_records_both_authorizers() {
        let lc = AnonymizationLifecycle::<Raw, String>::new("data".to_string());
        let pseudo = lc.pseudonymize(|s| s);
        let auth1 = ReversalAuthorization::new("alice", "review");
        let auth2 = ReversalAuthorization::new("bob", "review");
        let raw = pseudo
            .reverse_dual_auth(auth1, auth2, |s| s)
            .unwrap_or_else(|_| AnonymizationLifecycle::new("fail".to_string()));
        let entries = raw.audit().entries();
        let reversal = entries.iter().find(|e| e.action == "reverse_dual");
        assert!(reversal.is_some());
        let reason = &reversal.map(|e| e.reason.clone()).unwrap_or_default();
        assert!(reason.contains("alice"));
        assert!(reason.contains("bob"));
    }
}
