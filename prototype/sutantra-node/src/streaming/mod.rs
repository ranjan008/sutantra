use anyhow::Result;
use tokio::sync::mpsc;
use tracing::{info, warn};
use serde::{Serialize, Deserialize};

pub mod engine;
pub mod webrtc; // Mock WebRTC implementation
pub mod real_webrtc_fixed; // Fixed Real WebRTC implementation
pub mod discovery;
pub mod test_trigger; // Auto-trigger streams for testing

pub use engine::StreamingEngine;

/// Configuration for the streaming engine
#[derive(Debug, Clone)]
pub struct StreamingConfig {
    pub webrtc_port: u16,
    pub max_streams: u32,
    pub max_viewers_per_stream: u32,
    pub enable_relay: bool,
    pub relay_capacity: u32, // Max concurrent streams a relay can handle
    pub discovery_interval_seconds: u64,
    pub use_real_webrtc: bool, // Toggle between mock and real WebRTC
}

/// Events emitted by the streaming layer
#[derive(Debug, Clone)]
pub enum StreamingEvent {
    /// A stream was started by a creator
    StreamStarted { 
        stream_id: String, 
        creator: String 
    },
    
    /// A viewer connected to a stream
    ViewerConnected { 
        stream_id: String, 
        viewer_id: String 
    },
    
    /// A viewer disconnected from a stream
    ViewerDisconnected { 
        stream_id: String, 
        viewer_id: String,
        reason: String,
    },
    
    /// Stream ended
    StreamEnded { 
        stream_id: String, 
        duration_seconds: u64 
    },
    
    /// Stream quality metrics updated
    QualityUpdate { 
        stream_id: String, 
        metrics: StreamMetrics 
    },
    
    /// Bandwidth usage changed
    BandwidthUpdate { 
        total_bandwidth_mbps: f64 
    },
    
    /// Error occurred in streaming
    StreamingError { 
        stream_id: String, 
        error: String 
    },
}

/// Commands that can be sent to the streaming layer
#[derive(Debug, Clone)]
pub enum StreamingCommand {
    /// Prepare for a new stream
    PrepareStream {
        stream_id: String,
        creator: String,
    },
    
    /// Start streaming (creator side)
    StartStream {
        stream_id: String,
        creator: String,
        quality_settings: StreamQualitySettings,
    },
    
    /// Stop streaming
    StopStream {
        stream_id: String,
    },
    
    /// Grant access to a viewer
    GrantAccess {
        stream_id: String,
        viewer: String,
    },
    
    /// Revoke access from a viewer
    RevokeAccess {
        stream_id: String,
        viewer: String,
    },
    
    /// Connect to a stream as viewer
    ConnectToStream {
        stream_id: String,
        viewer: String,
    },
    
    /// Disconnect from a stream
    DisconnectFromStream {
        stream_id: String,
        viewer: String,
    },
    
    /// Update relay configuration
    UpdateRelayConfig {
        max_streams: u32,
        max_bandwidth_mbps: f64,
    },
}

/// Stream quality metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamMetrics {
    pub bandwidth_mbps: f64,
    pub latency_ms: u32,
    pub packet_loss_percent: f64,
    pub resolution: String,
    pub fps: u32,
    pub bitrate_kbps: u32,
    pub jitter_ms: u32,
}

/// Stream quality settings for creators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamQualitySettings {
    pub video_codec: String, // "H264", "VP8", "VP9"
    pub audio_codec: String, // "Opus", "AAC"
    pub max_bitrate_kbps: u32,
    pub target_fps: u32,
    pub resolution: String, // "1920x1080", "1280x720", etc.
    pub adaptive_bitrate: bool,
}

impl Default for StreamQualitySettings {
    fn default() -> Self {
        Self {
            video_codec: "H264".to_string(),
            audio_codec: "Opus".to_string(),
            max_bitrate_kbps: 2000,
            target_fps: 30,
            resolution: "1280x720".to_string(),
            adaptive_bitrate: true,
        }
    }
}

/// Active stream information
#[derive(Debug, Clone)]
pub struct ActiveStream {
    pub stream_id: String,
    pub creator: String,
    pub viewers: Vec<String>,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub quality_settings: StreamQualitySettings,
    pub current_metrics: Option<StreamMetrics>,
    pub total_bandwidth_usage: f64,
}

/// Viewer connection information
#[derive(Debug, Clone)]
pub struct ViewerConnection {
    pub viewer_id: String,
    pub stream_id: String,
    pub connected_at: chrono::DateTime<chrono::Utc>,
    pub peer_connection: Option<String>, // WebRTC peer connection ID
    pub quality_metrics: Option<StreamMetrics>,
}

/// Stream discovery information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamDiscoveryInfo {
    pub stream_id: String,
    pub creator: String,
    pub title: String,
    pub description: Option<String>,
    pub is_live: bool,
    pub viewer_count: u32,
    pub quality: String, // "HD", "SD", etc.
    pub price_per_minute: u64,
    pub category: Option<String>,
    pub tags: Vec<String>,
}

