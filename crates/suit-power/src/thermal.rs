//! # Thermal Derating
pub fn get_derate_factor(temp: f32) -> f32 {
    if temp > 80.0 {
        0.5
    } else if temp > 60.0 {
        0.8
    } else {
        1.0
    }
}
