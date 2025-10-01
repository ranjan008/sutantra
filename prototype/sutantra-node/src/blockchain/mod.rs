use anyhow::Result;
use tokio::sync::mpsc;
use tracing::{info, warn};
use serde::{Serialize, Deserialize};

use crate::integration::StreamQualityMetrics;

pub mod engine;
pub mod state;
pub mod consensus;
pub mod transactions;

pub use engine::BlockchainEngine;

/// Configuration for the blockchain engine
#[derive(Debug, Clone)]
pub struct BlockchainConfig {
    pub port: u16,
    pub is_validator: bool,
    pub data_dir: String,
}

/// Events emitted by the blockchain layer
#[derive(Debug, Clone)]
pub enum BlockchainEvent {
    /// A new stream was registered on-chain
    StreamRegistered { 
        stream_id: String, 
        creator: String 
    },
    
    /// Payment was processed for stream access
    PaymentProcessed { 
        stream_id: String, 
        viewer: String, 
        amount: u64 
    },
    
    /// Access was granted to a viewer
    AccessGranted { 
        stream_id: String, 
        viewer: String 
    },
    
    /// Access was revoked from a viewer
    AccessRevoked { 
        stream_id: String, 
        viewer: String,
        reason: String,
    },
    
    /// New block was produced
    BlockProduced { 
        block_number: u64, 
        block_hash: String 
    },
    
    /// Blockchain sync status changed
    SyncStatusChanged { 
        is_synced: bool, 
        best_block: u64 
    },
}

/// Commands that can be sent to the blockchain layer
#[derive(Debug, Clone)]
pub enum BlockchainCommand {
    /// Register a new stream
    RegisterStream {
        stream_id: String,
        creator: String,
        title: String,
        description: Option<String>,
        price_per_minute: u64,
    },
    
    /// Process a payment for stream access
    ProcessPayment {
        stream_id: String,
        viewer: String,
        amount: u64,
    },
    
    /// Check if viewer has access to stream
    CheckAccess {
        stream_id: String,
        viewer: String,
    },
    
    /// Record that a stream started
    RecordStreamStart {
        stream_id: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    
    /// Record that a stream ended
    RecordStreamEnd {
        stream_id: String,
        timestamp: chrono::DateTime<chrono::Utc>,
        duration_seconds: u64,
    },
    
    /// Report stream quality metrics for validator rewards
    ReportStreamQuality {
        stream_id: String,
        metrics: StreamQualityMetrics,
    },
    
    /// Transfer tokens between accounts
    Transfer {
        from: String,
        to: String,
        amount: u64,
    },
}

/// Stream registration data stored on blockchain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamRegistration {
    pub stream_id: String,
    pub creator: String,
    pub title: String,
    pub description: Option<String>,
    pub price_per_minute: u64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub is_active: bool,
    pub total_earnings: u64,
    pub total_viewers: u32,
    pub total_duration_minutes: u64,
}

/// Account balance and stream access information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub address: String,
    pub balance: u64,
    pub nonce: u64,
    pub stream_access: std::collections::HashMap<String, StreamAccess>,
    pub created_streams: Vec<String>,
}

/// Stream access token with payment and expiry info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamAccess {
    pub stream_id: String,
    pub paid_until: chrono::DateTime<chrono::Utc>,
    pub total_paid: u64,
    pub access_granted_at: chrono::DateTime<chrono::Utc>,
}

/// Transaction types supported by Sutantra blockchain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Transaction {
    /// Transfer STREAM tokens
    Transfer {
        from: String,
        to: String,
        amount: u64,
        nonce: u64,
        signature: String,
    },
    
    /// Register a new stream
    RegisterStream {
        creator: String,
        stream_data: StreamRegistration,
        signature: String,
    },
    
    /// Purchase access to a stream
    PurchaseAccess {
        viewer: String,
        stream_id: String,
        amount: u64,
        duration_minutes: u64,
        signature: String,
    },
    
    /// Report stream quality (validator only)
    ReportQuality {
        validator: String,
        stream_id: String,
        metrics: StreamQualityMetrics,
        signature: String,
    },
}

/// Blockchain block structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub number: u64,
    pub parent_hash: String,
    pub hash: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub transactions: Vec<Transaction>,
    pub state_root: String,
    pub validator: String,
    pub signature: String,
}

