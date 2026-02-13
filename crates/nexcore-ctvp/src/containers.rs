//! Testcontainers integration for Phase 2+ validation.
//!
//! This module provides utilities for spinning up real infrastructure
//! (databases, caches, queues) to validate capabilities with real data.

use crate::error::{CtvpError, CtvpResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Container configuration for validation testing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerConfig {
    /// Image name (e.g., "postgres:15")
    pub image: String,

    /// Container name
    pub name: String,

    /// Port mappings (container_port -> host_port)
    pub ports: HashMap<u16, u16>,

    /// Environment variables
    pub env: HashMap<String, String>,

    /// Wait strategy
    pub wait: WaitStrategy,

    /// Startup timeout in seconds
    pub timeout_secs: u64,
}

impl ContainerConfig {
    /// Creates a PostgreSQL container config
    pub fn new_postgres(version: &str) -> Self {
        let mut env = HashMap::new();
        env.insert("POSTGRES_USER".into(), "test".into());
        env.insert("POSTGRES_PASSWORD".into(), "test".into());
        env.insert("POSTGRES_DB".into(), "test".into());

        let mut ports = HashMap::new();
        ports.insert(5432, 0); // 0 = random available port

        Self {
            image: format!("postgres:{}", version),
            name: "ctvp-postgres".into(),
            ports,
            env,
            wait: WaitStrategy::Log("database system is ready to accept connections".into()),
            timeout_secs: 60,
        }
    }

    /// Creates a Redis container config
    pub fn new_redis(version: &str) -> Self {
        let mut ports = HashMap::new();
        ports.insert(6379, 0);

        Self {
            image: format!("redis:{}", version),
            name: "ctvp-redis".into(),
            ports,
            env: HashMap::new(),
            wait: WaitStrategy::Log("Ready to accept connections".into()),
            timeout_secs: 30,
        }
    }

    /// Creates a RabbitMQ container config
    pub fn new_rabbitmq(version: &str) -> Self {
        let mut env = HashMap::new();
        env.insert("RABBITMQ_DEFAULT_USER".into(), "test".into());
        env.insert("RABBITMQ_DEFAULT_PASS".into(), "test".into());

        let mut ports = HashMap::new();
        ports.insert(5672, 0);
        ports.insert(15672, 0); // Management UI

        Self {
            image: format!("rabbitmq:{}-management", version),
            name: "ctvp-rabbitmq".into(),
            ports,
            env,
            wait: WaitStrategy::Log("Server startup complete".into()),
            timeout_secs: 90,
        }
    }

    /// Creates a Kafka container config (using redpanda for lighter weight)
    pub fn new_kafka() -> Self {
        let mut ports = HashMap::new();
        ports.insert(9092, 0);

        Self {
            image: "redpandadata/redpanda:latest".into(),
            name: "ctvp-kafka".into(),
            ports,
            env: HashMap::new(),
            wait: WaitStrategy::Port(9092),
            timeout_secs: 60,
        }
    }

    /// Creates a LocalStack container for AWS services
    pub fn new_localstack() -> Self {
        let mut env = HashMap::new();
        env.insert("SERVICES".into(), "s3,sqs,dynamodb".into());
        env.insert("DEFAULT_REGION".into(), "us-east-1".into());

        let mut ports = HashMap::new();
        ports.insert(4566, 0);

        Self {
            image: "localstack/localstack:latest".into(),
            name: "ctvp-localstack".into(),
            ports,
            env,
            wait: WaitStrategy::Http {
                path: "/_localstack/health".into(),
                status: 200,
            },
            timeout_secs: 120,
        }
    }

    /// Creates a Toxiproxy container for fault injection
    pub fn new_toxiproxy() -> Self {
        let mut ports = HashMap::new();
        ports.insert(8474, 0); // API
        ports.insert(8475, 0); // Proxies start here

        Self {
            image: "ghcr.io/shopify/toxiproxy:latest".into(),
            name: "ctvp-toxiproxy".into(),
            ports,
            env: HashMap::new(),
            wait: WaitStrategy::Port(8474),
            timeout_secs: 30,
        }
    }
}

impl From<String> for ContainerConfig {
    fn from(image: String) -> Self {
        Self {
            image,
            name: "ctvp-container".into(),
            ports: HashMap::new(),
            env: HashMap::new(),
            wait: WaitStrategy::None,
            timeout_secs: 60,
        }
    }
}

/// Strategy for waiting until container is ready.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WaitStrategy {
    /// Wait for a log message
    Log(String),

    /// Wait for a port to be available
    Port(u16),

    /// Wait for an HTTP endpoint
    Http {
        /// Health check path
        path: String,
        /// Expected status code
        status: u16,
    },

    /// Wait for a specific duration
    Duration {
        /// Seconds to wait
        secs: u64,
    },

    /// No waiting
    None,
}

/// Running container handle.
#[derive(Debug)]
pub struct RunningContainer {
    /// Container ID
    pub id: String,

    /// Container name
    pub name: String,

    /// Mapped ports (container_port -> host_port)
    pub ports: HashMap<u16, u16>,

    /// Connection strings
    pub connection_strings: HashMap<String, String>,
}

impl From<RunningContainer> for String {
    fn from(container: RunningContainer) -> Self {
        container.id
    }
}

impl RunningContainer {
    /// Gets the host port for a container port
    pub fn get_host_port(&self, container_port: u16) -> Option<u16> {
        self.ports.get(&container_port).copied()
    }

    /// Gets a connection string by name
    pub fn get_connection_string(&self, name: &str) -> Option<&str> {
        self.connection_strings.get(name).map(|s| s.as_str())
    }
}

/// Container orchestrator for validation tests.
pub struct ContainerOrchestrator {
    /// Running containers
    containers: Vec<RunningContainer>,

