use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse, Extension, Json
};
use futures::{SinkExt, StreamExt};
use mongodb::Client;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::{ Mutex, mpsc::{unbounded_channel, UnboundedSender } };
use serde_json::to_string;

use crate::{add_ws_session, delete_ws_session, get_sessions, models::session_model::SessionUpdate, update_ws_session};

pub struct SharedState {
    clients: Vec<UnboundedSender<Message>>,
}

impl SharedState {
    pub fn new() -> Self {
        SharedState {
            clients: Vec::new(),
        }
    }

    pub fn broadcast(&self, action_type: &str, status: &str, data: Value) {
        let message = json!({
            "action_type": action_type,
            "status": status,
            "data": data
        });
        let message_text = to_string(&message).unwrap_or_else(|_| "{}".to_string());
        for client in &self.clients {
            if let Err(e) = client.send(Message::Text(message_text.clone())) {
                eprintln!("Failed to broadcast message: {}", e);
            }
        }
    }
}

pub async fn websocket_handler(ws: WebSocketUpgrade, client: Extension<Arc<Client>>, shared_state: Arc<Mutex<SharedState>>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, client, shared_state.clone()))
}

async fn handle_socket(socket: WebSocket, client: Extension<Arc<Client>>, shared_state: Arc<Mutex<SharedState>>) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = unbounded_channel::<Message>();

    tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            if let Err(e) = sender.send(message).await {
                eprintln!("Failed to send message: {}", e);
                break;
            }
        }
    });

    {
        let mut state = shared_state.lock().await;
        state.clients.push(tx);
    }
    
    while let Some(Ok(Message::Text(text))) = receiver.next().await {
        let request: Value = serde_json::from_str(&text).unwrap_or_else(|_| {
            eprintln!("Failed to parse request text to JSON.");
            serde_json::Value::Null
        });
        let state = shared_state.lock().await;

        match request["action"].as_str() {
            Some(action_type) => {
                let (status, data) = match action_type {
                    "get_sessions" => {
                        let response = get_sessions(client.clone()).await;
                        match response {
                            Ok(sessions) => ("success", json!(sessions)),
                            Err(e) => {
                                eprintln!("Failed to get sessions: {}", e);
                                ("error", json!({"error": "Failed to get sessions"}))
                            },
                        }
                    },
                    "add_session" => {
                        if let Ok(session_data) = serde_json::from_value::<SessionUpdate>(request["data"].clone()) {
                            let response = add_ws_session(client.clone(), Json(session_data)).await;
                            match response {
                                Ok(session) => ("success", json!(session)),
                                Err(e) => {
                                    eprintln!("Failed to add session: {}", e);
                                    ("error", json!({"error": "Failed to add session"}))
                                },
                            }
                        } else {
                            eprintln!("Failed to parse session data");
                            ("error", json!({"error": "Failed to parse session data"}))
                        }
                    },
                    "update_session" => {
                        if let Ok(session_update) = serde_json::from_value::<SessionUpdate>(request["data"].clone()) {
                            let id_str = request["id"].as_str().unwrap_or_default();
                            let response = update_ws_session(client.clone(), axum::extract::Path(id_str.to_string()), Json(session_update)).await;
                            match response {
                                Ok(session) => ("success", json!(session)),
                                Err(e) => {
                                    eprintln!("Failed to update session: {}", e);
                                    ("error", json!({"error": "Failed to update session"}))
                                },
                            }
                        } else {
                            eprintln!("Failed to parse session update data");
                            ("error", json!({"error": "Failed to parse session update data"}))
                        }
                    },
                    "delete_session" => {
                        let id_str = request["id"].as_str().unwrap_or_default();
                        let response = delete_ws_session(client.clone(), axum::extract::Path(id_str.to_string())).await;
                        match response {
                            Ok(_) => ("success", json!({"message": "Session deleted successfully", "_id": id_str})),
                            Err(e) => {
                                eprintln!("Failed to delete session: {}", e);
                                ("error", json!({"error": "Failed to delete session"}))
                            },
                        }
                    },
                    _ => {
                        eprintln!("Unsupported action received.");
                        ("error", json!({"error": "Unsupported action"}))
                    }
                };
                state.broadcast(action_type, status, data);
            },
            None => {
                eprintln!("Action type is missing.");
                state.broadcast("error", "error", json!({"error": "Action type is missing"}));
            }
        }
    }
}
