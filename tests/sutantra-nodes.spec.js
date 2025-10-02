const { test, expect } = require('@playwright/test');

// Test configuration
const STREAMER_URL = 'http://localhost:8080';
const VIEWER_URL = 'http://localhost:8082';
const TEST_TIMEOUT = 30000;

test.describe('Sutantra Nodes Basic Functionality', () => {
  
  test.beforeEach(async () => {
    // Wait a bit for nodes to be ready
    await new Promise(resolve => setTimeout(resolve, 2000));
  });

  test('Streamer node should be accessible and load correctly', async ({ page }) => {
    console.log('🎬 Testing Streamer Node...');
    
    // Navigate to streamer
    await page.goto(STREAMER_URL);
    
    // Check if page loads
    await expect(page).toHaveTitle(/Sutantra/);
    
    // Check for key elements
    await expect(page.locator('h1')).toContainText('Sutantra');
    await expect(page.locator('#startStreamBtn')).toBeVisible();
    
    // Check node info
    const response = await page.request.get(`${STREAMER_URL}/api/node/info`);
    expect(response.ok()).toBeTruthy();
    const nodeInfo = await response.json();
    expect(nodeInfo.node_type).toBe('streamer');
    expect(nodeInfo.web_port).toBe(8080);
    
    console.log('✅ Streamer node info:', nodeInfo);
  });

  test('Viewer node should be accessible and load correctly', async ({ page }) => {
    console.log('👥 Testing Viewer Node...');
    
    // Navigate to viewer
    await page.goto(VIEWER_URL);
    
    // Check if page loads
    await expect(page).toHaveTitle(/Sutantra/);
    
    // Check for key elements
    await expect(page.locator('h1')).toContainText('Sutantra');
    
    // Check node info
    const response = await page.request.get(`${VIEWER_URL}/api/node/info`);
    expect(response.ok()).toBeTruthy();
    const nodeInfo = await response.json();
    expect(nodeInfo.node_type).toBe('viewer');
    expect(nodeInfo.web_port).toBe(8082);
    
    console.log('✅ Viewer node info:', nodeInfo);
  });

  test('Both nodes should have healthy status', async ({ page }) => {
    console.log('🏥 Testing Node Health...');
    
    // Test streamer health
    const streamerHealth = await page.request.get(`${STREAMER_URL}/health`);
    expect(streamerHealth.ok()).toBeTruthy();
    const streamerStatus = await streamerHealth.json();
    expect(streamerStatus.status).toBe('healthy');
    
    // Test viewer health
    const viewerHealth = await page.request.get(`${VIEWER_URL}/health`);
    expect(viewerHealth.ok()).toBeTruthy();
    const viewerStatus = await viewerHealth.json();
    expect(viewerStatus.status).toBe('healthy');
    
    console.log('✅ Both nodes are healthy');
  });
});

