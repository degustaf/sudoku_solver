//! Provides ways to interact with a sudoku puzzle.

#![warn(missing_docs)]
pub mod types;

use types::SudokuErrors;

// TODO make public wrappers of these functions.
// Board::assign
// Board::eliminate

/// Place the digit `value` in the puzzle at location `idx`.
///
/// # Errors
/// This function can return an error if
/// - `idx` is out of bounds for the grid.
/// - `value` is not a valid digit for the puzzle.
pub fn assign(
    board: &mut types::Board,
    idx: usize,
    value: usize,
) -> Result<types::Elimination, SudokuErrors> {
    if idx >= board.len() {
        return Err(SudokuErrors::OutOfBounds);
    }
    let v = board.to_bits(value)?;
    if board.possible_value(idx, v) {
        Ok(board.assign(idx, v))
    } else {
        Ok(types::Elimination::Contradiction)
    }
}

/// Eliminate digits contained in `value` from the grid at location `idx`.
///
/// # Errors
/// This function can return an error if
/// - `idx` is out of bounds for the grid.
/// - `value` is not a valid digit for the puzzle.
pub fn eliminate(
    board: &mut types::Board,
    idx: usize,
    value: usize,
) -> Result<types::Elimination, SudokuErrors> {
    if idx >= board.len() {
        return Err(SudokuErrors::OutOfBounds);
    }
    let v = board.to_bits(value)?;
    Ok(board.eliminate(idx, v))
}

#[cfg(test)]
mod tests {
    use super::*;

    use types::Board;

    #[test]
    fn test_assign() {
        let mut board = Board::new(9, 9).unwrap();
        assert_eq!(
            assign(&mut board, 11, 6),
            Ok(types::Elimination::Eliminated)
        );
        assert_eq!(
            assign(&mut board, 11, 1),
            Ok(types::Elimination::Contradiction)
        );
        assert_eq!(assign(&mut board, 1111, 6), Err(SudokuErrors::OutOfBounds));
        assert_eq!(assign(&mut board, 21, 16), Err(SudokuErrors::ValueTooLarge));
    }

    #[test]
    fn test_eliminate() {
        let mut board = Board::new(9, 9).unwrap();
        assert_eq!(
            eliminate(&mut board, 11, 6),
            Ok(types::Elimination::Eliminated)
        );
        assign(&mut board, 11, 5).unwrap();
        assert_eq!(
            eliminate(&mut board, 11, 5),
            Ok(types::Elimination::Contradiction)
        );

        assert_eq!(
            eliminate(&mut board, 1111, 6),
            Err(SudokuErrors::OutOfBounds)
        );
        assert_eq!(
            eliminate(&mut board, 21, 16),
            Err(SudokuErrors::ValueTooLarge)
        );
    }
}
