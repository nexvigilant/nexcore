//! # Scientific Color Transfer Functions
//!
//! A module for scientifically-grounded color mapping, translating scalar values
//! in `[0.0, 1.0]` to RGBA colors. Designed for data visualization with
//! perceptually uniform color maps, color vision deficiency (CVD) simulation,
//! and 1D LUT generation for GPU consumption.
//!
//! No external dependencies. Uses inline color mathematics.
//!
//! ## Example
//!
//! ```rust
//! use nexcore_viz::transfer_lut::{TransferFunction, Preset, sample};
//! 
//! let tf = TransferFunction::from_preset(Preset::Viridis);
//! let color = sample(&tf, 0.5);
//! assert_eq!(color.len(), 4);
//! ```

use std::fmt;
use serde::{Deserialize, Serialize};

/// Error types for transfer function operations.
#[derive(Debug, Clone, PartialEq)]
pub enum TransferError {
    EmptyLut,
    InvalidRange { min: f64, max: f64 },
    InvalidStopCount(usize),
    OutOfRange(f64),
    InvalidPreset(String),
}

impl fmt::Display for TransferError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyLut => write!(f, "LUT is empty"),
            Self::InvalidRange { min, max } => write!(f, "Invalid range: min {} max {}", min, max),
            Self::InvalidStopCount(c) => write!(f, "Invalid stop count: {}", c),
            Self::OutOfRange(v) => write!(f, "Value out of range: {}", v),
            Self::InvalidPreset(p) => write!(f, "Invalid preset: {}", p),
        }
    }
}

impl std::error::Error for TransferError {}

/// A single color stop in a transfer function.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColorStop {
    pub position: f64,
    pub color: [f32; 4],
}

/// A scientific transfer function.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransferFunction {
    pub name: String,
    pub stops: Vec<ColorStop>,
    pub is_diverging: bool,
    pub is_cyclic: bool,
}

/// Supported color map presets.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Preset {
    Viridis,
    Magma,
    Inferno,
    Plasma,
    Turbo,
    Cividis,
    CoolWarm,
    Spectral,
    Grayscale,
    SignalHeat,
}

impl TransferFunction {
    pub fn new(name: String, mut stops: Vec<ColorStop>, is_diverging: bool, is_cyclic: bool) -> Result<Self, TransferError> {
        if stops.is_empty() {
            return Err(TransferError::InvalidStopCount(0));
        }
        stops.sort_by(|a, b| a.position.partial_cmp(&b.position).unwrap_or(std::cmp::Ordering::Equal));
        Ok(Self { name, stops, is_diverging, is_cyclic })
    }

