#!/usr/bin/env python3

import asyncio
import websockets
import json

async def test_viewer_discovery():
    uri = "ws://localhost:8081/ws"
    print(f"🔍 Connecting to {uri}...")
    
    try:
        async with websockets.connect(uri) as websocket:
            print("✅ Connected to viewer WebSocket")
            
            # Send handshake
            handshake = {
                "type": "handshake",
                "data": {"node_type": "viewer"}
            }
            print("📤 Sending handshake...")
            await websocket.send(json.dumps(handshake))
            
            # Wait for handshake response
            response = await websocket.recv()
            message = json.loads(response)
            print(f"📨 Handshake response: {json.dumps(message, indent=2)}")
            
            if message.get("type") == "handshakeResponse":
                # Request stream list
                stream_request = {
                    "type": "requestStreamList",
                    "data": {}
                }
                print("📤 Requesting stream list...")
                await websocket.send(json.dumps(stream_request))
                
                # Wait for stream list
                response = await websocket.recv()
                message = json.loads(response)
                print(f"📋 Stream list response: {json.dumps(message, indent=2)}")
                
                streams = message.get("data", {}).get("streams", [])
                print(f"✅ Found {len(streams)} streams")
                
    except Exception as e:
        print(f"❌ Error: {e}")

if __name__ == "__main__":
    asyncio.run(test_viewer_discovery())

