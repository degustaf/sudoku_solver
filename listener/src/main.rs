//! A websocket server that provides server-side sudoku solver functionality.
//! This has the goal of being a drop-in replacement for Rangsk's
//! [`SudokuSolver`](https://github.com/dclamage/SudokuSolver) using
//! [f-puzzles](https://www.f-puzzles.com) as a frontend.

#![warn(missing_docs)]
use crate::types::Response;
use futures_util::{SinkExt, StreamExt, TryFutureExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_util::sync::CancellationToken;
use tungstenite::protocol::Message;

mod requests;
mod types;

async fn handler(ws: TcpStream) {
    println!("Connection established.");
    let ws_stream = tokio_tungstenite::accept_async(ws)
        .await
        .expect("Error during the websocket handshake occurred");
    let (mut ws_tx, mut ws_rcv) = ws_stream.split();

    let (ch_tx, ch_rx) = mpsc::channel::<Response>(100); // I would be very surprised if we get anywhere close to this.
    let mut ch_rx = ReceiverStream::new(ch_rx);

    tokio::task::spawn(async move {
        while let Some(message) = ch_rx.next().await {
            let json = serde_json::to_string(&message).unwrap(); // to_string failing is a programmer error, and should panic.
            let msg = Message::text(json);
            ws_tx
                .send(msg)
                .unwrap_or_else(|e| {
                    eprintln!("websocket send error: {e}");
                })
                .await;
        }
    });

    while let Some(result) = ws_rcv.next().await {
        let token = CancellationToken::new();
        let msg = match result {
            Ok(msg) => {
                match msg {
                    Message::Text(msg) => msg,
                    Message::Close(_) => {
                        eprintln!("Connection closed.");
                        break;
                    }
                    _ => {
                        eprintln!("Unable to process binary message from websocket: {msg:?}");
                        continue;
                    }
                }
            }
            Err(e) => {
                eprintln!("websocket error: {e}");
                continue;
            }
        };

        requests::process_message(&msg, token, ch_tx.clone()).await;
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let addr = "127.0.0.1:4545";
    let listener = TcpListener::bind(&addr).await.expect("Can't listen");

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handler(stream));
    }
}
