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
    Device510k, DeviceClass, DeviceEvent, DevicePma, DeviceRecall, DeviceUdi, MdrDevice,
    MdrPatient, MdrText, UdiDeviceSize, UdiIdentifier, UdiSterilization,
};
pub use drug::{
    ActiveIngredient, DrugApplication, DrugEvent, DrugLabel, DrugNdc, DrugRecall, EventDrug,
    FdaApplicationDoc, FdaProduct, FdaSubmission, NdcPackaging, Patient, PrimarySource, Reaction,
};
pub use food::{FoodConsumer, FoodEvent, FoodOutcome, FoodProduct, FoodReaction, FoodRecall};
pub use substance::{Substance, SubstanceCode, SubstanceName};
