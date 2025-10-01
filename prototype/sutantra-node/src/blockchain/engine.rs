use anyhow::Result;
use tokio::sync::mpsc;
use tracing::{info, debug, warn, error};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::{BlockchainConfig, BlockchainEvent, BlockchainCommand, Block, Transaction, Account, StreamRegistration};

/// Core blockchain engine that integrates with streaming
pub struct BlockchainEngine {
    config: BlockchainConfig,
    
    // Communication channels
    command_rx: mpsc::Receiver<BlockchainCommand>,
    event_tx: mpsc::Sender<BlockchainEvent>,
    
    // Blockchain state
    state: Arc<RwLock<BlockchainState>>,
    
    // Block production
    is_validator: bool,
    current_block: u64,
}

/// Blockchain state including accounts and streams
#[derive(Debug)]
pub struct BlockchainState {
    /// Account balances and stream access
    pub accounts: HashMap<String, Account>,
    
    /// Registered streams
    pub streams: HashMap<String, StreamRegistration>,
    
    /// Chain metadata
    pub best_block: u64,
    pub finalized_block: u64,
    pub chain_id: String,
    
    /// Transaction pool
    pub pending_transactions: Vec<Transaction>,
}

impl BlockchainEngine {
    pub async fn new(
        config: BlockchainConfig,
        command_rx: mpsc::Receiver<BlockchainCommand>,
        event_tx: mpsc::Sender<BlockchainEvent>,
    ) -> Result<Self> {
        info!("🔗 Initializing Blockchain Engine");
        
        // Initialize blockchain state
        let mut state = BlockchainState {
            accounts: HashMap::new(),
            streams: HashMap::new(),
            best_block: 0,
            finalized_block: 0,
            chain_id: "sutantra-testnet".to_string(),
            pending_transactions: Vec::new(),
        };
        
        // Create genesis account with initial balance
        let genesis_account = Account {
            address: "genesis".to_string(),
            balance: 1_000_000_000, // 1 billion STREAM tokens
            nonce: 0,
            stream_access: HashMap::new(),
            created_streams: Vec::new(),
        };
        state.accounts.insert("genesis".to_string(), genesis_account);
        
        let is_validator = config.is_validator;
        
        Ok(Self {
            config,
            command_rx,
            event_tx,
            state: Arc::new(RwLock::new(state)),
            is_validator,
            current_block: 0,
        })
    }
    
    pub async fn run(&mut self) -> Result<()> {
        info!("🚀 Starting Blockchain Engine");
        info!("📁 Data directory: {}", self.config.data_dir);
        info!("⚡ Validator mode: {}", self.is_validator);
        
        // Start block production if validator
        let block_production_handle = if self.is_validator {
            let state = Arc::clone(&self.state);
            let event_tx = self.event_tx.clone();
            Some(tokio::spawn(async move {
                Self::block_production_loop(state, event_tx).await;
            }))
        } else {
            None
        };
        
        // Main command processing loop
        loop {
            tokio::select! {
                // Process commands from integration layer
                Some(command) = self.command_rx.recv() => {
                    if let Err(e) = self.handle_command(command).await {
                        error!("Error handling blockchain command: {}", e);
                    }
                }
                
                else => {
                    warn!("Blockchain command channel closed");
                    break;
                }
            }
        }
        
        // Cleanup
        if let Some(handle) = block_production_handle {
            handle.abort();
        }
        
        Ok(())
    }
    
