//! Binary word MCP tools.
//!
//! Width-dispatched wrappers over `nexcore-word` trait algebra:
//! PopulationOps, PositionOps, ArithmeticOps, RotationOps, ManipulationOps.

use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;
use serde_json::json;

use crate::params::word::{
    WordAlignUpParams, WordAnalyzeParams, WordBinaryGcdParams, WordBitTestParams,
    WordHammingDistanceParams, WordIsqrtParams, WordLog2Params, WordParityParams,
    WordPopcountParams, WordRotateParams,
};

// ── Helpers ──────────────────────────────────────────────────────────────

fn ok_json(value: serde_json::Value) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&value).unwrap_or_else(|_| format!("{value}")),
    )]))
}

fn err_result(msg: &str) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::error(vec![rmcp::model::Content::text(
        msg.to_string(),
    )]))
}

/// Dispatch macro: creates a Word of the correct width, runs an expression.
/// Returns u64 from the result.
macro_rules! width_dispatch {
    ($val:expr, $width:expr, |$w:ident| $body:expr) => {{
        match $width {
            8 => {
                let $w = nexcore_word::prelude::Word8::new($val as u8);
                $body
            }
            16 => {
                let $w = nexcore_word::prelude::Word16::new($val as u16);
                $body
            }
            32 => {
                let $w = nexcore_word::prelude::Word32::new($val as u32);
                $body
            }
            64 => {
                let $w = nexcore_word::prelude::Word64::new($val);
                $body
            }
            _ => return err_result("width must be 8, 16, 32, or 64"),
        }
    }};
}

/// Two-value dispatch macro for operations needing two words of the same width.
macro_rules! width_dispatch2 {
    ($a:expr, $b:expr, $width:expr, |$wa:ident, $wb:ident| $body:expr) => {{
        match $width {
            8 => {
                let $wa = nexcore_word::prelude::Word8::new($a as u8);
                let $wb = nexcore_word::prelude::Word8::new($b as u8);
                $body
            }
            16 => {
                let $wa = nexcore_word::prelude::Word16::new($a as u16);
                let $wb = nexcore_word::prelude::Word16::new($b as u16);
                $body
            }
            32 => {
                let $wa = nexcore_word::prelude::Word32::new($a as u32);
                let $wb = nexcore_word::prelude::Word32::new($b as u32);
                $body
            }
            64 => {
                let $wa = nexcore_word::prelude::Word64::new($a);
                let $wb = nexcore_word::prelude::Word64::new($b);
                $body
            }
            _ => return err_result("width must be 8, 16, 32, or 64"),
        }
    }};
}

fn resolve_width(w: Option<u8>) -> u8 {
    w.unwrap_or(64)
}

// ── Tools ────────────────────────────────────────────────────────────────

/// Comprehensive binary word analysis.
pub fn word_analyze(p: WordAnalyzeParams) -> Result<CallToolResult, McpError> {
    use nexcore_word::prelude::*;

    let w = resolve_width(p.width);
    width_dispatch!(p.value, w, |word| {
        let popcount = word.popcount().value();
        let zero_count = word.zero_count().value();
        let parity = format!("{}", word.parity());
        let leading_zeros = word.leading_zeros().value();
        let trailing_zeros = word.trailing_zeros().value();
        let is_power_of_two = word.is_power_of_two();
        let log2 = word.log2().ok().map(|b| b.value());
        let alignment = format!("{}", word.check_alignment());
        let binary = format!("{word:b}");

        ok_json(json!({
            "value": p.value,
            "width": w,
            "binary": binary,
            "popcount": popcount,
            "zero_count": zero_count,
            "parity": parity,
            "leading_zeros": leading_zeros,
            "trailing_zeros": trailing_zeros,
            "is_power_of_two": is_power_of_two,
            "log2": log2,
            "alignment": alignment,
        }))
    })
}

/// Count set bits (population count).
pub fn word_popcount(p: WordPopcountParams) -> Result<CallToolResult, McpError> {
    use nexcore_word::prelude::*;

    let w = resolve_width(p.width);
    width_dispatch!(p.value, w, |word| {
        ok_json(json!({
            "value": p.value,
            "width": w,
            "popcount": word.popcount().value(),
        }))
    })
}

