//! A library for solving Yin-Yang  puzzles
//!
//! # Rules
//!
//! Shade the grid 2 colors such that all cells of each cell are connected orthoganally and no 2 by
//! 2 region is completely shaded either color.

use solution_iter::Solvable;
use std::collections::VecDeque;
use std::error::Error;
use std::fmt::Display;
use std::ops::BitAndAssign;
use std::ops::BitOrAssign;
use std::sync::Arc;
use strength_reduce::StrengthReducedUsize;

/// Errors that can be generated when working with Yin-Yang puzzles.
#[derive(Debug, PartialEq)]
pub enum YinYangError {
    /// The dimensions provided for a yin-yang puzzle doesn't match the length of the string
    /// representation.
    BadDimensions(usize, usize, usize),

    /// A character used in the string representation of a yin-yang puzzle is invalid.
    BadEncoding(char),

    /// The puzzle has a contradiction in it.
    Contradiction,
}

impl Display for YinYangError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            YinYangError::BadDimensions(height, width, length) => {
                write!(f, "Bad dimensions: height is {height} and width is {width}, but length of the string representation is {length}.")
            }
            YinYangError::BadEncoding(c) => {
                write!(
                    f,
                    "Can't encode '{c}' as shaded or unshaded in a yin-yang puzzle."
                )
            }
            YinYangError::Contradiction => {
                write!(f, "Puzzle has a contradiction.")
            }
        }
    }
}

impl Error for YinYangError {}

/// Tracks if a strategy is sucessful.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Deduction {
    /// The strategy succeeded in setting a cell in the puzzle.
    Deduction,

    /// The strategy failed to set any cells in the puzzle.
    Same,
}

impl BitAndAssign for Deduction {
    fn bitand_assign(&mut self, rhs: Self) {
        if rhs == Deduction::Deduction {
            *self = rhs;
        }
    }
}

/// A representation of a yin-yang puzzle.
#[derive(Clone, Debug)]
pub struct YinYang {
    height: usize,
    width: usize,
    data: Vec<usize>,
    border: Arc<[usize]>,
    divisor: StrengthReducedUsize,
}

