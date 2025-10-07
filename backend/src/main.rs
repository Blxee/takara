use std::error::Error;

use axum::{
    Router,
    extract::{WebSocketUpgrade, ws::WebSocket},
    response::IntoResponse,
    routing::any,
    serve,
};
use futures_util::{SinkExt, StreamExt};
use tokio::{join, net::TcpListener, spawn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let app = Router::new().route("/ws", any(ws_handler));
    let listener = TcpListener::bind("127.0.0.1:8888").await?;

    println!("listening at {}", listener.local_addr().unwrap());
    serve(listener, app).await?;
    Ok(())
}

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(socket: WebSocket) {
    let (mut sender, mut receiver) = socket.split();

    let send_task = spawn(async move {
        loop {
            let mut buf = String::new();
            std::io::stdin().read_line(&mut buf).unwrap();
            sender.send(buf.into()).await.unwrap();
        }
    });

    let recv_task = spawn(async move {
        loop {
            let msg = receiver.next().await;
            println!("Client: {:?}", msg.unwrap().unwrap().to_text().unwrap());
        }
    });

    let _ = join!(send_task, recv_task);
}
