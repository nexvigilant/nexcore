#![forbid(unsafe_code)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]

//! # nexcore-pv-wasm
//!
//! WASM bindings for NexVigilant PV signal detection math.
//! Wraps `nexcore-pv-math` with `wasm-bindgen` exports for browser consumption.
//!
//! ## Usage (JavaScript/TypeScript)
//!
//! ```js
//! import init, { compute_prr, compute_ror, compute_ic, compute_ebgm } from 'nexcore-pv-wasm';
//!
//! await init();
//! const prr = compute_prr(285, 12450, 8920, 1845000);
//! console.log(prr); // { value: 4.651, signal: true, ci_lower: 4.139, ci_upper: 5.227 }
//! ```

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use nexcore_pv_math::{SignalCriteria, TwoByTwoTable};
use serde::Serialize;
use wasm_bindgen::prelude::*;

/// Signal detection result returned to JavaScript.
#[derive(Serialize)]
struct JsSignalResult {
    value: f64,
    signal: bool,
    ci_lower: f64,
    ci_upper: f64,
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

/// Create an error result visible to JS callers (fix #4: no silent NULL returns).
fn js_error(method: &str, msg: &str) -> JsValue {
    let result = JsSignalResult {
        value: f64::NAN,
        signal: false,
        ci_lower: f64::NAN,
        ci_upper: f64::NAN,
        method: method.to_string(),
        error: Some(msg.to_string()),
    };
    serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
}

/// Compute PRR from a 2x2 table.
#[wasm_bindgen]
pub fn compute_prr(a: u64, b: u64, c: u64, d: u64) -> JsValue {
    let table = TwoByTwoTable::new(a, b, c, d);
    let criteria = SignalCriteria::evans();
    match nexcore_pv_math::calculate_prr(&table, &criteria) {
        Ok(r) => {
            let result = JsSignalResult {
                value: r.point_estimate,
                signal: r.is_signal,
                ci_lower: r.lower_ci,
                ci_upper: r.upper_ci,
                method: "PRR".to_string(),
                error: None,
            };
            serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
        }
        Err(e) => js_error("PRR", &e.to_string()),
    }
}

/// Compute ROR from a 2x2 table.
#[wasm_bindgen]
pub fn compute_ror(a: u64, b: u64, c: u64, d: u64) -> JsValue {
    let table = TwoByTwoTable::new(a, b, c, d);
    let criteria = SignalCriteria::evans();
    match nexcore_pv_math::calculate_ror(&table, &criteria) {
        Ok(r) => {
            let result = JsSignalResult {
                value: r.point_estimate,
                signal: r.is_signal,
                ci_lower: r.lower_ci,
                ci_upper: r.upper_ci,
                method: "ROR".to_string(),
                error: None,
            };
            serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
        }
        Err(e) => js_error("ROR", &e.to_string()),
    }
}

/// Compute IC (Information Component) from a 2x2 table.
#[wasm_bindgen]
pub fn compute_ic(a: u64, b: u64, c: u64, d: u64) -> JsValue {
    let table = TwoByTwoTable::new(a, b, c, d);
    let criteria = SignalCriteria::evans();
    match nexcore_pv_math::calculate_ic(&table, &criteria) {
        Ok(r) => {
            let result = JsSignalResult {
                value: r.point_estimate,
                signal: r.is_signal,
                ci_lower: r.lower_ci,
                ci_upper: r.upper_ci,
                method: "IC".to_string(),
                error: None,
            };
            serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
        }
        Err(e) => js_error("IC", &e.to_string()),
    }
}

/// Compute EBGM from a 2x2 table.
#[wasm_bindgen]
pub fn compute_ebgm(a: u64, b: u64, c: u64, d: u64) -> JsValue {
    let table = TwoByTwoTable::new(a, b, c, d);
    let criteria = SignalCriteria::evans();
    match nexcore_pv_math::calculate_ebgm(&table, &criteria) {
        Ok(r) => {
            let result = JsSignalResult {
                value: r.point_estimate,
                signal: r.is_signal,
                ci_lower: r.lower_ci,
                ci_upper: r.upper_ci,
                method: "EBGM".to_string(),
                error: None,
            };
            serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
        }
        Err(e) => js_error("EBGM", &e.to_string()),
    }
}

/// Naranjo causality quick score result.
#[derive(Serialize)]
struct JsNaranjoResult {
    score: i32,
    category: String,
}

/// Quick Naranjo causality assessment.
/// Args: temporal, dechallenge, rechallenge, alternatives, previous (1=yes, 0=unknown, -1=no).
#[wasm_bindgen]
pub fn compute_naranjo(
    temporal: i32,
    dechallenge: i32,
    rechallenge: i32,
    alternatives: i32,
    previous: i32,
) -> JsValue {
    let result = nexcore_pv_math::calculate_naranjo_quick(
        temporal,
        dechallenge,
        rechallenge,
        alternatives,
        previous,
    );
    let js = JsNaranjoResult {
        score: result.score,
        category: format!("{:?}", result.category),
    };
    serde_wasm_bindgen::to_value(&js).unwrap_or(JsValue::NULL)
}

/// WHO-UMC causality quick result.
#[derive(Serialize)]
struct JsWhoUmcResult {
    category: String,
    description: String,
}

/// Quick WHO-UMC causality classification.
/// Args: temporal, dechallenge, rechallenge, alternatives, plausibility (1/0/-1).
#[wasm_bindgen]
pub fn compute_who_umc(
    temporal: i32,
    dechallenge: i32,
    rechallenge: i32,
    alternatives: i32,
    plausibility: i32,
) -> JsValue {
    let result = nexcore_pv_math::calculate_who_umc_quick(
        temporal,
        dechallenge,
        rechallenge,
        alternatives,
        plausibility,
    );
    let js = JsWhoUmcResult {
        category: format!("{:?}", result.category),
        description: result.description.to_string(),
    };
    serde_wasm_bindgen::to_value(&js).unwrap_or(JsValue::NULL)
}
