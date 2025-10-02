/**
 * Sutantra WebRTC Client
 * Handles WebRTC connections to the Sutantra node
 */

class SutantraWebRTCClient {
    constructor() {
        this.peerConnection = null;
        this.localStream = null;
        this.websocket = null;
        this.isStreaming = false;
        // Determine WebSocket URL based on current page port
        const currentPort = window.location.port || '8080';
        this.nodeUrl = `ws://localhost:${currentPort}/ws`;
        this.streamId = null;
        this.stats = {
            uploadSpeed: 0,
            latency: 0,
            packetLoss: 0,
            jitter: 0,
            viewerCount: 0
        };
        
        // WebRTC Configuration with decentralized STUN servers
        this.rtcConfig = {
            iceServers: [
                { urls: 'stun:global.stun.twilio.com:3478' },
                { urls: 'stun:stun.relay.metered.ca:80' },
                { urls: 'stun:stun.cloudflare.com:3478' },
                { urls: 'stun:stun.l.google.com:19302' }
            ],
            iceCandidatePoolSize: 10
        };
        
        this.setupEventHandlers();
    }

    setupEventHandlers() {
        // Handle WebSocket connection events
        this.onWebSocketOpen = this.onWebSocketOpen.bind(this);
        this.onWebSocketMessage = this.onWebSocketMessage.bind(this);
        this.onWebSocketClose = this.onWebSocketClose.bind(this);
        this.onWebSocketError = this.onWebSocketError.bind(this);
    }

    async connectToNode(nodeUrl = null) {
        if (nodeUrl) {
            this.nodeUrl = nodeUrl;
        }

        try {
            console.log(`🔗 Connecting to Sutantra node: ${this.nodeUrl}`);
            
            // Close existing connection
            if (this.websocket) {
                this.websocket.close();
            }

            // Create WebSocket connection
            this.websocket = new WebSocket(this.nodeUrl);
            this.websocket.onopen = this.onWebSocketOpen;
            this.websocket.onmessage = this.onWebSocketMessage;
            this.websocket.onclose = this.onWebSocketClose;
            this.websocket.onerror = this.onWebSocketError;

            return new Promise((resolve, reject) => {
                this.websocket.onopen = () => {
                    this.onWebSocketOpen();
                    resolve();
                };
                this.websocket.onerror = (error) => {
                    this.onWebSocketError(error);
                    reject(error);
                };
            });
        } catch (error) {
            console.error('❌ Failed to connect to node:', error);
            throw error;
        }
    }

    onWebSocketOpen() {
        console.log('✅ Connected to Sutantra node');
        this.updateConnectionStatus('connected');
        
        // Initial connection established, handshake will be sent separately
    }

    onWebSocketMessage(event) {
        try {
            const message = JSON.parse(event.data);
            console.log('📨 Received message:', message);
            
            switch (message.type || message.message_type) {
                case 'handshakeResponse':
                case 'handshake-response':
                    this.handleHandshakeResponse(message);
                    break;
                case 'createStreamResponse':
                    this.handleCreateStreamResponse(message);
                    break;
                case 'stopStreamResponse':
                    this.handleStopStreamResponse(message);
                    break;
                case 'joinStreamResponse':
                    this.handleJoinStreamResponse(message);
                    break;
                case 'streamList':
                    this.handleStreamList(message);
                    break;
                case 'offer':
                    this.handleOffer(message);
                    break;
                case 'answer':
                    this.handleAnswer(message);
                    break;
                case 'ice-candidate':
                    this.handleIceCandidate(message);
                    break;
                case 'stream-stats':
                    this.handleStreamStats(message);
                    break;
                case 'pong':
                    console.log('🏓 Received pong from server');
                    break;
                case 'error':
                    this.handleError(message);
                    break;
                default:
                    console.warn('🤷 Unknown message type:', message.type || message.message_type);
            }
        } catch (error) {
            console.error('❌ Error parsing WebSocket message:', error);
        }
    }

    onWebSocketClose() {
        console.log('🔌 Disconnected from Sutantra node');
        this.updateConnectionStatus('disconnected');
        this.cleanup();
    }

    onWebSocketError(error) {
        console.error('❌ WebSocket error:', error);
        this.updateConnectionStatus('error');
    }

