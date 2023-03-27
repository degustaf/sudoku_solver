#![warn(missing_docs)]

use f_puzzles::FPuzzles;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
enum Command {
    #[serde(rename = "solvepath")]
    SolvePath,
    #[serde(rename = "step")]
    Step,
    #[serde(rename = "solve")]
    Solve,
    #[serde(rename = "check")]
    Check,
    #[serde(rename = "cancel")]
    Cancel,
    #[serde(rename = "count")]
    Count,
    #[serde(rename = "truecandidates")]
    TrueCandidates,
}

impl std::fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Request {
    Command {
        nonce: usize,
        command: Command,
        #[allow(dead_code)]
        #[serde(rename = "dataType")]
        data_type: String,
        data: String,
    },
    Cancel {
        nonce: usize,
        command: Command,
    },
}

#[derive(Debug, Serialize)]
pub struct LogicalCell {
    value: usize,
    candidates: Vec<usize>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum Response {
    #[serde(rename = "cancelled")]
    Cancelled { nonce: usize },
    #[serde(rename = "invalid")]
    Invalid { nonce: usize, message: String },
    #[serde(rename = "truecandidates")]
    #[allow(dead_code)]
    TrueCandidates {
        nonce: usize,
        #[serde(rename = "solutionsPerCandidate")]
        solutions_per_candidate: Vec<usize>,
    },
    #[serde(rename = "solved")]
    #[allow(dead_code)]
    Solved { nonce: usize, solution: Vec<usize> },
    #[serde(rename = "count")]
    #[allow(dead_code)]
    Count {
        nonce: usize,
        count: usize,
        #[serde(rename = "inProgress")]
        in_progress: bool,
    },
    #[serde(rename = "logical")]
    #[allow(dead_code)]
    LogicalResponse {
        nonce: usize,
        cells: Vec<LogicalCell>,
        message: String,
        #[serde(rename = "isValid")]
        is_valid: bool,
    },
}

pub fn process_message(msg: &str) -> Response {
    let v: Request = match serde_json::from_str(msg) {
        Ok(v) => v,
        Err(e) => {
            return Response::Invalid {
                nonce: 0,
                message: e.to_string(),
            };
        }
    };
    match v {
        Request::Cancel { nonce, command } => {
            println!("{command}: {nonce}");
            // TODO
            Response::Cancelled { nonce }
        }
        Request::Command {
            nonce,
            command,
            data_type: _,
            data,
        } => {
            let f_data = match lz_str::decompress_from_base64(&data) {
                Some(f) => match String::from_utf16(&f) {
                    Ok(f) => f,
                    Err(e) => {
                        return Response::Invalid {
                            nonce,
                            message: e.to_string(),
                        };
                    }
                },
                None => {
                    return Response::Invalid {
                        nonce,
                        message: "Corrupted N64 encoded data.".to_string(),
                    };
                }
            };
            println!("{command}");
            let _f_puz: FPuzzles = match serde_json::from_str(&f_data) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("Error parsing f-puzzles data: {e}");
                    return Response::Invalid {
                        nonce,
                        message: e.to_string(),
                    };
                }
            };
            Response::Invalid {
                nonce,
                message: String::new(),
            }
        }
    }
}
