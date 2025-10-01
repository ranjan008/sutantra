# Sutantra Node Integration Specification

## 🏗️ **Integrated Node Components**

### **Core Architecture: Single Process, Multiple Subsystems**

```
sutantra-node (single binary)
├── blockchain/          # Blockchain consensus and state
│   ├── consensus.rs     # PoS consensus engine
│   ├── state.rs         # Account, stream, and payment state
│   ├── transactions.rs  # Transaction processing
│   └── p2p.rs          # Blockchain networking
├── streaming/           # WebRTC streaming engine
│   ├── webrtc.rs       # WebRTC implementation
│   ├── relay.rs        # Stream relay and routing
│   ├── discovery.rs    # Stream discovery
│   └── nat.rs          # NAT traversal
├── integration/         # Bridge between blockchain and streaming
│   ├── events.rs       # Event routing between layers
│   ├── payments.rs     # Real-time payment processing
│   ├── access.rs       # Blockchain-based access control
│   └── metrics.rs      # Stream quality metrics
└── mobile/             # Mobile-specific optimizations
    ├── light_client.rs # Light node implementation
    ├── sync.rs         # Efficient state synchronization
    └── battery.rs      # Power optimization
```

---

## 🔄 **Integration Mechanisms**

### **1. Unified Event System**

```rust
// Central event bus that connects blockchain and streaming
#[derive(Debug, Clone)]
pub enum SutantraEvent {
    // Blockchain events that affect streaming
    StreamRegistered { stream_id: StreamId, creator: AccountId },
    PaymentReceived { stream_id: StreamId, amount: Balance },
    AccessGranted { viewer: AccountId, stream_id: StreamId },
    
    // Streaming events that affect blockchain
    StreamStarted { stream_id: StreamId, quality: QualityMetrics },
    ViewerJoined { stream_id: StreamId, viewer_count: u32 },
    StreamEnded { stream_id: StreamId, duration: Duration },
    
    // Cross-layer events
    PaymentDue { stream_id: StreamId, viewer: AccountId, amount: Balance },
    QualityReport { stream_id: StreamId, metrics: StreamQuality },
}

// Event router that ensures both layers stay synchronized
pub struct EventBridge {
    blockchain_tx: Sender<BlockchainCommand>,
    streaming_tx: Sender<StreamingCommand>,
    event_rx: Receiver<SutantraEvent>,
}
```

### **2. Real-Time Payment Integration**

```rust
// Payments flow directly through WebRTC data channels
pub struct StreamPaymentChannel {
    webrtc_channel: RTCDataChannel,
    blockchain_account: AccountId,
    rate_per_second: Balance,
    last_payment: Instant,
}

impl StreamPaymentChannel {
    // Send micropayment through WebRTC, settle on blockchain
    pub async fn process_payment(&mut self) -> Result<()> {
        let amount = self.calculate_payment_due();
        
        // Send payment instruction via WebRTC
        let payment_msg = PaymentMessage {
            amount,
            timestamp: now(),
            signature: self.sign_payment(amount),
        };
        self.webrtc_channel.send(&payment_msg).await?;
        
        // Queue blockchain transaction
        self.submit_blockchain_payment(amount).await?;
        
        Ok(())
    }
}
```

### **3. Blockchain-Controlled Access**

```rust
// Access control enforced by blockchain, implemented in WebRTC
pub struct StreamAccessControl {
    blockchain_state: Arc<BlockchainState>,
    active_streams: HashMap<StreamId, StreamSession>,
}

impl StreamAccessControl {
    pub async fn authorize_viewer(&self, stream_id: StreamId, viewer: AccountId) -> bool {
        // Check blockchain state for access rights
        match self.blockchain_state.get_stream_access(stream_id, viewer).await {
            Some(AccessToken { valid_until, payment_status }) => {
                payment_status.is_current() && valid_until > now()
            }
            None => false
        }
    }
    
    // Automatically called when WebRTC connection is established
    pub async fn on_webrtc_connect(&mut self, peer_id: PeerId, stream_id: StreamId) {
        let viewer = self.resolve_peer_to_account(peer_id).await;
        if !self.authorize_viewer(stream_id, viewer).await {
            self.reject_connection(peer_id, "Payment required").await;
        }
    }
}
```

---

## 🌐 **Network Integration**

### **Unified P2P Stack**