    async fn handle_command(&self, command: BlockchainCommand) -> Result<()> {
        match command {
            BlockchainCommand::RegisterStream { 
                stream_id, 
                creator, 
                title, 
                description, 
                price_per_minute 
            } => {
                self.register_stream(stream_id, creator, title, description, price_per_minute).await?;
            }
            
            BlockchainCommand::ProcessPayment { stream_id, viewer, amount } => {
                self.process_payment(stream_id, viewer, amount).await?;
            }
            
            BlockchainCommand::CheckAccess { stream_id, viewer } => {
                self.check_access(stream_id, viewer).await?;
            }
            
            BlockchainCommand::RecordStreamStart { stream_id, timestamp } => {
                self.record_stream_start(stream_id, timestamp).await?;
            }
            
            BlockchainCommand::ReportStreamQuality { stream_id, metrics } => {
                self.record_stream_quality(stream_id, metrics).await?;
            }
            
            BlockchainCommand::Transfer { from, to, amount } => {
                self.transfer_tokens(from, to, amount).await?;
            }
            
            _ => {
                debug!("Unhandled blockchain command: {:?}", command);
            }
        }
        
        Ok(())
    }
    
    async fn register_stream(
        &self,
        stream_id: String,
        creator: String,
        title: String,
        description: Option<String>,
        price_per_minute: u64,
    ) -> Result<()> {
        info!("📝 Registering stream: {} by {}", stream_id, creator);
        
        let mut state = self.state.write().await;
        
        // Check if stream already exists
        if state.streams.contains_key(&stream_id) {
            warn!("Stream {} already exists", stream_id);
            return Ok(());
        }
        
        // Create stream registration
        let registration = StreamRegistration {
            stream_id: stream_id.clone(),
            creator: creator.clone(),
            title,
            description,
            price_per_minute,
            created_at: chrono::Utc::now(),
            is_active: false,
            total_earnings: 0,
            total_viewers: 0,
            total_duration_minutes: 0,
        };
        
        // Add to state
        state.streams.insert(stream_id.clone(), registration);
        
        // Update creator's account
        state.accounts.entry(creator.clone())
            .or_insert_with(|| Account {
                address: creator.clone(),
                balance: 0,
                nonce: 0,
                stream_access: HashMap::new(),
                created_streams: Vec::new(),
            })
            .created_streams.push(stream_id.clone());
        
        // Emit event
        self.event_tx.send(BlockchainEvent::StreamRegistered { 
            stream_id, 
            creator 
        }).await?;
        
        Ok(())
    }
    
    async fn process_payment(&self, stream_id: String, viewer: String, amount: u64) -> Result<()> {
        info!("💳 Processing payment: {} STREAM from {} for {}", amount, viewer, stream_id);
        
        let mut state = self.state.write().await;
        
        // Get stream info (clone to avoid borrow conflicts)
        let stream_info = state.streams.get(&stream_id).cloned().ok_or_else(|| {
            anyhow::anyhow!("Stream {} not found", stream_id)
        })?;
        
        let creator = stream_info.creator.clone();
        let price_per_minute = stream_info.price_per_minute;
        
        // Check viewer balance
        let viewer_account = state.accounts.get_mut(&viewer).ok_or_else(|| {
            anyhow::anyhow!("Viewer account {} not found", viewer)
        })?;
        
        if viewer_account.balance < amount {
            warn!("Insufficient balance for viewer {}: {} < {}", viewer, viewer_account.balance, amount);
            return Ok(());
        }
        
        // Transfer tokens
        viewer_account.balance -= amount;
        viewer_account.nonce += 1;
        
        // Credit creator (drop viewer_account reference first)
        drop(viewer_account);
        
        state.accounts.entry(creator.clone())
            .or_insert_with(|| Account {
                address: creator.clone(),
                balance: 0,
                nonce: 0,
                stream_access: HashMap::new(),
                created_streams: Vec::new(),
            })
            .balance += amount;
        
        // Get viewer account again for access grant
        let viewer_account = state.accounts.get_mut(&viewer).unwrap();
        
        // Grant access (payment for 1 minute per 10 STREAM tokens for simplicity)
        let access_duration_minutes = amount / price_per_minute;
        let access_until = chrono::Utc::now() + chrono::Duration::minutes(access_duration_minutes as i64);
        
        viewer_account.stream_access.insert(stream_id.clone(), super::StreamAccess {
            stream_id: stream_id.clone(),
            paid_until: access_until,
            total_paid: amount,
            access_granted_at: chrono::Utc::now(),
        });
        
        // Update stream earnings
        if let Some(stream) = state.streams.get_mut(&stream_id) {
            stream.total_earnings += amount;
        }
        
        // Emit event
        self.event_tx.send(BlockchainEvent::PaymentProcessed { 
            stream_id, 
            viewer, 
            amount 
        }).await?;
        
        Ok(())
    }
    