    sendMessage(message) {
        if (this.websocket && this.websocket.readyState === WebSocket.OPEN) {
            this.websocket.send(JSON.stringify(message));
            console.log('📤 Sent message:', message);
        } else {
            console.warn('⚠️ WebSocket not connected, cannot send message');
        }
    }

    async startStream() {
        try {
            console.log('🎥 Starting video stream...');
            this.updateConnectionStatus('connecting');
            
            // First create a stream on the server
            await this.createStream('Live Stream', 'Live streaming from web UI');

            // Get user media
            this.localStream = await navigator.mediaDevices.getUserMedia({
                video: {
                    width: { ideal: 1280 },
                    height: { ideal: 720 },
                    frameRate: { ideal: 30 }
                },
                audio: {
                    echoCancellation: true,
                    noiseSuppression: true,
                    autoGainControl: true
                }
            });

            // Display local video
            const localVideo = document.getElementById('localVideo');
            localVideo.srcObject = this.localStream;

            // Create peer connection
            await this.createPeerConnection();

            // Add tracks to peer connection
            this.localStream.getTracks().forEach(track => {
                this.peerConnection.addTrack(track, this.localStream);
                console.log('➕ Added track:', track.kind);
            });

            // Create and send offer
            const offer = await this.peerConnection.createOffer();
            await this.peerConnection.setLocalDescription(offer);

            this.sendMessage({
                type: 'create-stream',
                offer: offer,
                streamConfig: this.getStreamConfig()
            });

            this.isStreaming = true;
            this.updateConnectionStatus('streaming');
            this.startStatsCollection();

            console.log('✅ Stream started successfully');
            return true;

        } catch (error) {
            console.error('❌ Failed to start stream:', error);
            this.updateConnectionStatus('error');
            throw error;
        }
    }

    async stopStream() {
        try {
            console.log('🛑 Stopping video stream...');

            if (this.streamId) {
                this.sendMessage({
                    type: 'stop-stream',
                    streamId: this.streamId
                });
            }

            this.cleanup();
            this.isStreaming = false;
            this.updateConnectionStatus('connected');

            console.log('✅ Stream stopped successfully');
            return true;

        } catch (error) {
            console.error('❌ Failed to stop stream:', error);
            throw error;
        }
    }

    async createPeerConnection() {
        this.peerConnection = new RTCPeerConnection(this.rtcConfig);

        // Handle ICE candidates
        this.peerConnection.onicecandidate = (event) => {
            if (event.candidate) {
                this.sendMessage({
                    type: 'ice-candidate',
                    candidate: event.candidate,
                    streamId: this.streamId
                });
            }
        };

        // Handle connection state changes
        this.peerConnection.onconnectionstatechange = () => {
            console.log('🔄 Connection state:', this.peerConnection.connectionState);
            
            switch (this.peerConnection.connectionState) {
                case 'connected':
                    this.updateConnectionStatus('streaming');
                    break;
                case 'disconnected':
                case 'failed':
                    this.updateConnectionStatus('error');
                    break;
            }
        };

        // Handle ICE connection state changes
        this.peerConnection.oniceconnectionstatechange = () => {
            console.log('🧊 ICE connection state:', this.peerConnection.iceConnectionState);
        };

        // Handle incoming remote streams
        this.peerConnection.ontrack = (event) => {
            console.log('📺 Received remote stream:', event);
            const remoteVideo = document.getElementById('remoteVideo');
            if (remoteVideo && event.streams[0]) {
                console.log('📺 Setting remote video stream');
                remoteVideo.srcObject = event.streams[0];
                this.remoteStream = event.streams[0];
                
                // Show remote video and hide local video for viewers
                remoteVideo.style.display = 'block';
                const localVideo = document.getElementById('localVideo');
                if (localVideo) {
                    localVideo.style.display = 'none';
                }
            }
        };

        console.log('✅ Peer connection created');
    }

    handleHandshakeResponse(message) {
        console.log('🤝 Handshake successful:', message);
        // Update UI with node info
        if (message.nodeInfo) {
            document.getElementById('nodeId').textContent = message.nodeInfo.nodeId || 'sutantra-node-1';
            document.getElementById('blockHeight').textContent = message.nodeInfo.blockHeight || '0';
            document.getElementById('peerCount').textContent = message.nodeInfo.peerCount || '0';
        }
    }

    async handleOffer(message) {
        // This would be used for receiving streams (viewer mode)
        console.log('📥 Received offer:', message);
    }