```rust
// Single networking layer serving both blockchain and streaming
pub struct SutantraNetwork {
    // Shared libp2p network for peer discovery
    libp2p_swarm: Swarm<SutantraBehaviour>,
    
    // Blockchain-specific protocols
    blockchain_protocols: BlockchainProtocols,
    
    // Streaming-specific protocols  
    streaming_protocols: StreamingProtocols,
    
    // Cross-protocol coordination
    routing_table: Arc<RwLock<RoutingTable>>,
}

// Peers can be blockchain validators AND streaming relays
#[derive(Clone)]
pub struct PeerInfo {
    peer_id: PeerId,
    
    // Blockchain capabilities
    is_validator: bool,
    blockchain_sync_status: SyncStatus,
    
    // Streaming capabilities
    max_streams: u32,
    bandwidth_capacity: u64,
    geographic_region: Region,
    
    // Quality metrics
    reliability_score: f64,
    average_latency: Duration,
}
```

### **Stream Discovery via Blockchain**

```rust
// Stream registry stored on blockchain, discovery via DHT
pub struct StreamRegistry {
    // On-chain stream metadata
    streams: BTreeMap<StreamId, StreamMetadata>,
    
    // Off-chain routing information
    routing_cache: HashMap<StreamId, Vec<PeerId>>,
}

impl StreamRegistry {
    // Register stream on blockchain, announce on DHT
    pub async fn register_stream(&mut self, stream: StreamMetadata) -> Result<()> {
        // Store metadata on blockchain
        self.blockchain_tx.send(RegisterStream(stream.clone())).await?;
        
        // Announce on DHT for discovery
        self.dht.put_value(stream.id.to_bytes(), stream.encode()).await?;
        
        // Update local routing cache
        self.update_routing_cache(stream.id).await?;
        
        Ok(())
    }
    
    // Find optimal relay nodes for stream
    pub async fn find_stream_relays(&self, stream_id: StreamId) -> Vec<PeerId> {
        // Combine blockchain validator info with streaming metrics
        let validators = self.get_nearby_validators().await;
        let streaming_nodes = self.get_streaming_capable_peers().await;
        
        // Select best nodes based on geographic location, capacity, and reliability
        self.select_optimal_relays(validators, streaming_nodes, stream_id).await
    }
}
```

---

## 📱 **Mobile Light Node Architecture**

### **Streaming-Capable Light Client**

```rust
// Mobile node that can stream without full blockchain state
pub struct SutantraLightNode {
    // Minimal blockchain state for payments and access
    light_client: LightClient,
    
    // Full streaming capabilities
    streaming_engine: StreamingEngine,
    
    // Efficient state synchronization
    sync_manager: LightSyncManager,
    
    // Battery and bandwidth optimization
    power_manager: PowerManager,
}

impl SutantraLightNode {
    // Start streaming with minimal blockchain verification
    pub async fn start_streaming(&mut self, stream_id: StreamId) -> Result<StreamHandle> {
        // Verify access using light client (merkle proofs)
        let access_proof = self.light_client.verify_stream_access(stream_id).await?;
        
        // Connect to stream using WebRTC
        let stream = self.streaming_engine.connect_to_stream(stream_id).await?;
        
        // Set up payment channel
        let payment_channel = self.setup_payment_channel(stream_id, access_proof).await?;
        
        Ok(StreamHandle {
            stream,
            payment_channel,
            stream_id,
        })
    }
    
    // Efficient payment without full blockchain sync
    pub async fn process_payment(&mut self, amount: Balance) -> Result<()> {
        // Create payment transaction
        let tx = self.create_payment_transaction(amount).await?;
        
        // Submit directly to validators (no local mempool needed)
        self.submit_to_validators(tx).await?;
        
        // Update local balance using light client proofs
        self.light_client.update_balance_proof().await?;
        
        Ok(())
    }
}
```

---

## 🔧 **Development Approach**

### **Prototype Implementation Plan**

1. **Week 1-2: Core Integration**
   - Build unified event system
   - Create basic blockchain + WebRTC in single process
   - Implement simple payment integration

2. **Week 3-4: Access Control**
   - Add blockchain-based stream authorization
   - Implement WebRTC connection gating
   - Test with multiple streams and viewers

3. **Week 5-6: Mobile Light Node**
   - Build light client streaming capabilities
   - Optimize for mobile constraints
   - Test on actual mobile devices

4. **Week 7-8: Network Integration**
   - Unified P2P discovery
   - Stream relay optimization
   - Load testing and performance tuning

This architecture ensures that WebRTC streaming is not an afterthought but a **core protocol feature** deeply integrated into every aspect of the Sutantra blockchain.

