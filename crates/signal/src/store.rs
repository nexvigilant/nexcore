//! # Signal Store
//!
//! Persistence layer for detection results and alerts.
//! Provides in-memory and JSON file backends.
//!
//! ## T1 Primitive: State
//! Store is mutable state holding the accumulated results.

use crate::core::{Alert, AlertState, DetectionResult, Result, SignalError, Store};
use nexcore_id::NexId;
use std::collections::HashMap;

/// In-memory store backed by `HashMap`.
#[derive(Default)]
pub struct MemoryStore {
    results: Vec<DetectionResult>,
    alerts: HashMap<NexId, Alert>,
}

impl MemoryStore {
    /// Create empty store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Count of stored results.
    pub fn result_count(&self) -> usize {
        self.results.len()
    }

    /// Count of stored alerts.
    pub fn alert_count(&self) -> usize {
        self.alerts.len()
    }
}

impl Store for MemoryStore {
    fn save_result(&mut self, result: &DetectionResult) -> Result<()> {
        self.results.push(result.clone());
        Ok(())
    }

    fn save_alert(&mut self, alert: &Alert) -> Result<()> {
        self.alerts.insert(alert.id, alert.clone());
        Ok(())
    }

    fn get_alerts(&self, state: Option<AlertState>) -> Result<Vec<Alert>> {
        let alerts: Vec<Alert> = match state {
            Some(s) => self
                .alerts
                .values()
                .filter(|a| a.state == s)
                .cloned()
                .collect(),
            None => self.alerts.values().cloned().collect(),
        };
        Ok(alerts)
    }
}

/// JSON file-based store that serializes to a path.
pub struct JsonFileStore {
    path: std::path::PathBuf,
    inner: MemoryStore,
}

impl JsonFileStore {
    /// Create store backed by a JSON file.
    pub fn new(path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            path: path.into(),
            inner: MemoryStore::new(),
        }
    }

    /// Flush current state to disk.
    pub fn flush(&self) -> Result<()> {
        let data = serde_json::json!({
            "results": self.inner.results,
            "alerts": self.inner.alerts.values().collect::<Vec<_>>(),
        });
        let json =
            serde_json::to_string_pretty(&data).map_err(|e| SignalError::Storage(e.to_string()))?;
        std::fs::write(&self.path, json).map_err(|e| SignalError::Storage(e.to_string()))?;
        Ok(())
    }
}

impl Store for JsonFileStore {
    fn save_result(&mut self, result: &DetectionResult) -> Result<()> {
        self.inner.save_result(result)
    }

    fn save_alert(&mut self, alert: &Alert) -> Result<()> {
        self.inner.save_alert(alert)
    }

    fn get_alerts(&self, state: Option<AlertState>) -> Result<Vec<Alert>> {
        self.inner.get_alerts(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::*;
    use nexcore_chrono::DateTime;

    fn make_alert() -> Alert {
        Alert {
            id: NexId::v4(),
            detection: DetectionResult {
                pair: DrugEventPair::new("drug", "event"),
                table: ContingencyTable {
                    a: 10,
                    b: 100,
                    c: 20,
                    d: 10_000,
                },
                prr: Some(Prr(3.0)),
                ror: Some(Ror(5.0)),
                ic: Some(Ic(1.5)),
                ebgm: Some(Ebgm(2.0)),
                chi_square: ChiSquare(10.0),
                strength: SignalStrength::Strong,
                detected_at: DateTime::now(),
            },
            state: AlertState::New,
            created_at: DateTime::now(),
            updated_at: DateTime::now(),
            notes: vec![],
        }
    }

    #[test]
    fn memory_store_crud() {
        let mut store = MemoryStore::new();
        let alert = make_alert();
        store.save_alert(&alert).unwrap();
        assert_eq!(store.alert_count(), 1);
        let new_alerts = store.get_alerts(Some(AlertState::New)).unwrap();
        assert_eq!(new_alerts.len(), 1);
    }
}
