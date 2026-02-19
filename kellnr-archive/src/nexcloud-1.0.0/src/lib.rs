#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![doc = "NexCloud: Rust-native cloud platform — process supervisor + reverse proxy."]
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