impl Display for YinYang {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        for i in 0..self.height {
            for j in 0..self.width {
                write!(f, "{} ", self.data[i * self.width + j])?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl YinYang {
    /// Generate an empty yin-yang puzzle of dimensions `height` by `width`.
    #[must_use]
    pub fn new(height: usize, width: usize) -> Self {
        let mut border = Vec::with_capacity(2 * height + 2 * width - 4);
        for i in 0..width {
            border.push(i);
        }
        for i in ((2 * width - 1)..(height * width)).step_by(width) {
            border.push(i);
        }
        for i in ((height - 1) * width..(height * width) - 1).rev() {
            border.push(i);
        }
        for i in (width..(height - 1) * width).step_by(width).rev() {
            border.push(i);
        }

        YinYang {
            height,
            width,
            data: vec![3; height * width],
            border: Arc::from(border),
            divisor: StrengthReducedUsize::new(width),
        }
    }

    /// Generate a puzzle from a string representation of a yin-yang puzzle.
    ///
    /// # Errors
    ///
    /// This function can throw an error if either `height` and `width` do not match the length of `repr`
    /// or the string contains characters other than `0` for empty, `1` for shaded or `2` for unshaded.
    ///
    /// ```
    /// use yy_engine::YinYang;
    ///
    /// let yy = YinYang::from_string(3, 3, "012000000");
    /// assert!(yy.is_ok());
    ///
    /// let bad_yy = YinYang::from_string(3, 3, "10");
    /// assert!(bad_yy.is_err());
    ///
    /// let other_bad_yy = YinYang::from_string(3, 3, "012000005");
    /// assert!(bad_yy.is_err());
    /// ```
    pub fn from_string(height: usize, width: usize, repr: &str) -> Result<YinYang, YinYangError> {
        if height * width != repr.len() {
            return Err(YinYangError::BadDimensions(height, width, repr.len()));
        }

        let mut yy = Self::new(height, width);

        for (i, c) in repr.char_indices() {
            match c {
                '0' => {}
                '1' => {
                    yy.data[i] = 1;
                }
                '2' => {
                    yy.data[i] = 2;
                }
                _ => {
                    return Err(YinYangError::BadEncoding(c));
                }
            }
        }

        Ok(yy)
    }

    fn two_by_two(&mut self, idx: usize) -> Deduction {
        debug_assert!(idx % self.divisor != self.width - 1); // So out algorithm won't go off the right edge of the puzzle.
        debug_assert!(idx + self.width < self.data.len()); // So our algorithm won't go off the bottom edge of the puzzle.

        let mut ones_count = 0;
        let mut twos_count = 0;
        let mut zero_idx = usize::MAX;

        for offset in [0, 1, self.width, self.width + 1] {
            let new_idx = idx + offset;
            if self.data[new_idx] == 1 {
                ones_count += 1;
            } else if self.data[new_idx] == 2 {
                twos_count += 1;
            } else {
                // self.data[new_idx] == 0
                zero_idx = new_idx;
            }
        }

        if ones_count == 3 && twos_count == 0 {
            self.data[zero_idx] = 2;
            Deduction::Deduction
        } else if twos_count == 3 && ones_count == 0 {
            self.data[zero_idx] = 1;
            Deduction::Deduction
        } else {
            Deduction::Same
        }
    }

    /// By the rules of yin-yang, you cannot have a 2 by 2 of either shaded or unshaded cells. This
    /// checks for those possibilities, and fills in empty cells in such a way to prevent a 2 by 2.
    fn two_by_two_all(&mut self) -> Deduction {
        let mut ret = Deduction::Same;

        for i in 0..self.height - 1 {
            for j in 0..self.width - 1 {
                ret &= self.two_by_two(i * self.width + j);
            }
        }
        ret
    }

    fn checkerboard(&mut self, idx: usize) -> Result<Deduction, YinYangError> {
        debug_assert!(idx % self.divisor != self.width - 1); // So our algorithm won't go off the right edge of the puzzle.
        debug_assert!(idx + self.width < self.data.len()); // So our algorithm won't go off the bottom edge of the puzzle.

        // The small part of the grid we're looking at is:
        // cell1 cell2
        // cell3 cell4
        let cell1 = self.data[idx];
        let cell2 = self.data[idx + 1];
        let cell3 = self.data[idx + self.width];
        let cell4 = self.data[idx + self.width + 1];

        if cell1 == cell4 {
            let other_color = 3 - cell1;
            if cell2 == cell3 && cell2 == other_color {
                return Err(YinYangError::Contradiction);
            }
            if cell2 == other_color && cell3 == 3 {
                self.data[idx + self.width] = cell1; // Set cell3 to cell1.
                return Ok(Deduction::Deduction);
            } else if cell3 == other_color && cell2 == 3 {
                self.data[idx + 1] = cell1; // Set cell2 to cell1.
                return Ok(Deduction::Deduction);
            }
        } else if cell2 == cell3 {
            let other_color = 3 - cell2;
            if cell1 == other_color && cell4 == 3 {
                self.data[idx + self.width + 1] = cell2; // Set cell4 to cell2.
                return Ok(Deduction::Deduction);
            } else if cell4 == other_color && cell1 == 3 {
                self.data[idx] = cell2; // Set cell1 to cell2.
                return Ok(Deduction::Deduction);
            }
        }

        Ok(Deduction::Same)
    }

    /// A checkerboard is an indication of a broken yin-yang. There is a connectivity argument that
    /// if there is a checkerboard, then either shaded, or unshaded cannot be connected. This
    /// function checks the board for possible checkerboards, and fills in cells to prevent a
    /// checkerboard.
    ///
    /// # Errors
    ///
    /// This function can return an error if there is a contradiction.
    fn checkerboard_all(&mut self) -> Result<Deduction, YinYangError> {
        let mut ret = Deduction::Same;

        for i in 0..self.height - 1 {
            for j in 0..self.width - 1 {
                ret &= self.checkerboard(i * self.width + j)?;
            }
        }
        Ok(ret)
    }

    fn deduce_border(&mut self) -> Result<Deduction, YinYangError> {
        let mut ret = Deduction::Same;
        let mut first_color = 3;
        let mut first_color_idx = usize::MAX;
        for (i, v) in self.border.iter().enumerate() {
            if self.data[*v] != 3 {
                first_color = self.data[*v];
                first_color_idx = i;
                break;
            }
        }
        if first_color == 3 {
            return Ok(ret);
        }

        let second_color = 3 - first_color;
        let mut second_color_idx = usize::MAX;
        for j in first_color_idx + 1..self.border.len() {
            if self.data[self.border[j]] == second_color {
                second_color_idx = j;
                break;
            }
        }
        if second_color_idx == usize::MAX {
            return Ok(ret);
        }

        let mut last_first_color_idx = first_color_idx;
        for i in first_color_idx + 1..second_color_idx {
            debug_assert_ne!(self.data[self.border[i]], second_color);
            if self.data[self.border[i]] == first_color {
                for j in last_first_color_idx + 1..i {
                    self.data[self.border[j]] = first_color;
                    ret = Deduction::Deduction;
                }
                last_first_color_idx = i;
            }
        }

        let mut last_second_color_idx = second_color_idx;
        last_first_color_idx = usize::MAX;
        for i in second_color_idx + 1..self.border.len() {
            if self.data[self.border[i]] == first_color {
                last_first_color_idx = i;
                break;
            }
            if self.data[self.border[i]] == second_color {
                for j in last_second_color_idx + 1..i {
                    self.data[self.border[j]] = second_color;
                    ret = Deduction::Deduction;
                }
                last_second_color_idx = i;
            }
        }

        if last_first_color_idx != usize::MAX {
            for i in last_first_color_idx + 1..self.border.len() {
                if self.data[self.border[i]] == second_color {
                    return Err(YinYangError::Contradiction);
                }
                if self.data[self.border[i]] != first_color {
                    self.data[self.border[i]] = first_color;
                    ret = Deduction::Deduction;
                }
            }

            for i in 0..first_color_idx {
                debug_assert_ne!(self.data[self.border[i]], second_color);
                if self.data[self.border[i]] != first_color {
                    self.data[self.border[i]] = first_color;
                    ret = Deduction::Deduction;
                }
            }
        }

        Ok(ret)
    }

    fn adjacent_cells(&self, idx: usize, ret: &mut [usize]) -> usize {
        let mut count = 0;

        let x = idx / self.divisor;
        let y = idx % self.divisor;

        if x > 0 {
            ret[count] = idx - self.width;
            count += 1;
        }
        if y > 0 {
            ret[count] = idx - 1;
            count += 1;
        }
        if y < self.width - 1 {
            ret[count] = idx + 1;
            count += 1;
        }
        if x < self.height - 1 {
            ret[count] = idx + self.width;
            count += 1;
        }

        count
    }

    fn check_helper(
        &self,
        shaded: &mut [bool],
        shaded_queue: &mut VecDeque<usize>,
        color: usize,
        starting_idx: usize,
    ) -> bool {
        for i in starting_idx..self.data.len() {
            if self.data[i] == color {
                shaded_queue.push_back(i);
                break;
            }
        }
        if shaded_queue.is_empty() {
            // we've checked every chunk of color, they all look okay.
            return true;
        }
        let mut way_out = false;
        let mut adjacent = [0; 4];
        while let Some(i) = shaded_queue.pop_front() {
            shaded[i] = true;
            let count = self.adjacent_cells(i, &mut adjacent);
            for new_idx in adjacent.iter().take(count) {
                if shaded[*new_idx] {
                    continue;
                }
                if self.data[*new_idx] == 3 {
                    way_out = true;
                } else if self.data[*new_idx] == color {
                    shaded_queue.push_back(*new_idx);
                }
            }
        }
        if !way_out {
            for (idx, v) in self.data.iter().enumerate() {
                if *v == color && !shaded[idx] {
                    // No way to reach other cells that are the same color. We're broken.
                    return false;
                }
            }
        }

        // Nothing appears to be wrong with this block
        // for i in starting_idx..self.data.len() {
        for (i, v) in shaded
            .iter()
            .enumerate()
            .take(self.data.len())
            .skip(starting_idx)
        {
            if self.data[i] == color && !v {
                for v in shaded.iter_mut() {
                    *v = false;
                }
                return self.check_helper(shaded, shaded_queue, color, i);
            }
        }

        // No more blocks to check.
        true
    }

    fn check_two_by_two(&self) -> bool {
        for i in 0..self.height - 1 {
            for j in 0..self.width - 1 {
                let idx = i * self.width + j;
                let mut ones_count = 0;
                let mut twos_count = 0;
                for offset in [0, 1, self.width, self.width + 1] {
                    let new_idx = idx + offset;
                    if self.data[new_idx] == 1 {
                        ones_count += 1;
                    } else if self.data[new_idx] == 2 {
                        twos_count += 1;
                    }
                }
                if ones_count == 4 || twos_count == 4 {
                    return false;
                }
            }
        }
        true
    }

    #[must_use]
    fn check(&self) -> bool {
        let mut shaded_queue = VecDeque::with_capacity(self.data.len() / 2);
        let mut shaded = vec![false; self.data.len()];
        if !self.check_helper(&mut shaded, &mut shaded_queue, 1, 0) {
            return false;
        }
        for v in &mut shaded {
            *v = false;
        }
        self.check_helper(&mut shaded, &mut shaded_queue, 2, 0) && self.check_two_by_two()
    }

    fn deduce(&mut self) -> Result<Deduction, YinYangError> {
        let mut ret = self.deduce_border()?;
        loop {
            while self.two_by_two_all() == Deduction::Deduction {
                ret = Deduction::Deduction;
            }
            if self.checkerboard_all()? == Deduction::Same {
                break;
            }
            if self.deduce_border()? == Deduction::Same {
                break;
            }
            ret = Deduction::Deduction;
        }

        Ok(ret)
    }
}

impl Solvable for YinYang {
    type Guess = usize;

    fn assign(&mut self, idx: usize, g: <Self as Solvable>::Guess) -> bool {
        self.data[idx] = g;
        self.check()
    }

    fn deduce(&mut self) -> bool {
        self.deduce().is_ok()
    }

    fn next_idx_to_guess(&self) -> Option<usize> {
        for (i, v) in self.data.iter().enumerate() {
            if *v == 3 {
                return Some(i);
            }
        }
        None
    }

    fn guesses(&self, _: usize) -> Vec<<Self as Solvable>::Guess> {
        vec![1, 2]
    }

    fn solved(&self) -> bool {
        for v in &self.data {
            if *v == 3 {
                return false;
            }
        }

        self.check()
    }

    fn indices(&self) -> Vec<usize> {
        let mut ret: Vec<usize> = self.border.iter().copied().collect();
        ret.extend(0..self.data.len());
        ret
    }

    fn possibility(&self, idx: usize, g: <Self as Solvable>::Guess) -> bool {
        self.data[idx] & g != 0
    }
}

impl BitOrAssign for YinYang {
    fn bitor_assign(&mut self, rhs: Self) {
        debug_assert_eq!(self.height, rhs.height);
        debug_assert_eq!(self.width, rhs.width);

        for (i, v) in self.data.iter_mut().enumerate() {
            *v |= rhs.data[i];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_util::sync::CancellationToken;

    #[test]
    fn deduction_bitand() {
        let mut deduc = Deduction::Same;

        deduc &= Deduction::Same;
        assert_eq!(deduc, Deduction::Same);

        deduc &= Deduction::Deduction;
        assert_eq!(deduc, Deduction::Deduction);

        deduc &= Deduction::Same;
        assert_eq!(deduc, Deduction::Deduction);

        deduc &= Deduction::Deduction;
        assert_eq!(deduc, Deduction::Deduction);
    }

    #[test]
    fn make_new() {
        let yy = YinYang::new(13, 42);
        assert_eq!(yy.height, 13);
        assert_eq!(yy.width, 42);
        assert_eq!(yy.data.len(), 13 * 42);
        assert_eq!(yy.border.len(), 2 * 13 + 2 * 42 - 4);
    }

    #[test]
    fn from_string_doctest() {
        let yy = YinYang::from_string(3, 3, "012000000");
        assert!(yy.is_ok());
        let yy_unwrap = yy.unwrap();
        assert_eq!(format!("{yy_unwrap}"), "3 1 2 \n3 3 3 \n3 3 3 \n");
        assert_eq!(*yy_unwrap.border, [0, 1, 2, 5, 8, 7, 6, 3]);

        let bad_yy = YinYang::from_string(3, 3, "10");
        assert!(bad_yy.is_err());
        assert_eq!(format!("{}", bad_yy.unwrap_err()), "Bad dimensions: height is 3 and width is 3, but length of the string representation is 2.");

        let other_bad_yy = YinYang::from_string(3, 3, "012000005");
        assert!(other_bad_yy.is_err());
        assert_eq!(
            format!("{}", other_bad_yy.unwrap_err()),
            "Can't encode '5' as shaded or unshaded in a yin-yang puzzle."
        );
    }

    #[test]
    fn two_by_two() {
        let mut yy = YinYang::from_string(5, 2, "0001112202").unwrap();
        assert_eq!(yy.two_by_two(0), Deduction::Same);

        assert_eq!(yy.two_by_two(2), Deduction::Deduction);
        assert_eq!(yy.data[2], 2);

        assert_eq!(yy.two_by_two(6), Deduction::Deduction);
        assert_eq!(yy.data[8], 1);
    }

    #[test]
    fn checkerboard() {
        let mut yy = YinYang::from_string(8, 2, "0112210210211000").unwrap();
        assert_eq!(
            *yy.border,
            [0, 1, 3, 5, 7, 9, 11, 13, 15, 14, 12, 10, 8, 6, 4, 2]
        );
        let mut response = yy.checkerboard(0);
        assert!(response.is_ok());
        assert_eq!(response.unwrap(), Deduction::Deduction);
        assert_eq!(yy.data[0], 1);

        response = yy.checkerboard(2);
        assert!(response.is_err());
        assert_eq!(response.unwrap_err(), YinYangError::Contradiction);

        response = yy.checkerboard(4);
        assert!(response.is_ok());
        assert_eq!(response.unwrap(), Deduction::Deduction);
        assert_eq!(yy.data[6], 2);

        response = yy.checkerboard(8);
        assert!(response.is_ok());
        assert_eq!(response.unwrap(), Deduction::Deduction);
        assert_eq!(yy.data[9], 1);

        response = yy.checkerboard(10);
        assert!(response.is_ok());
        assert_eq!(response.unwrap(), Deduction::Deduction);
        assert_eq!(yy.data[13], 1);

        response = yy.checkerboard(12);
        assert!(response.is_ok());
        assert_eq!(response.unwrap(), Deduction::Same);
    }

    #[test]
    fn two_by_two_all() {
        let mut yy = YinYang::from_string(3, 4, "110010000220").unwrap();
        assert_eq!(*yy.border, [0, 1, 2, 3, 7, 11, 10, 9, 8, 4]);
        let response = yy.two_by_two_all();
        assert_eq!(response, Deduction::Deduction);
        assert_eq!(format!("{yy}"), "1 1 3 3 \n1 2 1 3 \n3 2 2 3 \n");
    }

    #[test]
    fn checkerboard_all() {
        let mut yy = YinYang::from_string(3, 4, "121220010120").unwrap();
        let response = yy.checkerboard_all();
        assert!(response.is_err());
        assert_eq!(response.unwrap_err(), YinYangError::Contradiction);
    }

    #[test]
    fn deduce() {
        let mut yy = YinYang::from_string(3, 3, "100112100").unwrap();
        let response = yy.deduce();
        assert!(response.is_ok());
        assert_eq!(response.unwrap(), Deduction::Deduction);
        assert_eq!(format!("{yy}"), "1 2 2 \n1 1 2 \n1 2 2 \n");
    }

    #[test]
    fn yy_true_candidates_dfs() {
        let yy = YinYang::from_string(4, 4, "0020000020010000").unwrap();
        let tc = solution_iter::true_candidates_dfs(&yy);
        assert!(tc.is_some());
        assert_eq!(
            format!("{}", tc.unwrap()),
            "2 2 2 3 \n2 1 3 1 \n2 3 3 1 \n3 3 3 3 \n"
        );
    }

    #[test]
    fn yy_true_candidates_bfs() {
        let yy = YinYang::from_string(4, 4, "0020000020010000").unwrap();
        let token = CancellationToken::new();
        let tc = solution_iter::true_candidates_bfs(&yy, &token);
        assert!(tc.is_some());
        assert_eq!(
            format!("{}", tc.unwrap()),
            "2 2 2 3 \n2 1 3 1 \n2 3 3 1 \n3 3 3 3 \n"
        );
    }

    #[test]
    fn second_block_isolated() {
        let yy = YinYang::from_string(4, 5, "22222212122122200000").unwrap();
        let mut shaded_queue = VecDeque::with_capacity(yy.data.len() / 2);
        let mut shaded = vec![false; yy.data.len()];
        assert!(!yy.check_helper(&mut shaded, &mut shaded_queue, 1, 0));
    }

    #[test]
    fn deduce_border() {
        let mut yy = YinYang::from_string(10, 10, "0000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000210000").unwrap();
        assert_eq!(yy.deduce_border(), Ok(Deduction::Deduction));
        assert_eq!(
            format!("{yy}"),
            "2 2 2 2 2 2 2 2 2 2 
2 3 3 3 3 3 3 3 3 2 
2 3 3 3 3 3 3 3 3 2 
2 3 3 3 3 3 3 3 3 2 
2 3 3 3 3 3 3 3 3 3 
2 3 3 3 3 3 3 3 3 3 
2 3 3 3 3 3 3 3 3 3 
2 3 3 3 3 3 3 3 3 3 
2 3 3 3 3 3 3 3 3 3 
2 2 2 2 2 1 3 3 3 3 
"
        );
    }

    #[test]
    fn bad_border() {
        let mut yy = YinYang::from_string(5, 5, "1000200000000000000020001").unwrap();
        assert_eq!(yy.deduce_border(), Err(YinYangError::Contradiction));
    }

    #[test]
    fn empty_border() {
        let mut yy = YinYang::from_string(5, 5, "0000000000000000000000000").unwrap();
        assert_eq!(yy.deduce_border(), Ok(Deduction::Same));
    }

    #[test]
    fn border_only_one_color() {
        let mut yy = YinYang::from_string(5, 5, "1000000000000000000000001").unwrap();
        assert_eq!(yy.deduce_border(), Ok(Deduction::Same));
    }
}
