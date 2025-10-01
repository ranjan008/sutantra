use anyhow::Result;
use tokio::sync::mpsc;
use tracing::{info, debug, warn, error};
use std::collections::HashMap;

use super::{StreamingConfig, StreamingEvent, StreamingCommand, StreamQualitySettings};
use super::webrtc::{MockWebRTCEngine, StreamData};
use super::real_webrtc_fixed::RealWebRTCEngine;

/// Core streaming engine that handles WebRTC connections
pub struct StreamingEngine {
    config: StreamingConfig,
    command_rx: mpsc::Receiver<StreamingCommand>,
    event_tx: mpsc::Sender<StreamingEvent>,
    webrtc_engine: WebRTCEngine,
    active_streams: HashMap<String, String>, // stream_id -> creator
}

/// Enum to handle both mock and real WebRTC engines
pub enum WebRTCEngine {
    Mock(MockWebRTCEngine),
    Real(RealWebRTCEngine),
}

impl WebRTCEngine {
    pub async fn create_stream(&mut self, stream_id: String, creator: String, quality: StreamQualitySettings) -> Result<()> {
        match self {
            WebRTCEngine::Mock(engine) => engine.create_stream(stream_id, creator, quality).await,
            WebRTCEngine::Real(engine) => engine.create_stream(stream_id, creator, quality).await,
        }
    }

    pub async fn connect_viewer(&mut self, stream_id: String, viewer: String) -> Result<()> {
        match self {
            WebRTCEngine::Mock(engine) => engine.connect_viewer(stream_id, viewer).await,
            WebRTCEngine::Real(engine) => engine.connect_viewer(stream_id, viewer).await,
        }
    }

    pub async fn disconnect_viewer(&mut self, stream_id: String, viewer: String) -> Result<()> {
        match self {
            WebRTCEngine::Mock(engine) => engine.disconnect_viewer(stream_id, viewer).await,
            WebRTCEngine::Real(engine) => engine.disconnect_viewer(stream_id, viewer).await,
        }
    }

    pub async fn list_active_streams(&self) -> Vec<String> {
        match self {
            WebRTCEngine::Mock(engine) => engine.list_active_streams().await,
            WebRTCEngine::Real(engine) => engine.list_active_streams().await,
        }
    }

    pub async fn get_stream_metrics(&self, stream_id: &str) -> Option<super::StreamMetrics> {
        match self {
            WebRTCEngine::Mock(engine) => engine.get_stream_metrics(stream_id).await,
            WebRTCEngine::Real(engine) => engine.get_stream_metrics(stream_id).await,
        }
    }
}

impl StreamingEngine {
    pub async fn new(
        config: StreamingConfig,
        command_rx: mpsc::Receiver<StreamingCommand>,
        event_tx: mpsc::Sender<StreamingEvent>,
    ) -> Result<Self> {
        info!("🎥 Initializing Streaming Engine");
        info!("📡 WebRTC port: {}", config.webrtc_port);
        info!("📺 Max streams: {}", config.max_streams);
        
        let webrtc_engine = if config.use_real_webrtc {
            info!("🎥 SELECTED: REAL WebRTC Engine (Production Mode)");
            info!("📡 Initializing webrtc-rs crate v0.7.3");
            let mut engine = RealWebRTCEngine::new(config.webrtc_port).await?;
            engine.start().await?;
            WebRTCEngine::Real(engine)
        } else {
            info!("🎥 SELECTED: Mock WebRTC Engine (Testing Mode)");
            let mut engine = MockWebRTCEngine::new(config.webrtc_port).await?;
            engine.start().await?;
            WebRTCEngine::Mock(engine)
        };
        
        Ok(Self {
            config,
            command_rx,
            event_tx,
            webrtc_engine,
            active_streams: HashMap::new(),
        })
    }
    
    pub async fn run(&mut self) -> Result<()> {
        info!("🚀 Starting Streaming Engine");
        
        // Start accepting WebRTC connections in background
        let _webrtc_engine = &self.webrtc_engine;
        tokio::spawn(async move {
            // Note: In the actual implementation, we'd clone webrtc_engine here
            // For now, we'll just simulate connection acceptance
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                debug!("🔄 WebRTC connection handler running");
            }
        });
        
