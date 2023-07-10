//! Implementations related to sudoku boards.

use crate::constraints;
use crate::types::{
    to_bits, Bits, Board, BoardMeta, Constraint, Contradiction, Elimination, MoreBits,
    SudokuErrors, DIMENSIONS,
};
use core::iter::Iterator;
use core::ops::BitOrAssign;
use f_puzzles::FPuzzles;
use itertools::Itertools;
use rayon::prelude::*;
use solution_iter::SolutionIterator;
use solution_iter::Solvable;
use std::fmt::Display;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::TrySendError;
use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{self:?}")
    }
}

impl BitOrAssign for Board {
    fn bitor_assign(&mut self, rhs: Self) {
        for (i, v) in self.grid.iter_mut().enumerate() {
            *v |= rhs.grid[i];
        }
    }
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
        Self::new_with_regions(size, max_val, build_default_regions(size)?, Vec::new())
    }

    fn new_with_constraints(
        size: usize,
        max_val: usize,
        constraints: Vec<Constraint>,
    ) -> Result<Self, SudokuErrors> {
        Self::new_with_regions(size, max_val, build_default_regions(size)?, constraints)
    }

    pub(crate) fn new_with_regions(
        size: usize,
        max_val: usize,
        regions: Vec<Vec<usize>>,
        constraints: Vec<Constraint>,
    ) -> Result<Self, SudokuErrors> {
        if max_val < size {
            return Err(SudokuErrors::MaxTooLarge);
        }

        let full = Self::empty_cell(max_val)?;
        let mut grid = vec![0; size * size];
        grid.fill(full);

        let mut rows = Vec::with_capacity(size);
        for r in 0..size {
            rows.push((r * size..(r + 1) * size).collect());
        }

        let mut columns = Vec::with_capacity(size);
        for c in 0..size {
            columns.push((c..size * size).step_by(size).collect());
        }

        let mut b = Board {
            used_digits: 0,
            solved_digits: MoreBits::ZERO,
            grid,
            meta: Arc::new(BoardMeta {
                size,
                max_val,
                rows,
                columns,
                regions,
                constraints,
            }),
        };

        let mut init_status = Elimination::Eliminated;
        let meta = b.meta.clone();
        while init_status == Elimination::Eliminated {
            init_status = Elimination::Same;
            for c in &meta.constraints {
                init_status &= b.init_constraint(c)?;
            }
        }

        Ok(b)
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
                if b.grid[i] & d == 0 {
                    return Err(SudokuErrors::Contradiction);
                }
                b.assign(i, *d)?;
            }
        }

        Ok(b)
    }

    pub(crate) fn iter_ones(&self, idx: usize) -> Vec<usize> {
        let mut ret = Vec::with_capacity(self.meta.max_val);
        let value = self.grid[idx];
        for i in 1..=self.meta.max_val {
            if 1 << i & value != 0 {
                ret.push(i);
            }
        }

        ret
    }

    /// The boils down to left shifting and subtracting. It is off by one from what is expected,
    /// because we are not using the 0th bit.
    /// eq. If `max_val` is 9, then we are doing 1 << 10 - 2 = `0111_1111_11`
    fn empty_cell(max_val: usize) -> Result<Bits, SudokuErrors> {
        let mut full: Bits = 1;
        if usize::BITS as usize <= max_val {
            return Err(SudokuErrors::MaxTooLarge);
        }
        full <<= max_val + 1;
        full -= 2;

        Ok(full)
    }

    /// Get the length of the underlying data structure. This provides a way to determine if a
    /// particular index is safe to use.
    #[allow(clippy::len_without_is_empty)]
    #[must_use]
    pub fn len(&self) -> usize {
        self.grid.len()
    }

    /// Get the size of one side of the sudoku.
    #[must_use]
    pub fn size(&self) -> usize {
        self.meta.size
    }

    pub(crate) fn to_bits(&self, value: usize) -> Result<Bits, SudokuErrors> {
        if value > self.meta.max_val {
            return Err(SudokuErrors::ValueTooLarge);
        }
        let v = 1 << value;
        Ok(v)
    }

    /// Checks if a particular digit is still a candidate in cell `idx`.
    #[must_use]
    pub fn possible_value(&self, idx: usize, value: Bits) -> bool {
        debug_assert!(idx < self.len());
        self.grid[idx] & value != 0
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
            if self.used_digits.count_ones() as usize > self.meta.size {
                return Err(Contradiction(()));
            }
        }

        let row = idx / self.meta.size;
        let column = idx - self.meta.size * row;

        let meta = self.meta.clone();
        let mut ret = Elimination::Same;
        for i in &meta.rows[row] {
            if *i == idx {
                continue;
            }
            ret &= self.eliminate(*i, value)?;
        }

        for i in &meta.columns[column] {
            if *i == idx {
                continue;
            }
            ret &= self.eliminate(*i, value)?;
        }

        for region in &meta.regions {
            if region.contains(&idx) {
                for cell in region {
                    if *cell == idx {
                        continue;
                    }
                    ret &= self.eliminate(*cell, value)?;
                }
                break;
            }
        }

        for c in &meta.constraints {
            ret &= self.enforce_constraint_consistency(c, idx, value)?;
        }

        Ok(ret)
    }

    /// Eliminate digits contained in `value` from the grid at location `idx`.
    ///
    /// This function should do nothing except call `types::eliminate`. That function is called on
    /// grids during board construction, and should be consistent with this method.
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
        crate::types::eliminate(idx, value, &mut self.grid)
    }

    #[must_use]
    fn check_constraint(&self, c: &Constraint) -> bool {
        match c {
            Constraint::Quad(idx, single, double) => {
                constraints::check_quad(*idx, *single, *double, self.meta.size, &self.grid)
            }
            Constraint::Region(region) => constraints::check_region(region, &self.grid),
        }
    }

    fn init_constraint(&mut self, c: &Constraint) -> Result<Elimination, Contradiction> {
        match c {
            Constraint::Quad(idx, single, double) => {
                constraints::init_quad(*idx, *single, *double, self.meta.size, &mut self.grid)
            }
            Constraint::Region(_) => Ok(Elimination::Same),
        }
    }

    fn enforce_constraint_consistency(
        &mut self,
        c: &Constraint,
        idx: usize,
        value: Bits,
    ) -> Result<Elimination, Contradiction> {
        match c {
            Constraint::Quad(quad_idx, single, double) => constraints::quad_enforce_consistency(
                idx,
                *quad_idx,
                *single,
                *double,
                self.meta.size,
                &mut self.grid,
            ),
            Constraint::Region(region) => {
                constraints::region_enforce_consistency(idx, value, region, &mut self.grid)
            }
        }
    }

    fn get_additional_region_from_constraint<'a>(
        meta: &'a Arc<BoardMeta>,
        c: &'a Constraint,
    ) -> Option<&'a [usize]> {
        match c {
            Constraint::Quad(_, _, _) => None,
            Constraint::Region(region) => {
                if region.len() == meta.size {
                    Some(region)
                } else {
                    None
                }
            }
        }
    }

    /// Check whether the puzzle is solved.
    #[must_use]
    pub fn solved(&self) -> bool {
        if self.solved_digits.count_ones() != self.meta.size * self.meta.size {
            return false;
        }

        for c in &self.meta.constraints {
            if !self.check_constraint(c) {
                return false;
            }
        }

        true
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

    fn hidden_singles_helper(&mut self, unit: &[usize]) -> Result<Elimination, Contradiction> {
        let mut ret = Elimination::Same;
        let cells: Vec<(usize, Bits)> = unit.iter().map(|idx| (*idx, self.grid[*idx])).collect();
        for i in 1..=self.meta.max_val {
            let v = 1 << i;
            let mut f = cells.iter().filter(|(_, x)| *x & v == v);
            if let Some((idx, _)) = f.next() {
                if f.next().is_none() {
                    if self.grid[*idx] & v == 0 {
                        // Apparently, we've already removed v from this cell since we cached it
                        // above. So, we've got a contradiction.
                        return Err(Contradiction(()));
                    }
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
        let meta = self.meta.clone();
        // rows
        for r in &meta.rows {
            ret &= self.hidden_singles_helper(r)?;
        }
        // columns
        for c in &meta.columns {
            ret &= self.hidden_singles_helper(c)?;
        }
        // regions
        for reg in &meta.regions {
            ret &= self.hidden_singles_helper(reg)?;
        }
        // Additional regions from constraints.
        for c in &meta.constraints {
            if let Some(reg) = Self::get_additional_region_from_constraint(&meta, c) {
                ret &= self.hidden_singles_helper(reg)?;
            }
        }
        Ok(ret)
    }

    fn naked_tuple_helper(
        &mut self,
        n: usize,
        unit: &[usize],
    ) -> Result<Elimination, Contradiction> {
        let mut used_digits = 0;

        for i in unit {
            let v = self.grid[*i];
            if v.count_ones() == 1 {
                used_digits |= v;
            }
        }

        let mut ret = Elimination::Same;
        for vs in (1..=self.meta.max_val)
            .filter(|v| (1 << *v) & used_digits == 0)
            .combinations(n)
        {
            let mut digits = 0;
            for v in vs {
                digits |= 1 << v;
            }

            let mut indices = Vec::new();
            for i in unit {
                if self.solved_digits[*i] {
                    continue;
                }

                let v = self.grid[*i];
                if v & digits == v {
                    indices.push(*i);
                }
            }

            if indices.len() == n {
                for i in unit {
                    if indices.contains(i) {
                        continue;
                    }
                    ret &= self.eliminate(*i, digits)?;
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
        let meta = self.meta.clone();
        // rows
        for r in &meta.rows {
            ret &= self.naked_tuple_helper(n, r)?;
        }
        // columns
        for c in &meta.columns {
            ret &= self.naked_tuple_helper(n, c)?;
        }
        // regions
        for reg in &meta.regions {
            ret &= self.naked_tuple_helper(n, reg)?;
        }
        // Additional regions from constraints.
        for c in &meta.constraints {
            if let Some(reg) = Self::get_additional_region_from_constraint(&meta, c) {
                ret &= self.naked_tuple_helper(n, reg)?;
            }
        }
        Ok(ret)
    }

    /*
    #[allow(dead_code)]
    fn hidden_tuples_helper<T: Iterator<Item = usize>>(
        &mut self,
        _n: usize,
        _unit: &T,
    ) -> Result<Elimination, Contradiction> {
        let ret = Elimination::Same;
        Ok(ret)
    }
    */

    pub(crate) fn next_idx_to_guess(&self) -> Option<usize> {
        let mut count = self.meta.size + 1;
        let mut ret = None;

        for (i, d) in self.grid.iter().enumerate() {
            if d.count_ones() == 1 {
                continue;
            }
            if (d.count_ones() as usize) < count {
                count = d.count_ones() as usize;
                ret = Some(i);
            }
        }
        ret
    }

    #[allow(dead_code)]
    pub(crate) fn get_values(&self, idx: usize) -> Bits {
        self.grid[idx]
    }

    pub(crate) fn deduce(&mut self) -> Result<(), Contradiction> {
        loop {
            while self.naked_singles()? == Elimination::Eliminated {}
            if self.hidden_singles()? == Elimination::Eliminated {
                continue;
            }
            if self.solved() {
                break;
            }
            // if self.naked_tuples(2)? == Elimination::Eliminated {
            //     continue;
            // }
            // if self.solved() {
            //     break;
            // }
            // if self.naked_tuples(3)? == Elimination::Eliminated {
            //     continue;
            // }
            // if self.solved() {
            //     break;
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
    pub fn solutions(&self) -> SolutionIterator<Board> {
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
        let count = self
            .iter_ones(idx)
            .par_iter()
            .panic_fuse()
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
            while let Err(TrySendError::Full(_)) = tx.try_send(count) {
                if token.is_cancelled() {
                    return count;
                }
            }
            0
        } else {
            count
        }
    }

    /// Count the number of solutions to a puzzle. A partial count is periodically transmitted
    /// through the channel `tx`.
    pub fn solution_count(&mut self, token: &CancellationToken, tx: &Sender<usize>) {
        let count = self.solution_count_helper(token, tx);
        while let Err(TrySendError::Full(_)) = tx.try_send(count) {
            if token.is_cancelled() {
                break;
            }
        }
    }

    /// Count the number of solutions to a puzzle and return the result. Computation is cancelled
    /// when `max_count` is reached.
    #[tokio::main(flavor = "current_thread")]
    pub async fn solution_count_max(&mut self, max_count: usize) -> usize {
        let (tx, mut rx) = mpsc::channel::<usize>(100);
        let token = CancellationToken::new();
        let mut b = self.clone();
        let token_clone = token.clone();
        rayon::spawn(move || {
            b.solution_count(&token_clone, &tx);
        });
        let mut count = 0;
        while let Some(n) = rx.recv().await {
            count += n;
            if count > max_count {
                token.cancel();
                break;
            }
        }
        while let Some(n) = rx.recv().await {
            count += n;
        }
        count
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

pub(crate) fn rc_to_idx(s: &str, size: usize) -> Result<usize, SudokuErrors> {
    let Some(offset) = s.find('C') else {
        return Err(SudokuErrors::BadRCEncoding);
    };
    let (r_row, c_col) = s.split_at(offset);
    let Some(row) = r_row.strip_prefix('R') else {
        return Err(SudokuErrors::BadRCEncoding);
    };
    let Some(col) = c_col.strip_prefix('C') else {
        return Err(SudokuErrors::BadRCEncoding);
    };
    let Ok(x) = col.parse::<usize>() else {
        return Err(SudokuErrors::BadRCEncoding);
    };
    let Ok(y) = row.parse::<usize>() else {
        return Err(SudokuErrors::BadRCEncoding);
    };
    if x > size {
        return Err(SudokuErrors::BadRCEncoding);
    };
    if y > size {
        return Err(SudokuErrors::BadRCEncoding);
    };

    Ok((y - 1) * size + (x - 1))
}

impl TryFrom<&FPuzzles> for Board {
    type Error = SudokuErrors;

    fn try_from(f: &FPuzzles) -> Result<Self, SudokuErrors> {
        if f.size == 0 || f.size > DIMENSIONS.len() {
            return Err(SudokuErrors::OutOfBounds);
        }

        let mut constraints = Vec::new();

        for q in &f.quadruple {
            let idx = rc_to_idx(&q.cells[0], f.size)?;
            let mut single = 0;
            let mut double = 0;
            for v in &q.values {
                let bits = to_bits(*v);
                if bits & double != 0 {
                    return Err(SudokuErrors::Contradiction);
                }
                double |= single & bits;
                single ^= bits;
            }
            constraints.push(Constraint::Quad(idx, single, double));
        }

        for r in &f.extraregion {
            let mut region = Vec::with_capacity(f.size);
            for c in &r.cells {
                region.push(rc_to_idx(c, f.size)?);
            }
            constraints.push(Constraint::Region(region));
        }

        if f.negative_diagonal {
            let mut region = Vec::with_capacity(f.size);
            for i in 0..f.size {
                region.push(i * (f.size + 1));
            }
            constraints.push(Constraint::Region(region));
        }

        if f.positive_diagonal {
            let mut region = Vec::with_capacity(f.size);
            for i in 0..f.size {
                region.push((i + 1) * (f.size - 1));
            }
            constraints.push(Constraint::Region(region));
        }

        let mut ret = if f.is_irregular() {
            let reg = regions(f);
            if reg.iter().any(|x| x.len() != f.size) {
                return Err(SudokuErrors::IrregularWrongSizes);
            }
            Board::new_with_regions(f.size, f.size, reg, constraints)?
        } else {
            Board::new_with_constraints(f.size, f.size, constraints)?
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

impl Solvable for Board {
    type Guess = usize;

    fn assign(&mut self, next_idx: usize, guess: Self::Guess) -> bool {
        self.assign(next_idx, guess).is_ok()
    }

    fn deduce(&mut self) -> bool {
        self.deduce().is_ok()
    }

    fn guesses(&self, g: usize) -> Vec<Self::Guess> {
        self.iter_ones(g)
            .iter()
            .map(|i| self.to_bits(*i).unwrap())
            .collect()
    }

    fn next_idx_to_guess(&self) -> Option<usize> {
        self.next_idx_to_guess()
    }

    fn solved(&self) -> bool {
        self.solved()
    }

    fn indices(&self) -> Vec<usize> {
        (0..self.grid.len()).collect()
    }

    fn possibility(&self, idx: usize, g: <Self as Solvable>::Guess) -> bool {
        self.grid[idx] & g != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use tokio::sync::mpsc::channel;
    use tokio_util::sync::CancellationToken;

    use crate::from_string;

    const ONE: Bits = 1 << 1;
    const TWO: Bits = 1 << 2;
    const THREE: Bits = 1 << 3;
    const FOUR: Bits = 1 << 4;
    const FIVE: Bits = 1 << 5;
    const SIX: Bits = 1 << 6;
    const SEVEN: Bits = 1 << 7;
    const EIGHT: Bits = 1 << 8;
    const NINE: Bits = 1 << 9;

    #[test]
    fn good_new() {
        let res = Board::new(9, 9);
        assert!(res.is_ok());
        let board = res.unwrap();
        assert_eq!(board.meta.size, 9);
        assert_eq!(board.meta.max_val, 9);
        assert_eq!(board.used_digits, 0);
        assert_eq!(board.len(), 81);
        for v in board.grid {
            assert_eq!(v.count_ones(), 9);
            assert_eq!(v & 1 << 0, 0);
            for i in 1..=9 {
                assert_ne!(v & 1 << i, 0);
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
        value |= 1 << 2;
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
            assert_eq!(board.grid[i] & value, 0);
        }
        for i in (0..81).filter(|x| !sees.contains(x)) {
            assert_ne!(board.grid[i] & value, 0);
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
        assert_eq!(b.grid[71], 14);
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
    fn from_f_puzzles_quad() {
        let mut f = FPuzzles::new(9);
        f.quadruple.push(f_puzzles::Quad {
            cells: [
                "R1C1".to_string(),
                "R1C2".to_string(),
                "R2C1".to_string(),
                "R2C2".to_string(),
            ],
            values: vec![5, 6],
        });
        let res_b = Board::try_from(&f);
        assert!(res_b.is_ok());
        let b = res_b.unwrap();
        assert_eq!(b.meta.constraints[0], Constraint::Quad(0, FIVE | SIX, 0));
    }

    #[test]
    fn from_f_puzzles_extraregion() {
        let mut f = FPuzzles::new(9);
        f.extraregion.push(f_puzzles::Region {
            cells: vec![
                "R2C2".to_string(),
                "R2C3".to_string(),
                "R2C4".to_string(),
                "R3C2".to_string(),
                "R3C3".to_string(),
                "R3C4".to_string(),
                "R4C2".to_string(),
                "R4C3".to_string(),
                "R4C4".to_string(),
            ],
        });
        let res_b = Board::try_from(&f);
        assert!(res_b.is_ok());
    }

    #[test]
    fn from_f_puzzles_quad_repeated_digit_in_solution() {
        let mut f = FPuzzles::new(6);
        f.quadruple.push(f_puzzles::Quad {
            cells: [
                "R4C5".to_string(),
                "R4C6".to_string(),
                "R5C5".to_string(),
                "R562".to_string(),
            ],
            values: vec![1, 5],
        });
        f.quadruple.push(f_puzzles::Quad {
            cells: [
                "R4C2".to_string(),
                "R4C3".to_string(),
                "R5C2".to_string(),
                "R5C3".to_string(),
            ],
            values: vec![1, 3, 5, 6],
        });
        f.quadruple.push(f_puzzles::Quad {
            cells: [
                "R3C1".to_string(),
                "R3C2".to_string(),
                "R4C1".to_string(),
                "R4C2".to_string(),
            ],
            values: vec![1, 3, 5],
        });
        f.quadruple.push(f_puzzles::Quad {
            cells: [
                "R2C4".to_string(),
                "R2C5".to_string(),
                "R3C4".to_string(),
                "R3C5".to_string(),
            ],
            values: vec![5, 6],
        });
        f.quadruple.push(f_puzzles::Quad {
            cells: [
                "R2C3".to_string(),
                "R2C4".to_string(),
                "R3C3".to_string(),
                "R3C4".to_string(),
            ],
            values: vec![1, 2, 3, 6],
        });
        f.quadruple.push(f_puzzles::Quad {
            cells: [
                "R1C5".to_string(),
                "R1C6".to_string(),
                "R2C5".to_string(),
                "R2C6".to_string(),
            ],
            values: vec![3, 4],
        });

        let res_b = Board::try_from(&f);
        assert!(res_b.is_ok());
        let mut b = res_b.unwrap();

        assert!(b.assign(4, FOUR).is_ok());
        assert!(b.deduce().is_ok());

        assert!(b.assign(8, ONE).is_ok());
        assert!(b.deduce().is_ok());

        assert!(b.assign(9, SIX).is_ok());
        assert!(b.deduce().is_ok());

        assert!(b.assign(10, FIVE).is_ok());
        assert!(b.deduce().is_ok());

        assert!(b.assign(11, THREE).is_ok());
        assert!(b.deduce().is_ok());

        assert!(b.assign(3, ONE).is_ok());
        assert!(b.deduce().is_ok());

        assert!(b.assign(6, TWO).is_ok());
        assert!(b.deduce().is_ok());

        assert!(b.assign(16, SIX).is_ok());
        assert!(b.deduce().is_ok());

        eprintln!("{b:?}");
        let token = CancellationToken::new();
        let tc = solution_iter::true_candidates_bfs(&b, &token);
        assert!(tc.is_some());
        eprintln!("{tc:?}");
    }

    #[test]
    fn solution_iter() {
        let res = from_string(
            "1.2........62.3.........3.454..6........5.9......1.76..87.........9.8.........1.9",
        );
        assert!(res.is_ok());
        let board = res.unwrap();
        let iter = SolutionIterator::new(&board);
        assert_eq!(iter.count(), 78);
    }

    #[test]
    fn solution_iter_for_contradiction() {
        let res = from_string(
            "152........62.3.........3.454..6........5.9......1.76..87.........9.8.........1.9",
        );
        assert!(res.is_ok());
        let board = res.unwrap();
        let iter = SolutionIterator::new(&board);
        assert_eq!(iter.count(), 0);
    }

    #[test]
    fn solution_iter_for_initially_solved() {
        let res = from_string(
            "152........62.3.........3.454..6........5.9......1.76..87.........9.8.........1.9",
        );
        assert!(res.is_ok());
        let board = res.unwrap();
        let iter = SolutionIterator::new(&board);
        assert_eq!(iter.count(), 0);
    }

    #[test]
    fn test_rc_to_idx() {
        assert_eq!(rc_to_idx("R1C1", 9), Ok(0));
        assert_eq!(rc_to_idx("R16C16", 16), Ok(255));

        assert_eq!(rc_to_idx("R11", 9), Err(SudokuErrors::BadRCEncoding));
        assert_eq!(rc_to_idx("1C1", 9), Err(SudokuErrors::BadRCEncoding));
        assert_eq!(rc_to_idx("RoneC1", 9), Err(SudokuErrors::BadRCEncoding));
        assert_eq!(rc_to_idx("R1Cone", 9), Err(SudokuErrors::BadRCEncoding));
        assert_eq!(rc_to_idx("R16C3", 9), Err(SudokuErrors::BadRCEncoding));
        assert_eq!(rc_to_idx("R3C16", 9), Err(SudokuErrors::BadRCEncoding));
        assert_eq!(rc_to_idx("", 9), Err(SudokuErrors::BadRCEncoding));
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
    fn solution_count() {
        let b = from_string(
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

    #[test]
    fn solution_count_with_midcount_reporting() {
        let b = from_string(
            "19..7..5.....8..........37.2.5.....4...4.5.....6.....97.1....2....82.....4....91.",
        );
        assert!(b.is_ok());
        let mut board = b.unwrap();
        let token = CancellationToken::new();
        let (ch_tx, mut ch_rx) = channel::<usize>(10);
        rayon::scope(move |_| {
            board.solution_count(&token, &ch_tx);
        });
        let mut count = 0;
        while let Ok(n) = ch_rx.try_recv() {
            count += n;
        }
        assert_eq!(count, 753);
    }

    #[test]
    fn solution_count_with_cancellation() {
        let b = from_string(
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
        let b = from_string(
            "19..7..5....28..........37.2.5.....4...4.5.....6.....9731....2....82.....4....91.",
        );
        let mut board = b.unwrap();
        let res = board.deduce();
        eprintln!("{board:?}");
        assert!(res.is_ok());
    }

    #[test]
    fn test_indices() {
        for n in 4..=16 {
            let b = Board::new(n, n).unwrap();
            assert_eq!(b.indices().len(), n * n);
        }
    }

    #[test]
    fn naked_tuples() {
        let res = from_string(
            "1234..5.........67...............................................................",
        );
        assert!(res.is_ok());
        let mut board = res.unwrap();
        assert_eq!(board.naked_tuples(2), Ok(Elimination::Eliminated));
        assert!(!board.possibility(4, board.to_bits(8).unwrap()));
        assert!(!board.possibility(4, board.to_bits(9).unwrap()));
        assert!(!board.possibility(15, board.to_bits(8).unwrap()));
        assert!(!board.possibility(15, board.to_bits(9).unwrap()));

        // It's an implementation detail that we don't want to test if this is caught in the first
        // or second run of naked_tuples.
        let _ = board.naked_tuples(2);
        assert!(!board.possibility(13, board.to_bits(6).unwrap()));
        assert!(!board.possibility(13, board.to_bits(7).unwrap()));
    }

    #[test]
    fn windoku() {
        let mut f = FPuzzles::new(9);
        f.extraregion.push(f_puzzles::Region {
            cells: vec![
                "R2C2".to_string(),
                "R2C3".to_string(),
                "R2C4".to_string(),
                "R3C2".to_string(),
                "R3C3".to_string(),
                "R3C4".to_string(),
                "R4C2".to_string(),
                "R4C3".to_string(),
                "R4C4".to_string(),
            ],
        });
        f.extraregion.push(f_puzzles::Region {
            cells: vec![
                "R2C6".to_string(),
                "R2C7".to_string(),
                "R2C8".to_string(),
                "R3C6".to_string(),
                "R3C7".to_string(),
                "R3C8".to_string(),
                "R4C6".to_string(),
                "R4C7".to_string(),
                "R4C8".to_string(),
            ],
        });
        f.extraregion.push(f_puzzles::Region {
            cells: vec![
                "R6C6".to_string(),
                "R6C7".to_string(),
                "R6C8".to_string(),
                "R7C6".to_string(),
                "R7C7".to_string(),
                "R7C8".to_string(),
                "R8C6".to_string(),
                "R8C7".to_string(),
                "R8C8".to_string(),
            ],
        });
        f.extraregion.push(f_puzzles::Region {
            cells: vec![
                "R6C2".to_string(),
                "R6C3".to_string(),
                "R6C4".to_string(),
                "R7C2".to_string(),
                "R7C3".to_string(),
                "R7C4".to_string(),
                "R8C2".to_string(),
                "R8C3".to_string(),
                "R8C4".to_string(),
            ],
        });

        let res_b = Board::try_from(&f);
        assert!(res_b.is_ok());
        let mut b = res_b.unwrap();
        assert!(b.assign(4, THREE).is_ok());
        assert!(b.assign(14, TWO).is_ok());
        assert!(b.assign(16, EIGHT).is_ok());
        assert!(b.assign(20, SIX).is_ok());
        assert!(b.assign(24, FIVE).is_ok());
        assert!(b.assign(34, THREE).is_ok());
        assert!(b.assign(36, NINE).is_ok());
        assert!(b.assign(44, FOUR).is_ok());
        assert!(b.assign(46, SEVEN).is_ok());
        assert!(b.assign(56, TWO).is_ok());
        assert!(b.assign(60, NINE).is_ok());
        assert!(b.assign(66, FIVE).is_ok());
        assert!(b.assign(76, EIGHT).is_ok());
        assert!(b.assign(78, SEVEN).is_ok());

        assert_eq!(b.solution_count_max(100), 1);
    }

    #[test]
    fn diagonals() {
        let mut f = FPuzzles::new(9);
        f.grid[0][1].value = Some(3);
        f.grid[0][3].value = Some(1);
        f.grid[0][8].value = Some(6);
        f.grid[1][1].value = Some(9);
        f.grid[1][8].value = Some(5);
        f.grid[2][3].value = Some(8);
        f.grid[3][0].value = Some(5);
        f.grid[3][4].value = Some(6);
        f.grid[3][6].value = Some(2);
        f.grid[4][0].value = Some(4);
        f.grid[4][5].value = Some(2);
        f.grid[5][7].value = Some(1);
        f.grid[6][2].value = Some(3);
        f.grid[7][4].value = Some(7);
        f.grid[8][6].value = Some(6);
        f.grid[8][7].value = Some(4);

        f.positive_diagonal = true;
        f.negative_diagonal = true;

        let res_b = Board::try_from(&f);
        assert!(res_b.is_ok());
        let mut b = res_b.unwrap();
        assert_eq!(b.solution_count_max(100), 1);
    }
}