    async handleAnswer(message) {
        try {
            console.log('📥 Received answer:', message);
            await this.peerConnection.setRemoteDescription(message.answer);
            this.streamId = message.streamId;
            console.log('✅ Remote description set, stream ID:', this.streamId);
        } catch (error) {
            console.error('❌ Error handling answer:', error);
        }
    }

    async handleIceCandidate(message) {
        try {
            console.log('🧊 Received ICE candidate:', message);
            await this.peerConnection.addIceCandidate(message.candidate);
        } catch (error) {
            console.error('❌ Error adding ICE candidate:', error);
        }
    }

    handleStreamStats(message) {
        console.log('📊 Received stream stats:', message);
        this.stats = { ...this.stats, ...message.stats };
        this.updateStatsUI();
    }

    handleError(message) {
        console.error('❌ Received error:', message);
        this.updateConnectionStatus('error');
        // Show error to user
        alert(`Error: ${message.error}`);
    }

    getStreamConfig() {
        const qualitySelect = document.getElementById('qualitySelect');
        const bitrateSlider = document.getElementById('bitrateSlider');
        const fpsSelect = document.getElementById('fpsSelect');

        const quality = qualitySelect ? qualitySelect.value : '720p';
        const bitrate = bitrateSlider ? parseInt(bitrateSlider.value) : 2500;
        const fps = fpsSelect ? parseInt(fpsSelect.value) : 30;

        let resolution = { width: 1280, height: 720 };
        switch (quality) {
            case '1080p':
                resolution = { width: 1920, height: 1080 };
                break;
            case '720p':
                resolution = { width: 1280, height: 720 };
                break;
            case '480p':
                resolution = { width: 854, height: 480 };
                break;
            case '360p':
                resolution = { width: 640, height: 360 };
                break;
        }

        return {
            resolution,
            bitrate,
            fps,
            codec: 'VP8'
        };
    }

    startStatsCollection() {
        if (this.statsInterval) {
            clearInterval(this.statsInterval);
        }

        this.statsInterval = setInterval(async () => {
            if (this.peerConnection && this.isStreaming) {
                try {
                    const stats = await this.peerConnection.getStats();
                    this.processWebRTCStats(stats);
                } catch (error) {
                    console.warn('⚠️ Error collecting stats:', error);
                }
            }
        }, 1000);
    }

    processWebRTCStats(stats) {
        stats.forEach(report => {
            if (report.type === 'outbound-rtp' && report.mediaType === 'video') {
                // Calculate upload speed
                if (this.lastStats && this.lastStats.bytesSent) {
                    const bytesDiff = report.bytesSent - this.lastStats.bytesSent;
                    const timeDiff = report.timestamp - this.lastStats.timestamp;
                    this.stats.uploadSpeed = Math.round((bytesDiff * 8) / (timeDiff / 1000) / 1000000 * 100) / 100; // Mbps
                }

                this.lastStats = {
                    bytesSent: report.bytesSent,
                    timestamp: report.timestamp
                };
            }

            if (report.type === 'candidate-pair' && report.state === 'succeeded') {
                this.stats.latency = report.currentRoundTripTime ? Math.round(report.currentRoundTripTime * 1000) : 0;
            }
        });

        this.updateStatsUI();
    }

    updateStatsUI() {
        const elements = {
            uploadSpeed: document.getElementById('uploadSpeed'),
            latency: document.getElementById('latency'),
            packetLoss: document.getElementById('packetLoss'),
            jitter: document.getElementById('jitter'),
            viewerCount: document.getElementById('viewerCount')
        };

        if (elements.uploadSpeed) elements.uploadSpeed.textContent = `${this.stats.uploadSpeed} Mbps`;
        if (elements.latency) elements.latency.textContent = `${this.stats.latency} ms`;
        if (elements.packetLoss) elements.packetLoss.textContent = `${this.stats.packetLoss}%`;
        if (elements.jitter) elements.jitter.textContent = `${this.stats.jitter} ms`;
        if (elements.viewerCount) elements.viewerCount.textContent = this.stats.viewerCount;
    }

