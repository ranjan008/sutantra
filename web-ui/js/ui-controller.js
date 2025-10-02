/**
 * Sutantra UI Controller
 * Handles UI interactions and state management
 */

class SutantraUIController {
    constructor() {
        this.streamTime = 0;
        this.streamTimer = null;
        this.isStreaming = false;
        this.nodeType = null; // Don't set default - wait for server
        this.availableStreams = []; // For viewer mode
        
        // Don't setup UI yet - wait for node type from server
        console.log('🎭 UI Controller created, waiting for node type from server...');
    }

    setNodeType(nodeType) {
        console.log(`🎭 setNodeType called with: ${nodeType}`);
        console.log(`🎭 Previous nodeType was: ${this.nodeType}`);
        
        const isFirstTime = this.nodeType === null;
        this.nodeType = nodeType;
        
        console.log(`🎭 UI Controller set to ${nodeType} mode`);
        
        if (isFirstTime) {
            console.log(`🎭 First time setup - initializing UI and event listeners`);
            this.setupEventListeners();
            this.initializeUI();
        }
        
        console.log(`🎭 About to call updateUIForNodeType()`);
        this.updateUIForNodeType();
        console.log(`🎭 updateUIForNodeType() completed`);
    }

    updateUIForNodeType() {
        console.log(`🔧 updateUIForNodeType called with nodeType: ${this.nodeType}`);
        
        const streamControls = document.querySelector('.stream-controls');
        const streamInfo = document.querySelector('.stream-info');
        
        console.log(`🔧 Found streamControls element: ${!!streamControls}`);
        console.log(`🔧 Found streamInfo element: ${!!streamInfo}`);
        
        if (this.nodeType === 'streamer') {
            console.log(`🎬 Setting up STREAMER UI`);
            this.setupStreamerUI();
        } else if (this.nodeType === 'viewer') {
            console.log(`👥 Setting up VIEWER UI`);
            this.setupViewerUI();
        } else {
            console.log(`❓ Unknown nodeType: ${this.nodeType}`);
        }
        
        // Update page title
        document.title = `Sutantra - ${this.nodeType.charAt(0).toUpperCase() + this.nodeType.slice(1)}`;
        console.log(`📄 Updated page title to: ${document.title}`);
        
        // Update header
        const headerTitle = document.querySelector('.logo h1');
        if (headerTitle) {
            headerTitle.innerHTML = `
                <i class="fas fa-broadcast-tower"></i>
                Sutantra
                <span style="font-size: 0.6em; color: #4fc3f7; margin-left: 0.5em;">
                    ${this.nodeType.toUpperCase()}
                </span>
            `;
            console.log(`🏷️ Updated header to show: ${this.nodeType.toUpperCase()}`);
        } else {
            console.log(`❌ Header title element not found`);
        }
    }

    setupStreamerUI() {
        console.log('🎬 Setting up Streamer UI');
        
        // Update main controls
        const streamControls = document.querySelector('.stream-controls');
        if (streamControls) {
            console.log('🎬 Found streamControls, updating innerHTML for STREAMER');
            streamControls.innerHTML = `
                <button id="startStreamBtn" class="btn btn-primary">
                    <i class="fas fa-play"></i>
                    Start Stream
                </button>
                <button id="stopStreamBtn" class="btn btn-danger" style="display: none;">
                    <i class="fas fa-stop"></i>
                    Stop Stream
                </button>
                <button id="settingsBtn" class="btn btn-secondary">
                    <i class="fas fa-cog"></i>
                    Settings
                </button>
            `;
            console.log('🎬 STREAMER UI innerHTML updated successfully');
        } else {
            console.error('❌ streamControls element not found for STREAMER UI');
        }
        
        // Re-setup event listeners for new buttons
        this.setupEventListeners();
    }

    setupViewerUI() {
        console.log('👥 Setting up Viewer UI');
        
        // Update main controls - simpler for viewers
        const streamControls = document.querySelector('.stream-controls');
        if (streamControls) {
            console.log('👥 Found streamControls, updating innerHTML for VIEWER');
            streamControls.innerHTML = `
                <button id="joinStreamBtn" class="btn btn-success" style="display: none;">
                    <i class="fas fa-play"></i>
                    Join Stream
                </button>
                <button id="leaveStreamBtn" class="btn btn-danger" style="display: none;">
                    <i class="fas fa-stop"></i>
                    Leave Stream
                </button>
                <button id="settingsBtn" class="btn btn-secondary">
                    <i class="fas fa-cog"></i>
                    Settings
                </button>
            `;
            console.log('👥 VIEWER UI main controls updated successfully');
        } else {
            console.error('❌ streamControls element not found for VIEWER UI');
        }
        
        // Show the Available Streams section in sidebar
        const availableStreamsSection = document.getElementById('availableStreamsSection');
        if (availableStreamsSection) {
            availableStreamsSection.style.display = 'block';
            console.log('👥 Available Streams section shown in sidebar');
        } else {
            console.error('❌ availableStreamsSection element not found');
        }
        
        // Setup viewer-specific event listeners
        this.setupViewerEventListeners();
    }

