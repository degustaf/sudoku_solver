//! A rust representation of the f-puzzles json format. It includes serialization, and
//! deserialization using [serde](https://serde.rs/) crate.

#![warn(missing_docs)]
use serde::{Deserialize, Serialize};

/// A description of the state of an individual cell in the grid.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Cell {
    /// The filled in value in a grid cell.
    pub value: Option<usize>,

    #[serde(default)]
    given: bool,
    #[serde(rename = "centerPencilMarks")]
    #[serde(default)]
    center_pencil_marks: Vec<usize>,
    #[serde(rename = "cornerPencilMarks")]
    #[serde(default)]
    corner_pencil_marks: Vec<usize>,

    /// Any pencilmarks that are to be treated as given.
    #[serde(rename = "givenPencilMarks")]
    #[serde(default)]
    pub given_pencil_marks: Vec<usize>,

    /// Which region this cell should be a part of.
    pub region: Option<usize>,
}

impl Cell {
    fn new() -> Self {
        Cell {
            value: None,
            given: false,
            center_pencil_marks: Vec::new(),
            corner_pencil_marks: Vec::new(),
            given_pencil_marks: Vec::new(),
            region: None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
enum Logic {
    #[serde(rename = "tuples")]
    Tuples,
    #[serde(rename = "pointing")]
    Pointing,
    #[serde(rename = "fishes")]
    Fishes,
    #[serde(rename = "wings")]
    Wings,
    #[serde(rename = "aic")]
    Aic,
    #[serde(rename = "contradictions")]
    Contradictions,
}

#[derive(Debug, Deserialize, Serialize)]
enum TrueCandidatesOption {
    #[serde(rename = "colored")]
    Colored,
    #[serde(rename = "logical")]
    Logical,
}

#[derive(Debug, Deserialize, Serialize)]
struct CellPair {
    cells: [String; 2],
}

#[derive(Debug, Deserialize, Serialize)]
struct Quad {
    cells: Vec<String>,
    values: Vec<usize>,
}

/// A rust representation of a sudoku puzzle. It uses the f-puzzles format.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FPuzzles {
    /// The dimension of the Sudoku grid.
    pub size: usize,

    /// The individual cells in the board.
    pub grid: Vec<Vec<Cell>>,
    #[serde(rename = "diagonal+")]
    #[serde(default)]
    positive_diagonal: bool,
    #[serde(rename = "diagonal-")]
    #[serde(default)]
    negative_diagonal: bool,
    #[serde(default)]
    antiknight: bool,
    #[serde(default)]
    antiking: bool,
    #[serde(default)]
    disjointgroups: bool,
    #[serde(default)]
    nonconsecutive: bool,
    #[serde(default)]
    disabledlogic: Vec<Logic>,
    #[serde(default)]
    truecandidatesoptions: Vec<TrueCandidatesOption>,
    #[serde(default)]
    difference: Vec<CellPair>,
    #[serde(default)]
    ratio: Vec<CellPair>,
    #[serde(default)]
    quadruple: Vec<Quad>,
}

impl FPuzzles {
    /// Create a new sudoku puzzle in the f-puzzles format.
    #[must_use]
    pub fn new(size: usize) -> Self {
        FPuzzles {
            size,
            grid: vec![vec!(Cell::new(); size); size],
            positive_diagonal: false,
            negative_diagonal: false,
            antiknight: false,
            antiking: false,
            disjointgroups: false,
            nonconsecutive: false,
            disabledlogic: Vec::new(),
            truecandidatesoptions: Vec::new(),
            difference: Vec::new(),
            ratio: Vec::new(),
            quadruple: Vec::new(),
        }
    }

    /// Test if the puzzle has an irregular grid.
    #[must_use]
    pub fn is_irregular(&self) -> bool {
        for row in &self.grid {
            for cell in row {
                if cell.region.is_some() {
                    return true;
                }
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn irregular_region() {
        let mut f = FPuzzles::new(9);
        assert!(!f.is_irregular());

        f.grid[0][0].region = Some(1);
        assert!(f.is_irregular());
    }
}
