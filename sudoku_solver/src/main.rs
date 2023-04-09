//! A command line utility for solving sudoku.

use clap::{Parser, Subcommand};
use sudoku_engine;

#[derive(Subcommand)]
enum Command {
    /// Take a string representation of a grid from the command line and solve it.
    Solve { repr: String },
}

fn solve_puzzle(repr: String) {
    fn helper(repr: String) -> Result<sudoku_engine::Board, sudoku_engine::SudokuErrors> {
        let board = sudoku_engine::from_string(&repr)?;
        sudoku_engine::solve(&board)
    }
    match helper(repr) {
        Ok(_board) => {
            println!("Solved!");
        }
        Err(e) => {
            eprintln!("Error: {e}");
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
        Command::Solve { repr } => solve_puzzle(repr),
    }
}