    setupViewerEventListeners() {
        const refreshBtn = document.getElementById('refreshStreamsBtn');
        const joinBtn = document.getElementById('joinStreamBtn');
        const leaveBtn = document.getElementById('leaveStreamBtn');
        
        if (refreshBtn) {
            refreshBtn.addEventListener('click', () => this.refreshAvailableStreams());
        }
        
        if (joinBtn) {
            joinBtn.addEventListener('click', () => this.handleJoinStream());
        }
        
        if (leaveBtn) {
            leaveBtn.addEventListener('click', () => this.handleLeaveStream());
        }
    }

    setupEventListeners() {
        // Stream control buttons
        const startStreamBtn = document.getElementById('startStreamBtn');
        const stopStreamBtn = document.getElementById('stopStreamBtn');
        const settingsBtn = document.getElementById('settingsBtn');

        if (startStreamBtn) {
            startStreamBtn.addEventListener('click', () => this.handleStartStream());
        }
        
        if (stopStreamBtn) {
            stopStreamBtn.addEventListener('click', () => this.handleStopStream());
        }
        
        if (settingsBtn) {
            settingsBtn.addEventListener('click', () => this.showModal('settingsModal'));
        }

        // Settings controls
        const bitrateSlider = document.getElementById('bitrateSlider');
        const bitrateValue = document.getElementById('bitrateValue');
        
        if (bitrateSlider && bitrateValue) {
            bitrateSlider.addEventListener('input', (e) => {
                bitrateValue.textContent = e.target.value;
            });
        }

        // Quality selector
        const qualitySelect = document.getElementById('qualitySelect');
        if (qualitySelect) {
            qualitySelect.addEventListener('change', (e) => {
                this.updateQualityDisplay(e.target.value);
            });
        }

        // Modal close handlers
        document.addEventListener('click', (e) => {
            if (e.target.classList.contains('modal')) {
                this.closeModal(e.target.id);
            }
        });

        // Keyboard shortcuts
        document.addEventListener('keydown', (e) => {
            this.handleKeyboardShortcuts(e);
        });

        // Window beforeunload
        window.addEventListener('beforeunload', (e) => {
            if (this.isStreaming) {
                e.preventDefault();
                e.returnValue = 'You are currently streaming. Are you sure you want to leave?';
            }
        });
    }

    initializeUI() {
        // Initialize quality display
        const qualitySelect = document.getElementById('qualitySelect');
        if (qualitySelect) {
            this.updateQualityDisplay(qualitySelect.value);
        }

        // Initialize bitrate display
        const bitrateSlider = document.getElementById('bitrateSlider');
        const bitrateValue = document.getElementById('bitrateValue');
        if (bitrateSlider && bitrateValue) {
            bitrateValue.textContent = bitrateSlider.value;
        }

        // Check for camera/microphone permissions
        this.checkMediaPermissions();
    }

    async checkMediaPermissions() {
        try {
            const permissions = await navigator.permissions.query({ name: 'camera' });
            console.log('📹 Camera permission:', permissions.state);
            
            if (permissions.state === 'denied') {
                this.showNotification('Camera access is required for streaming', 'warning');
            }
        } catch (error) {
            console.warn('⚠️ Could not check camera permissions:', error);
        }
    }

    async handleStartStream() {
        try {
            this.showLoadingOverlay('Starting stream...');
            
            // Disable start button, enable stop button
            this.updateStreamButtons(true);
            
            // Start the stream via WebRTC client
            if (window.webrtcClient) {
                await window.webrtcClient.startStream();
                this.isStreaming = true;
                this.startStreamTimer();
                this.showNotification('Stream started successfully!', 'success');
            } else {
                throw new Error('WebRTC client not initialized');
            }
            
        } catch (error) {
            console.error('❌ Failed to start stream:', error);
            this.showNotification(`Failed to start stream: ${error.message}`, 'error');
            this.updateStreamButtons(false);
        } finally {
            this.hideLoadingOverlay();
        }
    }

