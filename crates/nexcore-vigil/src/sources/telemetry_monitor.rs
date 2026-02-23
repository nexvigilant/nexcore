//! Telemetry Monitor Source
//!
//! Watches Claude Code telemetry data and emits events for FRIDAY to process.
//! Monitors session logs, token usage, and prompt patterns.

use std::path::PathBuf;
use std::time::Duration;

use chrono::{DateTime, Utc};
use nexcore_id::NexId;
use notify::{Event as NotifyEvent, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::{info, warn};

use crate::events::EventBus;
use crate::models::{Event, Urgency};

/// Telemetry metrics tracked by the monitor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetrySnapshot {
    pub timestamp: DateTime<Utc>,
    pub session_count: u32,
    pub total_messages: u32,
    pub total_tokens: u64,
    pub cache_tokens: u64,
    pub estimated_cost_usd: f64,
    pub tool_calls: u32,
    pub active_time_seconds: u64,
}

/// Alert thresholds for telemetry monitoring
#[derive(Debug, Clone)]
pub struct TelemetryThresholds {
    /// Warn if tokens exceed this in a session
    pub token_warning: u64,
    /// Alert if cost exceeds this USD
    pub cost_alert: f64,
    /// Warn if session is idle for this many seconds
    pub idle_warning_seconds: u64,
    /// Alert if cache hit rate drops below this
    pub cache_hit_rate_min: f64,
}

impl Default for TelemetryThresholds {
    fn default() -> Self {
        Self {
            token_warning: 100_000,
            cost_alert: 10.0,
            idle_warning_seconds: 300,
            cache_hit_rate_min: 0.5,
        }
    }
}

/// Telemetry Monitor that watches Claude Code data
pub struct TelemetryMonitor {
    bus: EventBus,
    claude_dir: PathBuf,
    thresholds: TelemetryThresholds,
    last_snapshot: Option<TelemetrySnapshot>,
}

impl TelemetryMonitor {
    /// Create a new telemetry monitor
    pub fn new(bus: EventBus) -> Self {
        let claude_dir = std::env::var("HOME")
            .map(|h| PathBuf::from(h).join(".claude"))
            .unwrap_or_else(|_| PathBuf::from(".claude"));

        Self {
            bus,
            claude_dir,
            thresholds: TelemetryThresholds::default(),
            last_snapshot: None,
        }
    }

    /// Configure alert thresholds
    pub fn with_thresholds(mut self, thresholds: TelemetryThresholds) -> Self {
        self.thresholds = thresholds;
        self
    }

