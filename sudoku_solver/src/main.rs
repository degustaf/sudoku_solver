//! A command line utility for solving sudoku.

use crate::build_irregular::build_irregular;
use clap::{Parser, Subcommand, ValueEnum};
use solution_iter::{true_candidates_bfs, SolutionIterator};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use yy_engine::YinYang;

mod build_irregular;

#[derive(Clone, ValueEnum)]
enum YyComputation {
    SolutionCount,
    TrueCandidates,
    Candidates,
}

#[derive(Subcommand)]
enum Command {
    /// Take a string representation of a grid from the command line and solve it.
    Solve { repr: String },

    /// Treat each line of a file as an individual puzzle, and solve all of them.
    FromFile { path: PathBuf },

    BuildIrregular {
        size: usize,
        out_file: PathBuf,
        start: Option<Vec<usize>>,
    },

    /// Solve a Yin-Yang Puzzle.
    YinYang {
        #[arg(default_value_t = YyComputation::TrueCandidates)]
        #[arg(value_enum)]
        computation: YyComputation,
        path: PathBuf,
    },
}

fn solve_helper(repr: &str) -> Result<sudoku_engine::Board, sudoku_engine::SudokuErrors> {
    let board = sudoku_engine::from_string(repr)?;
    sudoku_engine::solve(&board)
}

fn solve_puzzle(repr: &str) {
    match solve_helper(repr) {
        Ok(_board) => {
            println!("Solved!");
        }
        Err(e) => {
            eprintln!("Error: {e}");
        }
    }
}

fn solve_file(path: &Path) {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Error: {e}");
            return;
        }
    };
    let data = BufReader::new(file);

    for (i, line) in data.lines().enumerate() {
        let repr = match line {
            Ok(repr) => repr,
            Err(e) => {
                eprintln!("Error on line {i}: {e}");
                return;
            }
        };
        match solve_helper(&repr) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error on line {i}: {e}");
            }
        }
    }
}

fn solve_yin_yang<R: std::io::BufRead, W: std::io::Write, W2: std::io::Write>(
    computation: &YyComputation,
    file: R,
    mut output: W,
    mut error: W2,
) {
    let mut data = file.lines();
    let mut repr = match data.next() {
        Some(Ok(repr)) => repr,
        Some(Err(e)) => {
            let _ = writeln!(error, "Error on line 1: {e}");
            return;
        }
        None => {
            let _ = writeln!(error, "Error: File is empty.");
            return;
        }
    };

    let mut height = 1;
    let width = repr.len();
    for (i, line) in data.enumerate() {
        let line_repr = match line {
            Ok(repr) => repr,
            Err(e) => {
                let _ = writeln!(error, "Error on line {}: {e}", i + 1);
                return;
            }
        };
        if line_repr.len() != width {
            let _ = writeln!(
                error,
                "Error: line {} is a different length from previous lines.",
                i + 1
            );
            return;
        }
        height += 1;
        repr.push_str(&line_repr);
    }

    let yy = match YinYang::from_string(height, width, &repr) {
        Ok(yy) => yy,
        Err(e) => {
            let _ = writeln!(error, "Error: {e}");
            return;
        }
    };
    match computation {
        YyComputation::TrueCandidates => match true_candidates_bfs(&yy) {
            Some(tc) => {
                let _ = writeln!(output, "{tc}");
            }
            None => {
                let _ = writeln!(output, "No solutions found.");
            }
        },
        YyComputation::SolutionCount => {
            let mut i: usize = 0;
            for _ in SolutionIterator::new(&yy) {
                i += 1;
                if i.trailing_zeros() >= 10 {
                    let _ = writeln!(output, "{i}");
                }
            }
            let _ = writeln!(output, "{i}");
        }
        YyComputation::Candidates => {
            for cand in SolutionIterator::new(&yy) {
                let _ = writeln!(output, "{cand}");
            }
        }
    }
}

#[derive(Parser)]
#[command(author, version, about, name = "sudoku_solver")]
struct Args {
    #[command(subcommand)]
    cmd: Command,
}

