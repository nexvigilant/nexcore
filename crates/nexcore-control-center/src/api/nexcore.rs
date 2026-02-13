//! nexcore status API client (Guardian, FRIDAY, Brain)

use serde::{Deserialize, Serialize};

/// Guardian status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardianStatus {
    pub status: String,
    pub iteration_count: u64,
    pub sensor_count: usize,
    pub actuator_count: usize,
    pub sensors: Vec<Sensor>,
    pub actuators: Vec<Actuator>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sensor {
    pub name: String,
    pub description: String,
    #[serde(rename = "type")]
    pub sensor_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Actuator {
    pub name: String,
    pub description: String,
    pub priority: u32,
}

/// FRIDAY status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FridayStatus {
    pub status: String,
    pub process: ProcessStatus,
    pub components: FridayComponents,
    pub sources: Vec<FridaySource>,
    pub executors: Vec<FridayExecutor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessStatus {
    pub name: String,
    pub running: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FridayComponents {
    pub event_bus: serde_json::Value,
    pub memory_layer: serde_json::Value,
    pub decision_engine: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FridaySource {
    pub name: String,
    pub description: String,
    #[serde(rename = "type")]
    pub source_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FridayExecutor {
    pub name: String,
    pub description: String,
    #[serde(rename = "type")]
    pub executor_type: String,
}

/// Brain sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainSessionsResponse {
    pub count: usize,
    pub sessions: Vec<BrainSession>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainSession {
    pub session_id: String,
    pub project: Option<String>,
    pub description: Option<String>,
    pub created_at: String,
}
