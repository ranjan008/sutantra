/**
 * Sutantra Web UI Application
 * Main application entry point
 */

class SutantraApp {
    constructor() {
        this.webrtcClient = null;
        this.uiController = null;
        this.isInitialized = false;
        this.nodeType = this.detectNodeType(); // 'streamer' or 'viewer'
    }

    detectNodeType() {
        // Detect node type based on port
        const currentPort = window.location.port || '8080';
        
        // Port 8080 = Streamer (Node 1), Port 8081 = Viewer (Node 2)
        if (currentPort === '8080') {
            return 'streamer';
        } else if (currentPort === '8081') {
            return 'viewer';
        } else {
            // Default to streamer for other ports
            return 'streamer';
        }
    }

    async initialize() {
        try {
            console.log(`🚀 Initializing Sutantra Web UI (detected as ${this.nodeType}, waiting for server confirmation)...`);
            
            // Initialize UI Controller WITHOUT setting node type yet
            this.uiController = new SutantraUIController();
            // Don't set node type yet - wait for server handshake
            window.uiController = this.uiController;
            
            // Load saved settings
            const settings = this.uiController.loadSettings();
            
            // Initialize WebRTC Client
            this.webrtcClient = new SutantraWebRTCClient();
            window.webrtcClient = this.webrtcClient;
            
            // Apply settings if available
            if (settings && settings.nodeUrl) {
                this.webrtcClient.nodeUrl = settings.nodeUrl;
            }
            
            // Show loading overlay
            this.uiController.showLoadingOverlay('Connecting to Sutantra node...');
            
            // Connect to node
            try {
                await this.webrtcClient.connectToNode();
                
                // Send handshake with node type
                await this.webrtcClient.sendHandshake(this.nodeType);
                
                console.log(`✅ Successfully connected to Sutantra node as ${this.nodeType}`);
                this.uiController.showNotification(`Connected as ${this.nodeType}!`, 'success');
            } catch (error) {
                console.warn('⚠️ Could not connect to node on startup:', error);
                this.uiController.showNotification('Could not connect to node. Check if the Sutantra node is running.', 'warning');
            }
            
            this.isInitialized = true;
            console.log('✅ Sutantra Web UI initialized successfully');
            
        } catch (error) {
            console.error('❌ Failed to initialize Sutantra Web UI:', error);
            this.uiController?.showNotification(`Initialization failed: ${error.message}`, 'error');
        } finally {
            this.uiController?.hideLoadingOverlay();
        }
    }

    async reconnectToNode() {
        if (!this.webrtcClient) return;
        
        try {
            this.uiController?.showLoadingOverlay('Reconnecting to node...');
            await this.webrtcClient.connectToNode();
            this.uiController?.showNotification('Reconnected to Sutantra node!', 'success');
        } catch (error) {
            console.error('❌ Reconnection failed:', error);
            this.uiController?.showNotification(`Reconnection failed: ${error.message}`, 'error');
        } finally {
            this.uiController?.hideLoadingOverlay();
        }
    }

    async checkNodeConnection() {
        if (!this.webrtcClient) return false;
        
        return this.webrtcClient.websocket && 
               this.webrtcClient.websocket.readyState === WebSocket.OPEN;
    }

    getAppInfo() {
        return {
            version: '1.0.0',
            name: 'Sutantra Web UI',
            description: 'Decentralized Live Streaming Interface',
            features: [
                'WebRTC Live Streaming',
                'Blockchain Integration',
                'Decentralized STUN Servers',
                'Real-time Statistics',
                'P2P Networking'
            ]
        };
    }
}

// Global app instance
let sutantraApp = null;

// Initialize when DOM is loaded
document.addEventListener('DOMContentLoaded', async () => {
    console.log('📄 DOM loaded, initializing Sutantra App...');
    
    try {
        sutantraApp = new SutantraApp();
        window.sutantraApp = sutantraApp;
        
        await sutantraApp.initialize();
        
        // Set up periodic connection check
        setInterval(async () => {
            const isConnected = await sutantraApp.checkNodeConnection();
            if (!isConnected && sutantraApp.isInitialized) {
                console.warn('⚠️ Lost connection to node, attempting reconnect...');
                await sutantraApp.reconnectToNode();
            }
        }, 10000); // Check every 10 seconds
        
    } catch (error) {
        console.error('❌ Failed to initialize app:', error);
        
        // Show fallback error message
        document.body.innerHTML = `
            <div style="
                display: flex;
                justify-content: center;
                align-items: center;
                height: 100vh;
                background: linear-gradient(135deg, #1e3c72 0%, #2a5298 100%);
                color: white;
                font-family: 'Segoe UI', sans-serif;
                text-align: center;
                padding: 2rem;
            ">
                <div>
                    <h1 style="color: #f44336; margin-bottom: 1rem;">
                        <i class="fas fa-exclamation-triangle"></i>
                        Initialization Failed
                    </h1>
                    <p style="margin-bottom: 2rem; color: #b0bec5;">
                        Could not initialize the Sutantra Web UI.<br>
                        Please check the console for more details.
                    </p>
                    <button onclick="location.reload()" style="
                        background: linear-gradient(45deg, #4fc3f7, #29b6f6);
                        color: white;
                        border: none;
                        padding: 0.75rem 1.5rem;
                        border-radius: 8px;
                        font-size: 1rem;
                        cursor: pointer;
                    ">
                        <i class="fas fa-refresh"></i>
                        Retry
                    </button>
                </div>
            </div>
        `;
    }
});

// Handle page visibility changes
document.addEventListener('visibilitychange', () => {
    if (document.visibilityState === 'visible' && sutantraApp) {
        // Page became visible, check connection
        setTimeout(async () => {
            const isConnected = await sutantraApp.checkNodeConnection();
            if (!isConnected) {
                console.log('🔄 Page visible, reconnecting to node...');
                await sutantraApp.reconnectToNode();
            }
        }, 1000);
    }
});

// Handle online/offline events
window.addEventListener('online', async () => {
    console.log('🌐 Network connection restored');
    if (sutantraApp) {
        await sutantraApp.reconnectToNode();
    }
});

window.addEventListener('offline', () => {
    console.log('📡 Network connection lost');
    if (window.uiController) {
        window.uiController.showNotification('Network connection lost', 'warning');
    }
});

// Global error handler
window.addEventListener('error', (event) => {
    console.error('🚨 Global error:', event.error);
    if (window.uiController) {
        window.uiController.showNotification('An unexpected error occurred', 'error');
    }
});

// Unhandled promise rejection handler
window.addEventListener('unhandledrejection', (event) => {
    console.error('🚨 Unhandled promise rejection:', event.reason);
    if (window.uiController) {
        window.uiController.showNotification('An unexpected error occurred', 'error');
    }
});

// Export for debugging
window.SutantraApp = SutantraApp;
