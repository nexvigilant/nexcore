#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]
#![doc = "NexCloud: Rust-native cloud platform — 35-type taxonomy + process supervisor + reverse proxy."]
#![doc = ""]
#![doc = "Tier: T3 (full domain platform)"]
#![doc = "Primitives: σ Sequence + ς State + μ Mapping + ∂ Boundary + ρ Recursion + ν Frequency + π Persistence"]

pub mod deploy;
pub mod error;
pub mod ethics;
pub mod events;
pub mod foundations;
pub mod manifest;
pub mod process;
pub mod proxy;
pub mod status;
pub mod supervisor;

// Cloud taxonomy (35 types grounded to Lex Primitiva)
pub mod composites;
pub mod grounding;
pub mod prelude;
pub mod primitives;
pub mod service_models;
pub mod transfer;

pub use deploy::{DeployPipeline, DeployTarget};
pub use error::{NexCloudError, Result};
pub use ethics::{EthicalAudit, OperatorRight, Prohibition, Virtue};
pub use events::{CloudEvent, EventBus};
pub use foundations::{
    ComputingParadigm, EngineeringPrinciple, GroundingRecord, ScientificDiscipline, Standard,
};
pub use manifest::CloudManifest;
pub use status::CloudStatus;
pub use supervisor::CloudSupervisor;

pub use composites::*;
pub use primitives::*;
pub use service_models::*;