        loop {
            tokio::select! {
                Some(command) = self.command_rx.recv() => {
                    if let Err(e) = self.handle_command(command).await {
                        error!("Error handling streaming command: {}", e);
                    }
                }
                
                else => {
                    warn!("Streaming command channel closed");
                    break;
                }
            }
        }
        
        Ok(())
    }
    
    async fn handle_command(&mut self, command: StreamingCommand) -> Result<()> {
        match command {
                StreamingCommand::PrepareStream { stream_id, creator } => {
                    info!("🎬 Preparing stream: {} by {}", stream_id, creator);

                    let quality = StreamQualitySettings::default();
                    self.webrtc_engine.create_stream(stream_id.clone(), creator.clone(), quality).await?;
                    self.active_streams.insert(stream_id.clone(), creator.clone());
                    
                    // Emit stream started event
                    self.event_tx.send(StreamingEvent::StreamStarted {
                        stream_id,
                        creator
                    }).await?;
                }
            
            StreamingCommand::StartStream { stream_id, creator, quality_settings } => {
                info!("▶️ Starting stream: {} by {}", stream_id, creator);
                
                // Create the stream with specified quality
                self.webrtc_engine.create_stream(stream_id.clone(), creator.clone(), quality_settings).await?;
                self.active_streams.insert(stream_id.clone(), creator.clone());
                
                // Start generating some demo stream data
                let stream_id_clone = stream_id.clone();
                let event_tx = self.event_tx.clone();
                
                tokio::spawn(async move {
                    // Simulate stream data generation
                    for frame in 0..30 { // 30 frames for demo
                        // In the demo, we just simulate successful data transmission
                        debug!("📡 Simulated stream data frame {}", frame);
                        
                        tokio::time::sleep(tokio::time::Duration::from_millis(33)).await; // ~30 FPS
                    }
                    
                    // Emit stream ended event
                    let _ = event_tx.send(StreamingEvent::StreamEnded { 
                        stream_id: stream_id_clone, 
                        duration_seconds: 1 
                    }).await;
                });
                
                self.event_tx.send(StreamingEvent::StreamStarted { 
                    stream_id, 
                    creator 
                }).await?;
            }
            
            StreamingCommand::GrantAccess { stream_id, viewer } => {
                info!("✅ Granting access: {} to stream {}", viewer, stream_id);
                
                self.webrtc_engine.connect_viewer(stream_id.clone(), viewer.clone()).await?;
                
                self.event_tx.send(StreamingEvent::ViewerConnected { 
                    stream_id, 
                    viewer_id: viewer 
                }).await?;
            }
            
            StreamingCommand::RevokeAccess { stream_id, viewer } => {
                info!("❌ Revoking access: {} from stream {}", viewer, stream_id);
                
                self.webrtc_engine.disconnect_viewer(stream_id.clone(), viewer.clone()).await?;
                
                self.event_tx.send(StreamingEvent::ViewerDisconnected { 
                    stream_id, 
                    viewer_id: viewer,
                    reason: "Access revoked".to_string(),
                }).await?;
            }
            
            StreamingCommand::ConnectToStream { stream_id, viewer } => {
                info!("🔗 Connecting {} to stream {}", viewer, stream_id);
                
                self.webrtc_engine.connect_viewer(stream_id.clone(), viewer.clone()).await?;
                
                self.event_tx.send(StreamingEvent::ViewerConnected { 
                    stream_id, 
                    viewer_id: viewer 
                }).await?;
            }
            
            _ => {
                debug!("Unhandled streaming command: {:?}", command);
            }
        }
        
        Ok(())
    }
    
    pub async fn get_active_streams(&self) -> Vec<String> {
        self.webrtc_engine.list_active_streams().await
    }
    
    pub async fn get_stream_metrics(&self, stream_id: &str) -> Option<super::StreamMetrics> {
        self.webrtc_engine.get_stream_metrics(stream_id).await
    }
}
