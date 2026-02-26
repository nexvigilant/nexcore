//! Board state management.
//!
//! The board is the σ (sequence) + ρ (state) composite: an ordered grid
//! of clues with mutable answered/unanswered state.

use crate::error::{JeopardyError, Result};
use crate::types::{Category, Clue, CluePosition, ClueValue, Round};
use serde::{Deserialize, Serialize};

/// A single cell on the board: either available or already answered.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Cell {
    /// Clue is still available to be selected.
    Available(Clue),
    /// Clue has been answered (stores the original value for scoring reference).
    Answered(ClueValue),
}

impl Cell {
    /// Returns true if this cell is still available.
    pub fn is_available(&self) -> bool {
        matches!(self, Cell::Available(_))
    }

    /// Returns the clue if available.
    pub fn clue(&self) -> Option<&Clue> {
        match self {
            Cell::Available(c) => Some(c),
            Cell::Answered(_) => None,
        }
    }

    /// Returns the value regardless of state.
    pub fn value(&self) -> ClueValue {
        match self {
            Cell::Available(c) => c.value,
            Cell::Answered(v) => *v,
        }
    }
}

/// The game board: a 2D grid of cells (rows x categories).
///
/// Convention: row 0 = lowest value, row 4 = highest value.
/// Holzhauer strategy traverses from row 4 downward.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Board {
    /// Grid storage: `cells[row][col]`.
    cells: Vec<Vec<Cell>>,
    /// Number of rows (typically 5).
    num_rows: usize,
    /// Categories for column headers.
    categories: Vec<Category>,
    /// Which round this board belongs to.
    round: Round,
}

impl Board {
    /// Create a new board for the given round with standard layout.
    ///
    /// Daily doubles are placed at the specified positions (if any).
    pub fn new(round: Round, daily_double_positions: &[CluePosition]) -> Result<Self> {
        let categories = Category::all().to_vec();
        let values = ClueValue::for_round(round);
        let num_rows = values.len();
        let num_cols = categories.len();

        if num_rows == 0 || num_cols == 0 {
            return Err(JeopardyError::InvalidBoardDimensions {
                rows: num_rows,
                cols: num_cols,
            });
        }

        let mut cells = Vec::with_capacity(num_rows);
        for (row_idx, value) in values.iter().enumerate() {
            let mut row = Vec::with_capacity(num_cols);
            for (col_idx, cat) in categories.iter().enumerate() {
                let pos = CluePosition::new(row_idx, col_idx);
                let is_dd = daily_double_positions.contains(&pos);
                // Difficulty scales linearly with row (higher value = harder)
                let difficulty = (row_idx as f64 + 1.0) / num_rows as f64;
                row.push(Cell::Available(Clue::new(*cat, *value, difficulty, is_dd)));
            }
            cells.push(row);
        }

        Ok(Board {
            cells,
            num_rows,
            categories,
            round,
        })
    }

    /// Get the cell at the given position.
    pub fn get(&self, pos: CluePosition) -> Option<&Cell> {
        self.cells.get(pos.row).and_then(|row| row.get(pos.col))
    }

    /// Mark a clue as answered. Returns the clue that was there.
    pub fn answer(&mut self, pos: CluePosition) -> Result<Clue> {
        let cell = self
            .cells
            .get_mut(pos.row)
            .and_then(|row| row.get_mut(pos.col));

        match cell {
            Some(Cell::Available(clue)) => {
                let clue_copy = clue.clone();
                self.cells[pos.row][pos.col] = Cell::Answered(clue_copy.value);
                Ok(clue_copy)
            }
            Some(Cell::Answered(_)) => Err(JeopardyError::ClueAlreadyAnswered {
                row: pos.row,
                category: pos.col,
            }),
            None => Err(JeopardyError::InvalidBoardDimensions {
                rows: pos.row,
                cols: pos.col,
            }),
        }
    }

    /// All positions with available (unanswered) clues.
    pub fn available_positions(&self) -> Vec<CluePosition> {
        let mut positions = Vec::new();
        for (row_idx, row) in self.cells.iter().enumerate() {
            for (col_idx, cell) in row.iter().enumerate() {
                if cell.is_available() {
                    positions.push(CluePosition::new(row_idx, col_idx));
                }
            }
        }
        positions
    }

    /// Count of remaining available clues.
    pub fn remaining_count(&self) -> usize {
        self.cells
            .iter()
            .flat_map(|row| row.iter())
            .filter(|c| c.is_available())
            .count()
    }

    /// Whether the board is fully answered.
    pub fn is_empty(&self) -> bool {
        self.remaining_count() == 0
    }

    /// Total value of all remaining clues.
    pub fn remaining_value(&self) -> u64 {
        self.cells
            .iter()
            .flat_map(|row| row.iter())
            .filter_map(|c| c.clue().map(|clue| clue.value.0))
            .sum()
    }

    /// Number of rows on this board.
    pub fn num_rows(&self) -> usize {
        self.num_rows
    }

    /// Number of categories (columns).
    pub fn num_cols(&self) -> usize {
        self.categories.len()
    }

    /// The round this board belongs to.
    pub fn round(&self) -> Round {
        self.round
    }

    /// The categories on this board.
    pub fn categories(&self) -> &[Category] {
        &self.categories
    }

    /// All daily double positions that are still available.
    pub fn remaining_daily_doubles(&self) -> Vec<CluePosition> {
        self.available_positions()
            .into_iter()
            .filter(|pos| {
                self.get(*pos)
                    .and_then(|c| c.clue())
                    .is_some_and(|clue| clue.is_daily_double)
            })
            .collect()
    }
}
