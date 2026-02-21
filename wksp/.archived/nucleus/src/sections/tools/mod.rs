//! Engineering Tools & Capability Accelerators
//!
//! Tools for automated code generation, AI-assisted debugging,
//! and performance optimization.

mod api_explorer;
mod brain_storage;
mod code_gen;
mod debug;
mod hub;
mod perf;
mod registry;
mod store;
mod visualizer;

pub use api_explorer::ApiExplorerPage;
pub use brain_storage::ArtifactManagerPage;
pub use code_gen::CodeGenPage;
pub use debug::DebugPage;
pub use hub::HubPage;
pub use perf::PerfPage;
pub use registry::RegistryHudPage;
pub use store::StorePage;
pub use visualizer::ArchVisualizerPage;
