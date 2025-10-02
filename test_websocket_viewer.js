const WebSocket = require('ws');

console.log('🔍 Testing viewer WebSocket connection...');

const ws = new WebSocket('ws://localhost:8081/ws');

ws.on('open', () => {
    console.log('✅ Connected to viewer WebSocket');
    
    // Send handshake
    console.log('📤 Sending handshake...');
    ws.send(JSON.stringify({
        type: 'handshake',
        data: { node_type: 'viewer' }
    }));
});

ws.on('message', (data) => {
    const message = JSON.parse(data.toString());
    console.log('📨 Received message:', JSON.stringify(message, null, 2));
    
    if (message.type === 'handshakeResponse') {
        console.log('📤 Sending requestStreamList...');
        ws.send(JSON.stringify({
            type: 'requestStreamList',
            data: {}
        }));
    } else if (message.type === 'streamList') {
        console.log('📋 Stream list received:', message.data.streams);
        process.exit(0);
    }
});

ws.on('error', (error) => {
    console.error('❌ WebSocket error:', error);
    process.exit(1);
});

ws.on('close', () => {
    console.log('🔌 WebSocket connection closed');
    process.exit(0);
});

// Timeout after 10 seconds
setTimeout(() => {
    console.log('⏰ Timeout - closing connection');
    ws.close();
}, 10000);

