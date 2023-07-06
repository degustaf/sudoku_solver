//! Types for interacting with a sudoku puzzle.

use bitvec::array as bit_array;
use core::num::TryFromIntError;
use core::ops::BitAnd;
use core::ops::BitAndAssign;
use core::ops::Not;
use fmt::Display;
use std::fmt;
use std::sync::Arc;

/// Errors for creating and solving sudoku.
#[derive(Debug, PartialEq)]
pub enum SudokuErrors {
    /// Attempted to make a grid with a `max_val` that is too large for our data structures.
    MaxTooLarge,

    /// Attempted to place a digit that is bigger than `Board::max_val`
    ValueTooLarge,

    /// Attempt to access an invalid index.
    OutOfBounds,

    /// In a context that doesn't return an `Elimination`, a contradiction was found.
    Contradiction,

    /// Attempt to create a non-square board.
    BadSize,

    /// Attempted to use an invalid character as a digit in a sudoku.
    BadDigit,

    /// An irregular
    IrregularWrongSizes,

    /// A puzzle has multiple solutions. Used as an error in cases where a unique solution is
    /// expected.
    MultipleSolutions,

    /// Attempt to encode a cell location in RC format that is corrupted.
    BadRCEncoding,
}

impl From<TryFromIntError> for SudokuErrors {
    fn from(_: TryFromIntError) -> Self {
        Self::BadDigit
    }
}

impl From<Contradiction> for SudokuErrors {
    fn from(_: Contradiction) -> Self {
        Self::Contradiction
    }
}

impl Display for SudokuErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{self:?}")
    }
}

/// Represents the deduction that a board is invalid.
#[derive(Debug, PartialEq)]
pub struct Contradiction(pub(crate) ());

/// Tracks if a strategy is sucessful.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Elimination {
    /// The strategy succeeded in eiliminating candidate(s).
    Eliminated,

    /// The strategy failed to eliminate any candidates.
    Same,
}

impl BitAnd<Elimination> for Elimination {
    type Output = Elimination;

    fn bitand(self, rhs: Elimination) -> Self::Output {
        if self == Self::Eliminated {
            Self::Eliminated
        } else {
            rhs
        }
    }
}

impl BitAndAssign for Elimination {
    fn bitand_assign(&mut self, rhs: Self) {
        if *self != Self::Eliminated {
            *self = rhs;
        }
    }
}

pub(crate) type Bits = usize;
pub(crate) type MoreBits = bit_array::BitArray<[u64; 4]>;
pub const MAX_SIZE: usize = 16;

pub(crate) const DIMENSIONS: [(usize, usize); MAX_SIZE] = [
    (1, 1),
    (2, 1),
    (3, 1),
    (2, 2),
    (5, 1),
    (3, 2),
    (7, 1),
    (4, 2),
    (3, 3),
    (5, 2),
    (11, 1),
    (4, 3),
    (13, 1),
    (7, 2),
    (5, 3),
    (4, 4),
];

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Constraint {
    Quad(usize, Bits, Bits),
    Region(Vec<usize>),
}

#[derive(Debug)]
pub(crate) struct BoardMeta {
    /// The size of a side of the board.
    pub(crate) size: usize,

    /// The maximum value that is used in this sudoku.
    pub(crate) max_val: usize,

    pub(crate) rows: Vec<Vec<usize>>,
    pub(crate) columns: Vec<Vec<usize>>,

    /// In a regular sudoku, these will represent the 9 3x3 boxes. We aren't hardcoding that in
    /// anticipation of irregular sudoku.
    pub(crate) regions: Vec<Vec<usize>>,

    pub(crate) constraints: Vec<Constraint>,
}

/// A representation of a sudoku board.
#[derive(Clone, Debug)]
pub struct Board {
    /// Helps us count which values we've used for mean mini puzzles.
    pub(crate) used_digits: Bits,

    /// Indices where we have placed a digit.
    pub(crate) solved_digits: MoreBits,

    /// space to store the data.
    pub(crate) grid: Vec<Bits>,

    /// Data that will remain constant during a solve. When we make a guess and copy a board, this
    /// doesn't need to be copied.
    pub(crate) meta: Arc<BoardMeta>,
}

pub(crate) fn to_bits(value: usize) -> Bits {
    debug_assert!(value < usize::BITS as usize);
    1 << value
}

pub(crate) fn eliminate(
    idx: usize,
    value: Bits,
    grid: &mut [Bits],
) -> Result<Elimination, Contradiction> {
    debug_assert!(idx <= grid.len());

    if grid[idx] & value == 0 {
        return Ok(Elimination::Same);
    }

    grid[idx] &= value.not();
    if grid[idx] == 0 {
        Err(Contradiction(()))
    } else {
        Ok(Elimination::Eliminated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIX: Bits = 1 << 6;

    #[test]
    fn parsing_bad_digits() {
        fn bad_digit() -> Result<usize, SudokuErrors> {
            Ok(usize::try_from(-1)?)
        }
        assert_eq!(bad_digit(), Err(SudokuErrors::BadDigit));
    }

    #[test]
    fn test_to_bits() {
        assert_eq!(to_bits(6), SIX);
    }

    #[test]
    fn bad_to_bits() {
        let board = Board::new(9, 9).unwrap();
        assert_eq!(board.to_bits(16), Err(SudokuErrors::ValueTooLarge));
    }

    #[test]
    fn eliminations_combine() {
        assert_eq!(
            Elimination::Eliminated & Elimination::Eliminated,
            Elimination::Eliminated
        );
        assert_eq!(
            Elimination::Eliminated & Elimination::Same,
            Elimination::Eliminated
        );

        assert_eq!(
            Elimination::Same & Elimination::Eliminated,
            Elimination::Eliminated
        );
        assert_eq!(Elimination::Same & Elimination::Same, Elimination::Same);
    }
}
