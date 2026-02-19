//! Claude Metrics API client

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const DEFAULT_URL: &str = "https://claude-metrics-17740146943.us-central1.run.app";

/// Health response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

/// Service status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusResponse {
    pub uptime_seconds: i64,
    pub metrics_count: usize,
    pub dashboards_count: usize,
    pub alerts_count: usize,
    pub version: String,
}

/// Metrics response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsResponse {
    pub timestamp: DateTime<Utc>,
    pub metrics: HashMap<String, f64>,
}

/// Alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub name: String,
    pub expr: String,
    pub severity: String,
    pub created_at: DateTime<Utc>,
    pub silenced_until: Option<DateTime<Utc>>,
}

/// Service config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub scrape_interval_seconds: u64,
    pub retention_days: u64,
    pub rate_limit_per_minute: u64,
}

/// Metrics API client
#[derive(Clone)]
pub struct MetricsClient {
    base_url: String,
    api_key: Option<String>,
}

impl MetricsClient {
    pub fn new(api_key: Option<String>) -> Self {
        Self {
            base_url: DEFAULT_URL.to_string(),
            api_key,
        }
    }

    pub fn with_url(mut self, url: &str) -> Self {
        self.base_url = url.to_string();
        self
    }
}

// Server-side implementations using reqwest
#[cfg(feature = "ssr")]
impl MetricsClient {
    pub async fn health(&self) -> anyhow::Result<HealthResponse> {
        let resp = reqwest::get(format!("{}/health", self.base_url))
            .await?
            .json()
            .await?;
        Ok(resp)
    }

    pub async fn status(&self) -> anyhow::Result<StatusResponse> {
        let client = reqwest::Client::new();
        let mut req = client.get(format!("{}/api/v1/status", self.base_url));
        if let Some(key) = &self.api_key {
            req = req.header("X-API-Key", key);
        }
        let resp = req.send().await?.json().await?;
        Ok(resp)
    }

    pub async fn get_metrics(&self) -> anyhow::Result<MetricsResponse> {
        let client = reqwest::Client::new();
        let mut req = client.get(format!("{}/api/v1/metrics", self.base_url));
        if let Some(key) = &self.api_key {
            req = req.header("X-API-Key", key);
        }
        let resp = req.send().await?.json().await?;
        Ok(resp)
    }

    pub async fn get_alerts(&self) -> anyhow::Result<Vec<Alert>> {
        let client = reqwest::Client::new();
        let mut req = client.get(format!("{}/api/v1/alerts", self.base_url));
        if let Some(key) = &self.api_key {
            req = req.header("X-API-Key", key);
        }
        let resp = req.send().await?.json().await?;
        Ok(resp)
    }

    pub async fn get_config(&self) -> anyhow::Result<ServiceConfig> {
        let client = reqwest::Client::new();
        let mut req = client.get(format!("{}/api/v1/config", self.base_url));
        if let Some(key) = &self.api_key {
            req = req.header("X-API-Key", key);
        }
        let resp = req.send().await?.json().await?;
        Ok(resp)
    }

    pub async fn push_metrics(&self, metrics: HashMap<String, f64>) -> anyhow::Result<()> {
        let client = reqwest::Client::new();
        let mut req = client
            .post(format!("{}/api/v1/metrics/push", self.base_url))
            .json(&metrics);
        if let Some(key) = &self.api_key {
            req = req.header("X-API-Key", key);
        }
        req.send().await?;
        Ok(())
    }
}
