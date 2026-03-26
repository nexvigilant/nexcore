//! Counter-Awareness MCP tools
//!
//! Exposes the counter-awareness detection/counter-detection framework as MCP tools:
//! - Single-sensor detection probability
//! - Multi-sensor fused detection
//! - Optimal countermeasure loadout
//! - 8×8 effectiveness matrix query
//! - Sensor/countermeasure catalog

use crate::params::{
    CaCatalogParams, CaDetectParams, CaFusionParams, CaMatrixParams, CaOptimizeParams,
};
use counter_awareness::detection::compute_detection;
use counter_awareness::device::catalog;
use counter_awareness::fusion::{compute_fusion, optimize_loadout};
use counter_awareness::matrix::EffectivenessMatrix;
use counter_awareness::primitives::{CounterPrimitive, SensingPrimitive};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

/// Parse a list of counter-primitive names, returning an error message for unknowns.
fn parse_counters(names: &[String]) -> Result<Vec<CounterPrimitive>, nexcore_error::NexError> {
    let mut counters = Vec::with_capacity(names.len());
    for name in names {
        match CounterPrimitive::from_name(name) {
            Some(cp) => counters.push(cp),
            None => {
                return Err(nexcore_error::nexerror!(
                    "Unknown counter-primitive: '{name}'. Valid: absorption, thermal_equilibrium, \
                 homogenization, diffusion, attenuation, band_denial, range_denial, sub_resolution"
                ));
            }
        }
    }
    Ok(counters)
}

/// Compute single-sensor detection probability.
pub fn detect(params: CaDetectParams) -> Result<CallToolResult, McpError> {
    let sensor = match catalog::lookup_sensor(&params.sensor) {
        Some(s) => s,
        None => {
            let names = catalog::sensor_names().join(", ");
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Unknown sensor: '{}'. Available: {names}",
                params.sensor
            ))]));
        }
    };

    let counters = match parse_counters(&params.counters) {
        Ok(c) => c,
        Err(msg) => return Ok(CallToolResult::error(vec![Content::text(msg)])),
    };

    let matrix = EffectivenessMatrix::default_physics();
    let result = compute_detection(
        &sensor,
        &counters,
        &matrix,
        params.range_m,
        params.raw_signature,
    );

    let contributions: Vec<serde_json::Value> = result
        .primitive_contributions
        .iter()
        .map(|pc| {
            json!({
                "primitive": format!("{:?}", pc.primitive),
                "raw": pc.raw,
                "residual": pc.residual,
                "reduction": pc.reduction,
            })
        })
        .collect();

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&json!({
            "sensor": result.sensor_name,
            "raw_signature": result.raw_signature,
            "residual_signature": result.residual_signature,
            "range_factor": result.range_factor,
            "detection_probability": result.detection_probability,
            "counters_applied": params.counters,
            "range_m": params.range_m,
            "primitive_contributions": contributions,
        }))
        .unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// Compute multi-sensor fused detection probability.
pub fn fusion(params: CaFusionParams) -> Result<CallToolResult, McpError> {
    let mut sensors = Vec::with_capacity(params.sensors.len());
    for name in &params.sensors {
        match catalog::lookup_sensor(name) {
            Some(s) => sensors.push(s),
            None => {
                let names = catalog::sensor_names().join(", ");
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Unknown sensor: '{name}'. Available: {names}"
                ))]));
            }
        }
    }

    let counters = match parse_counters(&params.counters) {
        Ok(c) => c,
        Err(msg) => return Ok(CallToolResult::error(vec![Content::text(msg)])),
    };

    let threshold = params.threshold.unwrap_or(0.5);
    let matrix = EffectivenessMatrix::default_physics();
    let result = compute_fusion(
        &sensors,
        &counters,
        &matrix,
        params.range_m,
        params.raw_signature,
        threshold,
    );

    let per_sensor: Vec<serde_json::Value> = result
        .sensor_assessments
        .iter()
        .map(|a| {
            json!({
                "sensor": a.sensor_name,
                "detection_probability": a.detection_probability,
                "residual_signature": a.residual_signature,
                "range_factor": a.range_factor,
            })
        })
        .collect();

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&json!({
            "fused_probability": result.fused_probability,
            "detected": result.detected,
            "threshold": result.threshold,
            "sensor_count": sensors.len(),
            "counters_applied": params.counters,
            "range_m": params.range_m,
            "raw_signature": params.raw_signature,
            "per_sensor": per_sensor,
        }))
        .unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// Find optimal countermeasure loadout under weight budget.
