// Test WebSocket stream discovery
const WebSocket = require('ws');

console.log('🧪 Testing WebSocket Stream Discovery');
console.log('=====================================');

// Test viewer node (8081)
const ws = new WebSocket('ws://localhost:8081/ws');

ws.on('open', function() {
    console.log('✅ Connected to viewer node WebSocket');
    
    // Send handshake
    const handshake = {
        type: 'handshake',
        data: { client_type: 'test' }
    };
    
    console.log('📤 Sending handshake:', JSON.stringify(handshake));
    ws.send(JSON.stringify(handshake));
});

ws.on('message', function(data) {
    const message = JSON.parse(data.toString());
    console.log('📥 Received:', JSON.stringify(message, null, 2));
    
    if (message.type === 'handshakeResponse') {
        console.log('🤝 Handshake successful, requesting stream list...');
        
        // Request stream list
        const streamRequest = {
            type: 'requestStreamList',
            data: {}
        };
        
        console.log('📤 Sending stream list request:', JSON.stringify(streamRequest));
        ws.send(JSON.stringify(streamRequest));
    } else if (message.type === 'streamList') {
        console.log('📺 Stream list received!');
        console.log('📺 Number of streams:', message.data?.streams?.length || 0);
        
        if (message.data?.streams?.length > 0) {
            console.log('✅ SUCCESS: Streams found!');
            message.data.streams.forEach((stream, i) => {
                console.log(`  Stream ${i + 1}: ${stream.title} (${stream.stream_id})`);
            });
        } else {
            console.log('⚠️  No streams found (this is expected if no streams are active)');
        }
        
        ws.close();
        process.exit(0);
    }
});

ws.on('error', function(error) {
    console.error('❌ WebSocket error:', error.message);
    process.exit(1);
});

ws.on('close', function() {
    console.log('🔌 WebSocket connection closed');
});

// Timeout after 10 seconds
setTimeout(() => {
    console.log('⏰ Test timeout - closing connection');
    ws.close();
    process.exit(1);
}, 10000);

