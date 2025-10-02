#!/bin/bash

echo "🧪 Testing Stream Discovery API"
echo "================================"

echo ""
echo "📋 Testing stream list request on viewer node (8081)..."

# Test with curl (this won't work for WebSocket, but let's check if there's an HTTP endpoint)
echo "🔍 Checking if there's an HTTP endpoint for streams..."
curl -s -w "\nHTTP Status: %{http_code}\n" http://localhost:8081/api/streams || echo "No HTTP streams endpoint"

echo ""
echo "🔍 Checking node health..."
curl -s http://localhost:8081/health
echo ""

echo "🔍 Checking node info..."
curl -s http://localhost:8081/api/node/info
echo ""

echo ""
echo "📋 Now let's check the streamer node (8080)..."
curl -s http://localhost:8080/health
echo ""
curl -s http://localhost:8080/api/node/info
echo ""

echo "✅ Basic API tests completed"

