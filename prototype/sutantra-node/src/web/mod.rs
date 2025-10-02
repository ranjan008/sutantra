use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use warp::Filter;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use futures_util::{SinkExt, StreamExt};
use warp::ws::{WebSocket, Message, Ws};

use crate::integration::SutantraEvent;
use crate::streaming::StreamingCommand;

pub mod handlers;
pub mod websocket;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebUIMessage {
    #[serde(rename = "type")]
    pub message_type: String,
    pub data: serde_json::Value,
    pub client_id: Option<String>,
    pub stream_id: Option<String>,
}

pub struct WebServer {
    port: u16,
    event_sender: mpsc::UnboundedSender<SutantraEvent>,
    streaming_sender: mpsc::UnboundedSender<StreamingCommand>,
    clients: Arc<RwLock<HashMap<String, mpsc::UnboundedSender<Result<Message, warp::Error>>>>>,
}

impl WebServer {
    pub fn new(
        port: u16,
        event_sender: mpsc::UnboundedSender<SutantraEvent>,
        streaming_sender: mpsc::UnboundedSender<StreamingCommand>,
    ) -> Self {
        Self {
            port,
            event_sender,
            streaming_sender,
            clients: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        tracing::info!("🌐 Starting Sutantra Web Server on port {}", self.port);

        let clients = self.clients.clone();
        let event_sender = self.event_sender.clone();
        let streaming_sender = self.streaming_sender.clone();

        // Static file serving
        let static_files = warp::fs::dir("../../web-ui");

        // WebSocket endpoint
        let ws_clients = clients.clone();
        let ws_event_sender = event_sender.clone();
        let ws_streaming_sender = streaming_sender.clone();
        
        let websocket = warp::path("ws")
            .and(warp::ws())
            .map(move |ws: Ws| {
                let clients = ws_clients.clone();
                let event_sender = ws_event_sender.clone();
                let streaming_sender = ws_streaming_sender.clone();
                
                ws.on_upgrade(move |websocket| {
                    websocket::handle_websocket(websocket, clients, event_sender, streaming_sender)
                })
            });

        // API endpoints
        let health = warp::path("health")
            .map(|| warp::reply::json(&serde_json::json!({"status": "healthy"})));

        let node_info = warp::path!("api" / "node" / "info")
            .map(|| warp::reply::json(&serde_json::json!({
                "node_id": "sutantra-node-1",
                "version": "0.1.0",
                "network": "sutantra-testnet",
                "streaming_enabled": true
            })));

        // Combine all routes
        let routes = static_files
            .or(websocket)
            .or(health)
            .or(node_info)
            .with(warp::cors().allow_any_origin().allow_headers(vec!["content-type"]).allow_methods(vec!["GET", "POST"]));

        tracing::info!("✅ Web server ready on http://localhost:{}", self.port);
        tracing::info!("🔌 WebSocket endpoint: ws://localhost:{}/ws", self.port);

        warp::serve(routes)
            .run(([127, 0, 0, 1], self.port))
            .await;

        Ok(())
    }

    pub async fn broadcast_event(&self, event: SutantraEvent) {
        let message = WebUIMessage {
            message_type: "event".to_string(),
            data: serde_json::to_value(event).unwrap_or_default(),
            client_id: None,
            stream_id: None,
        };

        let json_message = serde_json::to_string(&message).unwrap_or_default();
        let ws_message = Message::text(json_message);

        let clients = self.clients.read().await;
        for (client_id, sender) in clients.iter() {
            if let Err(e) = sender.send(Ok(ws_message.clone())) {
                tracing::warn!("Failed to send message to client {}: {}", client_id, e);
            }
        }
    }
}