fn main() {
    let args = Args::parse();

    match args.cmd {
        Command::Solve { repr } => solve_puzzle(&repr),
        Command::FromFile { path } => solve_file(&path),
        Command::BuildIrregular {
            size,
            out_file,
            start,
        } => {
            let mut file = match File::options().create(true).append(true).open(out_file) {
                Ok(file) => file,
                Err(e) => {
                    eprintln!("Error: {e}");
                    return;
                }
            };
            match start {
                None => {
                    build_irregular(size, &mut file, None);
                }
                Some(v) => {
                    if v.len() == size * size {
                        build_irregular(size, &mut file, Some(&v));
                    } else {
                        eprintln!("Wrong size of starting array");
                    }
                }
            };
        }
        Command::YinYang { computation, path } => {
            let file = match File::open(path) {
                Ok(file) => file,
                Err(e) => {
                    eprintln!("Error: {e}");
                    return;
                }
            };
            solve_yin_yang(
                &computation,
                BufReader::new(file),
                std::io::stdout(),
                std::io::stderr(),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::str::from_utf8;

    #[test]
    fn yy_solution_count_one() {
        let input = b"1000000001
0002000220
0020200200
0002002100
0101000100
0000012000
0000200010
0020020020
0022000000
0000020000";
        let mut output = Vec::new();
        let mut error = Vec::new();
        solve_yin_yang(
            &YyComputation::SolutionCount,
            &input[..],
            &mut output,
            &mut error,
        );
        assert_eq!(from_utf8(&output).unwrap(), "1\n");
    }

    #[test]
    fn yy_solution_count_none() {
        let input = b"1000200001
0002000220
0020200200
0002002100
0101000100
0000012000
0000200010
0020020020
0022000000
0000020000";
        let mut output = Vec::new();
        let mut error = Vec::new();
        solve_yin_yang(
            &YyComputation::SolutionCount,
            &input[..],
            &mut output,
            &mut error,
        );
        assert_eq!(from_utf8(&output).unwrap(), "0\n");
    }

    #[test]
    fn yy_solution_count_many() {
        let input = b"1000000001
0002000220
0020200200
0002002100
0100000100
0000000000
0000000010
0000000020
0022000000
0000020000";
        let mut output = Vec::new();
        let mut error = Vec::new();
        solve_yin_yang(
            &YyComputation::SolutionCount,
            &input[..],
            &mut output,
            &mut error,
        );
        assert_eq!(from_utf8(&output).unwrap(), "1024\n2048\n2515\n");
    }

    #[test]
    fn yy_tc_none() {
        let input = b"1000200001
0002000220
0020200200
0002002100
0101000100
0000012000
0000200010
0020020020
0022000000
0000020000";
        let mut output = Vec::new();
        let mut error = Vec::new();
        solve_yin_yang(
            &YyComputation::TrueCandidates,
            &input[..],
            &mut output,
            &mut error,
        );
        assert_eq!(from_utf8(&output).unwrap(), "No solutions found.\n");
    }

    #[test]
    fn yy_tc_many() {
        let input = b"1000000001
0002000220
0020200200
0002002100
0101000100
0000000000
0000000010
0020020020
0022000000
0000020000";
        let mut output = Vec::new();
        let mut error = Vec::new();
        solve_yin_yang(
            &YyComputation::TrueCandidates,
            &input[..],
            &mut output,
            &mut error,
        );
        assert_eq!(from_utf8(&output).unwrap(), "1 1 1 1 1 1 1 1 1 1 \n1 2 1 2 1 2 1 2 2 3 \n1 2 2 2 2 2 2 2 1 3 \n1 2 1 2 1 1 2 1 1 2 \n1 1 1 1 1 2 2 1 2 2 \n1 2 1 2 3 3 3 1 1 2 \n1 2 2 2 3 3 3 2 1 2 \n1 1 2 1 1 2 3 2 2 2 \n1 2 2 2 3 3 1 1 1 2 \n1 1 1 3 3 2 2 2 2 2 \n\n");
    }

    #[test]
    fn bad_yy() {
        let input = b"1000000001
00020002200020200200
0002002100
0101000100
0000000000
0000000010
0020020020
0022000000
0000020000";
        let mut output = Vec::new();
        let mut error = Vec::new();
        solve_yin_yang(
            &YyComputation::TrueCandidates,
            &input[..],
            &mut output,
            &mut error,
        );
        assert_eq!(
            from_utf8(&error).unwrap(),
            "Error: line 1 is a different length from previous lines.\n"
        );
    }

    #[test]
    fn yy_empty_file() {
        let input = b"";
        let mut output = Vec::new();
        let mut error = Vec::new();
        solve_yin_yang(
            &YyComputation::TrueCandidates,
            &input[..],
            &mut output,
            &mut error,
        );
        assert_eq!(from_utf8(&error).unwrap(), "Error: File is empty.\n");
    }
}
