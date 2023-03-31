#![warn(missing_docs)]

use crate::types::{Command, Error, Request, Response};
use f_puzzles::FPuzzles;
use sudoku_engine::Board;
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::TrySendError;
use tokio_util::sync::CancellationToken;

fn check_solutions(
    nonce: usize,
    f_puz: &FPuzzles,
    token: &CancellationToken,
    ch_tx: &mpsc::Sender<Response>,
) -> Result<(), Error> {
    let b = Board::try_from(f_puz)?;
    let mut solns = b.solutions();
    for count in 0..2 {
        if token.is_cancelled() {
            return Ok(());
        }
        if solns.next().is_none() {
            while let Err(TrySendError::Full(_)) = ch_tx.try_send(Response::Count {
                nonce,
                count,
                in_progress: false,
            }) {}
            return Ok(());
        }
    }

    while let Err(TrySendError::Full(_)) = ch_tx.try_send(Response::Count {
        nonce,
        count: 2,
        in_progress: false,
    }) {}
    Ok(())
}

#[allow(clippy::needless_pass_by_value)]
async fn process_fpuzzles_data(
    nonce: usize,
    command: &Command,
    data: &str,
    token: CancellationToken,
    ch_tx: mpsc::Sender<Response>,
) -> Result<(), Error> {
    let f_data = match lz_str::decompress_from_base64(data) {
        Some(f) => String::from_utf16(&f)?,
        None => {
            return Err(Error {
                msg: "Corrupted N64 encoded data.".to_string(),
            });
        }
    };
    let f_puz: FPuzzles = serde_json::from_str(&f_data)?;
    match command {
        Command::Check => {
            // This should spawn to a new thread where we do parallel CPU bound computations.
            check_solutions(nonce, &f_puz, &token, &ch_tx)?;
        }
        Command::Cancel => {
            token.cancel();
            if (ch_tx.send(Response::Cancelled { nonce }).await).is_ok() {};
        }
        _ => {
            todo!();
        }
    }
    Ok(())
}