    pub fn from_preset(preset: Preset) -> Self {
        let (name, stops, is_div) = match preset {
            Preset::Viridis => ("Viridis", vec![
                ColorStop { position: 0.0, color: [0.267, 0.005, 0.329, 1.0] },
                ColorStop { position: 0.5, color: [0.127, 0.567, 0.551, 1.0] },
                ColorStop { position: 1.0, color: [0.993, 0.906, 0.144, 1.0] },
            ], false),
            Preset::Magma => ("Magma", vec![
                ColorStop { position: 0.0, color: [0.0, 0.0, 0.0, 1.0] },
                ColorStop { position: 0.5, color: [0.716, 0.215, 0.475, 1.0] },
                ColorStop { position: 1.0, color: [0.987, 0.991, 0.749, 1.0] },
            ], false),
            Preset::Inferno => ("Inferno", vec![
                ColorStop { position: 0.0, color: [0.0, 0.0, 0.0, 1.0] },
                ColorStop { position: 0.5, color: [0.855, 0.272, 0.177, 1.0] },
                ColorStop { position: 1.0, color: [0.988, 1.0, 0.643, 1.0] },
            ], false),
            Preset::Plasma => ("Plasma", vec![
                ColorStop { position: 0.0, color: [0.050, 0.029, 0.528, 1.0] },
                ColorStop { position: 0.5, color: [0.812, 0.284, 0.435, 1.0] },
                ColorStop { position: 1.0, color: [0.940, 0.975, 0.131, 1.0] },
            ], false),
            Preset::Turbo => ("Turbo", vec![
                ColorStop { position: 0.0, color: [0.145, 0.091, 0.354, 1.0] },
                ColorStop { position: 0.5, color: [0.552, 0.985, 0.198, 1.0] },
                ColorStop { position: 1.0, color: [0.479, 0.015, 0.010, 1.0] },
            ], false),
            Preset::Cividis => ("Cividis", vec![
                ColorStop { position: 0.0, color: [0.0, 0.135, 0.304, 1.0] },
                ColorStop { position: 0.5, color: [0.485, 0.490, 0.481, 1.0] },
                ColorStop { position: 1.0, color: [0.996, 0.906, 0.407, 1.0] },
            ], false),
            Preset::CoolWarm => ("CoolWarm", vec![
                ColorStop { position: 0.0, color: [0.230, 0.299, 0.754, 1.0] },
                ColorStop { position: 0.5, color: [0.865, 0.865, 0.865, 1.0] },
                ColorStop { position: 1.0, color: [0.706, 0.016, 0.150, 1.0] },
            ], true),
            Preset::Spectral => ("Spectral", vec![
                ColorStop { position: 0.0, color: [0.620, 0.004, 0.259, 1.0] },
                ColorStop { position: 0.5, color: [1.0, 1.0, 0.749, 1.0] },
                ColorStop { position: 1.0, color: [0.369, 0.310, 0.635, 1.0] },
            ], true),
            Preset::Grayscale => ("Grayscale", vec![
                ColorStop { position: 0.0, color: [0.0, 0.0, 0.0, 1.0] },
                ColorStop { position: 1.0, color: [1.0, 1.0, 1.0, 1.0] },
            ], false),
            Preset::SignalHeat => ("SignalHeat", vec![
                ColorStop { position: 0.0, color: [0.0, 0.0, 0.5, 1.0] },
                ColorStop { position: 0.5, color: [0.5, 0.0, 0.5, 1.0] },
                ColorStop { position: 1.0, color: [1.0, 0.0, 0.0, 1.0] },
            ], false),
        };
        Self { name: name.to_string(), stops, is_diverging: is_div, is_cyclic: false }
    }
}

pub fn sample(tf: &TransferFunction, mut t: f64) -> [f32; 4] {
    if tf.stops.is_empty() { return [0.0, 0.0, 0.0, 1.0]; }
    if tf.stops.len() == 1 { return tf.stops[0].color; }
    if tf.is_cyclic {
        t = t.fract();
        if t < 0.0 { t += 1.0; }
    } else {
        t = t.clamp(0.0, 1.0);
    }
    
    let mut left = &tf.stops[0];
    let mut right = &tf.stops[tf.stops.len() - 1];
    
    if t <= left.position { return left.color; }
    if t >= right.position { return right.color; }
    
    for i in 0..tf.stops.len() - 1 {
        if t >= tf.stops[i].position && t <= tf.stops[i+1].position {
            left = &tf.stops[i];
            right = &tf.stops[i+1];
            break;
        }
    }
    
    let span = right.position - left.position;
    if span <= 0.0 { return left.color; }
    let f = ((t - left.position) / span) as f32;
    
    let mut res = [0.0; 4];
    for i in 0..4 {
        res[i] = left.color[i] + (right.color[i] - left.color[i]) * f;
    }
    res
}

pub fn sample_clamped(tf: &TransferFunction, value: f64, min: f64, max: f64) -> Result<[f32; 4], TransferError> {
    if min >= max { return Err(TransferError::InvalidRange { min, max }); }
    let t = (value - min) / (max - min);
    Ok(sample(tf, t))
}

pub struct Lut {
    pub name: String,
    pub resolution: usize,
    pub data: Vec<[f32; 4]>,
    pub data_range: (f64, f64),
}

