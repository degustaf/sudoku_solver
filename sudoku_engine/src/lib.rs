//! Provides ways to interact with a sudoku puzzle.

#![warn(missing_docs)]
mod types;

use types::board::Bits;
pub use types::board::Board;
use types::board::Elimination;
pub use types::board::SudokuErrors;

/// Convert a string of digits into the associated Board.
///
/// # Errors
/// This function can return an error if
/// - The string is not the right length to make a square board.
/// - A character that can't be converted into a digit is in the string.
pub fn from_string(repr: &str) -> Result<Board, SudokuErrors> {
    #[allow(clippy::cast_precision_loss)]
    // If our length overflows an f32, nothing later is going to work.
    #[allow(clippy::cast_sign_loss)]
    // sqrt is never negative.
    #[allow(clippy::cast_possible_truncation)]
    // We check if there's truncation, and return an error.
    let size = f32::sqrt(repr.len() as f32) as usize;
    if size * size != repr.len() {
        return Err(SudokuErrors::BadSize);
    }

    let mut digits: Vec<Option<Bits>> = vec![None; size * size];
    let mut max_val = size;
    for (i, c) in repr.chars().enumerate() {
        digits[i] = if let Some(d) = c.to_digit(16) {
            let x = usize::try_from(d)?;
            max_val = usize::max(max_val, x);
            Some(types::board::to_bits(x))
        } else {
            None
        }
    }

    Board::from_digits(size, max_val, &digits)
}

/// Place the digit `value` in the puzzle at location `idx`.
///
/// # Errors
/// This function can return an error if
/// - `idx` is out of bounds for the grid.
/// - `value` is not a valid digit for the puzzle.
pub fn assign(board: &mut Board, idx: usize, value: usize) -> Result<Elimination, SudokuErrors> {
    if idx >= board.len() {
        return Err(SudokuErrors::OutOfBounds);
    }
    let v = board.to_bits(value)?;
    if board.possible_value(idx, v) {
        Ok(board.assign(idx, v)?)
    } else {
        Err(SudokuErrors::Contradiction)
    }
}

/// Eliminate digits contained in `value` from the grid at location `idx`.
///
/// # Errors
/// This function can return an error if
/// - `idx` is out of bounds for the grid.
/// - `value` is not a valid digit for the puzzle.
pub fn eliminate(board: &mut Board, idx: usize, value: usize) -> Result<Elimination, SudokuErrors> {
    if idx >= board.len() {
        return Err(SudokuErrors::OutOfBounds);
    }
    let v = board.to_bits(value)?;
    Ok(board.eliminate(idx, v)?)
}

/// Attempt to solve the puzzle given in `board`.
///
/// # Errors
/// This function can return an error if
/// - The puzzle has no solutions.
/// - The puzzle has multiple solutions.
pub fn solve(board: &Board) -> Result<Board, SudokuErrors> {
    let mut slns = board.solutions();
    let Some(result) = slns.next() else {
        return Err(SudokuErrors::Contradiction)
    };

    match slns.next() {
        Some(_) => Err(SudokuErrors::MultipleSolutions),
        None => Ok(result),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assign() {
        let mut board = Board::new(9, 9).unwrap();
        assert_eq!(assign(&mut board, 11, 6), Ok(Elimination::Eliminated));
        assert_eq!(assign(&mut board, 11, 1), Err(SudokuErrors::Contradiction));
        assert_eq!(assign(&mut board, 1111, 6), Err(SudokuErrors::OutOfBounds));
        assert_eq!(assign(&mut board, 21, 16), Err(SudokuErrors::ValueTooLarge));
    }

    #[test]
    fn test_eliminate() {
        let mut board = Board::new(9, 9).unwrap();
        assert_eq!(eliminate(&mut board, 11, 6), Ok(Elimination::Eliminated));
        assign(&mut board, 11, 5).unwrap();
        assert_eq!(
            eliminate(&mut board, 11, 5),
            Err(SudokuErrors::Contradiction)
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

    #[test]
    fn good_from_string() {
        let resp = from_string(
            "1...5.3..9.2..........3.4...8.....4..7..........6..81.6..2.8.........5.7.....1..9",
        );
        assert!(resp.is_ok());
        // let board = resp.unwrap();

        // Naked Singles
        // assert_eq!(board.grid[70], bitarr!(u32, Lsb0; 0,1));
    }

    #[test]
    fn bad_from_string() {
        let resp = from_string("12345678");
        assert!(resp.is_err());
        assert_eq!(resp.unwrap_err(), SudokuErrors::BadSize);
    }
}
