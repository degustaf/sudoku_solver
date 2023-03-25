/// Implementation of the solution iterator. This iterator is used to generate solutions to given
/// puzzle. This provides the core engine used to find **a** solution, and to count solutions.
use crate::Board;
use crate::Elimination;
// use crate::SolutionCount;
use crate::Contradiction;
// use crate::types::board;

#[allow(dead_code)]
/// An iter that will generate solutions to the puzzle.
pub struct SolutionIter {
    stack: Vec<(Board, usize, Vec<usize>)>,
}

#[allow(dead_code)]
impl SolutionIter {
    fn solve_helper(board: &mut Board) -> Result<(), Contradiction> {
        // let mut status = Elimination::Same;
        while board.naked_singles()? == Elimination::Eliminated {
            // status = Elimination::Eliminated;
        }

        Ok(())
    }

    pub(crate) fn new(b: &Board) -> Self {
        let mut board = b.clone();

        match Self::solve_helper(&mut board) {
            Err(_) => Self { stack: Vec::new() },
            Ok(_) => match board.next_idx_to_guess() {
                None => Self { stack: Vec::new() },
                Some(next_idx) => {
                    let values: Vec<usize> = board.get_values(next_idx).iter_ones().collect();
                    Self {
                        stack: vec![(board, next_idx, values)],
                    }
                }
            },
        }
    }
}

impl std::iter::Iterator for SolutionIter {
    type Item = Board;

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
                    let bit = board.to_bits(value).unwrap();
                    if board.assign(next_idx, bit).is_err() {
                        continue;
                    }
                }
            }
            if Self::solve_helper(&mut board).is_err() {
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
                    let new_values: Vec<usize> = board.get_values(idx).iter_ones().collect();
                    self.stack.push((board, idx, new_values));
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solution_iter() {
        let res = crate::from_string(
            "1.2........62.3.........3.454..6........5.9......1.76..87.........9.8.........1.9",
        );
        assert!(res.is_ok());
        let board = res.unwrap();
        let iter = SolutionIter::new(&board);
        assert_eq!(iter.count(), 78);
    }

    #[test]
    fn solution_iter_for_contradiction() {
        let res = crate::from_string(
            "152........62.3.........3.454..6........5.9......1.76..87.........9.8.........1.9",
        );
        assert!(res.is_ok());
        let board = res.unwrap();
        let iter = SolutionIter::new(&board);
        assert_eq!(iter.count(), 0);
    }

    #[test]
    fn solution_iter_for_initially_solved() {
        let res = crate::from_string(
            "152........62.3.........3.454..6........5.9......1.76..87.........9.8.........1.9",
        );
        assert!(res.is_ok());
        let board = res.unwrap();
        let iter = SolutionIter::new(&board);
        assert_eq!(iter.count(), 0);
    }
}
