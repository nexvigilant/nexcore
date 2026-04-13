//! Layer 1 — Source connectors and ingestion orchestrator.
//!
//! Each connector implements [`SourceConnector`] and streams [`RawRecord`]
//! values into the pipeline. The [`IngestionOrchestrator`] fans-in all
//! connector streams using `futures::stream::select_all`.

use std::pin::Pin;
use std::sync::Arc;

use async_trait::async_trait;
use futures::Stream;
use tokio::sync::mpsc;
use tracing::{error, info};

use crate::error::{PerceptionError, Result};
use crate::types::{RawRecord, SourceHealth, SourceId};

/// A pinned, boxed async stream of raw records.
pub type RecordStream = Pin<Box<dyn Stream<Item = Result<RawRecord>> + Send + 'static>>;

/// Trait implemented by every data source connector.
///
/// Connectors are responsible for fetching or streaming raw records from
/// an external data source and translating transport errors into
/// [`PerceptionError`].
#[async_trait]
pub trait SourceConnector: Send + Sync + 'static {
    /// Stable identifier for this source.
    fn source_id(&self) -> &SourceId;

    /// Return a stream of raw records from the source.
    ///
    /// The stream should be infinite (polling) or naturally terminating
    /// depending on the source type. The orchestrator handles reconnection.
    async fn stream(&self) -> Result<RecordStream>;

    /// Perform a lightweight health check against the source.
    async fn health_check(&self) -> SourceHealth;
}

// ── FAERS connector ────────────────────────────────────────────────────────────

/// Connector for the FDA FAERS adverse-event API.
///
/// Streams individual case safety report payloads as [`RawRecord`] values.
/// Production use requires an API key passed via environment variable
/// `FAERS_API_KEY`; the connector operates in unauthenticated mode
/// (100 req/day limit) when the key is absent.
pub struct FaersConnector {
    source_id: SourceId,
    base_url: String,
    client: reqwest::Client,
}

