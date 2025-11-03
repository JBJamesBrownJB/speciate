//! WebSocket server for real-time simulation state broadcasting

use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use futures::{sink::SinkExt, stream::StreamExt};
use std::sync::Arc;
use tokio::sync::broadcast;

const BROADCAST_CAPACITY: usize = 100;

#[derive(Clone)]
pub struct AppState {
    pub tx: broadcast::Sender<String>,
    _rx: std::sync::Arc<tokio::sync::broadcast::Receiver<String>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        let (tx, rx) = broadcast::channel(BROADCAST_CAPACITY);
        Self {
            tx,
            _rx: std::sync::Arc::new(rx),
        }
    }

    pub fn broadcast<T: serde::Serialize>(&self, message: T) -> Result<usize, String> {
        let json = serde_json::to_string(&message).map_err(|e| e.to_string())?;
        self.tx.send(json).map_err(|e| e.to_string())
    }
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.tx.subscribe();

    let mut send_task = tokio::spawn(async move {
        while let Ok(json) = rx.recv().await {
            if sender.send(Message::Text(json.into())).await.is_err() {
                break;
            }
        }
    });

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(_msg)) = receiver.next().await {
            // Client messages ignored in Phase 1
        }
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_creation() {
        let state = AppState::new();
        // AppState keeps one receiver alive to prevent channel closure
        assert_eq!(state.tx.receiver_count(), 1);
    }
}
