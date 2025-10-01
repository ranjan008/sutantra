use anyhow::Result;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{info, error};

use crate::blockchain::{BlockchainEngine, BlockchainEvent, BlockchainCommand};
use crate::streaming::{StreamingEngine, StreamingEvent, StreamingCommand};

pub mod events;
pub mod node;

pub use node::SutantraNode;

/// Central event that coordinates between blockchain and streaming layers
#[derive(Debug, Clone)]
pub enum SutantraEvent {
    // Blockchain → Streaming events
    StreamRegistered { stream_id: String, creator: String },
    PaymentReceived { stream_id: String, viewer: String, amount: u64 },
    AccessGranted { stream_id: String, viewer: String },
    AccessRevoked { stream_id: String, viewer: String },
    
    // Streaming → Blockchain events  
    StreamStarted { stream_id: String, creator: String },
    ViewerJoined { stream_id: String, viewer: String },
    ViewerLeft { stream_id: String, viewer: String },
    StreamEnded { stream_id: String, duration_seconds: u64 },
    QualityReport { stream_id: String, metrics: StreamQualityMetrics },
    
    // Cross-layer events
    PaymentDue { stream_id: String, viewer: String, amount: u64 },
    ConnectionFailed { stream_id: String, viewer: String, reason: String },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StreamQualityMetrics {
    pub bandwidth_mbps: f64,
    pub latency_ms: u32,
    pub packet_loss_percent: f64,
    pub resolution: String,
    pub fps: u32,
}

/// Event bridge that coordinates between blockchain and streaming
pub struct EventBridge {
    // Communication channels
    blockchain_tx: mpsc::Sender<BlockchainCommand>,
    streaming_tx: mpsc::Sender<StreamingCommand>,
    
    // Event receivers
    blockchain_events: mpsc::Receiver<BlockchainEvent>,
    streaming_events: mpsc::Receiver<StreamingEvent>,
    
    // Internal event bus
    event_bus: mpsc::Sender<SutantraEvent>,
    event_rx: mpsc::Receiver<SutantraEvent>,
    