/// Hamming distance between two binary words.
pub fn word_hamming_distance(p: WordHammingDistanceParams) -> Result<CallToolResult, McpError> {
    use nexcore_word::prelude::*;

    let w = resolve_width(p.width);
    width_dispatch2!(p.a, p.b, w, |wa, wb| {
        ok_json(json!({
            "a": p.a,
            "b": p.b,
            "width": w,
            "hamming_distance": wa.hamming_distance(&wb).value(),
        }))
    })
}

/// Parity check (even/odd set bit count).
pub fn word_parity(p: WordParityParams) -> Result<CallToolResult, McpError> {
    use nexcore_word::prelude::*;

    let w = resolve_width(p.width);
    width_dispatch!(p.value, w, |word| {
        let parity = word.parity();
        ok_json(json!({
            "value": p.value,
            "width": w,
            "parity": format!("{parity}"),
            "is_even": parity.is_even(),
        }))
    })
}

/// Rotate a binary word left or right.
pub fn word_rotate(p: WordRotateParams) -> Result<CallToolResult, McpError> {
    use nexcore_word::prelude::*;

    let w = resolve_width(p.width);
    let dir = p.direction.as_deref().unwrap_or("left");

    width_dispatch!(p.value, w, |word| {
        let result_raw = match dir {
            "left" | "l" => word.rotate_left(p.amount).raw().to_u64(),
            "right" | "r" => word.rotate_right(p.amount).raw().to_u64(),
            _ => return err_result("direction must be 'left' or 'right'"),
        };
        ok_json(json!({
            "value": p.value,
            "width": w,
            "direction": dir,
            "amount": p.amount,
            "result": result_raw,
            "result_binary": format!("{:0width$b}", result_raw, width = w as usize),
        }))
    })
}

/// Floor log base 2.
pub fn word_log2(p: WordLog2Params) -> Result<CallToolResult, McpError> {
    use nexcore_word::prelude::*;

    let w = resolve_width(p.width);
    width_dispatch!(p.value, w, |word| {
        match word.log2() {
            Ok(bc) => ok_json(json!({
                "value": p.value,
                "width": w,
                "log2": bc.value(),
            })),
            Err(e) => err_result(&format!("{e}")),
        }
    })
}

/// Integer square root (Newton's method).
pub fn word_isqrt(p: WordIsqrtParams) -> Result<CallToolResult, McpError> {
    use nexcore_word::prelude::*;

    let w = resolve_width(p.width);
    width_dispatch!(p.value, w, |word| {
        match word.isqrt() {
            Ok(r) => ok_json(json!({
                "value": p.value,
                "width": w,
                "isqrt": r.raw().to_u64(),
            })),
            Err(e) => err_result(&format!("{e}")),
        }
    })
}

/// Binary GCD (Stein's algorithm).
pub fn word_binary_gcd(p: WordBinaryGcdParams) -> Result<CallToolResult, McpError> {
    use nexcore_word::prelude::*;

    let w = resolve_width(p.width);
    width_dispatch2!(p.a, p.b, w, |wa, wb| {
        match wa.binary_gcd(&wb) {
            Ok(r) => ok_json(json!({
                "a": p.a,
                "b": p.b,
                "width": w,
                "gcd": r.raw().to_u64(),
            })),
            Err(e) => err_result(&format!("{e}")),
        }
    })
}

/// Test a specific bit position.
pub fn word_bit_test(p: WordBitTestParams) -> Result<CallToolResult, McpError> {
    use nexcore_word::prelude::*;

    let w = resolve_width(p.width);
    if p.position >= w as u32 {
        return err_result(&format!("position {} exceeds width {} bits", p.position, w));
    }

    width_dispatch!(p.value, w, |word| {
        match word.test_bit(p.position) {
            Ok(set) => ok_json(json!({
                "value": p.value,
                "width": w,
                "position": p.position,
                "is_set": set,
            })),
            Err(e) => err_result(&format!("{e}")),
        }
    })
}

/// Align value up to next power-of-two boundary.
pub fn word_align_up(p: WordAlignUpParams) -> Result<CallToolResult, McpError> {
    use nexcore_word::prelude::*;

    let w = resolve_width(p.width);
    width_dispatch!(p.value, w, |word| {
        match word.align_up(p.alignment) {
            Ok(r) => ok_json(json!({
                "value": p.value,
                "alignment": p.alignment,
                "width": w,
                "aligned_value": r.raw().to_u64(),
            })),
            Err(e) => err_result(&format!("{e}")),
        }
    })
}
