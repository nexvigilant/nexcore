//! Cytokine emitter trait and implementations.
//!
//! ## T1 Grounding
//!
//! - `Emitter` → → (causality) - emission causes downstream effects
//! - `emit()` → π (persistence) - signal persists in bus until consumed
//! - `emit_batch()` → σ (sequence) - ordered emission

use crate::{Cytokine, CytokineFamily, Scope, ThreatLevel};
use std::future::Future;
use std::pin::Pin;

/// Error type for emission failures
#[derive(Debug, nexcore_error::Error)]
pub enum EmitError {
    /// Channel closed or full
    #[error("emission channel unavailable: {0}")]
    ChannelError(String),

    /// Signal validation failed
    #[error("invalid signal: {0}")]
    ValidationError(String),

    /// Rate limited
    #[error("rate limited: {0}")]
    RateLimited(String),
}

/// Result type for emission operations
pub type EmitResult<T = ()> = Result<T, EmitError>;

/// Trait for components that can emit cytokine signals.
///
/// # Tier: T2-P (Cross-Domain Primitive)
/// Grounds to: → (causality)
pub trait Emitter: Send + Sync {
    /// Emit a cytokine signal (fire-and-forget).
    ///
    /// Returns immediately after queueing the signal.
    fn emit(&self, signal: Cytokine) -> Pin<Box<dyn Future<Output = EmitResult> + Send + '_>>;

    /// Emit multiple signals in order.
    fn emit_batch(
        &self,
        signals: Vec<Cytokine>,
    ) -> Pin<Box<dyn Future<Output = EmitResult<usize>> + Send + '_>> {
        Box::pin(async move {
            let mut count = 0;
            for signal in signals {
                self.emit(signal).await?;
                count += 1;
            }
            Ok(count)
        })
    }

    /// Get the emitter's source identifier
    fn source_id(&self) -> &str;
}

/// Convenience methods for common signal emissions
pub trait EmitterExt: Emitter {
    /// Emit an alarm (IL-1)
    fn alarm(
        &self,
        name: impl Into<String>,
    ) -> Pin<Box<dyn Future<Output = EmitResult> + Send + '_>> {
        let signal = Cytokine::alarm(name).with_source(self.source_id());
        self.emit(signal)
    }

    /// Emit a termination signal (TNF-α)
    fn terminate(
        &self,
        name: impl Into<String>,
    ) -> Pin<Box<dyn Future<Output = EmitResult> + Send + '_>> {
        let signal = Cytokine::terminate(name).with_source(self.source_id());
        self.emit(signal)
    }

    /// Emit a suppression signal (IL-10)
    fn suppress(
        &self,
        name: impl Into<String>,
    ) -> Pin<Box<dyn Future<Output = EmitResult> + Send + '_>> {
        let signal = Cytokine::suppress(name).with_source(self.source_id());
        self.emit(signal)
    }

    /// Emit an amplification signal (IFN-γ)
    fn amplify(
        &self,
        name: impl Into<String>,
    ) -> Pin<Box<dyn Future<Output = EmitResult> + Send + '_>> {
        let signal = Cytokine::amplify(name).with_source(self.source_id());
        self.emit(signal)
    }

    /// Emit a custom signal with full parameters
    fn emit_custom(
        &self,
        family: CytokineFamily,
        name: impl Into<String>,
        severity: ThreatLevel,
        scope: Scope,
        payload: serde_json::Value,
    ) -> Pin<Box<dyn Future<Output = EmitResult> + Send + '_>> {
        let signal = Cytokine::new(family, name)
            .with_severity(severity)
            .with_scope(scope)
            .with_payload(payload)
            .with_source(self.source_id());
        self.emit(signal)
    }
}

// Blanket implementation
impl<T: Emitter + ?Sized> EmitterExt for T {}

/// A null emitter that discards all signals (for testing)
#[derive(Debug, Default)]
pub struct NullEmitter {
    source: String,
}

impl NullEmitter {
    /// Create a new null emitter
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
        }
    }
}

impl Emitter for NullEmitter {
    fn emit(&self, _signal: Cytokine) -> Pin<Box<dyn Future<Output = EmitResult> + Send + '_>> {
        Box::pin(async { Ok(()) })
    }

    fn source_id(&self) -> &str {
        &self.source
    }
}

/// A logging emitter that logs signals to tracing
#[derive(Debug, Default)]
pub struct TracingEmitter {
    source: String,
}

impl TracingEmitter {
    /// Create a new tracing emitter
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
        }
    }
}

impl Emitter for TracingEmitter {
    fn emit(&self, signal: Cytokine) -> Pin<Box<dyn Future<Output = EmitResult> + Send + '_>> {
        Box::pin(async move {
            tracing::info!(
                family = %signal.family,
                name = %signal.name,
                severity = %signal.severity,
                scope = %signal.scope,
                source = ?signal.source,
                target = ?signal.target,
                "🧬 Cytokine emitted"
            );
            Ok(())
        })
    }

    fn source_id(&self) -> &str {
        &self.source
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;

    #[tokio::test]
    async fn test_null_emitter() {
        let emitter = NullEmitter::new("test");
        let signal = Cytokine::new(CytokineFamily::Il1, "test");

        let result = emitter.emit(signal).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_emitter_ext_alarm() {
        let emitter = NullEmitter::new("guardian");
        let result = emitter.alarm("threat_detected").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_emit_batch() {
        let emitter = NullEmitter::new("test");
        let signals = vec![
            Cytokine::alarm("one"),
            Cytokine::alarm("two"),
            Cytokine::alarm("three"),
        ];

        let result = emitter.emit_batch(signals).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap_or(0), 3);
    }
}