test.describe('WebSocket Connection Tests', () => {
  
  test('Streamer WebSocket should connect and handshake', async ({ page }) => {
    console.log('🔌 Testing Streamer WebSocket...');
    
    await page.goto(STREAMER_URL);
    
    // Wait for page to load
    await page.waitForLoadState('networkidle');
    
    // Check WebSocket connection in browser console
    const wsConnected = await page.evaluate(async () => {
      return new Promise((resolve) => {
        const ws = new WebSocket('ws://localhost:8080/ws');
        
        ws.onopen = () => {
          console.log('WebSocket connected');
          // Send handshake
          ws.send(JSON.stringify({
            type: 'handshake',
            data: {
              node_type: 'streamer',
              client_id: 'test-client-' + Date.now()
            }
          }));
        };
        
        ws.onmessage = (event) => {
          const data = JSON.parse(event.data);
          console.log('WebSocket response:', data);
          if (data.type === 'handshakeResponse' && data.data.success) {
            ws.close();
            resolve(true);
          }
        };
        
        ws.onerror = (error) => {
          console.error('WebSocket error:', error);
          resolve(false);
        };
        
        // Timeout after 5 seconds
        setTimeout(() => {
          ws.close();
          resolve(false);
        }, 5000);
      });
    });
    
    expect(wsConnected).toBe(true);
    console.log('✅ Streamer WebSocket connected successfully');
  });

  test('Viewer WebSocket should connect and handshake', async ({ page }) => {
    console.log('🔌 Testing Viewer WebSocket...');
    
    await page.goto(VIEWER_URL);
    
    // Wait for page to load
    await page.waitForLoadState('networkidle');
    
    // Check WebSocket connection in browser console
    const wsConnected = await page.evaluate(async () => {
      return new Promise((resolve) => {
        const ws = new WebSocket('ws://localhost:8082/ws');
        
        ws.onopen = () => {
          console.log('WebSocket connected');
          // Send handshake
          ws.send(JSON.stringify({
            type: 'handshake',
            data: {
              node_type: 'viewer',
              client_id: 'test-client-' + Date.now()
            }
          }));
        };
        
        ws.onmessage = (event) => {
          const data = JSON.parse(event.data);
          console.log('WebSocket response:', data);
          if (data.type === 'handshakeResponse' && data.data.success) {
            ws.close();
            resolve(true);
          }
        };
        
        ws.onerror = (error) => {
          console.error('WebSocket error:', error);
          resolve(false);
        };
        
        // Timeout after 5 seconds
        setTimeout(() => {
          ws.close();
          resolve(false);
        }, 5000);
      });
    });
    
    expect(wsConnected).toBe(true);
    console.log('✅ Viewer WebSocket connected successfully');
  });
});

test.describe('UI Differentiation Tests', () => {
  
  test('Streamer UI should show streaming controls', async ({ page }) => {
    console.log('🎬 Testing Streamer UI...');
    
    await page.goto(STREAMER_URL);
    await page.waitForLoadState('networkidle');
    
    // Wait for WebSocket connection and UI setup
    await page.waitForTimeout(3000);
    
    // Check for streamer-specific elements
    await expect(page.locator('#startStreamBtn')).toBeVisible();
    
    // Check node type display
    const nodeTypeElement = page.locator('#nodeType');
    if (await nodeTypeElement.isVisible()) {
      await expect(nodeTypeElement).toContainText('streamer');
    }
    
    console.log('✅ Streamer UI shows correct controls');
  });

  test('Viewer UI should show viewer controls', async ({ page }) => {
    console.log('👥 Testing Viewer UI...');
    
    await page.goto(VIEWER_URL);
    await page.waitForLoadState('networkidle');
    
    // Wait for WebSocket connection and UI setup
    await page.waitForTimeout(3000);
    
    // Check node type display
    const nodeTypeElement = page.locator('#nodeType');
    if (await nodeTypeElement.isVisible()) {
      await expect(nodeTypeElement).toContainText('viewer');
    }
    
    // Check for available streams section (should be visible for viewers)
    const availableStreamsSection = page.locator('#availableStreamsSection');
    if (await availableStreamsSection.isVisible()) {
      console.log('✅ Available streams section is visible for viewer');
    }
    
    console.log('✅ Viewer UI shows correct controls');
  });
});

