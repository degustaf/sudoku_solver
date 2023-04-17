//! An interface

use arrayvec::ArrayVec;
use std::io::stdout;
use std::io::Write;

struct Partition {
    size: usize,
    data: ArrayVec<usize, { Self::CAPACITY }>,
    scratch: ArrayVec<usize, { Self::MAX_SIZE }>,
}

impl Partition {
    const MAX_SIZE: usize = 16;
    const CAPACITY: usize = Self::MAX_SIZE * Self::MAX_SIZE;
    fn new(size: usize, start: Option<&[usize]>) -> Self {
        // let data = Vec::with_capacity(size * size);
        let data = match start {
            Some(x) => x.iter().copied().collect(),
            None => ArrayVec::new(),
        };
        let mut scratch = ArrayVec::new();
        for _ in 0..size {
            scratch.push(0);
        }
        Self {
            size,
            data,
            scratch,
        }
    }
}

impl std::iter::Iterator for Partition {
    type Item = ArrayVec<usize, { Self::CAPACITY }>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.data.is_empty() {
            for i in 0..self.size {
                for _ in 0..self.size {
                    self.data.push(i);
                }
            }
            return Some(self.data.clone());
        }

        for i in 0..self.size {
            debug_assert_eq!(self.scratch[i], 0);
        }

        let scratch = self.scratch.as_mut_slice();
        let data = self.data.as_mut_slice();
        let mut i = self.size * self.size - 1;
        while i > 0 {
            unsafe {
                debug_assert!(i < data.len());
                let current_value = *data.get_unchecked(i);
                debug_assert!(current_value < scratch.len());
                *scratch.get_unchecked_mut(current_value) += 1;
                'outer: for new_value in current_value + 1..self.size {
                    debug_assert!(new_value < scratch.len());
                    if *scratch.get_unchecked(new_value) > 0 {
                        let mut max = 0;
                        for k in 0..i {
                            debug_assert!(k < data.len());
                            if *data.get_unchecked(k) > max + 1 {
                                break 'outer;
                            }
                            if *data.get_unchecked(k) > max {
                                max = *data.get_unchecked(k);
                            }
                        }
                        if new_value > max + 1 {
                            break 'outer;
                        }
                        debug_assert!(i < data.len());
                        *data.get_unchecked_mut(i) = new_value;
                        debug_assert!(new_value < scratch.len());
                        *scratch.get_unchecked_mut(new_value) -= 1;
                        i += 1;
                        for k in 0..self.size {
                            debug_assert!(k < scratch.len());
                            let mut count = *scratch.get_unchecked(k);
                            while count > 0 {
                                *data.get_unchecked_mut(i) = k;
                                count -= 1;
                                i += 1;
                            }
                            debug_assert!(k < scratch.len());
                            *scratch.get_unchecked_mut(k) = 0;
                        }
                        return Some(self.data.clone());
                    }
                }
            }

            i -= 1;
        }

        None
    }
}

/// This array has the number of solved grids in a sudoku of a given size, up to permuting the
/// digits.
const SUDOKU_COUNT: [usize; 10] = [
    0,
    1,
    1,
    1,
    1,
    2,
    46_080,
    100_000_000, /*7*/
    100_000_000, /*8*/
    18_383_222_420_692_992,
];

fn print_regions<T: std::io::Write>(mut file: &mut T, rng: &[usize]) -> std::io::Result<()> {
    write!(&mut file, "[")?;
    for i in rng {
        write!(&mut file, "{i} ")?;
    }
    writeln!(&mut file, "]")
}

pub(crate) fn build_irregular<T: std::io::Write>(
    size: usize,
    file: &mut T,
    start: Option<&[usize]>,
) {
    let iter = Partition::new(size, start);
    let target_count = SUDOKU_COUNT[size];
    let mut iter_count = 0;
    let mut total_count = 0;
    let mut std_out = stdout();
    'range_loop: for rngs in iter {
        if let Ok(mut b) = sudoku_engine::from_regions(size, size, &rngs) {
            for i in 0..size {
                if sudoku_engine::assign(&mut b, i, i + 1).is_err() {
                    break 'range_loop;
                }
            }
            let count = b.solution_count_max(target_count);
            if count > 0 && count <= target_count {
                writeln!(file, "{count}").unwrap();
                print_regions(file, &rngs).unwrap();
                print_regions(&mut std_out, &rngs).unwrap();
                iter_count += 1;
            }
            total_count += 1;
            if total_count > 100_000_000 {
                print_regions(&mut std_out, &rngs).unwrap();
                break;
            }
        }
    }
    println!("{iter_count} / {total_count}");
}

// sudoku_solver/src/build_irregular.rs: 19, 119-122, 124, 127-137, 140-145, 147-150, 154

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn count_two() {
        let iter = Partition::new(2, None);
        let n = iter.count();
        assert_eq!(n, 3);
    }

    #[test]
    fn count_three() {
        let iter = Partition::new(3, None);
        let n = iter.count();
        assert_eq!(n, 280);
    }

    #[test]
    fn count_four() {
        let iter = Partition::new(4, None);
        let n = iter.count();
        assert_eq!(n, 2_627_625);
    }
}
