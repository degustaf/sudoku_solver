#![warn(missing_docs)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub(crate) enum Command {
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

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub(crate) enum Request {
    Command {
        nonce: usize,
        command: Command,
        #[serde(rename = "dataType")]
        data_type: String,
        data: String,
    },
    Cancel {
        nonce: usize,
        command: Command,
    },
}

#[derive(Debug, PartialEq, Serialize)]
pub struct LogicalCell {
    value: usize,
    candidates: Vec<usize>,
}

#[derive(Debug, PartialEq, Serialize)]
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

impl From<serde_json::Error> for Response {
    fn from(e: serde_json::Error) -> Self {
        Response::Invalid {
            nonce: 0,
            message: e.to_string(),
        }
    }
}

pub(crate) struct Error {
    pub(crate) msg: String,
}

impl<T: ToString> From<T> for Error {
    fn from(e: T) -> Self {
        Error { msg: e.to_string() }
    }
}