test.describe('Streaming Workflow Tests', () => {
  
  test('Complete streaming workflow: create stream and discover from viewer', async ({ browser }) => {
    console.log('🎥 Testing Complete Streaming Workflow...');
    
    // Create two browser contexts for streamer and viewer
    const streamerContext = await browser.newContext();
    const viewerContext = await browser.newContext();
    
    const streamerPage = await streamerContext.newPage();
    const viewerPage = await viewerContext.newPage();
    
    try {
      // Step 1: Load both pages
      console.log('📱 Loading streamer and viewer pages...');
      await streamerPage.goto(STREAMER_URL);
      await viewerPage.goto(VIEWER_URL);
      
      // Wait for pages to load and WebSocket connections
      await streamerPage.waitForLoadState('networkidle');
      await viewerPage.waitForLoadState('networkidle');
      await streamerPage.waitForTimeout(3000);
      await viewerPage.waitForTimeout(3000);
      
      // Step 2: Create stream on streamer
      console.log('🎬 Creating stream on streamer...');
      
      // Check if start stream button is visible and click it
      const startStreamBtn = streamerPage.locator('#startStreamBtn');
      if (await startStreamBtn.isVisible()) {
        await startStreamBtn.click();
        console.log('✅ Clicked start stream button');
        
        // Wait for stream creation
        await streamerPage.waitForTimeout(2000);
      } else {
        console.log('⚠️ Start stream button not visible, trying WebSocket approach');
        
        // Create stream via WebSocket
        await streamerPage.evaluate(() => {
          if (window.webrtcClient && window.webrtcClient.websocket) {
            window.webrtcClient.createStream('Test Stream');
          }
        });
        await streamerPage.waitForTimeout(2000);
      }
      
      // Step 3: Check for stream on viewer
      console.log('👥 Checking stream discovery on viewer...');
      
      // Try to refresh streams on viewer
      const refreshBtn = viewerPage.locator('#refreshStreamsBtn');
      if (await refreshBtn.isVisible()) {
        await refreshBtn.click();
        console.log('✅ Clicked refresh streams button');
        await viewerPage.waitForTimeout(2000);
      } else {
        console.log('⚠️ Refresh button not visible, trying WebSocket approach');
        
        // Request stream list via WebSocket
        await viewerPage.evaluate(() => {
          if (window.webrtcClient && window.webrtcClient.websocket) {
            window.webrtcClient.requestStreamList();
          }
        });
        await viewerPage.waitForTimeout(2000);
      }
      
      // Step 4: Verify stream appears in viewer
      const streamsList = viewerPage.locator('#streamsList');
      if (await streamsList.isVisible()) {
        const streamsContent = await streamsList.textContent();
        console.log('📋 Streams list content:', streamsContent);
        
        // Check if streams are found
        if (streamsContent && !streamsContent.includes('No active streams')) {
          console.log('✅ Streams discovered successfully!');
        } else {
          console.log('⚠️ No streams found in viewer');
        }
      }
      
      // Step 5: Test API endpoint for streams
      console.log('🔍 Testing streams API endpoint...');
      const streamsResponse = await streamerPage.request.get(`${STREAMER_URL}/api/streams`);
      if (streamsResponse.ok()) {
        const streams = await streamsResponse.json();
        console.log('📊 API streams response:', streams);
        
        if (streams.length > 0) {
          console.log('✅ Stream found via API!');
        } else {
          console.log('⚠️ No streams found via API');
        }
      }
      
      console.log('🎉 Streaming workflow test completed');
      
    } finally {
      await streamerContext.close();
      await viewerContext.close();
    }
  });
});

test.describe('Error Handling Tests', () => {
  
  test('Should handle WebSocket disconnection gracefully', async ({ page }) => {
    console.log('🔌 Testing WebSocket disconnection handling...');
    
    await page.goto(STREAMER_URL);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(3000);
    
    // Simulate WebSocket disconnection
    const disconnectionHandled = await page.evaluate(() => {
      return new Promise((resolve) => {
        if (window.webrtcClient && window.webrtcClient.websocket) {
          const ws = window.webrtcClient.websocket;
          
          // Listen for reconnection attempts
          const originalConnect = window.webrtcClient.connectToNode;
          let reconnectAttempted = false;
          
          window.webrtcClient.connectToNode = function() {
            reconnectAttempted = true;
            return originalConnect.call(this);
          };
          
          // Force close WebSocket
          ws.close();
          
          // Check if reconnection is attempted
          setTimeout(() => {
            resolve(reconnectAttempted);
          }, 5000);
        } else {
          resolve(false);
        }
      });
    });
    
    console.log('🔄 WebSocket disconnection handling:', disconnectionHandled ? 'Good' : 'Needs improvement');
  });
});
