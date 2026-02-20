//! Response types for all openFDA API endpoints.
//!
//! All types derive `Debug`, `Clone`, `Serialize`, and `Deserialize`.
//! Fields use `#[serde(default)]` so missing JSON keys map to sensible defaults
//! rather than deserialization errors.

pub mod common;
pub mod device;
pub mod drug;
pub mod food;
pub mod substance;

// Flatten the most-used types for ergonomic imports.
pub use common::{OpenFdaEnrichment, OpenFdaMeta, OpenFdaResponse, ResultsMeta};
pub use device::{
    Device510k, DeviceClass, DeviceEvent, DevicePma, DeviceRecall, DeviceUdi, MdrDevice, MdrText,
    UdiIdentifier,
};
pub use drug::{
    ActiveIngredient, DrugApplication, DrugEvent, DrugLabel, DrugNdc, DrugRecall, EventDrug,
    FdaProduct, FdaSubmission, Patient, PrimarySource, Reaction,
};
pub use food::{FoodConsumer, FoodEvent, FoodProduct, FoodRecall, FoodReaction};
pub use substance::{Substance, SubstanceCode, SubstanceName};
