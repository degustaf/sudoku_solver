//! A command line utility for solving sudoku.

use clap::{Parser, Subcommand};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

#[derive(Subcommand)]
enum Command {
    /// Take a string representation of a grid from the command line and solve it.
    Solve { repr: String },

    /// Treat each line of a file as an individual puzzle, and solve all of them.
    FromFile { path: PathBuf },
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
    }
}