pub async fn process_message(msg: &str, token: CancellationToken, ch_tx: mpsc::Sender<Response>) {
    let v: Request = match serde_json::from_str(msg) {
        Ok(v) => v,
        Err(e) => {
            if (ch_tx.send(Response::from(e)).await).is_ok() {};
            return;
        }
    };
    match v {
        Request::Cancel { nonce, command } => {
            println!("{command:?}: {nonce}");
            token.cancel();
            if (ch_tx.send(Response::Cancelled { nonce }).await).is_ok() {};
        }
        Request::Command {
            nonce,
            command,
            data_type,
            data,
        } => {
            if data_type != "fpuzzles" {
                if (ch_tx
                    .send(Response::Invalid {
                        nonce,
                        message: "Invalid data format".to_string(),
                    })
                    .await)
                    .is_ok()
                {};
                return;
            }
            println!("{command:?}");
            if let Err(e) =
                process_fpuzzles_data(nonce, &command, &data, token, ch_tx.clone()).await
            {
                if (ch_tx
                    .send(Response::Invalid {
                        nonce,
                        message: e.msg,
                    })
                    .await)
                    .is_ok()
                {};
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use futures_util::StreamExt;
    use serde_json::Value;
    use tokio_stream::wrappers::ReceiverStream;

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

    #[tokio::test]
    async fn bad_n64_data() {
        let token = CancellationToken::new();
        let (ch_tx, _ch_rx) = mpsc::channel::<Response>(1);
        let processed = process_fpuzzles_data(0, &Command::Solve, "000000", token, ch_tx).await;
        assert!(processed.is_err());
        let err = processed.unwrap_err();
        assert_eq!(err.msg, "Corrupted N64 encoded data.".to_string());
    }

    #[tokio::test]
    async fn cancel_request() {
        let data = Request::Cancel {
            nonce: 9,
            command: Command::Cancel,
        };
        let request = serde_json::to_string(&data).unwrap();
        let token = CancellationToken::new();
        let (ch_tx, ch_rx) = mpsc::channel::<Response>(1);
        let mut ch_rx = ReceiverStream::new(ch_rx);
        process_message(&request, token.clone(), ch_tx).await;
        assert!(token.is_cancelled());
        let response = ch_rx.next().await;
        assert!(response.is_some());
        assert_eq!(response.unwrap(), Response::Cancelled { nonce: 9 });
    }

    #[tokio::test]
    async fn not_fpuzzles_data() {
        let data = Request::Command {
            nonce: 9,
            command: Command::Solve,
            data_type: "not f-puzzles".to_string(),
            data: String::new(),
        };
        let request = serde_json::to_string(&data).unwrap();
        let token = CancellationToken::new();
        let (ch_tx, ch_rx) = mpsc::channel::<Response>(1);
        let mut ch_rx = ReceiverStream::new(ch_rx);
        process_message(&request, token.clone(), ch_tx).await;
        let response = ch_rx.next().await;
        assert!(response.is_some());
        assert_eq!(
            response.unwrap(),
            Response::Invalid {
                nonce: 9,
                message: "Invalid data format".to_string()
            }
        );
    }

    #[tokio::test]
    async fn bad_n64_data_in_request() {
        let data = Request::Command {
            nonce: 9,
            command: Command::Solve,
            data_type: "fpuzzles".to_string(),
            data: "000000".to_string(),
        };
        let request = serde_json::to_string(&data).unwrap();
        let token = CancellationToken::new();
        let (ch_tx, ch_rx) = mpsc::channel::<Response>(1);
        let mut ch_rx = ReceiverStream::new(ch_rx);
        process_message(&request, token.clone(), ch_tx).await;
        let response = ch_rx.next().await;
        assert!(response.is_some());
        assert_eq!(
            response.unwrap(),
            Response::Invalid {
                nonce: 9,
                message: "Corrupted N64 encoded data.".to_string(),
            }
        );
    }

    #[test]
    fn check_puzzle() {
        let res_f = FPuzzles::try_from(
            "19..7..5....28..........37.2.5.....4...4.5.....6.....9731....2....82.....4....91.",
        );
        assert!(res_f.is_ok());
        let f = res_f.unwrap();
        let token = CancellationToken::new();
        let (ch_tx, mut ch_rx) = mpsc::channel::<Response>(1);
        assert!(check_solutions(42, &f, &token, &ch_tx).is_ok());
        let response = ch_rx.try_recv();
        assert!(response.is_ok());
        assert_eq!(
            response.unwrap(),
            Response::Count {
                nonce: 42,
                count: 1,
                in_progress: false,
            }
        );
        token.cancel();
        assert!(check_solutions(43, &f, &token, &ch_tx).is_ok());
    }

    #[test]
    fn check_puzzle_multiple_solutions() {
        let res_f = FPuzzles::try_from(
            ".9..7..5....28..........37.2.5.....4...4.5.....6.....9731....2....82.....4....91.",
        );
        assert!(res_f.is_ok());
        let f = res_f.unwrap();
        let token = CancellationToken::new();
        let (ch_tx, mut ch_rx) = mpsc::channel::<Response>(1);
        assert!(check_solutions(42, &f, &token, &ch_tx).is_ok());
        let response = ch_rx.try_recv();
        assert!(response.is_ok());
        assert_eq!(
            response.unwrap(),
            Response::Count {
                nonce: 42,
                count: 2,
                in_progress: false,
            }
        );
    }

    #[tokio::test]
    async fn process_fpuzzles_data_test() {
        let res_f = FPuzzles::try_from(
            ".9..7..5....28..........37.2.5.....4...4.5.....6.....9731....2....82.....4....91.",
        );
        assert!(res_f.is_ok());
        let f_str = serde_json::to_string(&res_f.unwrap());
        assert!(f_str.is_ok());
        let f_data = lz_str::compress_to_base64(&f_str.unwrap());
        let token = CancellationToken::new();
        let (ch_tx, mut ch_rx) = mpsc::channel::<Response>(1);
        assert!((process_fpuzzles_data(37, &Command::Check, &f_data, token, ch_tx).await).is_ok());
        let response = ch_rx.try_recv();
        assert!(response.is_ok());
        assert_eq!(
            response.unwrap(),
            Response::Count {
                nonce: 37,
                count: 2,
                in_progress: false,
            }
        );
    }
}