    // Shared state
    active_streams: Arc<RwLock<std::collections::HashMap<String, StreamState>>>,
}

#[derive(Debug, Clone)]
pub struct StreamState {
    pub stream_id: String,
    pub creator: String,
    pub viewers: Vec<String>,
    pub is_active: bool,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub total_earnings: u64,
    pub quality_metrics: Option<StreamQualityMetrics>,
}

impl EventBridge {
    pub fn new(
        blockchain_tx: mpsc::Sender<BlockchainCommand>,
        streaming_tx: mpsc::Sender<StreamingCommand>,
        blockchain_events: mpsc::Receiver<BlockchainEvent>,
        streaming_events: mpsc::Receiver<StreamingEvent>,
    ) -> Self {
        let (event_bus, event_rx) = mpsc::channel(1000);
        
        Self {
            blockchain_tx,
            streaming_tx,
            blockchain_events,
            streaming_events,
            event_bus,
            event_rx,
            active_streams: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }
    
    /// Start the event bridge loop
    pub async fn run(&mut self) -> Result<()> {
        info!("🌉 Starting Event Bridge");
        
        loop {
            tokio::select! {
                // Handle blockchain events
                Some(blockchain_event) = self.blockchain_events.recv() => {
                    self.handle_blockchain_event(blockchain_event).await?;
                }
                
                // Handle streaming events
                Some(streaming_event) = self.streaming_events.recv() => {
                    self.handle_streaming_event(streaming_event).await?;
                }
                
                // Handle internal coordination events
                Some(sutantra_event) = self.event_rx.recv() => {
                    self.handle_sutantra_event(sutantra_event).await?;
                }
                
                else => {
                    error!("All event channels closed");
                    break;
                }
            }
        }
        
        Ok(())
    }
    
    async fn handle_blockchain_event(&self, event: BlockchainEvent) -> Result<()> {
        match event {
            BlockchainEvent::StreamRegistered { stream_id, creator } => {
                info!("📝 Stream registered on blockchain: {}", stream_id);
                
                // Notify streaming layer to prepare for stream
                let cmd = StreamingCommand::PrepareStream { 
                    stream_id: stream_id.clone(), 
                    creator: creator.clone() 
                };
                self.streaming_tx.send(cmd).await?;
                
                // Update local state
                let mut streams = self.active_streams.write().await;
                streams.insert(stream_id.clone(), StreamState {
                    stream_id: stream_id.clone(),
                    creator,
                    viewers: Vec::new(),
                    is_active: false,
                    start_time: chrono::Utc::now(),
                    total_earnings: 0,
                    quality_metrics: None,
                });
            }
            
            BlockchainEvent::PaymentProcessed { stream_id, viewer, amount } => {
                info!("💳 Payment processed: {} STREAM from {} for {}", amount, viewer, stream_id);
                
                // Grant access to streaming layer
                let cmd = StreamingCommand::GrantAccess { 
                    stream_id: stream_id.clone(), 
                    viewer: viewer.clone() 
                };
                self.streaming_tx.send(cmd).await?;
                
                // Update earnings
                let mut streams = self.active_streams.write().await;
                if let Some(stream) = streams.get_mut(&stream_id) {
                    stream.total_earnings += amount;
                }
            }
            
            _ => {
                // Handle other blockchain events
            }
        }
        
        Ok(())
    }
    
    async fn handle_streaming_event(&self, event: StreamingEvent) -> Result<()> {
        match event {
            StreamingEvent::StreamStarted { stream_id, creator } => {
                info!("🎬 Stream started: {} by {}", stream_id, creator);
                
                // Update blockchain state
                let cmd = BlockchainCommand::RecordStreamStart { 
                    stream_id: stream_id.clone(), 
                    timestamp: chrono::Utc::now() 
                };
                self.blockchain_tx.send(cmd).await?;
                
                // Update local state
                let mut streams = self.active_streams.write().await;
                if let Some(stream) = streams.get_mut(&stream_id) {
                    stream.is_active = true;
                    stream.start_time = chrono::Utc::now();
                }
            }
            
            StreamingEvent::ViewerConnected { stream_id, viewer_id } => {
                info!("👥 Viewer connected: {} to stream {}", viewer_id, stream_id);
                
                // Check if payment is required and valid
                let cmd = BlockchainCommand::CheckAccess { 
                    stream_id: stream_id.clone(), 
                    viewer: viewer_id.clone() 
                };
                self.blockchain_tx.send(cmd).await?;
                
                // Update local state
                let mut streams = self.active_streams.write().await;
                if let Some(stream) = streams.get_mut(&stream_id) {
                    if !stream.viewers.contains(&viewer_id) {
                        stream.viewers.push(viewer_id);
                    }
                }
            }
            
            StreamingEvent::QualityUpdate { stream_id, metrics } => {
                // Convert to internal metrics format
                let quality_metrics = StreamQualityMetrics {
                    bandwidth_mbps: metrics.bandwidth_mbps,
                    latency_ms: metrics.latency_ms,
                    packet_loss_percent: metrics.packet_loss_percent,
                    resolution: metrics.resolution,
                    fps: metrics.fps,
                };
                
                // Update local state
                let mut streams = self.active_streams.write().await;
                if let Some(stream) = streams.get_mut(&stream_id) {
                    stream.quality_metrics = Some(quality_metrics.clone());
                }
                
                // Report to blockchain for validator rewards
                let cmd = BlockchainCommand::ReportStreamQuality { 
                    stream_id, 
                    metrics: quality_metrics 
                };
                self.blockchain_tx.send(cmd).await?;
            }
            
            _ => {
                // Handle other streaming events
            }
        }
        
        Ok(())
    }
    
    async fn handle_sutantra_event(&self, event: SutantraEvent) -> Result<()> {
        match event {
            SutantraEvent::PaymentDue { stream_id, viewer, amount } => {
                info!("⏰ Payment due: {} STREAM from {} for {}", amount, viewer, stream_id);
                
                // Trigger payment processing
                let cmd = BlockchainCommand::ProcessPayment { 
                    stream_id, 
                    viewer, 
                    amount 
                };
                self.blockchain_tx.send(cmd).await?;
            }
            
            _ => {
                // Handle other coordination events
            }
        }
        
        Ok(())
    }
    
    /// Get current stream status
    pub async fn get_stream_status(&self, stream_id: &str) -> Option<StreamState> {
        let streams = self.active_streams.read().await;
        streams.get(stream_id).cloned()
    }
    
    /// Get all active streams
    pub async fn get_active_streams(&self) -> Vec<StreamState> {
        let streams = self.active_streams.read().await;
        streams.values()
            .filter(|stream| stream.is_active)
            .cloned()
            .collect()
    }
}