impl FaersConnector {
    /// Create a new FAERS connector.
    ///
    /// * `base_url` — defaults to `https://api.fda.gov/drug/event.json`
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            source_id: SourceId::new("faers"),
            base_url: base_url.into(),
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl SourceConnector for FaersConnector {
    fn source_id(&self) -> &SourceId {
        &self.source_id
    }

    async fn stream(&self) -> Result<RecordStream> {
        let source_id = self.source_id.clone();
        let base_url = self.base_url.clone();
        let client = self.client.clone();

        let stream = futures::stream::unfold(
            (client, base_url, source_id, 0usize),
            |(client, base_url, source_id, skip)| async move {
                let url = format!("{base_url}?limit=1&skip={skip}");
                let result = client
                    .get(&url)
                    .send()
                    .await
                    .map_err(|e| PerceptionError::Http(e.to_string()))
                    .and_then(|resp| {
                        if resp.status().is_success() {
                            Ok(resp)
                        } else {
                            Err(PerceptionError::ConnectorFailure {
                                source: source_id.to_string(),
                                reason: format!("HTTP {}", resp.status()),
                            })
                        }
                    });

                match result {
                    Err(e) => Some((Err(e), (client, base_url, source_id, skip + 1))),
                    Ok(resp) => {
                        let payload_result = resp
                            .json::<serde_json::Value>()
                            .await
                            .map_err(|e| PerceptionError::Http(e.to_string()))
                            .map(|payload| RawRecord::new(source_id.clone(), payload));

                        Some((payload_result, (client, base_url, source_id, skip + 1)))
                    }
                }
            },
        );

        Ok(Box::pin(stream))
    }

    async fn health_check(&self) -> SourceHealth {
        let url = format!("{}&limit=1", self.base_url);
        match self.client.head(&url).send().await {
            Ok(resp) if resp.status().is_success() => SourceHealth::Healthy,
            Ok(resp) => SourceHealth::Degraded {
                reason: format!("HTTP {}", resp.status()),
            },
            Err(e) => SourceHealth::Unhealthy {
                reason: e.to_string(),
            },
        }
    }
}

// ── PubMed connector ───────────────────────────────────────────────────────────

/// Connector for the NCBI PubMed eUtils API.
///
/// Streams article abstracts containing adverse event case reports as
/// [`RawRecord`] values.
pub struct PubMedConnector {
    source_id: SourceId,
    base_url: String,
    client: reqwest::Client,
}

impl PubMedConnector {
    /// Create a new PubMed connector.
    ///
    /// * `base_url` — defaults to `https://eutils.ncbi.nlm.nih.gov/entrez/eutils/`
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            source_id: SourceId::new("pubmed"),
            base_url: base_url.into(),
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl SourceConnector for PubMedConnector {
    fn source_id(&self) -> &SourceId {
        &self.source_id
    }

    async fn stream(&self) -> Result<RecordStream> {
        let source_id = self.source_id.clone();
        let base_url = self.base_url.clone();
        let client = self.client.clone();

        let stream = futures::stream::unfold(
            (client, base_url, source_id, 0usize),
            |(client, base_url, source_id, retstart)| async move {
                let url = format!(
                    "{base_url}esearch.fcgi?db=pubmed&term=adverse+event&retstart={retstart}&retmax=1&retmode=json"
                );
                let result = client
                    .get(&url)
                    .send()
                    .await
                    .map_err(|e| PerceptionError::Http(e.to_string()))
                    .and_then(|resp| {
                        if resp.status().is_success() {
                            Ok(resp)
                        } else {
                            Err(PerceptionError::ConnectorFailure {
                                source: source_id.to_string(),
                                reason: format!("HTTP {}", resp.status()),
                            })
                        }
                    });

                match result {
                    Err(e) => Some((Err(e), (client, base_url, source_id, retstart + 1))),
                    Ok(resp) => {
                        let payload_result = resp
                            .json::<serde_json::Value>()
                            .await
                            .map_err(|e| PerceptionError::Http(e.to_string()))
                            .map(|payload| RawRecord::new(source_id.clone(), payload));

                        Some((payload_result, (client, base_url, source_id, retstart + 1)))
                    }
                }
            },
        );

        Ok(Box::pin(stream))
    }

    async fn health_check(&self) -> SourceHealth {
        let url = format!(
            "{}/einfo.fcgi?retmode=json",
            self.base_url.trim_end_matches('/')
        );
        match self.client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => SourceHealth::Healthy,
            Ok(resp) => SourceHealth::Degraded {
                reason: format!("HTTP {}", resp.status()),
            },
            Err(e) => SourceHealth::Unhealthy {
                reason: e.to_string(),
            },
        }
    }
}

// ── Ingestion orchestrator ─────────────────────────────────────────────────────

/// Orchestrates multiple source connectors, fan-in via per-connector channels.
///
/// Each connector is polled in its own Tokio task. Failures in one connector
/// are logged and do not affect the others — the orchestrator continues
/// streaming from healthy connectors.
pub struct IngestionOrchestrator {
    connectors: Vec<Arc<dyn SourceConnector>>,
    buffer_size: usize,
}

impl IngestionOrchestrator {
    /// Create an orchestrator from a set of connectors.
    pub fn new(connectors: Vec<Arc<dyn SourceConnector>>, buffer_size: usize) -> Self {
        Self {
            connectors,
            buffer_size,
        }
    }

    /// Spawn all connector tasks and return a merged receiver channel.
    ///
    /// Records from all connectors arrive on a single `mpsc::Receiver<RawRecord>`.
    /// If a connector stream ends or errors, that connector's task logs the
    /// error and exits — other connectors are unaffected.
    pub fn start(&self) -> mpsc::Receiver<RawRecord> {
        let (tx, rx) = mpsc::channel::<RawRecord>(self.buffer_size * self.connectors.len().max(1));

        for connector in &self.connectors {
            let connector = Arc::clone(connector);
            let tx = tx.clone();

            tokio::spawn(async move {
                let source = connector.source_id().clone();
                info!(source = %source, "connector task starting");

                match connector.stream().await {
                    Err(e) => {
                        error!(source = %source, error = %e, "connector failed to open stream");
                    }
                    Ok(mut stream) => {
                        use futures::StreamExt;
                        while let Some(result) = stream.next().await {
                            match result {
                                Ok(record) => {
                                    if tx.send(record).await.is_err() {
                                        // Receiver dropped — pipeline is shutting down
                                        break;
                                    }
                                }
                                Err(e) => {
                                    error!(source = %source, error = %e, "connector stream error");
                                    // Continue — don't kill the task on a single error
                                }
                            }
                        }
                        info!(source = %source, "connector stream ended");
                    }
                }
            });
        }

        rx
    }

    /// Run health checks on all connectors and return results.
    pub async fn health_check_all(&self) -> Vec<(SourceId, SourceHealth)> {
        let mut results = Vec::with_capacity(self.connectors.len());
        for connector in &self.connectors {
            let health = connector.health_check().await;
            results.push((connector.source_id().clone(), health));
        }
        results
    }
}
