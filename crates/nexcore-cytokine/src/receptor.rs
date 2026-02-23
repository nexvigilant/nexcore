//! Cytokine receptor trait and implementations.
//!
//! ## T1 Grounding
//!
//! - `Receptor` → μ (mapping) - binds specific cytokine family
//! - `on_signal()` → → (causality) - signal causes handler invocation
//! - `Affinity` → N (quantity) - binding strength

use crate::{Cytokine, CytokineFamily, Scope, ThreatLevel};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// Receptor binding affinity (how strongly it binds to signals)
///
/// # Tier: T2-P
/// Grounds to: N (quantity)
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum Affinity {
    /// Low affinity - only binds strong signals
    Low = 1,
    /// Medium affinity - binds moderate signals
    #[default]
    Medium = 2,
    /// High affinity - binds even weak signals
    High = 3,
}

/// Filter for which signals a receptor accepts
#[derive(Debug, Clone, Default)]
pub struct ReceptorFilter {
    /// Which families to accept (empty = all)
    pub families: Vec<CytokineFamily>,
    /// Minimum severity to accept
    pub min_severity: Option<ThreatLevel>,
    /// Which scopes to accept (empty = all)
    pub scopes: Vec<Scope>,
    /// Specific signal names to match (empty = all)
    pub names: Vec<String>,
    /// Source component filter (empty = all)
    pub sources: Vec<String>,
}

impl ReceptorFilter {
    /// Create a filter for a specific family
    pub fn family(family: CytokineFamily) -> Self {
        Self {
            families: vec![family],
            ..Default::default()
        }
    }

    /// Create a filter for multiple families
    pub fn families(families: impl IntoIterator<Item = CytokineFamily>) -> Self {
        Self {
            families: families.into_iter().collect(),
            ..Default::default()
        }
    }

    /// Add minimum severity filter
    #[must_use]
    pub fn with_min_severity(mut self, severity: ThreatLevel) -> Self {
        self.min_severity = Some(severity);
        self
    }

    /// Add scope filter
    #[must_use]
    pub fn with_scope(mut self, scope: Scope) -> Self {
        self.scopes.push(scope);
        self
    }

    /// Add name filter
    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.names.push(name.into());
        self
    }

    /// Check if a signal matches this filter
    pub fn matches(&self, signal: &Cytokine) -> bool {
        // Family filter
        if !self.families.is_empty() && !self.families.contains(&signal.family) {
            return false;
        }

        // Severity filter
        if let Some(min_sev) = self.min_severity
            && signal.severity < min_sev
        {
            return false;
        }

        // Scope filter
        if !self.scopes.is_empty() && !self.scopes.contains(&signal.scope) {
            return false;
        }

        // Name filter
        if !self.names.is_empty() && !self.names.contains(&signal.name) {
            return false;
        }

        // Source filter
        if !self.sources.is_empty() {
            match &signal.source {
                Some(src) => {
                    if !self.sources.contains(src) {
                        return false;
                    }
                }
                None => return false,
            }
        }

        true
    }
}

/// Handler function type for received signals
pub type SignalHandler =
    Arc<dyn Fn(Cytokine) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

/// Trait for components that can receive cytokine signals.
///
/// # Tier: T2-P (Cross-Domain Primitive)
/// Grounds to: μ (mapping)
pub trait Receptor: Send + Sync {
    /// Get the receptor's identifier
    fn receptor_id(&self) -> &str;

    /// Get the filter for this receptor
    fn filter(&self) -> &ReceptorFilter;

    /// Get binding affinity
    fn affinity(&self) -> Affinity {
        Affinity::default()
    }

    /// Handle a received signal
    fn on_signal(&self, signal: Cytokine) -> Pin<Box<dyn Future<Output = ()> + Send + '_>>;

    /// Check if this receptor accepts a signal
    fn accepts(&self, signal: &Cytokine) -> bool {
        self.filter().matches(signal)
    }
}

/// A simple receptor implementation using a closure
pub struct FnReceptor {
    id: String,
    filter: ReceptorFilter,
    affinity: Affinity,
    handler: SignalHandler,
}