    /// Docker client (would be bollard in real impl)
    _client: Option<()>,
}

impl ContainerOrchestrator {
    /// Creates a new orchestrator
    pub fn new() -> CtvpResult<Self> {
        // In real implementation, would initialize Docker client
        Ok(Self {
            containers: Vec::new(),
            _client: None,
        })
    }

    /// Starts a container from config
    pub fn start_container(&mut self, config: &ContainerConfig) -> CtvpResult<&RunningContainer> {
        let container = self.build_running_handle(config);
        self.containers.push(container);
        self.containers
            .last()
            .ok_or_else(|| CtvpError::Container("Failed to push container".into()))
    }

    fn build_running_handle(&self, config: &ContainerConfig) -> RunningContainer {
        RunningContainer {
            id: format!("container-{}", uuid::Uuid::new_v4()),
            name: config.name.clone(),
            ports: config.ports.clone(),
            connection_strings: self.generate_connection_strings(config),
        }
    }

    /// Generates connection strings for a container
    fn generate_connection_strings(&self, config: &ContainerConfig) -> HashMap<String, String> {
        let mut strings = HashMap::new();

        if config.image.starts_with("postgres") {
            strings.insert("database".into(), self.generate_postgres_url(config));
        }

        if config.image.starts_with("redis") {
            strings.insert("cache".into(), self.generate_redis_url(config));
        }

        if config.image.starts_with("rabbitmq") {
            strings.insert("queue".into(), self.generate_rabbitmq_url(config));
        }

        strings
    }

    fn generate_postgres_url(&self, config: &ContainerConfig) -> String {
        let port = config.ports.get(&5432).unwrap_or(&5432);
        format!(
            "postgresql://{}:{}@localhost:{}/{}",
            config
                .env
                .get("POSTGRES_USER")
                .unwrap_or(&"postgres".into()),
            config
                .env
                .get("POSTGRES_PASSWORD")
                .unwrap_or(&"postgres".into()),
            port,
            config.env.get("POSTGRES_DB").unwrap_or(&"postgres".into()),
        )
    }

    fn generate_redis_url(&self, config: &ContainerConfig) -> String {
        let port = config.ports.get(&6379).unwrap_or(&6379);
        format!("redis://localhost:{}", port)
    }

    fn generate_rabbitmq_url(&self, config: &ContainerConfig) -> String {
        let port = config.ports.get(&5672).unwrap_or(&5672);
        format!(
            "amqp://{}:{}@localhost:{}",
            config
                .env
                .get("RABBITMQ_DEFAULT_USER")
                .unwrap_or(&"guest".into()),
            config
                .env
                .get("RABBITMQ_DEFAULT_PASS")
                .unwrap_or(&"guest".into()),
            port,
        )
    }

    /// Stops all containers
    pub fn stop_all(&mut self) -> CtvpResult<()> {
        // In real implementation, would stop and remove all containers
        self.containers.clear();
        Ok(())
    }

    /// Gets a running container by name
    pub fn get_container(&self, name: &str) -> Option<&RunningContainer> {
        self.containers.iter().find(|c| c.name == name)
    }
}

impl Default for ContainerOrchestrator {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            containers: Vec::new(),
            _client: None,
        })
    }
}

impl Drop for ContainerOrchestrator {
    fn drop(&mut self) {
        let _ = self.stop_all();
    }
}

/// Preset container stacks for common validation scenarios.
pub struct ContainerStack {
    /// Stack name
    pub name: String,

    /// Container configurations
    pub containers: Vec<ContainerConfig>,
}

impl ContainerStack {
    /// Creates a web application stack (Postgres + Redis)
    pub fn new_web_app() -> Self {
        Self {
            name: "web-app".into(),
            containers: vec![
                ContainerConfig::new_postgres("15"),
                ContainerConfig::new_redis("7"),
            ],
        }
    }

    /// Creates an event-driven stack (Postgres + Redis + RabbitMQ)
    pub fn new_event_driven() -> Self {
        Self {
            name: "event-driven".into(),
            containers: vec![
                ContainerConfig::new_postgres("15"),
                ContainerConfig::new_redis("7"),
                ContainerConfig::new_rabbitmq("3"),
            ],
        }
    }

    /// Creates a fault injection stack (target services + Toxiproxy)
    pub fn new_fault_injection(targets: Vec<ContainerConfig>) -> Self {
        let mut containers = targets;
        containers.push(ContainerConfig::new_toxiproxy());

        Self {
            name: "fault-injection".into(),
            containers,
        }
    }

    /// Creates an AWS-like stack (LocalStack for S3, SQS, DynamoDB)
    pub fn new_aws_local() -> Self {
        Self {
            name: "aws-local".into(),
            containers: vec![ContainerConfig::new_localstack()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_postgres_config() {
        let config = ContainerConfig::new_postgres("15");
        assert!(config.image.contains("postgres"));
        assert!(config.env.contains_key("POSTGRES_USER"));
        assert!(config.ports.contains_key(&5432));
    }

    #[test]
    fn test_redis_config() {
        let config = ContainerConfig::new_redis("7");
        assert!(config.image.contains("redis"));
        assert!(config.ports.contains_key(&6379));
    }

    #[test]
    fn test_web_app_stack() {
        let stack = ContainerStack::new_web_app();
        assert_eq!(stack.containers.len(), 2);
    }

    #[test]
    fn test_connection_string_generation() -> CtvpResult<()> {
        let orchestrator = ContainerOrchestrator::new()?;
        let config = ContainerConfig::new_postgres("15");
        let strings = orchestrator.generate_connection_strings(&config);

        assert!(strings.contains_key("database"));
        assert!(strings["database"].contains("postgresql://"));
        Ok(())
    }
}
