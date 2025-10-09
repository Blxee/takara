use std::{error::Error, sync::Arc, time::Duration};

use axum::{
    Router,
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::IntoResponse,
    routing::any,
    serve,
};
use futures_util::{SinkExt, StreamExt};
use tokio::{
    join,
    net::TcpListener,
    spawn,
    sync::{Mutex, mpsc},
    time::sleep,
};

struct Room {
    player_a: Option<mpsc::Sender<Message>>,
    player_b: Option<mpsc::Sender<Message>>,
}

impl Room {
    fn get_buddy(&self, id: u32) -> Option<&mpsc::Sender<Message>> {
        match id {
            0 => self.player_b.as_ref(),
            1 => self.player_a.as_ref(),
            _ => panic!(),
        }
    }
}

type GameState = Arc<Mutex<Room>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let state: GameState = Arc::new(Mutex::new(Room {
        player_a: None,
        player_b: None,
    }));

    let app = Router::new()
        .route("/ws", any(ws_handler))
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:8888").await?;

    println!("listening at {}", listener.local_addr().unwrap());
    serve(listener, app).await?;
    Ok(())
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<GameState>) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: GameState) {
    let (tx, mut rx) = mpsc::channel::<Message>(10);

    let (mut sender, mut receiver) = socket.split();

    let id: u32;

    {
        let mut state = state.lock().await;

        if state.player_a.is_none() {
            (*state).player_a = Some(tx);
            id = 0;
            dbg!("player_a connected..");
        } else {
            (*state).player_b = Some(tx);
            id = 1;
            dbg!("player_b connected..");
        }
    }

    while state.lock().await.get_buddy(id).is_none() {
        sleep(Duration::from_millis(200)).await;
    }

    let other = state.lock().await.get_buddy(id).unwrap().clone();

    let send_task = spawn(async move {
        loop {
            let msg = rx.recv().await.unwrap();
            sender.send(msg).await.unwrap();
        }
    });

    let recv_task = spawn(async move {
        loop {
            let msg = receiver.next().await.unwrap().unwrap();
            other.send(msg).await.unwrap();
        }
    });

    let _ = join!(send_task, recv_task);
}
