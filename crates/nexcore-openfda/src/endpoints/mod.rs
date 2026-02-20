//! OpenFDA endpoint functions, one module per FDA domain.

pub mod device;
pub mod drug;
pub mod food;
pub mod other;

// Flatten commonly-used functions.
pub use device::{
    fetch_device_510k, fetch_device_class, fetch_device_events, fetch_device_pma,
    fetch_device_recalls, fetch_device_udi,
};
pub use drug::{
    fetch_drug_events, fetch_drug_labels, fetch_drug_ndc, fetch_drug_recalls, fetch_drugs_at_fda,
};
pub use food::{fetch_food_events, fetch_food_recalls};
pub use other::fetch_substances;
