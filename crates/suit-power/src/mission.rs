//! # Mission Energy Planner
//! Plans energy consumption over a mission profile.

/// Forecast of energy usage for the mission.
pub struct MissionForecast {
    /// Forecasted load over the next hour (kW).
    pub load_forecast: Vec<f32>,
}