    async fn check_access(&self, stream_id: String, viewer: String) -> Result<()> {
        debug!("🔍 Checking access: {} for stream {}", viewer, stream_id);
        
        let state = self.state.read().await;
        
        let has_access = if let Some(account) = state.accounts.get(&viewer) {
            if let Some(access) = account.stream_access.get(&stream_id) {
                access.paid_until > chrono::Utc::now()
            } else {
                false
            }
        } else {
            false
        };
        
        if has_access {
            self.event_tx.send(BlockchainEvent::AccessGranted { 
                stream_id, 
                viewer 
            }).await?;
        } else {
            self.event_tx.send(BlockchainEvent::AccessRevoked { 
                stream_id, 
                viewer,
                reason: "Payment required".to_string(),
            }).await?;
        }
        
        Ok(())
    }
    
    async fn record_stream_start(&self, stream_id: String, _timestamp: chrono::DateTime<chrono::Utc>) -> Result<()> {
        debug!("🎬 Recording stream start: {}", stream_id);
        
        let mut state = self.state.write().await;
        
        if let Some(stream) = state.streams.get_mut(&stream_id) {
            stream.is_active = true;
        }
        
        Ok(())
    }
    
    async fn record_stream_quality(&self, stream_id: String, _metrics: crate::integration::StreamQualityMetrics) -> Result<()> {
        debug!("📊 Recording stream quality for: {}", stream_id);
        
        // TODO: Use quality metrics for validator rewards
        // For now, just acknowledge
        
        Ok(())
    }
    
    async fn transfer_tokens(&self, from: String, to: String, amount: u64) -> Result<()> {
        info!("💸 Transfer: {} STREAM from {} to {}", amount, from, to);
        
        let mut state = self.state.write().await;
        
        // Get sender account
        let sender = state.accounts.get_mut(&from).ok_or_else(|| {
            anyhow::anyhow!("Sender account {} not found", from)
        })?;
        
        if sender.balance < amount {
            return Err(anyhow::anyhow!("Insufficient balance"));
        }
        
        // Deduct from sender
        sender.balance -= amount;
        sender.nonce += 1;
        
        // Credit receiver
        state.accounts.entry(to.clone())
            .or_insert_with(|| Account {
                address: to.clone(),
                balance: 0,
                nonce: 0,
                stream_access: HashMap::new(),
                created_streams: Vec::new(),
            })
            .balance += amount;
        
        Ok(())
    }
    
    /// Block production loop for validators
    async fn block_production_loop(
        state: Arc<RwLock<BlockchainState>>,
        event_tx: mpsc::Sender<BlockchainEvent>,
    ) {
        info!("⛏️  Starting block production");
        
        let mut block_number = 1u64;
        
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(6)).await; // 6 second blocks
            
            let block_hash = format!("block_{}", block_number);
            
            // Update state
            {
                let mut state = state.write().await;
                state.best_block = block_number;
                state.finalized_block = block_number.saturating_sub(1);
            }
            
            // Emit block produced event
            if let Err(e) = event_tx.send(BlockchainEvent::BlockProduced { 
                block_number, 
                block_hash 
            }).await {
                error!("Failed to send block produced event: {}", e);
                break;
            }
            
            debug!("🧱 Produced block #{}", block_number);
            block_number += 1;
        }
    }
    
    /// Get current blockchain status
    pub async fn get_status(&self) -> (u64, u64, u32) {
        let state = self.state.read().await;
        (state.best_block, state.finalized_block, state.pending_transactions.len() as u32)
    }
}
