//! A rust representation of the f-puzzles json format. It includes serialization, and
//! deserialization using [serde](https://serde.rs/) crate.

#![warn(missing_docs)]
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Cell {
    value: Option<usize>,
    #[serde(default)]
    given: bool,
    #[serde(rename = "centerPencilMarks")]
    #[serde(default)]
    center_pencil_marks: Vec<usize>,
    #[serde(rename = "cornerPencilMarks")]
    #[serde(default)]
    corner_pencil_marks: Vec<usize>,
    #[serde(rename = "givenPencilMarks")]
    #[serde(default)]
    given_pencil_marks: Vec<usize>,
    region: Option<usize>,
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

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FPuzzles {
    size: usize,
    grid: Vec<Vec<Cell>>,
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

/*
 * {"size":9,"quadruple":[{"cells":["R6C3","R6C4","R7C3","R7C4"],"values":[1,2,2,3]},{"cells":["R3C6","R3C7","R4C6","R4C7"],"values":[4,5,6,7]},{"cells":["R6C6","R6C7","R7C6","R7C7"],"values":[8,9]}]
 */
