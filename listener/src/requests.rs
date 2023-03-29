#![warn(missing_docs)]

use f_puzzles::FPuzzles;
use serde::{Deserialize, Serialize};
use std::string::FromUtf16Error;

#[derive(Debug, Deserialize, Serialize)]
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

#[derive(Debug, Deserialize, Serialize)]
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

impl From<serde_json::Error> for Response {
    fn from(e: serde_json::Error) -> Self {
        Response::Invalid {
            nonce: 0,
            message: e.to_string(),
        }
    }
}

#[allow(dead_code)]
struct Error {
    msg: String,
}

impl From<FromUtf16Error> for Error {
    fn from(e: FromUtf16Error) -> Self {
        Error { msg: e.to_string() }
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error { msg: e.to_string() }
    }
}

fn process_fpuzzles_data(nonce: usize, _command: &Command, data: &str) -> Result<Response, Error> {
    let f_data = match lz_str::decompress_from_base64(data) {
        Some(f) => String::from_utf16(&f)?,
        None => {
            return Err(Error {
                msg: "Corrupted N64 encoded data.".to_string(),
            });
        }
    };
    let _f_puz: FPuzzles = serde_json::from_str(&f_data)?;
    Ok(Response::Invalid {
        nonce,
        message: String::new(),
    })
}

fn process_message_helper(msg: &str) -> Result<Response, Response> {
    let v: Request = serde_json::from_str(msg)?;
    match v {
        Request::Cancel { nonce, command } => {
            println!("{command:?}: {nonce}");
            // TODO
            Ok(Response::Cancelled { nonce })
        }
        Request::Command {
            nonce,
            command,
            data_type,
            data,
        } => {
            if data_type != "fpuzzles" {
                return Err(Response::Invalid {
                    nonce,
                    message: "Invalid data format".to_string(),
                });
            }
            println!("{command:?}");
            match process_fpuzzles_data(nonce, &command, &data) {
                Ok(res) => Ok(res),
                Err(e) => Err(Response::Invalid {
                    nonce,
                    message: e.msg,
                }),
            }
        }
    }
}

pub fn process_message(msg: &str) -> Response {
    match process_message_helper(msg) {
        Ok(res) => res,
        Err(e) => e,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json::Value;

    #[test]
    fn utf16_error() {
        let v = &[0xD834, 0xDD1E, 0x006d, 0x0075, 0xD800, 0x0069, 0x0063];
        let u = String::from_utf16(v);
        assert!(u.is_err());
        let e: Error = Error::from(u.unwrap_err());
        assert_eq!(e.msg, "invalid utf-16: lone surrogate found");
    }

    #[test]
    fn serde_json_error() {
        let bad_json = "{[[], []}]";
        let json_err: Result<Value, serde_json::Error> = serde_json::from_str(bad_json);
        assert!(json_err.is_err());
        let err = Error::from(json_err.unwrap_err());
        assert_eq!(err.msg, "key must be a string at line 1 column 2");
    }

    #[test]
    fn serde_json_rewponse() {
        let bad_json = "{[[], []}]";
        let json_err: Result<Value, serde_json::Error> = serde_json::from_str(bad_json);
        assert!(json_err.is_err());
        let resp = Response::from(json_err.unwrap_err());
        assert_eq!(
            resp,
            Response::Invalid {
                nonce: 0,
                message: "key must be a string at line 1 column 2".to_string()
            }
        );
    }

    #[test]
    fn bad_n64_data() {
        let processed = process_fpuzzles_data(0, &Command::Solve, "000000");
        assert!(processed.is_err());
        let err = processed.unwrap_err();
        assert_eq!(err.msg, "Corrupted N64 encoded data.".to_string());
    }

    #[test]
    fn cancel_request() {
        let data = Request::Cancel {
            nonce: 9,
            command: Command::Cancel,
        };
        let request = serde_json::to_string(&data).unwrap();
        let response = process_message(&request);
        assert_eq!(response, Response::Cancelled { nonce: 9 });
    }

    #[test]
    fn not_fpuzzles_data() {
        let data = Request::Command {
            nonce: 9,
            command: Command::Solve,
            data_type: "not f-puzzles".to_string(),
            data: String::new(),
        };
        let request = serde_json::to_string(&data).unwrap();
        let response = process_message(&request);
        assert_eq!(
            response,
            Response::Invalid {
                nonce: 9,
                message: "Invalid data format".to_string()
            }
        );
    }

    #[test]
    fn bad_n64_data_in_request() {
        let data = Request::Command {
            nonce: 9,
            command: Command::Solve,
            data_type: "fpuzzles".to_string(),
            data: "000000".to_string(),
        };
        let request = serde_json::to_string(&data).unwrap();
        let response = process_message(&request);
        assert_eq!(
            response,
            Response::Invalid {
                nonce: 9,
                message: "Corrupted N64 encoded data.".to_string(),
            }
        );
    }
}
