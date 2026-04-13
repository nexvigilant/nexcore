//! Causality assessment algorithms: Naranjo and WHO-UMC.

pub mod naranjo;
pub mod who_umc;

pub use naranjo::{
    NaranjoInput, NaranjoResult, NaranjoScore, calculate_naranjo, calculate_naranjo_quick,
};
pub use who_umc::{WhoUmcCategory, WhoUmcResult, calculate_who_umc_quick};