pub fn optimize(params: CaOptimizeParams) -> Result<CallToolResult, McpError> {
    let mut sensors = Vec::with_capacity(params.sensors.len());
    for name in &params.sensors {
        match catalog::lookup_sensor(name) {
            Some(s) => sensors.push(s),
            None => {
                let names = catalog::sensor_names().join(", ");
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Unknown sensor: '{name}'. Available: {names}"
                ))]));
            }
        }
    }

    let mut countermeasures = Vec::with_capacity(params.countermeasures.len());
    for name in &params.countermeasures {
        match catalog::lookup_countermeasure(name) {
            Some(cm) => countermeasures.push(cm),
            None => {
                let names = catalog::countermeasure_names().join(", ");
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Unknown countermeasure: '{name}'. Available: {names}"
                ))]));
            }
        }
    }

    let matrix = EffectivenessMatrix::default_physics();
    let result = optimize_loadout(
        &sensors,
        &countermeasures,
        &matrix,
        params.weight_budget_kg,
        params.range_m,
        params.raw_signature,
    );

    let selected_names: Vec<&str> = result
        .selected
        .iter()
        .filter_map(|&i| params.countermeasures.get(i).map(|s| s.as_str()))
        .collect();

    let active_counters: Vec<String> = result
        .active_counters
        .iter()
        .map(|c| format!("{c:?}"))
        .collect();

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&json!({
            "selected_countermeasures": selected_names,
            "total_weight_kg": result.total_weight_kg,
            "total_power_w": result.total_power_w,
            "fused_probability": result.fused_probability,
            "active_counter_primitives": active_counters,
            "weight_budget_kg": params.weight_budget_kg,
            "evaluated_combinations": 2_u32.pow(countermeasures.len() as u32),
        }))
        .unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// Query the 8×8 effectiveness matrix.
