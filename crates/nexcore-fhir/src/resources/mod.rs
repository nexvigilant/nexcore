//! FHIR R4 resource definitions — PV-focused subset.

pub mod adverse_event;
pub mod bundle;
pub mod condition;
pub mod encounter;
pub mod medication;
pub mod medication_administration;
pub mod observation;
pub mod organization;
pub mod patient;
pub mod practitioner;

pub use adverse_event::*;
pub use bundle::*;
pub use condition::*;
pub use encounter::*;
pub use medication::*;
pub use medication_administration::*;
pub use observation::*;
pub use organization::*;
pub use patient::*;
pub use practitioner::*;
