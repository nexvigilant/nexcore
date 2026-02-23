//! Cytokine bus - central signal routing and distribution.
//!
//! ## T1 Grounding
//!
//! - `CytokineBus` → σ (sequence) - ordered signal processing
//! - Signal routing → μ (mapping) - map signals to receptors
//! - Broadcast → ρ (recursion) - one signal to many

use crate::{
    CascadeRule, Cytokine, CytokineFamily, EmitResult, Emitter, Receptor, Scope, ThreatLevel,
};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};

/// Configuration for the cytokine bus
#[derive(Debug, Clone)]
pub struct BusConfig {
    /// Maximum channel capacity
    pub channel_capacity: usize,
    /// Enable cascade processing
    pub enable_cascades: bool,
    /// Maximum signals per second (0 = unlimited)
    pub rate_limit: u32,
    /// Log all signals
    pub logging_enabled: bool,
}

impl Default for BusConfig {
    fn default() -> Self {
        Self {
            channel_capacity: 1024,
            enable_cascades: true,
            rate_limit: 0,
            logging_enabled: false,
        }
    }
}

/// Statistics for the cytokine bus
#[derive(Debug, Clone, Default)]
pub struct BusStats {
    /// Total signals emitted
    pub signals_emitted: u64,
    /// Total signals delivered
    pub signals_delivered: u64,
    /// Signals dropped (channel full)
    pub signals_dropped: u64,
    /// Cascades triggered
    pub cascades_triggered: u64,
    /// Signals by family
    pub by_family: HashMap<String, u64>,
    /// Signals by severity
    pub by_severity: HashMap<String, u64>,
}

/// The central cytokine bus for signal routing.
///
/// # Tier: T2-C (Composite)
/// Grounds to: σ (sequence) + μ (mapping)
pub struct CytokineBus {
    config: BusConfig,
    sender: broadcast::Sender<Cytokine>,
    receptors: RwLock<Vec<Arc<dyn Receptor>>>,
    cascade_rules: RwLock<Vec<CascadeRule>>,
    stats: RwLock<BusStats>,
    source_id: String,
}

impl CytokineBus {
    /// Create a new cytokine bus
    pub fn new(source_id: impl Into<String>) -> Self {
        Self::with_config(source_id, BusConfig::default())
    }

    /// Create a bus with custom configuration
    pub fn with_config(source_id: impl Into<String>, config: BusConfig) -> Self {
        let (sender, _) = broadcast::channel(config.channel_capacity);
        Self {
            config,
            sender,
            receptors: RwLock::new(Vec::new()),
            cascade_rules: RwLock::new(Vec::new()),
            stats: RwLock::new(BusStats::default()),
            source_id: source_id.into(),
        }
    }

    /// Register a receptor to receive signals
    pub async fn register_receptor(&self, receptor: Arc<dyn Receptor>) {
        let mut receptors = self.receptors.write().await;
        receptors.push(receptor);
    }

    /// Register a cascade rule
    pub async fn register_cascade(&self, rule: CascadeRule) {
        let mut rules = self.cascade_rules.write().await;
        rules.push(rule);
    }

    /// Load pre-defined cascade patterns
    pub async fn load_default_cascades(&self) {
        use crate::cascade::patterns;

        self.register_cascade(patterns::inflammatory()).await;
        self.register_cascade(patterns::proliferation()).await;
        self.register_cascade(patterns::suppression()).await;
        self.register_cascade(patterns::activation()).await;
        self.register_cascade(patterns::critical_response()).await;
    }

    /// Get current statistics
    pub async fn stats(&self) -> BusStats {
        self.stats.read().await.clone()
    }

    /// Process a signal through the bus
    async fn process_signal(&self, signal: Cytokine, cascade_depth: u8) {
        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.signals_emitted += 1;
            *stats
                .by_family
                .entry(signal.family.to_string())
                .or_insert(0) += 1;
            *stats
                .by_severity
                .entry(signal.severity.to_string())
                .or_insert(0) += 1;
        }

        // Log if enabled
        if self.config.logging_enabled {
            tracing::debug!(
                id = %signal.id,
                family = %signal.family,
                name = %signal.name,
                severity = %signal.severity,
                cascade_depth = cascade_depth,
                "📡 Signal processing"
            );
        }

        // Deliver to receptors
        {
            let receptors = self.receptors.read().await;
            for receptor in receptors.iter() {
                if receptor.accepts(&signal) {
                    receptor.on_signal(signal.clone()).await;
                    let mut stats = self.stats.write().await;
                    stats.signals_delivered += 1;
                }
            }
        }

        // Process cascades if enabled and signal is cascadable
        if self.config.enable_cascades && signal.cascadable {
            let rules = self.cascade_rules.read().await;
            let mut cascade_signals = Vec::new();

            for rule in rules.iter() {
                let responses = rule.execute(&signal, cascade_depth);
                if !responses.is_empty() {
                    let mut stats = self.stats.write().await;
                    stats.cascades_triggered += 1;
                }
                cascade_signals.extend(responses);
            }

            // Process cascade signals (recursively but depth-limited)
            drop(rules); // Release lock before recursion
            for cascade_signal in cascade_signals {
                Box::pin(self.process_signal(cascade_signal, cascade_depth + 1)).await;
            }
        }