    /// Start monitoring telemetry
    pub async fn run(&mut self) -> nexcore_error::Result<()> {
        info!(path = ?self.claude_dir, "Starting telemetry monitor");

        // Initial scan
        self.scan_telemetry().await?;

        // Set up file watcher for real-time updates
        let (tx, mut rx) = mpsc::channel(100);

        let mut watcher = notify::recommended_watcher(move |res: Result<NotifyEvent, _>| {
            if let Ok(event) = res {
                // Best-effort send from sync callback - receiver may be dropped
                // during shutdown, which is expected and safe to ignore
                if tx.blocking_send(event).is_err() {
                    // Channel closed, watcher will be dropped soon
                }
            }
        })?;

        // Watch key telemetry files
        let stats_path = self.claude_dir.join("stats-cache.json");
        let history_path = self.claude_dir.join("history.jsonl");
        let kinetics_path = self.claude_dir.join("prompt_kinetics");

        if stats_path.exists() {
            watcher.watch(&stats_path, RecursiveMode::NonRecursive)?;
        }
        if history_path.exists() {
            watcher.watch(&history_path, RecursiveMode::NonRecursive)?;
        }
        if kinetics_path.exists() {
            watcher.watch(&kinetics_path, RecursiveMode::Recursive)?;
        }

        // Also do periodic scans every 30 seconds
        let mut interval = tokio::time::interval(Duration::from_secs(30));

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    if let Err(e) = self.scan_telemetry().await {
                        warn!(error = %e, "Failed to scan telemetry");
                    }
                }
                Some(event) = rx.recv() => {
                    info!(event = ?event.kind, "Telemetry file changed");
                    if let Err(e) = self.scan_telemetry().await {
                        warn!(error = %e, "Failed to scan telemetry after change");
                    }
                }
            }
        }
    }

    /// Scan telemetry files and emit events
    async fn scan_telemetry(&mut self) -> nexcore_error::Result<()> {
        let snapshot = self.collect_snapshot().await?;

        // Check for alerts
        self.check_alerts(&snapshot).await?;

        // Emit telemetry event
        let event = Event {
            id: NexId::v4(),
            source: "telemetry_monitor".to_string(),
            event_type: "telemetry_snapshot".to_string(),
            priority: Urgency::Low,
            payload: serde_json::to_value(&snapshot)?,
            timestamp: Utc::now(),
            correlation_id: None,
        };

        self.bus.emit(event).await;

        self.last_snapshot = Some(snapshot);
        Ok(())
    }

    /// Collect current telemetry snapshot
    async fn collect_snapshot(&self) -> nexcore_error::Result<TelemetrySnapshot> {
        let stats_path = self.claude_dir.join("stats-cache.json");

        let stats: serde_json::Value = if stats_path.exists() {
            let content = tokio::fs::read_to_string(&stats_path).await?;
            serde_json::from_str(&content)?
        } else {
            serde_json::json!({})
        };

        // Extract stats from cache
        let today = Utc::now().format("%Y-%m-%d").to_string();
        let empty_obj = serde_json::json!({});
        let daily = stats.get(&today).unwrap_or(&empty_obj);

        let session_count = daily
            .get("sessionCount")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        let total_messages = daily
            .get("messageCount")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        // Get model usage
        let models = daily.get("modelsUsed").unwrap_or(&empty_obj);
        let mut total_tokens: u64 = 0;
        let mut cache_tokens: u64 = 0;

        if let Some(obj) = models.as_object() {
            for (_, model_stats) in obj {
                total_tokens += model_stats
                    .get("inputTokens")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                total_tokens += model_stats
                    .get("outputTokens")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                cache_tokens += model_stats
                    .get("cacheReadInputTokens")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
            }
        }

        let tool_calls = daily.get("toolCalls").and_then(|v| v.as_u64()).unwrap_or(0) as u32;

        // Estimate cost (rough calculation)
        let input_cost = (total_tokens as f64) * 0.000003; // $3/M input
        let output_cost = (total_tokens as f64) * 0.000015; // $15/M output
        let estimated_cost_usd = input_cost + output_cost;

        Ok(TelemetrySnapshot {
            timestamp: Utc::now(),
            session_count,
            total_messages,
            total_tokens,
            cache_tokens,
            estimated_cost_usd,
            tool_calls,
            active_time_seconds: 0, // Would need to calculate from sessions
        })
    }

    /// Check thresholds and emit alerts
    async fn check_alerts(&self, snapshot: &TelemetrySnapshot) -> nexcore_error::Result<()> {
        // Check token usage
        if snapshot.total_tokens > self.thresholds.token_warning {
            let event = Event {
                id: NexId::v4(),
                source: "telemetry_monitor".to_string(),
                event_type: "token_warning".to_string(),
                priority: Urgency::Normal,
                payload: serde_json::json!({
                    "type": "token_warning",
                    "tokens": snapshot.total_tokens,
                    "threshold": self.thresholds.token_warning,
                    "message": format!(
                        "Token usage ({}) exceeds warning threshold ({})",
                        snapshot.total_tokens, self.thresholds.token_warning
                    )
                }),
                timestamp: Utc::now(),
                correlation_id: None,
            };
            self.bus.emit(event).await;
        }

        // Check cost
        if snapshot.estimated_cost_usd > self.thresholds.cost_alert {
            let event = Event {
                id: NexId::v4(),
                source: "telemetry_monitor".to_string(),
                event_type: "cost_alert".to_string(),
                priority: Urgency::High,
                payload: serde_json::json!({
                    "type": "cost_alert",
                    "cost_usd": snapshot.estimated_cost_usd,
                    "threshold": self.thresholds.cost_alert,
                    "message": format!(
                        "Estimated cost (${:.2}) exceeds alert threshold (${:.2})",
                        snapshot.estimated_cost_usd, self.thresholds.cost_alert
                    )
                }),
                timestamp: Utc::now(),
                correlation_id: None,
            };
            self.bus.emit(event).await;
        }

        // Check cache hit rate
        let total_with_cache = snapshot.total_tokens + snapshot.cache_tokens;
        if total_with_cache > 0 {
            let cache_rate = snapshot.cache_tokens as f64 / total_with_cache as f64;
            if cache_rate < self.thresholds.cache_hit_rate_min {
                let event = Event {
                    id: NexId::v4(),
                    source: "telemetry_monitor".to_string(),
                    event_type: "cache_rate_warning".to_string(),
                    priority: Urgency::Low,
                    payload: serde_json::json!({
                        "type": "cache_rate_warning",
                        "cache_rate": cache_rate,
                        "threshold": self.thresholds.cache_hit_rate_min,
                        "message": format!(
                            "Cache hit rate ({:.1}%) is below threshold ({:.1}%)",
                            cache_rate * 100.0, self.thresholds.cache_hit_rate_min * 100.0
                        )
                    }),
                    timestamp: Utc::now(),
                    correlation_id: None,
                };
                self.bus.emit(event).await;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_thresholds() {
        let t = TelemetryThresholds::default();
        assert_eq!(t.token_warning, 100_000);
        assert!((t.cost_alert - 10.0).abs() < f64::EPSILON);
    }
}