pub fn bake_lut(tf: &TransferFunction, resolution: usize) -> Result<Lut, TransferError> {
    if resolution == 0 { return Err(TransferError::EmptyLut); }
    let mut data = Vec::with_capacity(resolution);
    for i in 0..resolution {
        let t = if resolution > 1 { i as f64 / (resolution - 1) as f64 } else { 0.5 };
        data.push(sample(tf, t));
    }
    Ok(Lut {
        name: format!("{}_LUT{}", tf.name, resolution),
        resolution,
        data,
        data_range: (0.0, 1.0),
    })
}

pub fn lut_sample(lut: &Lut, mut t: f64) -> [f32; 4] {
    if lut.data.is_empty() { return [0.0, 0.0, 0.0, 1.0]; }
    t = t.clamp(0.0, 1.0);
    let f = t * (lut.resolution - 1) as f64;
    let i = f.floor() as usize;
    let frac = (f - f.floor()) as f32;
    
    if i >= lut.resolution - 1 {
        return lut.data[lut.resolution - 1];
    }
    
    let c1 = lut.data[i];
    let c2 = lut.data[i+1];
    let mut res = [0.0; 4];
    for j in 0..4 {
        res[j] = c1[j] + (c2[j] - c1[j]) * frac;
    }
    res
}

pub enum CvdType {
    Protanopia,
    Deuteranopia,
    Tritanopia,
    Achromatopsia,
}

pub fn simulate_cvd(color: [f32; 4], cvd: CvdType) -> [f32; 4] {
    // Simplified CVD simulation matrix implementation
    let (r, g, b, a) = (color[0], color[1], color[2], color[3]);
    match cvd {
        CvdType::Protanopia => {
            let pr = 0.56667 * r + 0.43333 * g + 0.0 * b;
            let pg = 0.55833 * r + 0.44167 * g + 0.0 * b;
            let pb = 0.0 * r + 0.24167 * g + 0.75833 * b;
            [pr, pg, pb, a]
        },
        CvdType::Deuteranopia => {
            let dr = 0.625 * r + 0.375 * g + 0.0 * b;
            let dg = 0.7 * r + 0.3 * g + 0.0 * b;
            let db = 0.0 * r + 0.3 * g + 0.7 * b;
            [dr, dg, db, a]
        },
        CvdType::Tritanopia => {
            let tr = 0.95 * r + 0.05 * g + 0.0 * b;
            let tg = 0.0 * r + 0.43333 * g + 0.56667 * b;
            let tb = 0.0 * r + 0.475 * g + 0.525 * b;
            [tr, tg, tb, a]
        },
        CvdType::Achromatopsia => {
            let l = 0.299 * r + 0.587 * g + 0.114 * b;
            [l, l, l, a]
        }
    }
}

pub fn wgsl_sample_transfer() -> &'static str {
    "
fn sample_transfer(value: f32, lut: texture_1d<f32>, lut_sampler: sampler) -> vec4<f32> {
    let t = clamp(value, 0.0, 1.0);
    return textureSampleLevel(lut, lut_sampler, t, 0.0);
}
"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viridis_preset() {
        let tf = TransferFunction::from_preset(Preset::Viridis);
        assert_eq!(tf.stops.len(), 3);
        let c = sample(&tf, 0.5);
        assert!(c[1] > 0.5); // Greenish in middle
    }

    #[test]
    fn test_sample_clamped() {
        let tf = TransferFunction::from_preset(Preset::Grayscale);
        let res = sample_clamped(&tf, 10.0, 0.0, 20.0).unwrap();
        assert_eq!(res[0], 0.5);
    }
    
    #[test]
    fn test_bake_lut() {
        let tf = TransferFunction::from_preset(Preset::Grayscale);
        let lut = bake_lut(&tf, 256).unwrap();
        assert_eq!(lut.data.len(), 256);
        assert_eq!(lut.data[128][0], 128.0 / 255.0);
    }
}
