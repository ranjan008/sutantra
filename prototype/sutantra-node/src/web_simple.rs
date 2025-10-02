// Simple web server with WebSocket support
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use warp::Filter;
use warp::ws::{WebSocket, Message, Ws};
use futures_util::{SinkExt, StreamExt};

use crate::integration::SutantraEvent;
use crate::streaming::StreamingCommand;

// Global stream state management
static ACTIVE_STREAMS: tokio::sync::OnceCell<Arc<RwLock<HashMap<String, StreamInfo>>>> = tokio::sync::OnceCell::const_new();

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct StreamInfo {
    stream_id: String,
    title: String,
    creator: String,
    creator_port: u16,
    viewers: u32,
    quality: String,
    status: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

async fn get_stream_state() -> Arc<RwLock<HashMap<String, StreamInfo>>> {
    ACTIVE_STREAMS.get_or_init(|| async {
        Arc::new(RwLock::new(HashMap::new()))
    }).await.clone()
}

async fn get_active_streams() -> Vec<serde_json::Value> {
    let streams = get_stream_state().await;
    let streams_guard = streams.read().await;
    
    streams_guard.values().map(|stream| {
        serde_json::json!({
            "stream_id": stream.stream_id,
            "title": stream.title,
            "creator": stream.creator,
            "viewers": stream.viewers,
            "quality": stream.quality,
            "status": stream.status,
            "created_at": stream.created_at
        })
    }).collect()
}

async fn add_stream(stream_info: StreamInfo) -> anyhow::Result<()> {
    let streams = get_stream_state().await;
    let mut streams_guard = streams.write().await;
    streams_guard.insert(stream_info.stream_id.clone(), stream_info);
    Ok(())
}

async fn remove_stream(stream_id: &str) -> anyhow::Result<()> {
    let streams = get_stream_state().await;
    let mut streams_guard = streams.write().await;
    streams_guard.remove(stream_id);
    Ok(())
}

pub struct SimpleWebServer {
    port: u16,
    event_sender: mpsc::UnboundedSender<SutantraEvent>,
    streaming_sender: mpsc::UnboundedSender<StreamingCommand>,
    clients: Arc<RwLock<HashMap<String, mpsc::UnboundedSender<Result<Message, warp::Error>>>>>,
}

impl SimpleWebServer {
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

    pub async fn start(&self) -> anyhow::Result<()> {
        tracing::info!("🌐 Starting Sutantra Web Server on port {}", self.port);

        let clients = self.clients.clone();
        let event_sender = self.event_sender.clone();
        let streaming_sender = self.streaming_sender.clone();

        // Static file serving - check for symlink first, then fallback to relative path
        let static_files = if std::path::Path::new("web-ui").exists() {
            warp::fs::dir("web-ui")
        } else {
            warp::fs::dir("../../web-ui")
        };

        // WebSocket endpoint
        let ws_clients = clients.clone();
        let ws_event_sender = event_sender.clone();
        let ws_streaming_sender = streaming_sender.clone();
        let ws_port = self.port;
        
        let websocket = warp::path("ws")
            .and(warp::ws())
            .map(move |ws: Ws| {
                let clients = ws_clients.clone();
                let event_sender = ws_event_sender.clone();
                let streaming_sender = ws_streaming_sender.clone();
                let port = ws_port;
                
                ws.on_upgrade(move |websocket| {
                    handle_websocket(websocket, clients, event_sender, streaming_sender, port)
                })
            });

        // API endpoints
        let health = warp::path("health")
            .map(|| warp::reply::json(&serde_json::json!({"status": "healthy"})));

        let port = self.port;
        let node_info = warp::path!("api" / "node" / "info")
            .map(move || {
                let node_id = if port == 8080 {
                    "sutantra-streamer-node"
                } else if port == 8082 {
                    "sutantra-viewer-node"
                } else {
                    "sutantra-node"
                };
                
                let node_type = if port == 8080 {
                    "streamer"
                } else if port == 8082 {
                    "viewer"
                } else {
                    "unknown"
                };
                
                warp::reply::json(&serde_json::json!({
                    "node_id": node_id,
                    "node_type": node_type,
                    "version": "0.1.0",
                    "network": "sutantra-testnet",
                    "streaming_enabled": true,
                    "web_port": port
                }))
            });

        // API endpoint to get active streams
        let streams_api = warp::path!("api" / "streams")
            .and(warp::get())
            .and_then(|| async move {
                let streams = get_active_streams().await;
                Ok::<_, warp::Rejection>(warp::reply::json(&streams))
            });

        // Combine all routes
        let routes = static_files
            .or(websocket)
            .or(health)
            .or(node_info)
            .or(streams_api)
            .with(warp::cors().allow_any_origin().allow_headers(vec!["content-type"]).allow_methods(vec!["GET", "POST"]));

        tracing::info!("✅ Web server ready on http://localhost:{}", self.port);
        tracing::info!("🔌 WebSocket endpoint: ws://localhost:{}/ws", self.port);

        warp::serve(routes)
            .run(([127, 0, 0, 1], self.port))
            .await;

        Ok(())
    }
}

async fn handle_websocket(
    websocket: WebSocket,
    clients: Arc<RwLock<HashMap<String, mpsc::UnboundedSender<Result<Message, warp::Error>>>>>,
    event_sender: mpsc::UnboundedSender<SutantraEvent>,
    streaming_sender: mpsc::UnboundedSender<StreamingCommand>,
    port: u16,
) {
    let client_id = format!("client_{}", chrono::Utc::now().timestamp_millis());
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
                    port,
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
    clients: &Arc<RwLock<HashMap<String, mpsc::UnboundedSender<Result<Message, warp::Error>>>>>,
    _event_sender: &mpsc::UnboundedSender<SutantraEvent>,
    _streaming_sender: &mpsc::UnboundedSender<StreamingCommand>,
    port: u16,
) -> anyhow::Result<()> {
    if msg.is_text() {
        let text = msg.to_str().map_err(|_| anyhow::anyhow!("Failed to convert message to string"))?;
        tracing::info!("📨 Received message from {}: {}", client_id, text);

        // Parse the message
        let ui_message: serde_json::Value = serde_json::from_str(text)?;
        
        // Handle different message types
        match ui_message.get("type").and_then(|v| v.as_str()) {
            Some("handshake") => {
                // Determine node type based on port
                let node_type = if port == 8080 {
                    "streamer"
                } else if port == 8082 {
                    "viewer"
                } else {
                    "unknown"
                };
                
                let node_id = if port == 8080 {
                    "sutantra-streamer-node"
                } else if port == 8082 {
                    "sutantra-viewer-node"
                } else {
                    "sutantra-node"
                };
                
                let response = serde_json::json!({
                    "type": "handshakeResponse",
                    "data": {
                        "success": true,
                        "client_id": client_id,
                        "node_type": node_type,
                        "node_id": node_id,
                        "web_port": port,
                        "message": format!("Connected to Sutantra {} node", node_type)
                    }
                });
                
                send_to_client(client_id, clients, response).await?;
            }
            Some("createStream") => {
                tracing::info!("🎬 Stream creation request from {}", client_id);
                
                // Generate a stream ID
                let stream_id = format!("stream_{}", chrono::Utc::now().timestamp_millis());
                let title = ui_message.get("data")
                    .and_then(|d| d.get("title"))
                    .and_then(|t| t.as_str())
                    .unwrap_or("Live Stream")
                    .to_string();
                
                // Create stream info and add to global state
                let stream_info = StreamInfo {
                    stream_id: stream_id.clone(),
                    title: title.clone(),
                    creator: if port == 8080 { "sutantra-streamer-node".to_string() } else { format!("node-{}", port) },
                    creator_port: port,
                    viewers: 0,
                    quality: "720p".to_string(),
                    status: "active".to_string(),
                    created_at: chrono::Utc::now(),
                };
                
                add_stream(stream_info).await?;
                tracing::info!("✅ Added stream {} to global state", stream_id);
                
                let response = serde_json::json!({
                    "type": "createStreamResponse",
                    "data": {
                        "success": true,
                        "stream_id": stream_id,
                        "title": title,
                        "message": "Stream created successfully"
                    }
                });
                
                send_to_client(client_id, clients, response).await?;
                
                // Broadcast to all viewers that a new stream is available
                broadcast_stream_list(clients, port).await?;
            }
            Some("requestStreamList") => {
                tracing::info!("📋 Stream list request from {} on port {} (port type: {})", client_id, port, std::any::type_name::<u16>());
                
                let mut streams = get_active_streams().await;
                
                // If this is a viewer node (port 8082), also query the streamer node for active streams
                tracing::info!("🔍 Checking if port {} == 8082: {}", port, port == 8082);
                if port == 8082 {
                    tracing::info!("👥 Viewer requesting streams, querying streamer node...");
                    
                    // Query streamer node for active streams
                    match query_streamer_node_for_streams().await {
                        Ok(streamer_streams) => {
                            tracing::info!("✅ Found {} streams from streamer node", streamer_streams.len());
                            streams.extend(streamer_streams);
                        }
                        Err(e) => {
                            tracing::warn!("⚠️ Failed to query streamer node: {}", e);
                        }
                    }
                } else {
                    tracing::info!("🚫 Not a viewer node (port {} != 8082), skipping cross-node query", port);
                }
                
                tracing::info!("📋 Returning {} total active streams to {}", streams.len(), client_id);
                
                let response = serde_json::json!({
                    "type": "streamList",
                    "data": {
                        "streams": streams
                    }
                });
                
                send_to_client(client_id, clients, response).await?;
            }
            Some("joinStream") => {
                tracing::info!("👥 Stream join request from {}", client_id);
                
                let stream_id = ui_message.get("stream_id")
                    .and_then(|s| s.as_str())
                    .unwrap_or("unknown");
                
                let offer = ui_message.get("offer");
                
                tracing::info!("👥 Joining stream: {} with offer: {}", stream_id, offer.is_some());
                
                // For now, simulate a successful join
                // In a full implementation, we would:
                // 1. Find the streamer for this stream_id
                // 2. Forward the offer to the streamer
                // 3. Wait for the streamer's answer
                // 4. Send the answer back to the viewer
                
                let response = serde_json::json!({
                    "type": "joinStreamResponse",
                    "data": {
                        "success": true,
                        "stream_id": stream_id,
                        "message": "Successfully joined stream - WebRTC handshake initiated"
                    }
                });
                
                send_to_client(client_id, clients, response).await?;
            }
            Some("leaveStream") => {
                tracing::info!("🚪 Stream leave request from {}", client_id);
                
                let response = serde_json::json!({
                    "type": "leaveStreamResponse",
                    "data": {
                        "success": true,
                        "message": "Successfully left stream"
                    }
                });
                
                send_to_client(client_id, clients, response).await?;
            }
            Some("stopStream") => {
                tracing::info!("🛑 Stream stop request from {}", client_id);
                
                if let Some(stream_id) = ui_message.get("data")
                    .and_then(|d| d.get("stream_id"))
                    .and_then(|s| s.as_str()) {
                    
                    remove_stream(stream_id).await?;
                    tracing::info!("✅ Removed stream {} from global state", stream_id);
                    
                    let response = serde_json::json!({
                        "type": "stopStreamResponse",
                        "data": {
                            "success": true,
                            "stream_id": stream_id,
                            "message": "Stream stopped successfully"
                        }
                    });
                    
                    send_to_client(client_id, clients, response).await?;
                    
                    // Broadcast updated stream list to all viewers
                    broadcast_stream_list(clients, port).await?;
                } else {
                    let response = serde_json::json!({
                        "type": "stopStreamResponse",
                        "data": {
                            "success": false,
                            "message": "Missing stream_id"
                        }
                    });
                    
                    send_to_client(client_id, clients, response).await?;
                }
            }
            Some("webrtcOffer") => {
                tracing::info!("📡 WebRTC offer from {}", client_id);
                
                let offer = ui_message.get("offer");
                let stream_id = ui_message.get("stream_id")
                    .and_then(|s| s.as_str())
                    .unwrap_or("unknown");
                
                tracing::info!("📡 Offer for stream: {}", stream_id);
                
                // For now, just acknowledge the offer
                let response = serde_json::json!({
                    "type": "webrtcOfferResponse",
                    "data": {
                        "success": true,
                        "stream_id": stream_id,
                        "message": "Offer received"
                    }
                });
                
                send_to_client(client_id, clients, response).await?;
            }
            Some("webrtcAnswer") => {
                tracing::info!("📡 WebRTC answer from {}", client_id);
                
                let answer = ui_message.get("answer");
                let stream_id = ui_message.get("stream_id")
                    .and_then(|s| s.as_str())
                    .unwrap_or("unknown");
                
                tracing::info!("📡 Answer for stream: {}", stream_id);
                
                let response = serde_json::json!({
                    "type": "webrtcAnswerResponse",
                    "data": {
                        "success": true,
                        "stream_id": stream_id,
                        "message": "Answer received"
                    }
                });
                
                send_to_client(client_id, clients, response).await?;
            }
            Some("iceCandidate") => {
                tracing::info!("🧊 ICE candidate from {}", client_id);
                
                let candidate = ui_message.get("candidate");
                let stream_id = ui_message.get("stream_id")
                    .and_then(|s| s.as_str())
                    .unwrap_or("unknown");
                
                tracing::info!("🧊 ICE candidate for stream: {}", stream_id);
                
                let response = serde_json::json!({
                    "type": "iceCandidateResponse",
                    "data": {
                        "success": true,
                        "stream_id": stream_id,
                        "message": "ICE candidate received"
                    }
                });
                
                send_to_client(client_id, clients, response).await?;
            }
            _ => {
                tracing::warn!("🤷 Unknown message type from {}: {:?}", client_id, ui_message.get("type"));
            }
        }
    }
    Ok(())
}

async fn send_to_client(
    client_id: &str,
    clients: &Arc<RwLock<HashMap<String, mpsc::UnboundedSender<Result<Message, warp::Error>>>>>,
    message: serde_json::Value,
) -> anyhow::Result<()> {
    let json_message = serde_json::to_string(&message)?;
    let ws_message = Message::text(json_message);

    let clients_guard = clients.read().await;
    if let Some(sender) = clients_guard.get(client_id) {
        sender.send(Ok(ws_message))?;
    }
    Ok(())
}

async fn broadcast_stream_list(
    clients: &Arc<RwLock<HashMap<String, mpsc::UnboundedSender<Result<Message, warp::Error>>>>>,
    _port: u16,
) -> anyhow::Result<()> {
    // Get actual active streams from global state
    let streams = get_active_streams().await;
    
    let broadcast_message = serde_json::json!({
        "type": "streamList",
        "data": {
            "streams": streams
        }
    });
    
    let json_message = serde_json::to_string(&broadcast_message)?;
    let ws_message = Message::text(json_message);
    
    let clients_guard = clients.read().await;
    for (client_id, sender) in clients_guard.iter() {
        tracing::info!("📡 Broadcasting stream list to client: {}", client_id);
        if let Err(e) = sender.send(Ok(ws_message.clone())) {
            tracing::warn!("Failed to send broadcast to {}: {}", client_id, e);
        }
    }
    
    Ok(())
}

// Function to query the streamer node for active streams
async fn query_streamer_node_for_streams() -> anyhow::Result<Vec<serde_json::Value>> {
    // Make HTTP request to streamer node's API endpoint
    let client = reqwest::Client::new();
    let url = "http://localhost:8080/api/streams";
    
    tracing::info!("🔍 Querying streamer node at: {}", url);
    
    match client.get(url).timeout(std::time::Duration::from_secs(5)).send().await {
        Ok(response) => {
            if response.status().is_success() {
                let streams: Vec<serde_json::Value> = response.json().await?;
                tracing::info!("✅ Successfully retrieved {} streams from streamer node", streams.len());
                Ok(streams)
            } else {
                tracing::warn!("⚠️ Streamer node returned status: {}", response.status());
                Ok(vec![])
            }
        }
        Err(e) => {
            tracing::warn!("⚠️ Failed to connect to streamer node: {}", e);
            Ok(vec![]) // Return empty list instead of error to not break the flow
        }
    }
}
