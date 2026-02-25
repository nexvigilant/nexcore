//! Game theory tools (normal-form games + forge pipeline)
//!
//! Provides equilibrium analysis for 2×2 and N×M payoff matrices,
//! forge quality scoring, and game-theory-driven Rust code generation.

use crate::params::{
    ForgeCodeGenerateParams, ForgeNashSolveParams, ForgePayoffMatrixParams,
    ForgeQualityScoreParams, GameTheoryNash2x2Params,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

const EPS: f64 = 1.0e-9;

fn to_matrix_2x2(matrix: &[Vec<f64>]) -> Option<[[f64; 2]; 2]> {
    if matrix.len() != 2 {
        return None;
    }
    if matrix[0].len() != 2 || matrix[1].len() != 2 {
        return None;
    }
    Some([[matrix[0][0], matrix[0][1]], [matrix[1][0], matrix[1][1]]])
}

/// Compute Nash equilibria for a 2x2 game matrix.
pub fn nash_2x2(params: GameTheoryNash2x2Params) -> Result<CallToolResult, McpError> {
    let row = match to_matrix_2x2(&params.row_payoffs) {
        Some(m) => m,
        None => {
            let err = json!({
                "error": "row_payoffs must be a 2x2 matrix",
                "row_payoffs": params.row_payoffs,
            });
            return Ok(CallToolResult::success(vec![Content::text(
                err.to_string(),
            )]));
        }
    };
    let col = match to_matrix_2x2(&params.col_payoffs) {
        Some(m) => m,
        None => {
            let err = json!({
                "error": "col_payoffs must be a 2x2 matrix",
                "col_payoffs": params.col_payoffs,
            });
            return Ok(CallToolResult::success(vec![Content::text(
                err.to_string(),
            )]));
        }
    };

    let mut pure = Vec::new();
    for i in 0..2 {
        for j in 0..2 {
            let row_best = row[i][j] + EPS >= row[1 - i][j];
            let col_best = col[i][j] + EPS >= col[i][1 - j];
            if row_best && col_best {
                pure.push(json!({
                    "row_strategy": if i == 0 { "R1" } else { "R2" },
                    "col_strategy": if j == 0 { "C1" } else { "C2" },
                    "payoffs": { "row": row[i][j], "col": col[i][j] }
                }));
            }
        }
    }

    let mut warnings = Vec::new();
    let a = row[0][0];
    let b = row[0][1];
    let c = row[1][0];
    let d = row[1][1];

    let e = col[0][0];
    let f = col[0][1];
    let g = col[1][0];
    let h = col[1][1];

    let denom_row = a - b - c + d;
    let denom_col = e - f - g + h;

    let mut mixed = None;
    if denom_row.abs() < EPS {
        warnings.push(
            "Row player indifferent denominator near zero; mixed strategy may be undefined"
                .to_string(),
        );
    }
    if denom_col.abs() < EPS {
        warnings.push(
            "Column player indifferent denominator near zero; mixed strategy may be undefined"
                .to_string(),
        );
    }

    if denom_row.abs() >= EPS && denom_col.abs() >= EPS {
        let q = (d - b) / denom_row; // Column plays C1 with prob q
        let p = (h - g) / denom_col; // Row plays R1 with prob p

        if (0.0 - EPS..=1.0 + EPS).contains(&p) && (0.0 - EPS..=1.0 + EPS).contains(&q) {
            let p_clamped = p.clamp(0.0, 1.0);
            let q_clamped = q.clamp(0.0, 1.0);
            let row_value = a * q_clamped + b * (1.0 - q_clamped);
            let col_value = e * p_clamped + g * (1.0 - p_clamped);
            mixed = Some(json!({
                "row_mixed_p": p_clamped,
                "col_mixed_q": q_clamped,
                "expected_payoff": { "row": row_value, "col": col_value }
            }));
        } else {
            warnings.push("Mixed strategy solution out of [0,1] bounds".to_string());
        }
    }

    let json = json!({
        "pure_equilibria": pure,
        "mixed_equilibrium": mixed,
        "warnings": warnings,
    });

    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

// ============================================================================
// Forge Game Theory Tools
// ============================================================================

/// Build a flat values array into a rows×cols matrix, or return an error result.
fn build_matrix(values: &[f64], rows: usize, cols: usize) -> Result<Vec<Vec<f64>>, String> {
    if rows == 0 || cols == 0 {
        return Err("rows and cols must be > 0".to_string());
    }
    if values.len() != rows * cols {
        return Err(format!(
            "values length {} != rows({}) × cols({})",
            values.len(),
            rows,
            cols
        ));
    }
    let mut matrix = Vec::with_capacity(rows);
    for r in 0..rows {
        let start = r * cols;
        matrix.push(values[start..start + cols].to_vec());
    }
    Ok(matrix)
}

/// Analyze an N×M payoff matrix: best responses, dominance, minimax.
pub fn forge_payoff_matrix(params: ForgePayoffMatrixParams) -> Result<CallToolResult, McpError> {
    let matrix = match build_matrix(&params.values, params.rows, params.cols) {
        Ok(m) => m,
        Err(e) => {
            return Ok(CallToolResult::success(vec![Content::text(
                json!({"error": e}).to_string(),
            )]));
        }
    };

    let rows = params.rows;
    let cols = params.cols;

    let row_labels: Vec<String> = params
        .row_labels
        .unwrap_or_else(|| (0..rows).map(|i| format!("R{}", i)).collect());
    let col_labels: Vec<String> = params
        .col_labels
        .unwrap_or_else(|| (0..cols).map(|j| format!("C{}", j)).collect());

    // Row best responses: for each row, which column gives max payoff?
    let row_best_responses: Vec<usize> = (0..rows)
        .map(|r| {
            let mut best_col = 0;
            for c in 1..cols {
                if matrix[r][c] > matrix[r][best_col] + EPS {
                    best_col = c;
                }
            }
            best_col
        })
        .collect();

    // Col best responses: for each column, which row gives min payoff (opponent minimizes)?
    // In zero-sum framing, column player picks row that minimizes row's payoff.
    let col_best_responses: Vec<usize> = (0..cols)
        .map(|c| {
            let mut best_row = 0;
            for r in 1..rows {
                if matrix[r][c] < matrix[best_row][c] - EPS {
                    best_row = r;
                }
            }
            best_row
        })
        .collect();

    // Dominant row: row i dominates if matrix[i][c] >= matrix[r][c] for all r, c
    let dominant_row = (0..rows).find(|&i| {
        (0..rows).all(|r| r == i || (0..cols).all(|c| matrix[i][c] + EPS >= matrix[r][c]))
    });

    // Dominant col (for row player): col j is dominated if another col always gives more
    let dominant_col = (0..cols).find(|&j| {
        (0..cols).all(|c| c == j || (0..rows).all(|r| matrix[r][j] + EPS >= matrix[r][c]))
    });

    // Minimax: row player picks row maximizing minimum payoff across columns
    let minimax_row = (0..rows)
        .map(|r| {
            let min_val = (0..cols)
                .map(|c| matrix[r][c])
                .fold(f64::INFINITY, f64::min);
            (r, min_val)
        })
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(r, v)| json!({"strategy": row_labels[r], "value": v}));

    // Maximin: for each col, max payoff (worst case for col player), then col minimizes that
    let maximin_col = (0..cols)
        .map(|c| {
            let max_val = (0..rows)
                .map(|r| matrix[r][c])
                .fold(f64::NEG_INFINITY, f64::max);
            (c, max_val)
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(c, v)| json!({"strategy": col_labels[c], "value": v}));

    // Expected payoff per row (average across columns)
    let row_expected: Vec<f64> = (0..rows)
        .map(|r| matrix[r].iter().sum::<f64>() / cols as f64)
        .collect();

    let result = json!({
        "matrix": matrix,
        "row_labels": row_labels,
        "col_labels": col_labels,
        "row_best_responses": row_best_responses.iter().enumerate()
            .map(|(r, &c)| json!({"row": row_labels[r], "best_col": col_labels[c]}))
            .collect::<Vec<_>>(),
        "col_best_responses": col_best_responses.iter().enumerate()
            .map(|(c, &r)| json!({"col": col_labels[c], "best_row": row_labels[r]}))
            .collect::<Vec<_>>(),
        "dominant_row": dominant_row.map(|r| &row_labels[r]),
        "dominant_col": dominant_col.map(|c| &col_labels[c]),
        "minimax_row": minimax_row,
        "maximin_col": maximin_col,
        "row_expected_payoffs": row_expected.iter().enumerate()
            .map(|(r, &v)| json!({"row": row_labels[r], "expected": (v * 1000.0).round() / 1000.0}))
            .collect::<Vec<_>>(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// Solve N×M mixed strategy Nash equilibrium via fictitious play.
pub fn forge_nash_solve(params: ForgeNashSolveParams) -> Result<CallToolResult, McpError> {
    let matrix = match build_matrix(&params.values, params.rows, params.cols) {
        Ok(m) => m,
        Err(e) => {
            return Ok(CallToolResult::success(vec![Content::text(
                json!({"error": e}).to_string(),
            )]));
        }
    };

    let rows = params.rows;
    let cols = params.cols;
    let max_iters = params.iterations.unwrap_or(1000);

    // Check for pure dominant strategy first
    let dominant = (0..rows).find(|&i| {
        (0..rows).all(|r| r == i || (0..cols).all(|c| matrix[i][c] + EPS >= matrix[r][c]))
    });

    // Fictitious play: each player tracks opponent's empirical frequency, best-responds
    let mut row_counts = vec![0u64; rows]; // how many times each row was played
    let mut col_counts = vec![0u64; cols]; // how many times each col was played

    // Seed: start with row 0, col 0
    row_counts[0] = 1;
    col_counts[0] = 1;

    let mut converged = false;
    let mut iterations_used = max_iters;

    for iter in 0..max_iters {
        // Row player best-responds to column's empirical distribution
        let total_col: f64 = col_counts.iter().sum::<u64>() as f64;
        let best_row = (0..rows)
            .max_by(|&a, &b| {
                let ev_a: f64 = (0..cols)
                    .map(|c| matrix[a][c] * col_counts[c] as f64 / total_col)
                    .sum();
                let ev_b: f64 = (0..cols)
                    .map(|c| matrix[b][c] * col_counts[c] as f64 / total_col)
                    .sum();
                ev_a.total_cmp(&ev_b)
            })
            .unwrap_or(0);
        row_counts[best_row] += 1;

        // Col player best-responds (minimizes row's payoff) to row's empirical distribution
        let total_row: f64 = row_counts.iter().sum::<u64>() as f64;
        let best_col = (0..cols)
            .min_by(|&a, &b| {
                let ev_a: f64 = (0..rows)
                    .map(|r| matrix[r][a] * row_counts[r] as f64 / total_row)
                    .sum();
                let ev_b: f64 = (0..rows)
                    .map(|r| matrix[r][b] * row_counts[r] as f64 / total_row)
                    .sum();
                ev_a.total_cmp(&ev_b)
            })
            .unwrap_or(0);
        col_counts[best_col] += 1;

        // Check convergence: strategy weights changed < EPS
        if iter > 100 {
            let total_r: f64 = row_counts.iter().sum::<u64>() as f64;
            let total_c: f64 = col_counts.iter().sum::<u64>() as f64;
            let max_row_delta = row_counts
                .iter()
                .map(|&c| {
                    let w = c as f64 / total_r;
                    let w_prev = if c > 0 {
                        (c - 1) as f64 / (total_r - 1.0)
                    } else {
                        0.0
                    };
                    (w - w_prev).abs()
                })
                .fold(0.0f64, f64::max);
            let max_col_delta = col_counts
                .iter()
                .map(|&c| {
                    let w = c as f64 / total_c;
                    let w_prev = if c > 0 {
                        (c - 1) as f64 / (total_c - 1.0)
                    } else {
                        0.0
                    };
                    (w - w_prev).abs()
                })
                .fold(0.0f64, f64::max);
            if max_row_delta < 1e-6 && max_col_delta < 1e-6 {
                converged = true;
                iterations_used = iter + 1;
                break;
            }
        }
    }

    // Compute final strategies
    let total_r: f64 = row_counts.iter().sum::<u64>() as f64;
    let total_c: f64 = col_counts.iter().sum::<u64>() as f64;

    let row_strategy: Vec<f64> = row_counts
        .iter()
        .map(|&c| (c as f64 / total_r * 10000.0).round() / 10000.0)
        .collect();
    let col_strategy: Vec<f64> = col_counts
        .iter()
        .map(|&c| (c as f64 / total_c * 10000.0).round() / 10000.0)
        .collect();

    // Expected payoff under these strategies
    let expected_payoff: f64 = (0..rows)
        .flat_map(|r| (0..cols).map(move |c| (r, c)))
        .map(|(r, c)| row_strategy[r] * col_strategy[c] * matrix[r][c])
        .sum();

    let result = json!({
        "row_strategy": row_strategy,
        "col_strategy": col_strategy,
        "expected_payoff": (expected_payoff * 10000.0).round() / 10000.0,
        "dominant_strategy": dominant,
        "iterations_used": iterations_used,
        "converged": converged,
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// Compute forge quality score: Q = 0.40×prim + 0.25×combat + 0.20×turn + 0.15×survival
pub fn forge_quality_score(params: ForgeQualityScoreParams) -> Result<CallToolResult, McpError> {
    let prim_coverage = params.primitives_collected as f64 / 16.0;
    let combat_eff = if params.enemies_seen > 0 {
        params.enemies_killed as f64 / params.enemies_seen as f64
    } else {
        0.0
    };
    let turn_eff = if params.actual_turns > 0 {
        (params.ideal_turns as f64 / params.actual_turns as f64).min(1.0)
    } else {
        0.0
    };
    let survival = if params.max_hp > 0 {
        (params.current_hp as f64 / params.max_hp as f64).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let total = 0.40 * prim_coverage + 0.25 * combat_eff + 0.20 * turn_eff + 0.15 * survival;

    // Letter grade
    let grade = match total {
        t if t >= 0.95 => "S",
        t if t >= 0.90 => "A+",
        t if t >= 0.85 => "A",
        t if t >= 0.80 => "A-",
        t if t >= 0.75 => "B+",
        t if t >= 0.70 => "B",
        t if t >= 0.65 => "B-",
        t if t >= 0.60 => "C+",
        t if t >= 0.50 => "C",
        t if t >= 0.40 => "C-",
        t if t >= 0.30 => "D",
        _ => "F",
    };

    // Code completeness: 16 primitive sections + 6 safety + 1 quality (if >=8 prims)
    let prim_sections = params.primitives_collected.min(16);
    let quality_section: usize = if params.primitives_collected >= 8 {
        1
    } else {
        0
    };
    let total_sections = prim_sections + quality_section;

    let result = json!({
        "total": (total * 10000.0).round() / 10000.0,
        "components": {
            "primitive_coverage": (prim_coverage * 10000.0).round() / 10000.0,
            "combat_efficiency": (combat_eff * 10000.0).round() / 10000.0,
            "turn_efficiency": (turn_eff * 10000.0).round() / 10000.0,
            "survival": (survival * 10000.0).round() / 10000.0,
        },
        "grade": grade,
        "code_completeness": format!("{}/23", total_sections),
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

// ============================================================================
// Forge Code Generation — Rust templates per primitive and safety key
// ============================================================================

/// Primitive names indexed 0-15 matching the Lex Primitiva
const PRIMITIVE_NAMES: [&str; 16] = [
    "Sequence",        // 0  σ
    "Mapping",         // 1  μ
    "State",           // 2  ς
    "Recursion",       // 3  ρ
    "Void",            // 4  ∅
    "Boundary",        // 5  ∂
    "Frequency",       // 6  ν
    "Existence",       // 7  ∃
    "Persistence",     // 8  π
    "Causality",       // 9  →
    "Comparison",      // 10 κ
    "Quantity",        // 11 N
    "Location",        // 12 λ
    "Irreversibility", // 13 ∝
    "Sum",             // 14 Σ
    "Product",         // 15 ×
];

const PRIMITIVE_SYMS: [&str; 16] = [
    "σ", "μ", "ς", "ρ", "∅", "∂", "ν", "∃", "π", "→", "κ", "N", "λ", "∝", "Σ", "×",
];

fn code_for_primitive(idx: usize) -> Option<(&'static str, &'static str, &'static str)> {
    // Returns (section_name, grounds_to, code)
    match idx {
        11 => Some((
            "Payoff",
            "N + κ",
            r#"/// T2-P: Payoff value (N Quantity + κ Comparison)
#[derive(Debug, Clone, Copy)]
pub struct Payoff(pub f64);

impl Payoff {
    pub fn new(v: f64) -> Self { Self(v) }
    pub fn value(self) -> f64 { self.0 }
    pub fn is_positive(self) -> bool { self.0 > 0.0 }
}"#,
        )),
        14 => Some((
            "Action",
            "Σ + ∂",
            r#"/// T2-P: Action enum (Σ Sum + ∂ Boundary)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    Explore,
    Fight,
    Collect,
    Descend,
    Wait,
}

impl Action {
    pub fn index(self) -> usize {
        match self {
            Self::Explore => 0,
            Self::Fight => 1,
            Self::Collect => 2,
            Self::Descend => 3,
            Self::Wait => 4,
        }
    }
}"#,
        )),
        15 => Some((
            "PlayerState",
            "× + ς + N",
            r#"/// T2-C: Player state (× Product + ς State + N Quantity)
#[derive(Debug, Clone)]
pub struct PlayerState {
    pub hp: i32,
    pub max_hp: i32,
    pub atk: i32,
    pub def: i32,
    pub primitives_collected: Vec<usize>,
    pub floor: u32,
    pub turn: u32,
}

impl PlayerState {
    pub fn hp_ratio(&self) -> f64 {
        if self.max_hp > 0 { self.hp as f64 / self.max_hp as f64 } else { 0.0 }
    }
}"#,
        )),
        0 => Some((
            "ActionSequence",
            "σ + N",
            r#"/// T2-P: Action sequence (σ Sequence + N Quantity)
#[derive(Debug, Clone)]
pub struct ActionSequence(Vec<Action>);

impl ActionSequence {
    pub fn new() -> Self { Self(Vec::new()) }
    pub fn push(&mut self, action: Action) { self.0.push(action); }
    pub fn len(&self) -> usize { self.0.len() }
    pub fn is_empty(&self) -> bool { self.0.is_empty() }
    pub fn iter(&self) -> impl Iterator<Item = &Action> { self.0.iter() }
}

impl IntoIterator for ActionSequence {
    type Item = Action;
    type IntoIter = std::vec::IntoIter<Action>;
    fn into_iter(self) -> Self::IntoIter { self.0.into_iter() }
}"#,
        )),
        1 => Some((
            "PayoffMatrix",
            "μ + N",
            r#"/// T2-P: Payoff matrix (μ Mapping + N Quantity)
#[derive(Debug, Clone)]
pub struct PayoffMatrix {
    data: Vec<Vec<f64>>,
    rows: usize,
    cols: usize,
}

impl PayoffMatrix {
    pub fn new(rows: usize, cols: usize) -> Self {
        Self { data: vec![vec![0.0; cols]; rows], rows, cols }
    }
    pub fn get(&self, r: usize, c: usize) -> Option<f64> {
        self.data.get(r).and_then(|row| row.get(c).copied())
    }
    pub fn set(&mut self, r: usize, c: usize, v: f64) {
        if r < self.rows && c < self.cols { self.data[r][c] = v; }
    }
    pub fn rows(&self) -> usize { self.rows }
    pub fn cols(&self) -> usize { self.cols }
}

pub fn map_payoff(m: &PayoffMatrix, f: impl Fn(f64) -> f64) -> PayoffMatrix {
    let mut out = PayoffMatrix::new(m.rows(), m.cols());
    for r in 0..m.rows() {
        for c in 0..m.cols() {
            if let Some(v) = m.get(r, c) { out.set(r, c, f(v)); }
        }
    }
    out
}"#,
        )),
        10 => Some((
            "PayoffOrd",
            "κ",
            r#"/// T1: Comparison ordering for Payoff (κ Comparison)
impl PartialEq for Payoff {
    fn eq(&self, other: &Self) -> bool { self.0.to_bits() == other.0.to_bits() }
}
impl Eq for Payoff {}

impl PartialOrd for Payoff {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Payoff {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering { self.0.total_cmp(&other.0) }
}

impl std::fmt::Display for Payoff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2}", self.0)
    }
}"#,
        )),
        2 => Some((
            "ForgePhase",
            "ς + ∂ + σ",
            r#"/// T2-C: Phase FSM (ς State + ∂ Boundary + σ Sequence)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForgePhase {
    Explore,
    Combat,
    Collect,
    Descend,
}

impl ForgePhase {
    pub fn valid_transitions(self) -> &'static [ForgePhase] {
        match self {
            Self::Explore => &[Self::Combat, Self::Collect, Self::Descend],
            Self::Combat => &[Self::Explore, Self::Collect],
            Self::Collect => &[Self::Explore, Self::Descend],
            Self::Descend => &[Self::Explore],
        }
    }
    pub fn can_transition_to(self, target: ForgePhase) -> bool {
        self.valid_transitions().contains(&target)
    }
}"#,
        )),
        9 => Some((
            "Outcome",
            "→ + ×",
            r#"/// T2-P: Causal outcome (→ Causality + × Product)
#[derive(Debug, Clone)]
pub struct Outcome {
    pub payoff: Payoff,
    pub new_phase: ForgePhase,
    pub description: String,
}

pub fn apply_action(phase: ForgePhase, action: Action) -> Outcome {
    match (phase, action) {
        (ForgePhase::Explore, Action::Fight) => Outcome {
            payoff: Payoff::new(-1.0), new_phase: ForgePhase::Combat,
            description: "Engaged enemy".to_string(),
        },
        (ForgePhase::Explore, Action::Collect) => Outcome {
            payoff: Payoff::new(2.0), new_phase: ForgePhase::Collect,
            description: "Found primitive".to_string(),
        },
        (ForgePhase::Combat, Action::Fight) => Outcome {
            payoff: Payoff::new(3.0), new_phase: ForgePhase::Explore,
            description: "Victory".to_string(),
        },
        (_, Action::Wait) => Outcome {
            payoff: Payoff::new(0.0), new_phase: phase,
            description: "Waited".to_string(),
        },
        _ => Outcome {
            payoff: Payoff::new(-0.5), new_phase: phase,
            description: "Invalid transition".to_string(),
        },
    }
}"#,
        )),
        5 => Some((
            "ForgeError",
            "∂ + ∅",
            r#"/// T2-P: Boundary validation (∂ Boundary + ∅ Void)
#[derive(Debug)]
pub enum ForgeError {
    PayoffOutOfRange(f64),
    InvalidRow(usize),
    InvalidCol(usize),
    EmptyMatrix,
    InvalidTransition(ForgePhase, Action),
}

impl std::fmt::Display for ForgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PayoffOutOfRange(v) => write!(f, "payoff {v} out of [-1000, 1000]"),
            Self::InvalidRow(r) => write!(f, "row {r} out of bounds"),
            Self::InvalidCol(c) => write!(f, "col {c} out of bounds"),
            Self::EmptyMatrix => write!(f, "matrix is empty"),
            Self::InvalidTransition(p, a) => write!(f, "cannot {a:?} in {p:?}"),
        }
    }
}

pub fn validate_payoff(v: f64) -> Result<Payoff, ForgeError> {
    if !v.is_finite() || v < -1000.0 || v > 1000.0 {
        Err(ForgeError::PayoffOutOfRange(v))
    } else {
        Ok(Payoff::new(v))
    }
}"#,
        )),
        3 => Some((
            "Minimax",
            "ρ + κ",
            r#"/// T2-P: Recursive minimax solver (ρ Recursion + κ Comparison)
pub fn minimax(
    matrix: &PayoffMatrix,
    depth: usize,
    is_maximizing: bool,
) -> (Option<usize>, f64) {
    if depth == 0 {
        return (None, 0.0);
    }
    if is_maximizing {
        let mut best_val = f64::NEG_INFINITY;
        let mut best_row = 0;
        for r in 0..matrix.rows() {
            let (_, val) = minimax(matrix, depth - 1, false);
            let avg: f64 = (0..matrix.cols())
                .filter_map(|c| matrix.get(r, c))
                .sum::<f64>() / matrix.cols() as f64;
            let score = avg + val;
            if score > best_val {
                best_val = score;
                best_row = r;
            }
        }
        (Some(best_row), best_val)
    } else {
        let mut worst_val = f64::INFINITY;
        let mut worst_col = 0;
        for c in 0..matrix.cols() {
            let (_, val) = minimax(matrix, depth - 1, true);
            let avg: f64 = (0..matrix.rows())
                .filter_map(|r| matrix.get(r, c))
                .sum::<f64>() / matrix.rows() as f64;
            let score = avg + val;
            if score < worst_val {
                worst_val = score;
                worst_col = c;
            }
        }
        (Some(worst_col), worst_val)
    }
}"#,
        )),
        6 => Some((
            "MixedStrategy",
            "ν + Σ + N",
            r#"/// T2-C: Mixed strategy (ν Frequency + Σ Sum + N Quantity)
#[derive(Debug, Clone)]
pub struct MixedStrategy {
    weights: Vec<(Action, f64)>,
}

impl MixedStrategy {
    pub fn uniform(actions: &[Action]) -> Self {
        let w = 1.0 / actions.len() as f64;
        Self { weights: actions.iter().map(|&a| (a, w)).collect() }
    }

    pub fn from_weights(weights: Vec<(Action, f64)>) -> Result<Self, ForgeError> {
        let sum: f64 = weights.iter().map(|(_, w)| w).sum();
        if (sum - 1.0).abs() > 0.01 {
            return Err(ForgeError::EmptyMatrix);
        }
        Ok(Self { weights })
    }

    pub fn best_action(&self) -> Option<Action> {
        self.weights.iter()
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(a, _)| *a)
    }

    pub fn weight_of(&self, action: Action) -> f64 {
        self.weights.iter()
            .find(|(a, _)| *a == action)
            .map(|(_, w)| *w)
            .unwrap_or(0.0)
    }
}"#,
        )),
        7 => Some((
            "DominantStrategy",
            "∃ + κ",
            r#"/// T2-P: Dominant strategy check (∃ Existence + κ Comparison)
pub fn has_dominant_strategy(matrix: &PayoffMatrix) -> Option<usize> {
    'outer: for i in 0..matrix.rows() {
        for r in 0..matrix.rows() {
            if r == i { continue; }
            for c in 0..matrix.cols() {
                let vi = matrix.get(i, c).unwrap_or(0.0);
                let vr = matrix.get(r, c).unwrap_or(0.0);
                if vi < vr { continue 'outer; }
            }
        }
        return Some(i);
    }
    None
}"#,
        )),
        4 => Some((
            "SafePayoff",
            "∅ + ∂",
            r#"/// T2-P: Safe access (∅ Void + ∂ Boundary)
pub fn safe_payoff(matrix: &PayoffMatrix, r: usize, c: usize) -> Payoff {
    Payoff::new(matrix.get(r, c).unwrap_or(0.0))
}

pub fn safe_row(matrix: &PayoffMatrix, r: usize) -> Vec<Payoff> {
    (0..matrix.cols())
        .map(|c| safe_payoff(matrix, r, c))
        .collect()
}"#,
        )),
        8 => Some((
            "Persistable",
            "π",
            r#"/// T1: Persistence trait (π Persistence)
pub trait Persistable {
    fn to_json(&self) -> String;
    fn from_json(s: &str) -> Result<Self, ForgeError> where Self: Sized;
}"#,
        )),
        12 => Some((
            "MatrixPos",
            "λ + N + ∂",
            r#"/// T2-C: Matrix position (λ Location + N Quantity + ∂ Boundary)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MatrixPos {
    pub row: usize,
    pub col: usize,
}

impl MatrixPos {
    pub fn new(row: usize, col: usize, max_rows: usize, max_cols: usize) -> Option<Self> {
        if row < max_rows && col < max_cols {
            Some(Self { row, col })
        } else {
            None
        }
    }

    pub fn neighbors(&self, max_rows: usize, max_cols: usize) -> Vec<MatrixPos> {
        let mut out = Vec::new();
        if self.row > 0 { out.push(MatrixPos { row: self.row - 1, col: self.col }); }
        if self.row + 1 < max_rows { out.push(MatrixPos { row: self.row + 1, col: self.col }); }
        if self.col > 0 { out.push(MatrixPos { row: self.row, col: self.col - 1 }); }
        if self.col + 1 < max_cols { out.push(MatrixPos { row: self.row, col: self.col + 1 }); }
        out
    }
}"#,
        )),
        13 => Some((
            "CommittedStrategy",
            "∝ + ν",
            r#"/// T2-P: Irreversible committed strategy (∝ Irreversibility + ν Frequency)
pub struct CommittedStrategy {
    strategy: MixedStrategy,
}

impl CommittedStrategy {
    pub fn commit(strategy: MixedStrategy) -> Self {
        Self { strategy }
    }
    pub fn best_action(&self) -> Option<Action> { self.strategy.best_action() }
    pub fn weights(&self) -> &[(Action, f64)] { &self.strategy.weights }
}"#,
        )),
        _ => None,
    }
}

fn code_for_safety(key: &str) -> Option<(&'static str, &'static str)> {
    match key {
        "unwrap" => Some((
            "SafeAccess",
            r#"/// Safety: unwrap eliminated — safe matrix access with defaults
pub fn safe_matrix_get(m: &PayoffMatrix, r: usize, c: usize) -> f64 {
    m.get(r, c).unwrap_or(0.0)
}"#,
        )),
        "panic" => Some((
            "CheckedDiv",
            r#"/// Safety: panic eliminated — checked division
pub fn checked_div(a: f64, b: f64) -> Result<f64, ForgeError> {
    if b.abs() < f64::EPSILON {
        Err(ForgeError::PayoffOutOfRange(f64::INFINITY))
    } else {
        Ok(a / b)
    }
}"#,
        )),
        "unsafe" => Some((
            "ForbidUnsafe",
            r#"// Safety: unsafe eliminated — enforced by #![forbid(unsafe_code)] in header"#,
        )),
        "deadlock" => Some((
            "CycleDetect",
            r#"/// Safety: deadlock eliminated — Floyd's cycle detection
pub fn has_cycle(transitions: &[(usize, usize)], start: usize) -> bool {
    let next = |node: usize| -> usize {
        transitions.iter()
            .find(|(from, _)| *from == node)
            .map(|(_, to)| *to)
            .unwrap_or(node)
    };
    let mut slow = start;
    let mut fast = start;
    loop {
        slow = next(slow);
        fast = next(next(fast));
        if slow == fast { return slow != start || next(start) == start; }
        if fast == start { return false; }
    }
}"#,
        )),
        "clone" => Some((
            "ZeroCopy",
            r#"/// Safety: clone eliminated — in-place Nash computation
pub fn nash_in_place(matrix: &mut PayoffMatrix) -> Vec<f64> {
    let rows = matrix.rows();
    let cols = matrix.cols();
    let mut strategy = vec![1.0 / rows as f64; rows];
    for _ in 0..100 {
        let mut best_r = 0;
        let mut best_val = f64::NEG_INFINITY;
        for r in 0..rows {
            let ev: f64 = (0..cols)
                .filter_map(|c| matrix.get(r, c))
                .sum::<f64>() / cols as f64;
            if ev > best_val { best_val = ev; best_r = r; }
        }
        for s in &mut strategy { *s *= 0.99; }
        strategy[best_r] += 0.01;
    }
    strategy
}"#,
        )),
        "leak" => Some((
            "DropImpl",
            r#"/// Safety: leak eliminated — Drop implementation for cleanup
impl Drop for ActionSequence {
    fn drop(&mut self) {
        self.0.clear();
    }
}"#,
        )),
        _ => None,
    }
}

fn quality_score_section() -> &'static str {
    r#"/// Quality score computation (N Quantity + κ Comparison + ∂ Boundary)
pub struct QualityScore {
    pub primitive_coverage: f64,
    pub combat_efficiency: f64,
    pub turn_efficiency: f64,
    pub survival: f64,
}

impl QualityScore {
    pub fn compute(
        prims: usize, enemies_killed: usize, enemies_seen: usize,
        actual_turns: u32, ideal_turns: u32, hp: i32, max_hp: i32,
    ) -> Self {
        let primitive_coverage = prims as f64 / 16.0;
        let combat_efficiency = if enemies_seen > 0 {
            enemies_killed as f64 / enemies_seen as f64
        } else { 0.0 };
        let turn_efficiency = if actual_turns > 0 {
            (ideal_turns as f64 / actual_turns as f64).min(1.0)
        } else { 0.0 };
        let survival = if max_hp > 0 { hp as f64 / max_hp as f64 } else { 0.0 };
        Self { primitive_coverage, combat_efficiency, turn_efficiency, survival }
    }

    pub fn total(&self) -> f64 {
        0.40 * self.primitive_coverage
      + 0.25 * self.combat_efficiency
      + 0.20 * self.turn_efficiency
      + 0.15 * self.survival
    }
}"#
}

/// Generate Rust code from collected primitives and defeated enemies.
pub fn forge_code_generate(params: ForgeCodeGenerateParams) -> Result<CallToolResult, McpError> {
    let collected = &params.collected_primitives;
    let enemies = params.defeated_enemies.unwrap_or_default();

    // Validate primitive indices
    for &idx in collected {
        if idx > 15 {
            return Ok(CallToolResult::success(vec![Content::text(
                json!({"error": format!("primitive index {} out of range 0-15", idx)}).to_string(),
            )]));
        }
    }

    let mut sections: Vec<(i32, String, String, String)> = Vec::new(); // (order, name, grounds_to, code)

    // Header always included
    sections.push((
        -1,
        "Header".to_string(),
        "".to_string(),
        format!(
            "//! forge_output.rs — Generated by Primitive Depths: Code Forge\n\
         //! Primitives: {} of 16 collected\n\
         //! Safety keys: {}\n\n\
         #![forbid(unsafe_code)]\n\
         #![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]",
            collected.len(),
            if enemies.is_empty() {
                "none".to_string()
            } else {
                enemies.join(", ")
            }
        ),
    ));

    // Primitive generation order from forge.html
    let generation_order: [(usize, i32); 16] = [
        (11, 0),
        (14, 1),
        (15, 2),
        (0, 3),
        (1, 4),
        (10, 5),
        (2, 6),
        (9, 7),
        (5, 8),
        (3, 9),
        (6, 10),
        (7, 11),
        (4, 12),
        (8, 13),
        (12, 14),
        (13, 15),
    ];

    let mut grounds_to_summary = Vec::new();

    for &(prim_idx, order) in &generation_order {
        if collected.contains(&prim_idx) {
            if let Some((name, grounds, code)) = code_for_primitive(prim_idx) {
                sections.push((
                    order,
                    name.to_string(),
                    grounds.to_string(),
                    code.to_string(),
                ));
                grounds_to_summary.push(json!({
                    "type": name,
                    "prims": grounds,
                    "prim_sym": PRIMITIVE_SYMS[prim_idx],
                    "prim_name": PRIMITIVE_NAMES[prim_idx],
                }));
            }
        }
    }

    // Safety sections (order 100+)
    let safety_order = ["unwrap", "panic", "unsafe", "deadlock", "clone", "leak"];
    for (i, key) in safety_order.iter().enumerate() {
        if enemies.iter().any(|e| e == key) {
            if let Some((name, code)) = code_for_safety(key) {
                sections.push((
                    100 + i as i32,
                    name.to_string(),
                    String::new(),
                    code.to_string(),
                ));
            }
        }
    }

    // Quality score section if >= 8 primitives
    if collected.len() >= 8 {
        sections.push((
            200,
            "QualityScore".to_string(),
            "N + κ + ∂".to_string(),
            quality_score_section().to_string(),
        ));
    }

    // Sort by order
    sections.sort_by_key(|s| s.0);

    // Build final code
    let code: String = sections
        .iter()
        .map(|(_, _, _, c)| c.as_str())
        .collect::<Vec<_>>()
        .join("\n\n");

    let lines_of_code = code.lines().count();

    let result = json!({
        "code": code,
        "sections_unlocked": sections.len(),
        "total_sections": 23,
        "lines_of_code": lines_of_code,
        "grounds_to_summary": grounds_to_summary,
        "primitives_collected": collected.iter()
            .filter(|&&i| i < 16)
            .map(|&i| json!({"idx": i, "sym": PRIMITIVE_SYMS[i], "name": PRIMITIVE_NAMES[i]}))
            .collect::<Vec<_>>(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}