pub fn matrix(params: CaMatrixParams) -> Result<CallToolResult, McpError> {
    let m = EffectivenessMatrix::default_physics();

    let sensing_names: Vec<&str> = vec![
        "Reflection",
        "Emission",
        "Contrast",
        "Boundary",
        "Intensity",
        "Frequency",
        "Distance",
        "Resolution",
    ];
    let counter_names: Vec<&str> = vec![
        "Absorption",
        "ThermalEquilibrium",
        "Homogenization",
        "Diffusion",
        "Attenuation",
        "BandDenial",
        "RangeDenial",
        "SubResolution",
    ];

    // Single cell query
    if let (Some(s_name), Some(c_name)) = (&params.sensing, &params.counter) {
        let sp = match SensingPrimitive::from_name(s_name) {
            Some(sp) => sp,
            None => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Unknown sensing primitive: '{s_name}'. Valid: {}",
                    sensing_names.join(", ").to_lowercase()
                ))]));
            }
        };
        let cp = match CounterPrimitive::from_name(c_name) {
            Some(cp) => cp,
            None => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Unknown counter-primitive: '{c_name}'. Valid: {}",
                    counter_names.join(", ").to_lowercase()
                ))]));
            }
        };
        let eff = m.get(sp, cp);
        return Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&json!({
                "sensing": s_name,
                "counter": c_name,
                "effectiveness": eff,
                "interpretation": format!(
                    "{c_name} is {:.0}% effective against {s_name} detection",
                    eff * 100.0
                ),
            }))
            .unwrap_or_else(|_| "{}".to_string()),
        )]));
    }

    // Single row query
    if let Some(ref s_name) = params.sensing {
        let sp = match SensingPrimitive::from_name(s_name) {
            Some(sp) => sp,
            None => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Unknown sensing primitive: '{s_name}'"
                ))]));
            }
        };
        let row = m.row(sp);
        let entries: Vec<serde_json::Value> = counter_names
            .iter()
            .zip(row.iter())
            .map(|(name, &val)| json!({"counter": name, "effectiveness": val}))
            .collect();

        return Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&json!({
                "sensing": s_name,
                "counters": entries,
            }))
            .unwrap_or_else(|_| "{}".to_string()),
        )]));
    }

    // Single column query
    if let Some(ref c_name) = params.counter {
        let cp = match CounterPrimitive::from_name(c_name) {
            Some(cp) => cp,
            None => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Unknown counter-primitive: '{c_name}'"
                ))]));
            }
        };
        let col = m.column(cp);
        let entries: Vec<serde_json::Value> = sensing_names
            .iter()
            .zip(col.iter())
            .map(|(name, &val)| json!({"sensing": name, "effectiveness": val}))
            .collect();

        return Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&json!({
                "counter": c_name,
                "against": entries,
            }))
            .unwrap_or_else(|_| "{}".to_string()),
        )]));
    }

    // Full matrix
    let mut rows = Vec::with_capacity(8);
    for (si, sp) in SensingPrimitive::all().iter().enumerate() {
        let row = m.row(*sp);
        let entries: serde_json::Map<String, serde_json::Value> = counter_names
            .iter()
            .zip(row.iter())
            .map(|(name, &val)| (name.to_string(), json!(val)))
            .collect();
        rows.push(json!({
            "sensing": sensing_names[si],
            "counters": entries,
        }));
    }

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&json!({
            "matrix_8x8": rows,
            "row_labels": sensing_names,
            "col_labels": counter_names,
            "note": "M[sensing][counter] = effectiveness of counter against sensing. Diagonal = primary counters.",
        }))
        .unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// List available sensors and countermeasures.
pub fn catalog_list(params: CaCatalogParams) -> Result<CallToolResult, McpError> {
    let category = params.category.as_deref().unwrap_or("all");

    let show_sensors = category == "all" || category == "sensors";
    let show_cms = category == "all" || category == "countermeasures";

    let mut result = serde_json::Map::new();

    if show_sensors {
        let sensors: Vec<serde_json::Value> = catalog::sensor_names()
            .iter()
            .filter_map(|name| {
                catalog::lookup_sensor(name).map(|s| {
                    let prims: Vec<String> = s
                        .primary_primitives
                        .iter()
                        .map(|p| format!("{p:?}"))
                        .collect();
                    json!({
                        "name": name,
                        "full_name": s.name,
                        "energy_mode": format!("{:?}", s.energy_mode),
                        "spectral_band": format!("{:?}", s.spectral_band),
                        "max_range_m": s.max_range_m,
                        "noise_floor": s.noise_floor,
                        "primary_primitives": prims,
                    })
                })
            })
            .collect();
        result.insert("sensors".into(), json!(sensors));
    }

    if show_cms {
        let cms: Vec<serde_json::Value> = catalog::countermeasure_names()
            .iter()
            .filter_map(|name| {
                catalog::lookup_countermeasure(name).map(|cm| {
                    let counters: Vec<String> = cm
                        .primary_counters
                        .iter()
                        .map(|c| format!("{c:?}"))
                        .collect();
                    json!({
                        "name": name,
                        "full_name": cm.name,
                        "energy_mode": format!("{:?}", cm.energy_mode),
                        "weight_kg": cm.weight_kg,
                        "power_w": cm.power_w,
                        "primary_counters": counters,
                        "effectiveness": cm.effectiveness,
                    })
                })
            })
            .collect();
        result.insert("countermeasures".into(), json!(cms));
    }

    result.insert("primitives".into(), json!({
        "sensing": ["reflection", "emission", "contrast", "boundary", "intensity", "frequency", "distance", "resolution"],
        "counter": ["absorption", "thermal_equilibrium", "homogenization", "diffusion", "attenuation", "band_denial", "range_denial", "sub_resolution"],
    }));

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&json!(result)).unwrap_or_else(|_| "{}".to_string()),
    )]))
}
