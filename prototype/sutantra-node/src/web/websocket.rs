use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use futures_util::{SinkExt, StreamExt};
use warp::ws::{WebSocket, Message};
use serde_json::Value;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

use crate::integration::SutantraEvent;
use crate::streaming::{StreamingCommand, StreamQualitySettings};
use crate::web::WebUIMessage;

pub type WebSocketSender = mpsc::UnboundedSender<Result<Message, warp::Error>>;
pub type WebSocketClients = Arc<RwLock<HashMap<String, WebSocketSender>>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeRequest {
    pub client_type: String, // "streamer" or "viewer"
    pub node_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateStreamRequest {
    pub title: String,
    pub description: Option<String>,
    pub quality: StreamQualitySettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateStreamResponse {
    pub success: bool,
    pub message: String,
    pub stream_id: Option<String>,
    pub error: Option<String>,
}

pub async fn handle_websocket(
    websocket: WebSocket,
    clients: WebSocketClients,
    event_sender: mpsc::UnboundedSender<SutantraEvent>,
    streaming_sender: mpsc::UnboundedSender<StreamingCommand>,
) {
    let client_id = Uuid::new_v4().to_string();
    tracing::info!("🔌 New WebSocket connection: {}", client_id);

    let (mut ws_tx, mut ws_rx) = websocket.split();
    let (client_tx, mut client_rx) = mpsc::unbounded_channel();

    // Add client to the clients map
    clients.write().await.insert(client_id.clone(), client_tx);

    // Handle outgoing messages to client
    let clients_for_cleanup = clients.clone();
    let client_id_for_cleanup = client_id.clone();
    tokio::spawn(async move {
        while let Some(msg) = client_rx.recv().await {
            let message = match msg {
                Ok(m) => m,
                Err(_) => break,
            };
            if ws_tx.send(message).await.is_err() {
                break;
            }
        }
        // Clean up client when connection closes
        clients_for_cleanup.write().await.remove(&client_id_for_cleanup);
        tracing::info!("🔌 WebSocket connection closed: {}", client_id_for_cleanup);
    });

    // Handle incoming messages from client
    while let Some(result) = ws_rx.next().await {
        match result {
            Ok(msg) => {
                if let Err(e) = handle_message(
                    msg,
                    &client_id,
                    &clients,
                    &event_sender,
                    &streaming_sender,
                ).await {
                    tracing::error!("Error handling message from {}: {}", client_id, e);
                    break;
                }
            }
            Err(e) => {
                tracing::error!("WebSocket error for client {}: {}", client_id, e);
                break;
            }
        }
    }

    // Clean up client
    clients.write().await.remove(&client_id);
    tracing::info!("🔌 WebSocket connection terminated: {}", client_id);
}

async fn handle_message(
    msg: Message,
    client_id: &str,
    clients: &WebSocketClients,
    event_sender: &mpsc::UnboundedSender<SutantraEvent>,
    streaming_sender: &mpsc::UnboundedSender<StreamingCommand>,
) -> anyhow::Result<()> {
    if msg.is_text() {
        let text = msg.to_str().map_err(|_| anyhow::anyhow!("Failed to convert message to string"))?;
        tracing::debug!("📨 Received message from {}: {}", client_id, text);

        let ui_message: WebUIMessage = serde_json::from_str(text)?;

        match ui_message.message_type.as_str() {
            "handshake" => {
                let handshake_req: HandshakeRequest = serde_json::from_value(ui_message.data.clone())?;
                tracing::info!("🤝 Handshake from {} (type: {})", client_id, handshake_req.client_type);

                let response_message = WebUIMessage {
                    message_type: "handshakeResponse".to_string(),
                    data: serde_json::to_value(serde_json::json!({
                        "success": true,
                        "client_id": client_id,
                        "message": "Connected to Sutantra node"
                    }))?,
                    client_id: Some(client_id.to_string()),
                    stream_id: None,
                };
                send_to_client(client_id, clients, response_message).await?;
            },
            
            "createStream" => {
                let mut create_req: CreateStreamRequest = serde_json::from_value(ui_message.data.clone())?;
                
                // Ensure quality settings have all required fields
                if create_req.quality.resolution.is_empty() {
                    create_req.quality.resolution = "1280x720".to_string();
                }
                let stream_id = Uuid::new_v4().to_string();

                tracing::info!("🎬 Creating stream '{}' for client {}", create_req.title, client_id);

                let response_message = WebUIMessage {
                    message_type: "createStreamResponse".to_string(),
                    data: serde_json::to_value(CreateStreamResponse {
                        success: true,
                        message: "Stream creation initiated".to_string(),
                        stream_id: Some(stream_id.clone()),
                        error: None,
                    })?,
                    client_id: Some(client_id.to_string()),
                    stream_id: Some(stream_id.clone()),
                };
                send_to_client(client_id, clients, response_message).await?;

                let streaming_command = StreamingCommand::PrepareStream {
                    stream_id: stream_id.clone(),
                    creator: client_id.to_string(),
                };

                if let Err(e) = streaming_sender.send(streaming_command) {
                    tracing::error!("Failed to send streaming command: {}", e);
                    send_error(client_id, clients, "Failed to create stream").await?;
                }
            },
            
            "joinStream" => {
                if let Some(stream_id) = ui_message.stream_id {
                    tracing::info!("👥 Client {} joining stream {}", client_id, stream_id);

                    let streaming_command = StreamingCommand::ConnectToStream {
                        stream_id: stream_id.clone(),
                        viewer: client_id.to_string(),
                    };

                    if let Err(e) = streaming_sender.send(streaming_command) {
                        tracing::error!("Failed to send join command: {}", e);
                        send_error(client_id, clients, "Failed to join stream").await?;
                    }
                }
            },
            
            "updateQuality" => {
                if let Some(stream_id) = ui_message.stream_id {
                    let mut quality: StreamQualitySettings = serde_json::from_value(ui_message.data)?;
                    
                    // Ensure quality settings have all required fields
                    if quality.resolution.is_empty() {
                        quality.resolution = "1280x720".to_string();
                    }
                    tracing::info!("⚙️ Updating quality for stream {} from client {}", stream_id, client_id);

                    // For now, we'll use StartStream to update quality
                    let streaming_command = StreamingCommand::StartStream {
                        stream_id: stream_id.clone(),
                        creator: client_id.to_string(),
                        quality_settings: quality,
                    };

                    if let Err(e) = streaming_sender.send(streaming_command) {
                        tracing::error!("Failed to send quality update: {}", e);
                        send_error(client_id, clients, "Failed to update quality").await?;
                    }
                }
            },
            
            "ping" => {
                let pong_message = WebUIMessage {
                    message_type: "pong".to_string(),
                    data: serde_json::to_value(serde_json::json!({
                        "timestamp": chrono::Utc::now().timestamp_millis()
                    }))?,
                    client_id: Some(client_id.to_string()),
                    stream_id: None,
                };
                send_to_client(client_id, clients, pong_message).await?;
            },
            
            _ => {
                tracing::warn!("Unknown message type '{}' from client {}", ui_message.message_type, client_id);
            }
        }
    }
    Ok(())
}

async fn send_to_client(
    client_id: &str,
    clients: &WebSocketClients,
    message: WebUIMessage,
) -> anyhow::Result<()> {
    let json_message = serde_json::to_string(&message)?;
    let ws_message = Message::text(json_message);

    let clients_guard = clients.read().await;
    if let Some(sender) = clients_guard.get(client_id) {
        sender.send(Ok(ws_message))?;
    }
    Ok(())
}

async fn send_error(
    client_id: &str,
    clients: &WebSocketClients,
    error_message: &str,
) -> anyhow::Result<()> {
    let error_msg = WebUIMessage {
        message_type: "error".to_string(),
        data: serde_json::to_value(serde_json::json!({
            "message": error_message
        }))?,
        client_id: Some(client_id.to_string()),
        stream_id: None,
    };
    send_to_client(client_id, clients, error_msg).await
}