//! Implementation of the solution iterator. This iterator is used to generate solutions to given
//! puzzle. This provides the core engine used to find **a** solution, and to count solutions.

/// A common trait for puzzles that can be solved using a depth first search.
pub trait Solvable: Clone {
    /// The type that is used to describe a guess as used in the `assign` function.
    type Guess;

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