    updateConnectionStatus(status) {
        const statusDot = document.getElementById('connectionStatus');
        const statusText = document.getElementById('statusText');

        if (statusDot) {
            statusDot.className = 'status-dot';
            switch (status) {
                case 'connected':
                    statusDot.classList.add('connected');
                    if (statusText) statusText.textContent = 'Connected';
                    break;
                case 'connecting':
                    statusDot.classList.add('connecting');
                    if (statusText) statusText.textContent = 'Connecting...';
                    break;
                case 'streaming':
                    statusDot.classList.add('connected');
                    if (statusText) statusText.textContent = 'Streaming';
                    break;
                case 'error':
                    if (statusText) statusText.textContent = 'Error';
                    break;
                default:
                    if (statusText) statusText.textContent = 'Disconnected';
            }
        }
    }

    cleanup() {
        // Stop stats collection
        if (this.statsInterval) {
            clearInterval(this.statsInterval);
            this.statsInterval = null;
        }

        // Close peer connection
        if (this.peerConnection) {
            this.peerConnection.close();
            this.peerConnection = null;
        }

        // Stop local stream
        if (this.localStream) {
            this.localStream.getTracks().forEach(track => track.stop());
            this.localStream = null;
        }

        // Clear video element
        const localVideo = document.getElementById('localVideo');
        if (localVideo) {
            localVideo.srcObject = null;
        }

        this.streamId = null;
        this.lastStats = null;
    }

    disconnect() {
        this.cleanup();
        
        if (this.websocket) {
            this.websocket.close();
            this.websocket = null;
        }
        
        this.updateConnectionStatus('disconnected');
    }

    // New methods for handshake and viewer functionality
    async sendHandshake(nodeType) {
        console.log(`🤝 Sending handshake as ${nodeType}`);
        this.sendMessage({
            type: 'handshake',
            data: {
                client_type: nodeType,
                node_id: null
            }
        });
    }

    handleHandshakeResponse(message) {
        console.log('🤝 Handshake response received:', message);
        if (message.data?.success) {
            console.log('✅ Handshake successful');
            this.clientId = message.data.client_id;
            
            // Update node type from server response
            if (message.data.node_type && window.uiController) {
                console.log(`🎭 Server confirmed node type: ${message.data.node_type}`);
                console.log(`🔧 Current UI controller node type: ${window.uiController.nodeType}`);
                
                // Force UI update
                window.uiController.setNodeType(message.data.node_type);
                
                // Double-check that it was set
                console.log(`🔧 After setNodeType, UI controller node type: ${window.uiController.nodeType}`);
                
                // Update node info display
                if (message.data.node_id) {
                    this.updateNodeInfo({
                        node_id: message.data.node_id,
                        node_type: message.data.node_type,
                        web_port: message.data.web_port
                    });
                }
            } else {
                console.error('❌ Missing node_type in handshake response or uiController not available');
                console.log('Available data:', message.data);
                console.log('uiController available:', !!window.uiController);
            }
            
            if (window.uiController) {
                window.uiController.showNotification(
                    message.data.message || 'Connected to Sutantra node!', 
                    'success'
                );
            }
        }
    }

    handleCreateStreamResponse(message) {
        console.log('🎬 Create stream response:', message);
        if (message.data?.success) {
            this.streamId = message.data.stream_id;
            this.isStreaming = true;
            console.log('✅ Stream created successfully:', this.streamId);
            if (window.uiController) {
                window.uiController.showNotification('Stream created successfully!', 'success');
                window.uiController.updateStreamButtons(true); // Set streaming to true
            }
        } else {
            console.error('❌ Failed to create stream:', message.data?.error);
            if (window.uiController) {
                window.uiController.showNotification('Failed to create stream: ' + (message.data?.error || 'Unknown error'), 'error');
            }
        }
    }

    handleStopStreamResponse(message) {
        console.log('🛑 Stream stop response:', message);
        if (message.data?.success) {
            this.streamId = null;
            this.isStreaming = false;
            console.log('✅ Stream stopped successfully');
            if (window.uiController) {
                window.uiController.showNotification('Stream stopped successfully!', 'success');
                window.uiController.updateStreamButtons(false); // Set streaming to false
            }
        } else {
            console.error('❌ Failed to stop stream:', message.data?.error);
            if (window.uiController) {
                window.uiController.showNotification('Failed to stop stream: ' + (message.data?.error || 'Unknown error'), 'error');
            }
        }
    }

