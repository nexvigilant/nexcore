use async_trait::async_trait;

#[async_trait]
pub trait Source: Send + Sync {
    async fn run(&self) -> nexcore_error::Result<()>;
    fn name(&self) -> &'static str;
}

pub mod filesystem;

pub mod webhook;

pub mod voice;

pub mod git_monitor;

pub mod telemetry_monitor;

pub mod workflow;

pub mod browser;
