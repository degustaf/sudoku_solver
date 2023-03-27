//! A websocket server that provides server-side sudoku solver functionality.
//! This has the goal of being a drop-in replacement for Rangsk's
//! [`SudokuSolver`](https://github.com/dclamage/SudokuSolver) using
//! [f-puzzles](https://www.f-puzzles.com) as a frontend.

#![warn(missing_docs)]
use futures_util::{SinkExt, StreamExt, TryFutureExt};
use warp::filters::ws::Message;
use warp::ws::{WebSocket, Ws};
use warp::{cors, Filter};

mod requests;

/// This function is heavily based on the
/// [warp example](https://github.com/seanmonstar/warp/blob/master/examples/websockets_chat.rs).
async fn handler(ws: WebSocket) {
    println!("Connection established.");
    let (mut ws_tx, mut ws_rcv) = ws.split();

    while let Some(result) = ws_rcv.next().await {
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

        let response = requests::process_message(&msg);
        let json = serde_json::to_string(&response).unwrap(); // to_string failing is a programmer error, and should panic.
        let msg = Message::text(json);
        ws_tx
            .send(msg)
            .unwrap_or_else(|e| {
                eprintln!("websocket send error: {e}");
            })
            .await;
    }
}

#[tokio::main]
async fn main() {
    let route = warp::path::end()
        .and(warp::ws())
        .map(|ws: Ws| ws.on_upgrade(handler))
        .with(cors().allow_any_origin());
    warp::serve(route).run(([127, 0, 0, 1], 4545)).await;
}

// listener/src/main.rs: 17-19, 21-25, 27-28, 30, 34-35, 40-46, 48, 52-58
