//! A command line utility for solving sudoku.

use crate::build_irregular::build_irregular;
use clap::{Parser, Subcommand, ValueEnum};
use solution_iter::{SolutionIterator, true_candidates_bfs};
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

fn solve_yin_yang(computation: YyComputation, path: &Path) {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Error: {e}");
            return;
        }
    };
    let mut data = BufReader::new(file).lines();
    let mut repr = match data.next() {
        Some(Ok(repr)) => repr,
        Some(Err(e)) => {
            eprintln!("Error on line 1: {e}");
            return;
        }
        None => {
            eprintln!("Error: {} is empty.", path.display());
            return;
        }
    };

    let mut height = 1;
    let width = repr.len();
    for (i, line) in data.enumerate() {
        let line_repr = match line {
            Ok(repr) => repr,
            Err(e) => {
                eprintln!("Error on line {}: {e}", i + 1);
                return;
            }
        };
        if line_repr.len() != width {
            eprintln!(
                "Error: line {} is a different length from previous lines.",
                i + 1
            );
        }
        height += 1;
        repr.push_str(&line_repr);
    }

    let yy = match YinYang::from_string(height, width, &repr) {
        Ok(yy) => yy,
        Err(e) => {
            eprintln!("Error: {e}");
            return;
        }
    };
    match computation {
        YyComputation::TrueCandidates => match true_candidates_bfs(&yy) {
            Some(tc) => {
                println!("{tc}");
            }
            None => {
                eprintln!("No solutions found.");
            }
        },
        YyComputation::SolutionCount => {
            let mut i = 0;
            for _ in SolutionIterator::new(&yy) {
                i += 1;
                if i & 0x3FF == 0 {
                    println!("{i}");
                }
            }
            println!("{i}");
        }
        YyComputation::Candidates => {
            for cand in SolutionIterator::new(&yy) {
                println!("{cand}");
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
        Command::YinYang { computation, path } => solve_yin_yang(computation, &path),
    }
}
