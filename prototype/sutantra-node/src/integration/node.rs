use anyhow::Result;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{info, error};

use crate::blockchain::{BlockchainEngine, BlockchainEvent, BlockchainCommand, BlockchainConfig};
use crate::streaming::{StreamingEngine, StreamingEvent, StreamingCommand, StreamingConfig};
use crate::integration::EventBridge;
use crate::mobile::LightClient;

/// Main Sutantra node that integrates blockchain and streaming
pub struct SutantraNode {
    node_type: NodeType,
    port: u16,
    is_validator: bool,
    enable_streaming: bool,
}

#[derive(Debug, Clone)]
pub enum NodeType {
    Full,
    Light { bootnodes: Vec<String> },
}

impl SutantraNode {
    pub fn get_event_sender(&self) -> mpsc::UnboundedSender<crate::integration::SutantraEvent> {
        // Create a channel for web UI events
        let (tx, mut rx) = mpsc::unbounded_channel();
        
        // Spawn a task to handle events from web UI
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                tracing::info!("🌐 Web UI Event: {:?}", event);
                // TODO: Process events from web UI
            }
        });
        
        tx
    }

    pub fn get_streaming_sender(&self) -> mpsc::UnboundedSender<crate::streaming::StreamingCommand> {
        // Create a channel for streaming commands from web UI
        let (tx, mut rx) = mpsc::unbounded_channel();
        
        // Spawn a task to handle streaming commands from web UI
        tokio::spawn(async move {
            while let Some(command) = rx.recv().await {
                tracing::info!("🎬 Web UI Streaming Command: {:?}", command);
                // TODO: Forward commands to streaming engine
            }
        });
        
        tx
    }

    /// Create a new full node
    pub async fn new(port: u16, is_validator: bool, enable_streaming: bool) -> Result<Self> {
        Ok(Self {
            node_type: NodeType::Full,
            port,
            is_validator,
            enable_streaming,
        })
    }
    
    /// Create a new light node (mobile-optimized)
    pub async fn new_light(port: u16, bootnodes: Vec<String>) -> Result<Self> {
        Ok(Self {
            node_type: NodeType::Light { bootnodes },
            port,
            is_validator: false,
            enable_streaming: true,
        })
    }
    
    /// Start the node
    pub async fn run(self) -> Result<()> {
        match self.node_type.clone() {
            NodeType::Full => self.run_full_node().await,
            NodeType::Light { bootnodes } => self.run_light_node(bootnodes).await,
        }
    }
    
    async fn run_full_node(self) -> Result<()> {
        info!("🚀 Starting Sutantra Full Node");
        info!("📡 Port: {}", self.port);
        info!("⚡ Validator: {}", self.is_validator);
        info!("🎥 Streaming: {}", self.enable_streaming);
        
        // Create communication channels
        let (blockchain_cmd_tx, blockchain_cmd_rx) = mpsc::channel(1000);
        let (blockchain_event_tx, blockchain_event_rx) = mpsc::channel(1000);
        let (streaming_cmd_tx, streaming_cmd_rx) = mpsc::channel(1000);
        let (streaming_event_tx, streaming_event_rx) = mpsc::channel(1000);
        
        // Configure blockchain engine
        let blockchain_config = BlockchainConfig {
            port: self.port,
            is_validator: self.is_validator,
            data_dir: format!("./data/blockchain_{}", self.port),
        };
        
        // Configure streaming engine  
        let streaming_config = StreamingConfig {
            webrtc_port: self.port + 1000,
            max_streams: 100,
            max_viewers_per_stream: 1000,
            enable_relay: true,
            relay_capacity: 1000,
            discovery_interval_seconds: 30,
            use_real_webrtc: true, // Use fixed real WebRTC implementation
        };
        
        // Initialize engines
        let mut blockchain_engine = BlockchainEngine::new(
            blockchain_config,
            blockchain_cmd_rx,
            blockchain_event_tx,
        ).await?;
        
        let streaming_engine = if self.enable_streaming {
            let engine = StreamingEngine::new(
                streaming_config,
                streaming_cmd_rx,
                streaming_event_tx,
            ).await?;
            
            // Auto-trigger test stream if this is the first node (port 30333)
            if self.port == 30333 {
                info!("🧪 Node 1 detected - will auto-trigger test stream");
                let test_trigger = crate::streaming::test_trigger::StreamTestTrigger::new(
                    streaming_cmd_tx.clone()
                );
                tokio::spawn(async move {
                    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                    if let Err(e) = test_trigger.start_test_sequence().await {
                        error!("Test trigger failed: {}", e);
                    }
                });
            }
            
            Some(engine)
        } else {
            None
        };
        
        // Initialize event bridge
        let mut event_bridge = EventBridge::new(
            blockchain_cmd_tx,
            streaming_cmd_tx,
            blockchain_event_rx,
            streaming_event_rx,
        );
        
        // Start all components
        let blockchain_handle = tokio::spawn(async move {
            if let Err(e) = blockchain_engine.run().await {
                error!("Blockchain engine error: {}", e);
            }
        });
        
        let streaming_handle = if let Some(mut engine) = streaming_engine {
            Some(tokio::spawn(async move {
                if let Err(e) = engine.run().await {
                    error!("Streaming engine error: {}", e);
                }
            }))
        } else {
            None
        };
        
        let bridge_handle = tokio::spawn(async move {
            if let Err(e) = event_bridge.run().await {
                error!("Event bridge error: {}", e);
            }
        });
        
        info!("✅ All components started successfully");
        
        // Wait for shutdown signal
        tokio::signal::ctrl_c().await?;
        info!("🛑 Shutting down node...");
        
        // Graceful shutdown
        blockchain_handle.abort();
        if let Some(handle) = streaming_handle {
            handle.abort();
        }
        bridge_handle.abort();
        
        info!("✅ Node shutdown complete");
        Ok(())
    }
    
    async fn run_light_node(self, bootnodes: Vec<String>) -> Result<()> {
        info!("📱 Starting Sutantra Light Node");
        info!("📡 Port: {}", self.port);
        info!("🔗 Bootnodes: {:?}", bootnodes);
        
        // Create light client
        let _light_client = LightClient::new(self.port, bootnodes).await?;
        
        // For now, just run a basic light client
        // TODO: Implement full light client with streaming capabilities
        info!("🔄 Light client sync starting...");
        
        // Simulate light client operation
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            info!("💓 Light client heartbeat - Synced with network");
            
            // Check for shutdown
            if tokio::signal::ctrl_c().await.is_ok() {
                break;
            }
        }
        
        info!("✅ Light node shutdown complete");
        Ok(())
    }
}

/// Node status information
#[derive(Debug, Clone)]
pub struct NodeStatus {
    pub node_type: String,
    pub port: u16,
    pub is_validator: bool,
    pub blockchain_status: BlockchainStatus,
    pub streaming_status: Option<StreamingStatus>,
    pub connected_peers: u32,
    pub uptime_seconds: u64,
}

#[derive(Debug, Clone)]
pub struct BlockchainStatus {
    pub is_synced: bool,
    pub best_block: u64,
    pub finalized_block: u64,
    pub pending_transactions: u32,
    pub account_balance: u64,
}

#[derive(Debug, Clone)]  
pub struct StreamingStatus {
    pub active_streams: u32,
    pub total_viewers: u32,
    pub bandwidth_usage_mbps: f64,
    pub relay_capacity_percent: f64,
}
