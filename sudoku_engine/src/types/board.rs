//! Types for interacting with a sudoku puzzle.

#![warn(missing_docs)]
use crate::types::iter::SolutionIterator;
use bitvec::array as bit_array;
use core::iter::Iterator;
use core::num::TryFromIntError;
use core::ops::BitAnd;
use core::ops::BitAndAssign;
use core::ops::Not;
use f_puzzles::FPuzzles;
use fmt::Display;
use itertools::Itertools;
use rayon::prelude::*;
use std::fmt;
use std::sync::Arc;
use tokio::sync::mpsc::error::TrySendError;
use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;

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

pub(crate) type Bits = bit_array::BitArray<[u32; 1]>;
type MoreBits = bit_array::BitArray<[u64; 4]>;
pub const MAX_SIZE: usize = 16;

const DIMENSIONS: [(usize, usize); MAX_SIZE] = [
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

#[derive(Debug)]
struct BoardMeta {
    /// The size of a side of the board.
    size: usize,

    /// The maximum value that is used in this sudoku.
    max_val: usize,

    /// In a regular sudoku, these will represent the 9 3x3 boxes. We aren't hardcoding that in
    /// anticipation of irregular sudoku.
    regions: Vec<Vec<usize>>,
}

/// A representation of a sudoku board.
#[derive(Clone, Debug)]
pub struct Board {
    /// Helps us count which values we've used for mean mini puzzles.
    used_digits: Bits,

    /// Indices where we have placed a digit.
    solved_digits: MoreBits,

    /// space to store the data.
    grid: Vec<Bits>,

    /// Data that will remain constant during a solve. When we make a guess and copy a board, this
    /// doesn't need to be copied.
    meta: Arc<BoardMeta>,
}

fn build_default_regions(size: usize) -> Result<Vec<Vec<usize>>, SudokuErrors> {
    let mut ret: Vec<Vec<usize>> = Vec::new();
    ret.resize_with(size, || vec![0; size]);

    if size == 0 || size > DIMENSIONS.len() {
        return Err(SudokuErrors::OutOfBounds);
    }
    let (width, height) = DIMENSIONS[size - 1];

    // If this becomes a bottleneck, we can hack a loop that can be const, make this function const
    // and compute a table of values at compile time.
    // https://stackoverflow.com/a/67941488
    for box_y in 0..width {
        for box_x in 0..height {
            let grid_box = &mut ret[box_y * height + box_x];
            let mut idx = 0;
            for y in 0..height {
                for x in 0..width {
                    grid_box[idx] = (box_y * height + y) * size + box_x * width + x;
                    idx += 1;
                }
            }
        }
    }

    Ok(ret)
}

impl Board {
    /// Generate an empty sudoku grid.
    ///
    /// # Errors
    /// This function can generate an error if either
    /// - `size` is 0
    /// - `size` is greater than `MAX_SIZE`
    /// - `max_val` is less than size.
    /// - `max_val` is greater than 32.
    pub fn new(size: usize, max_val: usize) -> Result<Self, SudokuErrors> {
        Self::new_with_regions(size, max_val, build_default_regions(size)?)
    }

    fn new_with_regions(
        size: usize,
        max_val: usize,
        regions: Vec<Vec<usize>>,
    ) -> Result<Self, SudokuErrors> {
        if max_val < size {
            return Err(SudokuErrors::MaxTooLarge);
        }

        let full = Self::empty_cell(max_val)?;
        let mut grid = vec![Bits::ZERO; size * size];
        grid.fill(full);

        Ok(Board {
            used_digits: Bits::ZERO,
            solved_digits: MoreBits::ZERO,
            grid,
            meta: Arc::new(BoardMeta {
                size,
                max_val,
                regions,
            }),
        })
    }

    pub(crate) fn from_digits(
        size: usize,
        max_val: usize,
        digits: &[Option<Bits>],
    ) -> Result<Self, SudokuErrors> {
        debug_assert_eq!(digits.len(), size * size);
        let mut b = Self::new(size, max_val)?;

        for (i, o) in digits.iter().enumerate() {
            if let Some(d) = o {
                if !(b.grid[i] & d).any() {
                    return Err(SudokuErrors::Contradiction);
                }
                b.assign(i, *d)?;
            }
        }

        Ok(b)
    }

    fn empty_cell(max_val: usize) -> Result<Bits, SudokuErrors> {
        let mut full: Bits = Bits::ZERO;
        if full.len() <= max_val {
            return Err(SudokuErrors::MaxTooLarge);
        }
        for i in 1..=max_val {
            full.set(i, true);
        }
        Ok(full)
    }

    /// Get the length of the underlying data structure. This provides a way to determine if a
    /// particular index is safe to use.
    #[allow(clippy::len_without_is_empty)]
    #[must_use]
    pub fn len(&self) -> usize {
        self.grid.len()
    }

    pub(crate) fn to_bits(&self, value: usize) -> Result<Bits, SudokuErrors> {
        if value > self.meta.max_val {
            return Err(SudokuErrors::ValueTooLarge);
        }
        let mut v = Bits::ZERO;
        debug_assert!(value < v.len());
        v.set(value, true);
        Ok(v)
    }

    /// Checks if a particular digit is still a candidate in cell `idx`.
    #[must_use]
    pub fn possible_value(&self, idx: usize, value: Bits) -> bool {
        debug_assert!(idx < self.len());
        (self.grid[idx] & value).any()
    }

    /// assigns `value` into the grid at `idx`.
    ///
    /// # Panics
    ///
    /// This function will panic in the following situations:
    /// - `value` doesn't represents a single digit.
    /// - `idx` is out of bounds for the grid.
    /// - `value` is not a possibility at `idx`.
    ///
    /// It is the callers responsability to pre verify that none of these conditions exist.
    /// We provide a wrapper [`sudoku_engine::assign`] that performs these checks so that library
    /// users may call this function directly without risking a panic.
    pub(crate) fn assign(&mut self, idx: usize, value: Bits) -> Result<Elimination, Contradiction> {
        debug_assert_eq!(value.count_ones(), 1);
        debug_assert!(idx < self.len());
        debug_assert_eq!(self.grid[idx] & value, value);

        if !self.solved_digits[idx] {
            self.grid[idx] = value;
            self.solved_digits.set(idx, true);
            self.used_digits |= value;
            if self.used_digits.count_ones() > self.meta.size {
                return Err(Contradiction(()));
            }
        }

        let row = self.meta.size * (idx / self.meta.size);
        let column = idx - row;

        let mut ret = Elimination::Same;
        for i in (row..(row + self.meta.size)).filter(move |x| *x != idx) {
            ret &= self.eliminate(i, value)?;
        }

        for i in (column..(self.meta.size * self.meta.size))
            .step_by(self.meta.size)
            .filter(move |x| *x != idx)
        {
            ret &= self.eliminate(i, value)?;
        }

        for region in &self.meta.regions {
            if region.contains(&idx) {
                for cell in region.clone().iter().filter(move |x| **x != idx) {
                    ret &= self.eliminate(*cell, value)?;
                }
                break;
            }
        }

        Ok(ret)
    }

    /// Eliminate digits contained in `value` from the grid at location `idx`.
    ///
    /// # Panics
    ///
    /// This function will panic in the following situations:
    /// - `idx` is out of bounds for the grid.
    ///
    /// It is the callers responsability to pre verify that none of these conditions exist.
    /// We provide a wrapper [`sudoku_engine::eliminate`] that performs these checks so that library
    /// users may call this function directly without risking a panic.
    pub(crate) fn eliminate(
        &mut self,
        idx: usize,
        value: Bits,
    ) -> Result<Elimination, Contradiction> {
        debug_assert!(idx <= self.len());

        if !(self.grid[idx] & value).any() {
            return Ok(Elimination::Same);
        }

        self.grid[idx] &= value.not();
        if self.grid[idx] == Bits::ZERO {
            Err(Contradiction(()))
        } else {
            Ok(Elimination::Eliminated)
        }
    }

    /// Check whether the puzzle is solved.
    #[must_use]
    pub fn solved(&self) -> bool {
        self.solved_digits.count_ones() == self.meta.size * self.meta.size
    }

    /// Search the grid for places where there is only a single candidate.
    ///
    /// # Errors
    /// This will throw an error if searching for naked singles leads to a contradiction.
    pub fn naked_singles(&mut self) -> Result<Elimination, Contradiction> {
        let mut ret = Elimination::Same;

        let temp: Vec<(usize, Bits)> = self
            .grid
            .iter()
            .enumerate()
            .filter(|(i, d)| d.count_ones() == 1 && !self.solved_digits[*i])
            .map(|(i, d)| (i, *d))
            .collect();

        for (i, d) in temp {
            ret &= self.assign(i, d)?;
        }

        Ok(ret)
    }

    fn hidden_singles_helper<T: Iterator<Item = usize>>(
        &mut self,
        unit: T,
    ) -> Result<Elimination, Contradiction> {
        let mut ret = Elimination::Same;
        let cells: Vec<(usize, Bits)> = unit.map(|idx| (idx, self.grid[idx])).collect();
        for i in 1..=self.meta.max_val {
            let mut v = Bits::ZERO;
            v.set(i, true);
            let mut f = cells.iter().filter(|(_, x)| *x & v == v);
            if let Some((idx, _)) = f.next() {
                if f.next().is_none() {
                    ret &= self.assign(*idx, v)?;
                }
            } else {
                return Err(Contradiction(()));
            }
        }

        Ok(ret)
    }

    /// Locate cells that are the only place for a digit to go in a row, column, or region.
    ///
    /// # Errors
    /// This will throw an error if searching for hidden singles leads to a contradiction.
    pub fn hidden_singles(&mut self) -> Result<Elimination, Contradiction> {
        let mut ret = Elimination::Same;
        // rows
        for r in 0..self.meta.size {
            ret &= self.hidden_singles_helper(r * self.meta.size..(r + 1) * self.meta.size)?;
        }
        // columns
        for c in 0..self.meta.size {
            ret &= self.hidden_singles_helper(
                (c..self.meta.size * self.meta.size).step_by(self.meta.size),
            )?;
        }
        // regions
        let regions = self.meta.regions.clone();
        for reg in &regions {
            ret &= self.hidden_singles_helper(reg.iter().copied())?;
        }
        Ok(ret)
    }

    fn naked_tuple_helper<T: Iterator<Item = usize> + Clone>(
        &mut self,
        n: usize,
        unit: &T,
    ) -> Result<Elimination, Contradiction> {
        let mut used_digits = Bits::ZERO;

        for i in unit.clone() {
            let v = self.grid[i];
            if v.count_ones() == 1 {
                used_digits |= v;
            }
        }

        let mut ret = Elimination::Same;
        for vs in (1..=self.meta.max_val)
            .filter(|v| !used_digits[*v])
            .combinations(n)
        {
            let mut digits = Bits::ZERO;
            for v in vs {
                digits.set(v, true);
            }

            let mut indices = Vec::new();
            for i in unit.clone() {
                if self.solved_digits[i] {
                    continue;
                }

                let v = self.grid[i];
                if v & digits == v {
                    indices.push(i);
                }
            }

            if indices.len() == n {
                for i in unit.clone().filter(|i| !indices.contains(i)) {
                    ret &= self.eliminate(i, digits)?;
                }
            }
        }

        Ok(ret)
    }

    /// Locate n cells that can only contain n values, and remove those digits from other seen
    /// cells.
    ///
    /// # Errors
    /// This will throw an error if searching for naked tuples leads to a contradiction.
    pub fn naked_tuples(&mut self, n: usize) -> Result<Elimination, Contradiction> {
        let mut ret = Elimination::Same;
        // rows
        for r in 0..self.meta.size {
            ret &= self.naked_tuple_helper(n, &(r * self.meta.size..(r + 1) * self.meta.size))?;
        }
        // columns
        for c in 0..self.meta.size {
            ret &= self.naked_tuple_helper(
                n,
                &(c..self.meta.size * self.meta.size).step_by(self.meta.size),
            )?;
        }
        // regions
        let regions = self.meta.regions.clone();
        for reg in &regions {
            ret &= self.naked_tuple_helper(n, &reg.iter().copied())?;
        }
        Ok(ret)
    }

    pub(crate) fn next_idx_to_guess(&self) -> Option<usize> {
        let mut count = self.meta.size + 1;
        let mut ret = None;

        for (i, d) in self.grid.iter().enumerate() {
            if d.count_ones() == 1 {
                continue;
            }
            if d.count_ones() < count {
                count = d.count_ones();
                ret = Some(i);
            }
        }
        ret
    }

    pub(crate) fn get_values(&self, idx: usize) -> Bits {
        self.grid[idx]
    }

    pub(crate) fn deduce(&mut self) -> Result<(), Contradiction> {
        loop {
            while self.naked_singles()? == Elimination::Eliminated {}
            if self.hidden_singles()? == Elimination::Eliminated {
                continue;
            }
            if self.naked_tuples(2)? == Elimination::Eliminated {
                continue;
            }
            // if self.naked_tuples(3)? == Elimination::Eliminated {
            //     continue;
            // }
            // if self.naked_tuples(4)? == Elimination::Eliminated {
            //     continue;
            // }
            break;
        }
        Ok(())
    }

    /// An iterator of all possible solutions to the given puzzle.
    #[must_use]
    pub fn solutions(&self) -> SolutionIterator {
        SolutionIterator::new(self)
    }

    fn solution_count_helper(&mut self, token: &CancellationToken, tx: &Sender<usize>) -> usize {
        if token.is_cancelled() {
            return 0;
        }
        if self.deduce().is_err() {
            return 0;
        }
        if self.solved() {
            return 1;
        }
        let Some(idx) = self.next_idx_to_guess() else {
                return 0;
            };
        let values: Vec<usize> = self.get_values(idx).iter_ones().collect();
        let count = values
            .par_iter()
            .fold(
                || 0,
                |acc, v| {
                    let mut board = self.clone();
                    let bit = board.to_bits(*v).unwrap();
                    let n = match board.assign(idx, bit) {
                        Ok(_) => board.solution_count_helper(token, tx),
                        Err(_) => 0,
                    };
                    acc + n
                },
            )
            .sum::<usize>();
        if count > 500 {
            while let Err(TrySendError::Full(_)) = tx.try_send(count) {}
            0
        } else {
            count
        }
    }

    /// count the number of solutions to a puzzle. A partial count is periodically transmitted
    /// through the channel `tx`.
    pub fn solution_count(&mut self, token: &CancellationToken, tx: &Sender<usize>) {
        let count = self.solution_count_helper(token, tx);
        while let Err(TrySendError::Full(_)) = tx.try_send(count) {}
    }
}

fn regions(f: &FPuzzles) -> Vec<Vec<usize>> {
    let (width, height) = DIMENSIONS[f.size - 1];
    let mut ret = vec![Vec::new(); f.size];
    for (r, row) in f.grid.iter().enumerate() {
        let box_r = (r / height) * height;
        for (c, cell) in row.iter().enumerate() {
            let idx = r * f.size + c;
            match cell.region {
                Some(i) => {
                    ret[i].push(idx);
                }
                None => {
                    ret[box_r + (c / width)].push(idx);
                }
            }
        }
    }

    ret
}

impl TryFrom<&FPuzzles> for Board {
    type Error = SudokuErrors;

    fn try_from(f: &FPuzzles) -> Result<Self, SudokuErrors> {
        if f.size == 0 || f.size > DIMENSIONS.len() {
            return Err(SudokuErrors::OutOfBounds);
        }
        let mut ret = if f.is_irregular() {
            let reg = regions(f);
            if reg.iter().any(|x| x.len() != f.size) {
                return Err(SudokuErrors::IrregularWrongSizes);
            }
            Board::new_with_regions(f.size, f.size, reg)?
        } else {
            Board::new(f.size, f.size)?
        };

        for (r, row) in f.grid.iter().enumerate() {
            for (c, cell) in row.iter().enumerate() {
                if let Some(v) = cell.value {
                    ret.assign(r * f.size + c, to_bits(v as usize))?;
                } else if !cell.given_pencil_marks.is_empty() {
                    for v in 1..=ret.meta.max_val {
                        if !cell.given_pencil_marks.contains(&v) {
                            ret.eliminate(r * f.size + c, to_bits(v))?;
                        }
                    }
                }
            }
        }

        Ok(ret)
    }
}

pub(crate) fn to_bits(value: usize) -> Bits {
    let mut v = Bits::ZERO;
    debug_assert!(value < v.len());
    v.set(value, true);
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    use bitvec::bitarr;
    use bitvec::prelude::Lsb0;
    use tokio::sync::mpsc::channel;

    const ONE: Bits = bitarr!(const u32, Lsb0; 0,1);
    const TWO: Bits = bitarr!(const u32, Lsb0; 0,0,1);
    const THREE: Bits = bitarr!(const u32, Lsb0; 0,0,0,1);
    const FOUR: Bits = bitarr!(const u32, Lsb0; 0,0,0,0,1);
    const FIVE: Bits = bitarr!(const u32, Lsb0; 0,0,0,0,0,1);
    const SIX: Bits = bitarr!(const u32, Lsb0; 0,0,0,0,0,0,1);
    const SEVEN: Bits = bitarr!(const u32, Lsb0; 0,0,0,0,0,0,0,1);

    #[test]
    fn parsing_bad_digits() {
        fn bad_digit() -> Result<usize, SudokuErrors> {
            Ok(usize::try_from(-1)?)
        }
        assert_eq!(bad_digit(), Err(SudokuErrors::BadDigit));
    }

    #[test]
    fn test_build_default_regions() {
        let too_small = build_default_regions(0);
        assert!(too_small.is_err());
        assert_eq!(too_small.unwrap_err(), SudokuErrors::OutOfBounds);

        let too_big = build_default_regions(17);
        assert!(too_big.is_err());
        assert_eq!(too_big.unwrap_err(), SudokuErrors::OutOfBounds);

        let just_right = build_default_regions(16);
        assert!(just_right.is_ok());
    }

    #[test]
    fn correct_9_by_9_regions() {
        const CORRECT_REGIONS: &[[usize; 9]] = &[
            [0, 1, 2, 9, 10, 11, 18, 19, 20],
            [3, 4, 5, 12, 13, 14, 21, 22, 23],
            [6, 7, 8, 15, 16, 17, 24, 25, 26],
            [27, 28, 29, 36, 37, 38, 45, 46, 47],
            [30, 31, 32, 39, 40, 41, 48, 49, 50],
            [33, 34, 35, 42, 43, 44, 51, 52, 53],
            [54, 55, 56, 63, 64, 65, 72, 73, 74],
            [57, 58, 59, 66, 67, 68, 75, 76, 77],
            [60, 61, 62, 69, 70, 71, 78, 79, 80],
        ];

        let regions = build_default_regions(9).unwrap();
        for (i, region) in CORRECT_REGIONS.iter().enumerate() {
            for idx in region {
                assert!(regions[i].contains(idx));
            }
        }
    }

    #[test]
    fn good_new() {
        let res = Board::new(9, 9);
        assert!(res.is_ok());
        let board = res.unwrap();
        assert_eq!(board.meta.size, 9);
        assert_eq!(board.meta.max_val, 9);
        assert_eq!(board.used_digits, Bits::ZERO);
        assert_eq!(board.len(), 81);
        for v in board.grid {
            assert_eq!(v.count_ones(), 9);
            assert_eq!(v.get(0).as_deref(), Some(&false));
            for i in 1..=9 {
                assert_eq!(v.get(i).as_deref(), Some(&true));
            }
        }
    }

    #[test]
    fn bad_new() {
        let mut res = Board::new(9, 100);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), SudokuErrors::MaxTooLarge);

        res = Board::new(16, 9);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), SudokuErrors::MaxTooLarge);
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
    fn possible_value() {
        let mut board = Board::new(9, 9).unwrap();
        let value = board.to_bits(5).unwrap();
        assert!(board.possible_value(65, value));
        assert_eq!(board.eliminate(65, value), Ok(Elimination::Eliminated));
        assert!(!board.possible_value(65, value));
    }

    #[test]
    fn eliminate_one() {
        let mut board = Board::new(9, 9).unwrap();
        let value = board.to_bits(6).unwrap();
        assert_eq!(board.eliminate(11, value), Ok(Elimination::Eliminated));
        assert_eq!(board.eliminate(11, value), Ok(Elimination::Same));
    }

    #[test]
    fn eliminate_multiple() {
        let mut board = Board::new(9, 9).unwrap();
        let mut value = board.to_bits(6).unwrap();
        assert_eq!(board.eliminate(11, value), Ok(Elimination::Eliminated));
        value.set(2, true);
        assert_eq!(board.eliminate(11, value), Ok(Elimination::Eliminated));
        assert_eq!(board.eliminate(11, value), Ok(Elimination::Same));
    }

    #[test]
    fn assign() {
        let mut board = Board::new(9, 9).unwrap();
        let value = board.to_bits(6).unwrap();
        assert_eq!(board.assign(11, value), Ok(Elimination::Eliminated));
        assert_eq!(board.assign(11, value), Ok(Elimination::Same));
        let sees = [
            0, 1, 2, 9, 10, 12, 13, 14, 15, 16, 17, 18, 19, 20, 29, 38, 47, 56, 65, 74,
        ];
        for i in sees {
            assert!((board.grid[i] & value).not_any());
        }
        for i in (0..81).filter(|x| !sees.contains(x)) {
            assert!((board.grid[i] & value).any());
        }
    }

    #[test]
    fn eliminate_after_assign() {
        let mut board = Board::new(9, 9).unwrap();
        let value = board.to_bits(6).unwrap();
        assert_eq!(board.assign(11, value), Ok(Elimination::Eliminated));
        assert_eq!(board.eliminate(11, value), Err(Contradiction(())));
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

    #[test]
    fn from_digits() {
        let mut digits: Vec<Option<Bits>> = vec![
            Some(ONE),
            None,
            None,
            None,
            None,
            Some(TWO),
            None,
            None,
            None,
            None,
            Some(SIX),
            None,
            None,
            None,
            Some(TWO),
            None,
            None,
            None,
            None,
            Some(SIX),
            None,
            None,
            None,
            Some(THREE),
            None,
            None,
            None,
            Some(FIVE),
            None,
            Some(ONE),
            None,
            Some(FOUR),
            None,
            None,
            None,
            None,
        ];

        assert_eq!(digits.len(), 36);
        let response = Board::from_digits(6, 6, digits.as_ref());
        assert!(response.is_ok());
        let mut board = response.unwrap();

        assert_eq!(board.grid[0], ONE);
        assert_eq!(board.grid[5], TWO);
        assert_eq!(board.grid[10], SIX);
        assert_eq!(board.grid[14], TWO);
        assert_eq!(board.grid[19], SIX);
        assert_eq!(board.grid[23], THREE);
        assert_eq!(board.grid[27], FIVE);
        assert_eq!(board.grid[29], ONE);
        assert_eq!(board.grid[31], FOUR);

        // Naked single
        assert_eq!(board.grid[35], SIX);
        assert_eq!(board.naked_singles(), Ok(Elimination::Eliminated));
        assert!(board.solved_digits[35]);
        assert!(!board.solved());

        digits[6] = Some(SIX);
        let err = Board::from_digits(6, 6, digits.as_ref());
        assert!(err.is_err());
        assert_eq!(err.unwrap_err(), SudokuErrors::Contradiction);
    }

    #[test]
    fn too_many_digits() {
        let mut board = Board::new(6, 9).unwrap();
        assert_eq!(board.assign(0, ONE), Ok(Elimination::Eliminated));
        assert_eq!(board.assign(1, TWO), Ok(Elimination::Eliminated));
        assert_eq!(board.assign(2, THREE), Ok(Elimination::Eliminated));
        assert_eq!(board.assign(3, FOUR), Ok(Elimination::Eliminated));
        assert_eq!(board.assign(4, FIVE), Ok(Elimination::Eliminated));
        assert_eq!(board.assign(5, SIX), Ok(Elimination::Eliminated));
        assert_eq!(board.assign(6, SEVEN), Err(Contradiction(())));
    }

    #[test]
    fn bits_size() {
        let x = Bits::ZERO;
        assert_eq!(x.len(), 32);
        let y = MoreBits::ZERO;
        assert_eq!(y.len(), MAX_SIZE * MAX_SIZE);
    }

    #[test]
    fn next_idx_to_guess() {
        let mut board = Board::new(6, 6).unwrap();
        assert_eq!(board.assign(0, ONE), Ok(Elimination::Eliminated));
        assert_eq!(board.assign(1, TWO), Ok(Elimination::Eliminated));
        assert_eq!(board.assign(2, THREE), Ok(Elimination::Eliminated));
        assert_eq!(board.assign(13, FOUR), Ok(Elimination::Eliminated));

        // At this point cell 7 is the most constrained as it sees all of the placed digits.
        assert_eq!(board.next_idx_to_guess(), Some(7));
    }

    #[test]
    fn get_values() {
        let mut board = Board::new(6, 6).unwrap();
        assert_eq!(board.assign(0, ONE), Ok(Elimination::Eliminated));
        assert_eq!(board.assign(1, TWO), Ok(Elimination::Eliminated));
        assert_eq!(board.assign(2, THREE), Ok(Elimination::Eliminated));
        for i in 0..36 {
            assert_eq!(board.get_values(i), board.grid[i]);
        }
    }

    #[test]
    fn irregular_regions() {
        let mut f = FPuzzles::new(9);
        f.grid[0][3].region = Some(0);
        f.grid[2][2].region = Some(1);
        let regions = regions(&f);
        assert_eq!(regions[0], vec![0, 1, 2, 3, 9, 10, 11, 18, 19]);
        assert_eq!(regions[1], vec![4, 5, 12, 13, 14, 20, 21, 22, 23]);
    }

    #[test]
    fn from_f_puzzles() {
        let mut f = FPuzzles::new(9);
        f.grid[1][3].value = Some(1);
        f.grid[4][4].value = Some(5);
        f.grid[7][8].given_pencil_marks = vec![1, 2, 3];
        let res_b = Board::try_from(&f);
        println!("{res_b:?}");
        assert!(res_b.is_ok());
        let b = res_b.unwrap();
        assert_eq!(b.grid[12], ONE);
        assert_eq!(b.grid[40], FIVE);
        assert_eq!(b.grid[71], bitarr!(u32, Lsb0; 0,1,1,1));
    }

    #[test]
    fn from_f_puzzles_too_big() {
        let f = FPuzzles::new(17);
        let res_b = Board::try_from(&f);
        assert!(res_b.is_err());
        assert_eq!(res_b.unwrap_err(), SudokuErrors::OutOfBounds);
    }

    #[test]
    fn from_f_puzzles_irregular() {
        let mut f = FPuzzles::new(9);
        f.grid[0][3].region = Some(0);
        let mut res_b = Board::try_from(&f);
        assert!(res_b.is_err());
        assert_eq!(res_b.unwrap_err(), SudokuErrors::IrregularWrongSizes);

        f.grid[2][2].region = Some(1);
        res_b = Board::try_from(&f);
        assert!(res_b.is_ok());
        let b = res_b.unwrap();
        assert_eq!(b.meta.regions[0], vec![0, 1, 2, 3, 9, 10, 11, 18, 19]);
        assert_eq!(b.meta.regions[1], vec![4, 5, 12, 13, 14, 20, 21, 22, 23]);
    }

    #[test]
    fn solution_count() {
        let b = crate::from_string(
            ".9..7..5....28..........37.2.5.....4...4.5.....6.....9731....2....82.....4....91.",
        );
        assert!(b.is_ok());
        let mut board = b.unwrap();
        let token = CancellationToken::new();
        let (ch_tx, mut ch_rx) = channel::<usize>(1);
        rayon::scope(move |_| {
            board.solution_count(&token, &ch_tx);
        });
        let response = ch_rx.try_recv();
        assert!(response.is_ok());
        assert_eq!(response.unwrap(), 38);
    }

    #[cfg_attr(tarpaulin, ignore)]
    #[test]
    fn solution_count_with_midcount_reporting() {
        let b = crate::from_string(
            "19..7..5.....8..........37.2.5.....4...4.5.....6.....97.1....2....82.....4....91.",
        );
        assert!(b.is_ok());
        let mut board = b.unwrap();
        let token = CancellationToken::new();
        let (ch_tx, mut ch_rx) = channel::<usize>(10);
        board.solution_count(&token, &ch_tx);
        let mut count = 0;
        while let Ok(n) = ch_rx.try_recv() {
            count += n;
        }
        assert_eq!(count, 753);
    }

    #[test]
    fn solution_count_with_cancellation() {
        let b = crate::from_string(
            "19..7..5.....8..........37.2.5.....4...4.5.....6.....97.1....2....82.....4....91.",
        );
        assert!(b.is_ok());
        let mut board = b.unwrap();
        let token = CancellationToken::new();
        let (ch_tx, mut ch_rx) = channel::<usize>(10);
        token.cancel();
        board.solution_count(&token, &ch_tx);
        let mut count = 0;
        while let Ok(n) = ch_rx.try_recv() {
            count += n;
        }
        assert_eq!(count, 0);
    }

    #[test]
    fn test_deduce() {
        let b = crate::from_string(
            "19..7..5....28..........37.2.5.....4...4.5.....6.....9731....2....82.....4....91.",
        );
        let mut board = b.unwrap();
        let res = board.deduce();
        eprintln!("{board:?}");
        assert!(res.is_ok());
    }
}
