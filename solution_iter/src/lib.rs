//! Implementation of the solution iterator. This iterator is used to generate solutions to given
//! puzzle. This provides the core engine used to find **a** solution, and to count solutions.

use std::ops::BitOrAssign;
// use std::fmt::Display;

/// A common trait for puzzles that can be solved using a depth first search.
pub trait Solvable: Clone {
    /// The type that is used to describe a guess as used in the `assign` function.
    type Guess: Copy;

    /// Assign `guess` at `next_idx`. Should return `false` if this breaks the puzzle, and `true`
    /// otherwise.
    fn assign(&mut self, next_idx: usize, guess: Self::Guess) -> bool;

    /// Apply the logic of the puzzle to move towards a solution. Should return `false` if the
    /// deductions lead to the conclusion that the puzzle is broken, `true` otherwise.
    fn deduce(&mut self) -> bool;

    /// Which index should we guess next? Returns `None` if there is nowhere left to guess.
    fn next_idx_to_guess(&self) -> Option<usize>;

    /// What are the possible values to guess at `idx`
    fn guesses(&self, idx: usize) -> Vec<Self::Guess>;

    /// Check if the puzzle is solved.
    fn solved(&self) -> bool;

    /// Get all possible indices for the puzzle.
    fn indices(&self) -> Vec<usize>;

    /// Is `g` a possible value at `i`?
    fn possibility(&self, idx: usize, g: Self::Guess) -> bool;
}

/// An iter that will generate solutions to the puzzle.
pub struct SolutionIterator<T: Solvable> {
    stack: Vec<(T, usize, Vec<T::Guess>)>,
}

impl<T: Solvable> SolutionIterator<T> {
    /// Create a new `SolutionIterator` that will iterate over all solutions of a puzzle.
    pub fn new(b: &T) -> Self {
        let mut board = b.clone();

        if board.deduce() {
            match board.next_idx_to_guess() {
                None => {
                    if board.solved() {
                        Self {
                            stack: vec![(board, 0, Vec::new())],
                        }
                    } else {
                        Self { stack: Vec::new() }
                    }
                }
                Some(next_idx) => {
                    let values = board.guesses(next_idx);
                    Self {
                        stack: vec![(board, next_idx, values)],
                    }
                }
            }
        } else {
            Self { stack: Vec::new() }
        }
    }
}

impl<T: Solvable> std::iter::Iterator for SolutionIterator<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        while !self.stack.is_empty() {
            let (mut board, next_idx, mut values) = self.stack.pop()?;

            if board.solved() {
                return Some(board);
            }

            match values.pop() {
                None => {
                    continue;
                }
                Some(value) => {
                    self.stack.push((board.clone(), next_idx, values));
                    if !board.assign(next_idx, value) {
                        continue;
                    }
                }
            }
            if !board.deduce() {
                continue;
            }
            // println!("{board}");
            if board.solved() {
                return Some(board);
            }
            match board.next_idx_to_guess() {
                None => {
                    continue;
                }
                Some(idx) => {
                    let new_values = board.guesses(idx);
                    self.stack.push((board, idx, new_values));
                }
            }
        }

        None
    }
}

/// Compute all possible values that can be placed in any index for a puzzle.
///
/// `BitOrAssign` is used to combine multiple solutions and retain what possibilities are available
/// at each index.
///
/// This uses a depth first search and may not be appropriate for puzzles with large branching
/// factors that have many solutions available.
pub fn true_candidates_dfs<T: Solvable + BitOrAssign>(puzzle: &T) -> Option<T> {
    let mut iter = SolutionIterator::new(puzzle);
    let Some(mut ret) = iter.next() else {
        return None;
    };
    for sln in iter {
        ret |= sln;
    }

    Some(ret)
}

fn tc_breadth_first<T: Solvable + BitOrAssign>(mut puzzle: T, ret: &mut T) {
    for i in puzzle.indices() {
        let mut count = 0;
        let mut val = None;
        for g in puzzle.guesses(i) {
            if puzzle.possibility(i, g) {
                if ret.possibility(i, g) {
                    count += 1;
                    val = Some(g);
                } else {
                    let mut new_puzzle = puzzle.clone();
                    if new_puzzle.assign(i, g) {
                        if let Some(p) = SolutionIterator::new(&new_puzzle).next() {
                            *ret |= p;
                            count += 1;
                            val = Some(g);
                        }
                    }
                }
            }
        }
        if count == 1 {
            puzzle.assign(i, val.unwrap());
            puzzle.deduce();
        }
    }
}

/// Compute all possible values that can be placed in any index for a puzzle.
///
/// `BitOrAssign` is used to combine multiple solutions and retain what possibilities are available
/// at each index.
///
/// This uses a breadth first search and may not be appropriate for puzzles that have only a few
/// solutions, but finding each one is slow and requires many contradictions.
pub fn true_candidates_bfs<T: Solvable + BitOrAssign>(puzzle: &T) -> Option<T> {
    let Some(mut ret) = SolutionIterator::new(puzzle).next() else {
        return None;
    };

    tc_breadth_first(puzzle.clone(), &mut ret);
    Some(ret)
}

/// Compute all possible values that can be placed in any index for a puzzle.
///
/// `BitOrAssign` is used to combine multiple solutions and retain what possibilities are available
/// at each index.
///
/// This uses a hybrid approach where a depth first search is attempted. If too many solutions are
/// found, we switch to a breadth first approach and begin filling in possible values that have
/// been skipped over so far.
pub fn true_candidates<T: Solvable + BitOrAssign>(puzzle: &T) -> Option<T> {
    let mut iter = SolutionIterator::new(puzzle);
    let Some(mut ret) = iter.next() else {
        return None;
    };
    let mut count = 1;
    for sln in iter {
        ret |= sln;
        count += 1;
        if count > 10_000 {
            tc_breadth_first(puzzle.clone(), &mut ret);
            break;
        }
    }

    Some(ret)
}