    async handleStopStream() {
        try {
            this.showLoadingOverlay('Stopping stream...');
            
            // Stop the stream via WebRTC client
            if (window.webrtcClient) {
                await window.webrtcClient.stopStream();
                this.isStreaming = false;
                this.stopStreamTimer();
                this.showNotification('Stream stopped successfully!', 'success');
            }
            
            // Update button states
            this.updateStreamButtons(false);
            
        } catch (error) {
            console.error('❌ Failed to stop stream:', error);
            this.showNotification(`Failed to stop stream: ${error.message}`, 'error');
        } finally {
            this.hideLoadingOverlay();
        }
    }

    updateStreamButtons(streaming) {
        const startBtn = document.getElementById('startStreamBtn');
        const stopBtn = document.getElementById('stopStreamBtn');
        
        console.log(`🎬 updateStreamButtons called with streaming: ${streaming}`);
        
        if (startBtn) {
            if (streaming) {
                startBtn.style.display = 'none';
                console.log('🎬 Hidden start button');
            } else {
                startBtn.style.display = 'inline-block';
                startBtn.innerHTML = '<i class="fas fa-play"></i> Start Stream';
                console.log('🎬 Shown start button');
            }
        }
        
        if (stopBtn) {
            if (streaming) {
                stopBtn.style.display = 'inline-block';
                console.log('🎬 Shown stop button');
            } else {
                stopBtn.style.display = 'none';
                console.log('🎬 Hidden stop button');
            }
        }
    }

    startStreamTimer() {
        this.streamTime = 0;
        this.updateStreamTimeDisplay();
        
        this.streamTimer = setInterval(() => {
            this.streamTime++;
            this.updateStreamTimeDisplay();
        }, 1000);
    }

    stopStreamTimer() {
        if (this.streamTimer) {
            clearInterval(this.streamTimer);
            this.streamTimer = null;
        }
        this.streamTime = 0;
        this.updateStreamTimeDisplay();
    }

    updateStreamTimeDisplay() {
        const streamTimeElement = document.getElementById('streamTime');
        if (streamTimeElement) {
            const hours = Math.floor(this.streamTime / 3600);
            const minutes = Math.floor((this.streamTime % 3600) / 60);
            const seconds = this.streamTime % 60;
            
            const timeString = hours > 0 ? 
                `${hours.toString().padStart(2, '0')}:${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}` :
                `${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`;
                
            streamTimeElement.textContent = timeString;
        }
    }

    updateQualityDisplay(quality) {
        const qualityElement = document.getElementById('quality');
        if (qualityElement) {
            qualityElement.textContent = quality;
        }
    }

    showModal(modalId) {
        const modal = document.getElementById(modalId);
        if (modal) {
            modal.style.display = 'block';
            document.body.style.overflow = 'hidden';
        }
    }

    closeModal(modalId) {
        const modal = document.getElementById(modalId);
        if (modal) {
            modal.style.display = 'none';
            document.body.style.overflow = 'auto';
        }
    }

    showLoadingOverlay(message = 'Loading...') {
        const overlay = document.getElementById('loadingOverlay');
        if (overlay) {
            const messageElement = overlay.querySelector('p');
            if (messageElement) {
                messageElement.textContent = message;
            }
            overlay.style.display = 'flex';
        }
    }

    hideLoadingOverlay() {
        const overlay = document.getElementById('loadingOverlay');
        if (overlay) {
            overlay.style.display = 'none';
        }
    }

    showNotification(message, type = 'info') {
        // Create notification element
        const notification = document.createElement('div');
        notification.className = `notification notification-${type}`;
        notification.innerHTML = `
            <div class="notification-content">
                <i class="fas ${this.getNotificationIcon(type)}"></i>
                <span>${message}</span>
                <button class="notification-close" onclick="this.parentElement.parentElement.remove()">
                    <i class="fas fa-times"></i>
                </button>
            </div>
        `;

        // Add styles if not already present
        if (!document.querySelector('#notification-styles')) {
            const styles = document.createElement('style');
            styles.id = 'notification-styles';
            styles.textContent = `
                .notification {
                    position: fixed;
                    top: 20px;
                    right: 20px;
                    z-index: 10000;
                    max-width: 400px;
                    border-radius: 8px;
                    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
                    animation: slideInRight 0.3s ease-out;
                }
                .notification-success { background: linear-gradient(45deg, #4caf50, #45a049); }
                .notification-error { background: linear-gradient(45deg, #f44336, #d32f2f); }
                .notification-warning { background: linear-gradient(45deg, #ff9800, #f57c00); }
                .notification-info { background: linear-gradient(45deg, #2196f3, #1976d2); }
                .notification-content {
                    display: flex;
                    align-items: center;
                    gap: 12px;
                    padding: 16px;
                    color: white;
                }
                .notification-close {
                    background: none;
                    border: none;
                    color: white;
                    cursor: pointer;
                    font-size: 14px;
                    opacity: 0.8;
                    margin-left: auto;
                }
                .notification-close:hover { opacity: 1; }
                @keyframes slideInRight {
                    from { transform: translateX(100%); opacity: 0; }
                    to { transform: translateX(0); opacity: 1; }
                }
            `;
            document.head.appendChild(styles);
        }

        // Add to DOM
        document.body.appendChild(notification);

        // Auto-remove after 5 seconds
        setTimeout(() => {
            if (notification.parentElement) {
                notification.remove();
            }
        }, 5000);
    }