    handleJoinStreamResponse(message) {
        console.log('🚀 Join stream response:', message);
        if (message.data?.success) {
            console.log('✅ Successfully joined stream:', message.data.stream_id);
            
            // Show remote video element and hide local video
            const localVideo = document.getElementById('localVideo');
            const remoteVideo = document.getElementById('remoteVideo');
            
            if (localVideo) {
                localVideo.style.display = 'none';
            }
            
            if (remoteVideo) {
                remoteVideo.style.display = 'block';
            }
            
            if (window.uiController) {
                window.uiController.showNotification('Joined stream successfully!', 'success');
            }
        } else {
            console.error('❌ Failed to join stream:', message.data?.error);
            if (window.uiController) {
                window.uiController.showNotification('Failed to join stream: ' + (message.data?.error || 'Unknown error'), 'error');
            }
        }
    }

    handleStreamList(message) {
        console.log('📺 Received stream list:', message);
        if (window.uiController && window.uiController.updateAvailableStreams) {
            window.uiController.updateAvailableStreams(message.data?.streams || []);
        }
    }

    async requestStreamList() {
        console.log('📋 Requesting stream list...');
        
        if (!this.websocket || this.websocket.readyState !== WebSocket.OPEN) {
            console.error('❌ WebSocket not connected');
            throw new Error('WebSocket not connected');
        }
        
        const message = {
            type: 'requestStreamList',
            data: {}
        };
        
        console.log('📋 Sending stream list request:', message);
        this.sendMessage(message);
        console.log('📋 Stream list request sent successfully');
    }

    async createStream(title, description = '') {
        console.log('🎬 Creating stream:', title);
        this.sendMessage({
            type: 'createStream',
            data: {
                title: title,
                description: description,
                quality: {
                    video_codec: 'VP8',
                    audio_codec: 'Opus',
                    max_bitrate_kbps: 2500,
                    target_fps: 30,
                    resolution: '1280x720',
                    adaptive_bitrate: true
                }
            }
        });
    }

    async stopStream() {
        console.log('🛑 Stopping stream:', this.streamId);
        if (!this.streamId) {
            console.warn('⚠️ No active stream to stop');
            return;
        }
        
        this.sendMessage({
            type: 'stopStream',
            data: {
                stream_id: this.streamId
            }
        });
    }

    async joinStream(streamId) {
        console.log('🚀 Joining stream:', streamId);
        this.streamId = streamId;
        
        try {
            // Set up peer connection for receiving stream
            await this.createPeerConnection();
            
            // Create offer to start WebRTC handshake
            const offer = await this.peerConnection.createOffer();
            await this.peerConnection.setLocalDescription(offer);
            
            console.log('📡 Created offer for joining stream:', streamId);
            
            // Send join request with WebRTC offer
            this.sendMessage({
                type: 'joinStream',
                stream_id: streamId,
                offer: offer,
                data: {}
            });
            
        } catch (error) {
            console.error('❌ Error joining stream:', error);
            if (window.uiController) {
                window.uiController.showNotification('Failed to join stream: ' + error.message, 'error');
            }
        }
    }

    async leaveStream() {
        console.log('🚪 Leaving stream...');
        if (this.streamId) {
            this.sendMessage({
                type: 'leaveStream',
                stream_id: this.streamId,
                data: {}
            });
            this.streamId = null;
        }
    }

    handleError(message) {
        const errorMsg = message.data?.message || message.error || 'Unknown error';
        console.error('❌ Server error:', errorMsg);
        if (window.uiController) {
            window.uiController.showNotification('Error: ' + errorMsg, 'error');
        }
    }

    updateNodeInfo(nodeInfo) {
        console.log('📊 Updating node info:', nodeInfo);
        
        // Update node ID display
        const nodeIdElement = document.getElementById('nodeId');
        if (nodeIdElement && nodeInfo.node_id) {
            nodeIdElement.textContent = nodeInfo.node_id;
        }
        
        // Update other node info if available
        if (nodeInfo.web_port) {
            const portElement = document.getElementById('webPort');
            if (portElement) {
                portElement.textContent = nodeInfo.web_port;
            }
        }
        
        // Update node type indicator
        if (nodeInfo.node_type) {
            const typeElement = document.getElementById('nodeType');
            if (typeElement) {
                typeElement.textContent = nodeInfo.node_type.toUpperCase();
            }
        }
    }
}

// Export for use in other modules
window.SutantraWebRTCClient = SutantraWebRTCClient;