        // Broadcast to subscribers
        let _ = self.sender.send(signal);
    }

    /// Subscribe to receive all signals
    pub fn subscribe(&self) -> broadcast::Receiver<Cytokine> {
        self.sender.subscribe()
    }

    /// Get receptor count
    pub async fn receptor_count(&self) -> usize {
        self.receptors.read().await.len()
    }

    /// Get cascade rule count
    pub async fn cascade_count(&self) -> usize {
        self.cascade_rules.read().await.len()
    }

    /// Convenience: emit alarm
    pub async fn alarm(&self, name: impl Into<String>) -> EmitResult {
        self.emit(Cytokine::alarm(name)).await
    }

    /// Convenience: emit termination
    pub async fn terminate(&self, name: impl Into<String>) -> EmitResult {
        self.emit(Cytokine::terminate(name)).await
    }

    /// Convenience: emit suppression
    pub async fn suppress(&self, name: impl Into<String>) -> EmitResult {
        self.emit(Cytokine::suppress(name)).await
    }

    /// Convenience: emit amplification
    pub async fn amplify(&self, name: impl Into<String>) -> EmitResult {
        self.emit(Cytokine::amplify(name)).await
    }

    /// Emit a custom signal
    pub async fn emit_custom(
        &self,
        family: CytokineFamily,
        name: impl Into<String>,
        severity: ThreatLevel,
        scope: Scope,
        payload: serde_json::Value,
    ) -> EmitResult {
        let signal = Cytokine::new(family, name)
            .with_severity(severity)
            .with_scope(scope)
            .with_payload(payload)
            .with_source(&self.source_id);
        self.emit(signal).await
    }
}

impl Emitter for CytokineBus {
    fn emit(&self, signal: Cytokine) -> Pin<Box<dyn Future<Output = EmitResult> + Send + '_>> {
        Box::pin(async move {
            self.process_signal(signal, 0).await;
            Ok(())
        })
    }

    fn source_id(&self) -> &str {
        &self.source_id
    }
}

/// A global bus instance for simple use cases
static GLOBAL_BUS: std::sync::OnceLock<CytokineBus> = std::sync::OnceLock::new();

/// Get or initialize the global bus
pub fn global_bus() -> &'static CytokineBus {
    GLOBAL_BUS.get_or_init(|| CytokineBus::new("global"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FnReceptor, ReceptorFilter};
    use std::sync::atomic::{AtomicU32, Ordering};

    #[tokio::test]
    async fn test_bus_creation() {
        let bus = CytokineBus::new("test");
        assert_eq!(bus.receptor_count().await, 0);
        assert_eq!(bus.cascade_count().await, 0);
    }

    #[tokio::test]
    async fn test_signal_delivery() {
        let received = Arc::new(AtomicU32::new(0));
        let received_clone = received.clone();

        let receptor = FnReceptor::new("counter", ReceptorFilter::default(), move |_signal| {
            let received = received_clone.clone();
            async move {
                received.fetch_add(1, Ordering::SeqCst);
            }
        });

        let bus = CytokineBus::new("test");
        bus.register_receptor(Arc::new(receptor)).await;

        bus.alarm("test1").await.ok();
        bus.alarm("test2").await.ok();
        bus.alarm("test3").await.ok();

        // Give async handlers time to complete
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        assert_eq!(received.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_filtered_receptor() {
        let received = Arc::new(AtomicU32::new(0));
        let received_clone = received.clone();

        let receptor = FnReceptor::new(
            "il1_only",
            ReceptorFilter::family(CytokineFamily::Il1),
            move |_signal| {
                let received = received_clone.clone();
                async move {
                    received.fetch_add(1, Ordering::SeqCst);
                }
            },
        );

        let bus = CytokineBus::new("test");
        bus.register_receptor(Arc::new(receptor)).await;

        // Emit IL-1 (should be received)
        bus.alarm("test").await.ok();

        // Emit TNF-α (should NOT be received by IL-1 receptor)
        bus.terminate("test").await.ok();

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // Only 1 signal should have been received
        assert_eq!(received.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_cascade_execution() {
        use crate::cascade::patterns;

        let received_il6 = Arc::new(AtomicU32::new(0));
        let received_il6_clone = received_il6.clone();

        let receptor = FnReceptor::new(
            "il6_counter",
            ReceptorFilter::family(CytokineFamily::Il6),
            move |_signal| {
                let received = received_il6_clone.clone();
                async move {
                    received.fetch_add(1, Ordering::SeqCst);
                }
            },
        );

        let bus = CytokineBus::new("test");
        bus.register_receptor(Arc::new(receptor)).await;
        bus.register_cascade(patterns::inflammatory()).await;

        // Emit IL-1 which should cascade to IL-6
        bus.alarm("trigger").await.ok();

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // IL-6 receptor should have received the cascaded signal
        assert!(received_il6.load(Ordering::SeqCst) >= 1);
    }

    #[tokio::test]
    async fn test_stats_tracking() {
        let bus = CytokineBus::new("test");

        bus.alarm("one").await.ok();
        bus.alarm("two").await.ok();
        bus.terminate("three").await.ok();

        let stats = bus.stats().await;
        assert_eq!(stats.signals_emitted, 3);
        assert_eq!(*stats.by_family.get("IL-1").unwrap_or(&0), 2);
        assert_eq!(*stats.by_family.get("TNF-α").unwrap_or(&0), 1);
    }
}
