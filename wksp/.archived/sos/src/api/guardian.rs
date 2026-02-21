/// Guardian API client — nu Frequency (real-time alerts) + varsigma State (guardian status)
///
/// HTTP for status, WebSocket for live alert stream
use futures::StreamExt;
use gloo_net::http::Request;
use gloo_net::websocket::{futures::WebSocket, Message};
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;

use super::{api_base_url, url, ApiError};

/// Guardian system status
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GuardianStatus {
    #[serde(default)]
    pub state: String,
    #[serde(default)]
    pub active_alerts: u32,
    #[serde(default)]
    pub uptime_seconds: u64,
    #[serde(default)]
    pub last_tick: String,
}

/// A single guardian alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardianAlert {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub severity: String,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub timestamp: String,
    #[serde(default)]
    pub source: String,
}

/// Fetch guardian status
pub async fn get_status() -> Result<GuardianStatus, ApiError> {
    let resp = Request::get(&url("/api/v1/guardian")).send().await?;

    if resp.ok() {
        let body = resp.text().await?;
        let parsed: GuardianStatus = serde_json::from_str(&body)?;
        Ok(parsed)
    } else {
        Err(ApiError {
            message: format!("Guardian API returned {}", resp.status()),
        })
    }
}

/// Build WebSocket URL for guardian alert stream
pub fn ws_url() -> String {
    let base = api_base_url();
    let ws_base = base
        .replace("https://", "wss://")
        .replace("http://", "ws://");
    format!("{ws_base}/api/v1/guardian/ws/bridge")
}

/// Connect to guardian WebSocket and invoke callback on each alert
pub fn connect_ws(on_alert: impl Fn(GuardianAlert) + 'static) {
    spawn_local(async move {
        let ws_endpoint = ws_url();
        match WebSocket::open(&ws_endpoint) {
            Ok(ws) => {
                let (_, mut read) = ws.split();
                while let Some(msg) = read.next().await {
                    match msg {
                        Ok(Message::Text(text)) => {
                            if let Ok(alert) = serde_json::from_str::<GuardianAlert>(&text) {
                                on_alert(alert);
                            }
                        }
                        Ok(Message::Bytes(_)) => { /* ignore binary frames */ }
                        Err(e) => {
                            log::error!("WebSocket error: {e:?}");
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                log::error!("WebSocket connect failed: {e:?}");
            }
        }
    });
}