impl FnReceptor {
    /// Create a new receptor with a handler function
    pub fn new<F, Fut>(id: impl Into<String>, filter: ReceptorFilter, handler: F) -> Self
    where
        F: Fn(Cytokine) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        Self {
            id: id.into(),
            filter,
            affinity: Affinity::default(),
            handler: Arc::new(move |signal| Box::pin(handler(signal))),
        }
    }

    /// Set the affinity
    #[must_use]
    pub fn with_affinity(mut self, affinity: Affinity) -> Self {
        self.affinity = affinity;
        self
    }
}

impl Receptor for FnReceptor {
    fn receptor_id(&self) -> &str {
        &self.id
    }

    fn filter(&self) -> &ReceptorFilter {
        &self.filter
    }

    fn affinity(&self) -> Affinity {
        self.affinity
    }

    fn on_signal(&self, signal: Cytokine) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        (self.handler)(signal)
    }
}

/// A logging receptor that logs all received signals
pub struct LoggingReceptor {
    id: String,
    filter: ReceptorFilter,
}

impl LoggingReceptor {
    /// Create a logging receptor that accepts all signals
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            filter: ReceptorFilter::default(),
        }
    }

    /// Create a logging receptor with a filter
    pub fn with_filter(id: impl Into<String>, filter: ReceptorFilter) -> Self {
        Self {
            id: id.into(),
            filter,
        }
    }
}

impl Receptor for LoggingReceptor {
    fn receptor_id(&self) -> &str {
        &self.id
    }

    fn filter(&self) -> &ReceptorFilter {
        &self.filter
    }

    fn on_signal(&self, signal: Cytokine) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            tracing::info!(
                receptor = %signal.id,
                family = %signal.family,
                name = %signal.name,
                severity = %signal.severity,
                "🎯 Signal received"
            );
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_receptor_filter_family() {
        let filter = ReceptorFilter::family(CytokineFamily::Il1);
        let il1 = Cytokine::new(CytokineFamily::Il1, "test");
        let tnf = Cytokine::new(CytokineFamily::TnfAlpha, "test");

        assert!(filter.matches(&il1));
        assert!(!filter.matches(&tnf));
    }

    #[test]
    fn test_receptor_filter_severity() {
        let filter = ReceptorFilter::default().with_min_severity(ThreatLevel::High);

        let low = Cytokine::new(CytokineFamily::Il1, "test").with_severity(ThreatLevel::Low);
        let high = Cytokine::new(CytokineFamily::Il1, "test").with_severity(ThreatLevel::High);

        assert!(!filter.matches(&low));
        assert!(filter.matches(&high));
    }

    #[test]
    fn test_receptor_filter_combined() {
        let filter = ReceptorFilter::family(CytokineFamily::Il1)
            .with_min_severity(ThreatLevel::Medium)
            .with_name("threat");

        let matching =
            Cytokine::new(CytokineFamily::Il1, "threat").with_severity(ThreatLevel::High);
        let wrong_family =
            Cytokine::new(CytokineFamily::TnfAlpha, "threat").with_severity(ThreatLevel::High);
        let wrong_name =
            Cytokine::new(CytokineFamily::Il1, "other").with_severity(ThreatLevel::High);

        assert!(filter.matches(&matching));
        assert!(!filter.matches(&wrong_family));
        assert!(!filter.matches(&wrong_name));
    }

    #[tokio::test]
    async fn test_fn_receptor() {
        use std::sync::atomic::{AtomicBool, Ordering};

        let received = Arc::new(AtomicBool::new(false));
        let received_clone = received.clone();

        let receptor = FnReceptor::new("test", ReceptorFilter::default(), move |_signal| {
            let received = received_clone.clone();
            async move {
                received.store(true, Ordering::SeqCst);
            }
        });

        let signal = Cytokine::new(CytokineFamily::Il1, "test");
        receptor.on_signal(signal).await;

        assert!(received.load(Ordering::SeqCst));
    }
}
