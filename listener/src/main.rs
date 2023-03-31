//! A websocket server that provides server-side sudoku solver functionality.
//! This has the goal of being a drop-in replacement for Rangsk's
//! [`SudokuSolver`](https://github.com/dclamage/SudokuSolver) using
//! [f-puzzles](https://www.f-puzzles.com) as a frontend.

#![warn(missing_docs)]
use crate::types::Response;
use futures_util::{SinkExt, StreamExt, TryFutureExt};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_util::sync::CancellationToken;
use warp::filters::ws::Message;
use warp::ws::{WebSocket, Ws};
use warp::{cors, Filter};

mod requests;
mod types;

/// This function is heavily based on the
/// [warp example](https://github.com/seanmonstar/warp/blob/master/examples/websockets_chat.rs).
async fn handler(ws: WebSocket) {
    println!("Connection established.");
    let (mut ws_tx, mut ws_rcv) = ws.split();

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
                if msg.is_close() {
                    eprintln!("Connection closed.");
                    break;
                } else if let Ok(msg) = msg.to_str() {
                    String::from(msg)
                } else {
                    eprintln!("Unable to process binary message from websocket: {msg:?}");
                    continue;
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
    let route = warp::path::end()
        .and(warp::ws())
        .map(|ws: Ws| ws.on_upgrade(handler))
        .with(cors().allow_any_origin());
    warp::serve(route).run(([127, 0, 0, 1], 4545)).await;
}