    getNotificationIcon(type) {
        switch (type) {
            case 'success': return 'fa-check-circle';
            case 'error': return 'fa-exclamation-circle';
            case 'warning': return 'fa-exclamation-triangle';
            default: return 'fa-info-circle';
        }
    }

    handleKeyboardShortcuts(e) {
        // Ctrl/Cmd + Enter: Start/Stop stream
        if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
            e.preventDefault();
            if (this.isStreaming) {
                this.handleStopStream();
            } else {
                this.handleStartStream();
            }
        }

        // Escape: Close modals
        if (e.key === 'Escape') {
            const openModal = document.querySelector('.modal[style*="block"]');
            if (openModal) {
                this.closeModal(openModal.id);
            }
        }

        // F11: Toggle fullscreen (for video element)
        if (e.key === 'F11') {
            e.preventDefault();
            this.toggleVideoFullscreen();
        }
    }

    toggleVideoFullscreen() {
        const videoContainer = document.querySelector('.video-container');
        if (!videoContainer) return;

        if (!document.fullscreenElement) {
            videoContainer.requestFullscreen().catch(err => {
                console.warn('⚠️ Could not enter fullscreen:', err);
            });
        } else {
            document.exitFullscreen();
        }
    }

    // Settings management
    saveSettings() {
        const settings = {
            nodeUrl: document.getElementById('nodeUrl')?.value || 'ws://localhost:30333',
            streamKey: document.getElementById('streamKey')?.value || '',
            enableP2P: document.getElementById('enableP2P')?.checked || true,
            enableBlockchain: document.getElementById('enableBlockchain')?.checked || true
        };

        // Save to localStorage
        localStorage.setItem('sutantra-settings', JSON.stringify(settings));

        // Apply settings
        if (window.webrtcClient && settings.nodeUrl !== window.webrtcClient.nodeUrl) {
            window.webrtcClient.nodeUrl = settings.nodeUrl;
            this.showNotification('Settings saved! Reconnect to apply changes.', 'success');
        } else {
            this.showNotification('Settings saved successfully!', 'success');
        }

        this.closeModal('settingsModal');
    }

    loadSettings() {
        try {
            const savedSettings = localStorage.getItem('sutantra-settings');
            if (savedSettings) {
                const settings = JSON.parse(savedSettings);
                
                // Apply to UI
                const nodeUrlInput = document.getElementById('nodeUrl');
                const streamKeyInput = document.getElementById('streamKey');
                const enableP2PCheckbox = document.getElementById('enableP2P');
                const enableBlockchainCheckbox = document.getElementById('enableBlockchain');

                if (nodeUrlInput) nodeUrlInput.value = settings.nodeUrl || 'ws://localhost:30333';
                if (streamKeyInput) streamKeyInput.value = settings.streamKey || '';
                if (enableP2PCheckbox) enableP2PCheckbox.checked = settings.enableP2P !== false;
                if (enableBlockchainCheckbox) enableBlockchainCheckbox.checked = settings.enableBlockchain !== false;

                return settings;
            }
        } catch (error) {
            console.warn('⚠️ Could not load settings:', error);
        }
        return null;
    }

    // Viewer-specific methods
    async refreshAvailableStreams() {
        console.log('🔄 Refreshing available streams...');
        this.showNotification('Refreshing streams...', 'info');
        
        try {
            // Request stream list from WebRTC client
            if (window.webrtcClient) {
                console.log('🔄 WebRTC client found, requesting stream list...');
                await window.webrtcClient.requestStreamList();
                console.log('🔄 Stream list request sent');
            } else {
                console.error('❌ WebRTC client not available');
                this.showNotification('WebRTC client not available', 'error');
            }
        } catch (error) {
            console.error('❌ Failed to refresh streams:', error);
            this.showNotification('Failed to refresh streams: ' + error.message, 'error');
        }
    }

    updateAvailableStreams(streams) {
        console.log('📺 Updating available streams:', streams);
        this.availableStreams = streams || [];
        
        const streamsList = document.getElementById('streamsList');
        if (!streamsList) return;
        
        if (this.availableStreams.length === 0) {
            streamsList.innerHTML = '<p class="no-streams">No active streams found. Click refresh to check again.</p>';
            return;
        }
        
        streamsList.innerHTML = this.availableStreams.map(stream => `
            <div class="stream-item" data-stream-id="${stream.stream_id}">
                <div class="stream-info">
                    <h4>${stream.title || 'Untitled Stream'}</h4>
                    <p class="stream-creator">by ${stream.creator}</p>
                    <p class="stream-viewers">${stream.viewers || 0} viewers</p>
                    <p class="stream-quality">${stream.quality || '720p'}</p>
                </div>
                <button class="btn btn-primary btn-sm" onclick="window.uiController.selectStream('${stream.stream_id}')">
                    <i class="fas fa-play"></i>
                    Select
                </button>
            </div>
        `).join('');
    }

    selectStream(streamId) {
        console.log('🎯 Selected stream:', streamId);
        this.selectedStreamId = streamId;
        
        // Show join button
        const joinBtn = document.getElementById('joinStreamBtn');
        if (joinBtn) {
            joinBtn.style.display = 'inline-block';
        }
        
        // Highlight selected stream
        document.querySelectorAll('.stream-item').forEach(item => {
            item.classList.remove('selected');
        });
        
        const selectedItem = document.querySelector(`[data-stream-id="${streamId}"]`);
        if (selectedItem) {
            selectedItem.classList.add('selected');
        }
        
        this.showNotification(`Selected stream: ${streamId}`, 'success');
    }

    async handleJoinStream() {
        if (!this.selectedStreamId) {
            this.showNotification('Please select a stream first', 'warning');
            return;
        }
        
        console.log('🚀 Joining stream:', this.selectedStreamId);
        this.showNotification('Joining stream...', 'info');
        
        try {
            if (window.webrtcClient) {
                await window.webrtcClient.joinStream(this.selectedStreamId);
                
                // Update UI
                const joinBtn = document.getElementById('joinStreamBtn');
                const leaveBtn = document.getElementById('leaveStreamBtn');
                
                if (joinBtn) joinBtn.style.display = 'none';
                if (leaveBtn) leaveBtn.style.display = 'inline-block';
                
                this.isStreaming = true;
                this.startStreamTimer();
                this.showNotification('Successfully joined stream!', 'success');
            }
        } catch (error) {
            console.error('❌ Failed to join stream:', error);
            this.showNotification('Failed to join stream: ' + error.message, 'error');
        }
    }

    async handleLeaveStream() {
        console.log('🚪 Leaving stream...');
        this.showNotification('Leaving stream...', 'info');
        
        try {
            if (window.webrtcClient) {
                await window.webrtcClient.leaveStream();
                
                // Update UI
                const joinBtn = document.getElementById('joinStreamBtn');
                const leaveBtn = document.getElementById('leaveStreamBtn');
                
                if (joinBtn) joinBtn.style.display = 'inline-block';
                if (leaveBtn) leaveBtn.style.display = 'none';
                
                this.isStreaming = false;
                this.stopStreamTimer();
                this.showNotification('Left stream successfully', 'success');
            }
        } catch (error) {
            console.error('❌ Failed to leave stream:', error);
            this.showNotification('Failed to leave stream: ' + error.message, 'error');
        }
    }
}

// Global functions for HTML onclick handlers
window.showModal = (modalId) => {
    if (window.uiController) {
        window.uiController.showModal(modalId);
    }
};

window.closeModal = (modalId) => {
    if (window.uiController) {
        window.uiController.closeModal(modalId);
    }
};

window.saveSettings = () => {
    if (window.uiController) {
        window.uiController.saveSettings();
    }
};

window.showAbout = () => {
    alert('Sutantra - Decentralized Live Streaming Platform\n\nBuilt with WebRTC and blockchain technology for truly decentralized streaming.');
};

window.showHelp = () => {
    alert('Sutantra Help:\n\n• Click "Start Stream" to begin streaming\n• Use Ctrl+Enter to start/stop streaming\n• Press F11 for fullscreen video\n• Configure settings in the Settings panel');
};

// Export for use in other modules
window.SutantraUIController = SutantraUIController;
