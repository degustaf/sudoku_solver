//! Implementations related to Sudoku constraints.

use crate::types::{eliminate, Bits, Contradiction, Elimination};

#[must_use]
pub(crate) fn check_quad(
    idx: usize,
    single: Bits,
    double: Bits,
    size: usize,
    grid: &[Bits],
) -> bool {
    let indices = [idx, idx + 1, idx + size, idx + size + 1];

    let mut seen = 0;
    let mut seen_twice = 0;
    for i in indices {
        let v = grid[i];
        let add_to_mask = v - (v & seen_twice);
        seen_twice |= seen & v;
        seen ^= add_to_mask;
    }
    ((seen | seen_twice) & single == single) && (seen_twice & double == double)
}

pub(crate) fn init_quad(
    idx: usize,
    single: Bits,
    double: Bits,
    size: usize,
    grid: &mut [Bits],
) -> Result<Elimination, Contradiction> {
    debug_assert_eq!(single & double, 0);
    let mut indices = [idx, idx + 1, idx + size, idx + size + 1];
    let (mask, double_mask, count) = {
        let mut mask: Bits = 0;
        let mut double_mask = 0;
        let mut count = 0;
        for i in &mut indices {
            let v = grid[*i];
            if v & (single | double) == 0 {
                *i = Bits::MAX;
                continue;
            }
            count += 1;
            let add_to_mask = v - (v & double_mask);
            double_mask |= mask & v;
            mask ^= add_to_mask;
        }
        (mask, double_mask, count)
    };
    debug_assert_eq!(mask & double_mask, 0);

    let needed = single.count_ones() + 2 * double.count_ones();
    if needed > count {
        return Err(Contradiction(()));
    }

    if (((mask | double_mask) & single) != single) || ((double_mask & double) != double) {
        return Err(Contradiction(()));
    }

    if needed < count {
        return Ok(Elimination::Same);
    }

    let mut ret = Elimination::Same;
    for i in indices {
        if i == Bits::MAX {
            continue;
        }
        ret &= eliminate(i, grid[i] - (grid[i] & (single | double)), grid)?;
    }
    Ok(ret)
}

pub(crate) fn quad_enforce_consistency(
    idx: usize,
    quad_idx: usize,
    single: Bits,
    double: Bits,
    size: usize,
    grid: &mut [Bits],
) -> Result<Elimination, Contradiction> {
    let indices = [quad_idx, quad_idx + 1, quad_idx + size, quad_idx + size + 1];
    if !indices.contains(&idx) {
        return Ok(Elimination::Same);
    }

    let (mask, double_mask, count) = {
        let mut mask: Bits = 0;
        let mut double_mask = 0;
        let mut count = 0;
        for i in indices {
            let v = grid[i];
            if v & (single | double) == 0 {
                continue;
            }
            count += 1;
            let add_to_mask = v - (v & double_mask);
            double_mask |= mask & v;
            mask ^= add_to_mask;
        }
        (mask, double_mask, count)
    };

    let needed = single.count_ones() + 2 * double.count_ones();
    if needed > count {
        return Err(Contradiction(()));
    }

    if ((mask | double_mask) & single != single) || (double_mask & double != double) {
        return Err(Contradiction(()));
    }

    Ok(Elimination::Same)
}

#[cfg(test)]
mod test {
    use super::*;

    const ONE: Bits = 1 << 1;
    const TWO: Bits = 1 << 2;
    const THREE: Bits = 1 << 3;
    const FOUR: Bits = 1 << 4;
    const FIVE: Bits = 1 << 5;
    const SIX: Bits = 1 << 6;
    const SEVEN: Bits = 1 << 7;
    const EIGHT: Bits = 1 << 8;
    const NINE: Bits = 1 << 9;
    const ALL_DIGITS: Bits = ONE | TWO | THREE | FOUR | FIVE | SIX | SEVEN | EIGHT | NINE;

    #[test]
    fn test_check_quad() {
        let mut grid = [0; 81];
        grid[11] = FOUR;
        grid[12] = TWO;
        grid[20] = ONE;
        grid[21] = THREE;

        let mut vals = ONE | TWO | THREE | FOUR;
        assert!(check_quad(11, vals, 0, 9, &grid));
        assert!(check_quad(11, TWO | THREE, 0, 9, &grid));

        vals = TWO | THREE | FOUR | FIVE;
        assert!(!check_quad(11, vals, 0, 9, &grid));
        assert!(!check_quad(11, TWO | THREE, FOUR, 9, &grid));
    }

    #[test]
    fn test_check_quad_repeated_digit_in_solution() {
        let grid = [
            32, 64, 8, 2, 16, 4, 4, 16, 2, 64, 32, 8, 16, 32, 4, 8, 64, 2, 8, 2, 64, 16, 4, 32, 64,
            8, 32, 4, 2, 16, 2, 4, 16, 32, 8, 64,
        ];
        assert!(check_quad(4, THREE | FOUR, 0, 6, &grid));
        assert!(check_quad(8, ONE | TWO | THREE | SIX, 0, 6, &grid));
        assert!(check_quad(9, FIVE | SIX, 0, 6, &grid));
        assert!(check_quad(12, ONE | THREE | FIVE, 0, 6, &grid));
        assert!(check_quad(19, ONE | THREE | FIVE | SIX, 0, 6, &grid));
        assert!(check_quad(22, ONE | FIVE, 0, 6, &grid));
    }

    #[test]
    fn test_init_quad() {
        let mut grid = [ALL_DIGITS; 81];

        assert_eq!(
            init_quad(0, ONE | TWO | THREE | FOUR, 0, 9, &mut grid),
            Ok(Elimination::Eliminated)
        );
        assert_eq!(grid[0], ONE | TWO | THREE | FOUR);

        grid[0] = NINE;
        assert!(init_quad(0, ONE | TWO | THREE | FOUR, 0, 9, &mut grid).is_err());

        grid[2] = FIVE;
        grid[12] = FOUR;
        assert!(init_quad(2, FIVE | SIX | SEVEN | EIGHT, 0, 9, &mut grid).is_err());

        assert_eq!(init_quad(4, FIVE, SIX, 9, &mut grid), Ok(Elimination::Same));

        grid[4] = FIVE;
        grid[5] = SIX;
        grid[13] = SIX;
        grid[14] = FIVE;
        assert!(init_quad(4, FOUR | FIVE, 0, 9, &mut grid).is_err());
    }

    #[test]
    fn test_quad_enforce_consistency() {
        let mut grid = [ALL_DIGITS; 81];
        grid[0] = ONE;
        assert_eq!(
            quad_enforce_consistency(0, 1, ONE | TWO | THREE | FOUR, 0, 9, &mut grid),
            Ok(Elimination::Same)
        );

        grid[0] = NINE;
        assert!(quad_enforce_consistency(0, 0, ONE | TWO | THREE | FOUR, 0, 9, &mut grid).is_err());

        grid[0] = FIVE;
        grid[1] = FOUR;
        assert!(
            quad_enforce_consistency(0, 0, FIVE | SIX | SEVEN | EIGHT, 0, 9, &mut grid).is_err()
        );

        grid[9] = FOUR;
        grid[10] = FIVE;
        assert!(quad_enforce_consistency(0, 0, FIVE | SIX, 0, 9, &mut grid).is_err());
        assert_eq!(
            quad_enforce_consistency(0, 0, FOUR | FIVE, 0, 9, &mut grid),
            Ok(Elimination::Same)
        );
    }
}
