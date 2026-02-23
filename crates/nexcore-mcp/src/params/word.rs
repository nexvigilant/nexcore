//! Parameter types for binary word MCP tools.

use schemars::JsonSchema;
use serde::Deserialize;

/// Analyze a binary word: popcount, parity, leading/trailing zeros, alignment, log2.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct WordAnalyzeParams {
    /// The value to analyze (as unsigned integer).
    pub value: u64,
    /// Bit width: 8, 16, 32, or 64 (default: 64).
    pub width: Option<u8>,
}

/// Count set (1) bits in a binary word.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct WordPopcountParams {
    /// The value to count bits in.
    pub value: u64,
    /// Bit width: 8, 16, 32, or 64 (default: 64).
    pub width: Option<u8>,
}

/// Compute Hamming distance between two binary words.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct WordHammingDistanceParams {
    /// First value.
    pub a: u64,
    /// Second value.
    pub b: u64,
    /// Bit width: 8, 16, 32, or 64 (default: 64).
    pub width: Option<u8>,
}

/// Check parity (even/odd set bit count) of a binary word.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct WordParityParams {
    /// The value to check.
    pub value: u64,
    /// Bit width: 8, 16, 32, or 64 (default: 64).
    pub width: Option<u8>,
}

/// Rotate a binary word left or right.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct WordRotateParams {
    /// The value to rotate.
    pub value: u64,
    /// Number of bit positions to rotate.
    pub amount: u32,
    /// Direction: "left" or "right" (default: "left").
    pub direction: Option<String>,
    /// Bit width: 8, 16, 32, or 64 (default: 64).
    pub width: Option<u8>,
}

/// Compute floor(log2(value)) for a binary word.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct WordLog2Params {
    /// The value (must be > 0).
    pub value: u64,
    /// Bit width: 8, 16, 32, or 64 (default: 64).
    pub width: Option<u8>,
}

/// Compute integer square root via Newton's method.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct WordIsqrtParams {
    /// The value (must be > 0).
    pub value: u64,
    /// Bit width: 8, 16, 32, or 64 (default: 64).
    pub width: Option<u8>,
}

/// Compute binary GCD (Stein's algorithm) of two values.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct WordBinaryGcdParams {
    /// First value.
    pub a: u64,
    /// Second value.
    pub b: u64,
    /// Bit width: 8, 16, 32, or 64 (default: 64).
    pub width: Option<u8>,
}

/// Test whether a specific bit is set.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct WordBitTestParams {
    /// The value to test.
    pub value: u64,
    /// Bit position (0-indexed from LSB).
    pub position: u32,
    /// Bit width: 8, 16, 32, or 64 (default: 64).
    pub width: Option<u8>,
}

/// Align a value up to the next multiple of a power-of-two alignment.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct WordAlignUpParams {
    /// The value to align.
    pub value: u64,
    /// Alignment (must be a power of two).
    pub alignment: u32,
    /// Bit width: 8, 16, 32, or 64 (default: 64).
    pub width: Option<u8>,
}
