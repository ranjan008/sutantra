#!/bin/bash

# Simple Web UI Demo - Start with mock implementation
# This will create a basic working demo without full integration

set -e

echo "🚀 Starting Simple Sutantra Web UI Demo"
echo "======================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
WEB_PORT=8080

echo -e "${BLUE}📋 Configuration:${NC}"
echo "  • Web UI Port: $WEB_PORT"
echo ""

# Check if Python is available for simple HTTP server
if command -v python3 &> /dev/null; then
    PYTHON_CMD="python3"
elif command -v python &> /dev/null; then
    PYTHON_CMD="python"
else
    echo -e "${RED}❌ Python not found. Please install Python to run the demo.${NC}"
    exit 1
fi

echo -e "${YELLOW}🌐 Starting simple HTTP server for Web UI...${NC}"
echo "  • Web UI will be available at: ${GREEN}http://localhost:$WEB_PORT${NC}"
echo ""

# Function to cleanup on exit
cleanup() {
    echo ""
    echo -e "${YELLOW}🧹 Cleaning up...${NC}"
    
    # Kill any background processes
    if [ ! -z "$SERVER_PID" ]; then
        kill $SERVER_PID 2>/dev/null || true
        wait $SERVER_PID 2>/dev/null || true
    fi
    
    echo -e "${GREEN}✅ Cleanup completed${NC}"
    exit 0
}

# Set up signal handlers
trap cleanup SIGINT SIGTERM

# Change to web-ui directory
cd /home/abhi/sutantra/web-ui

# Start simple HTTP server
echo -e "${GREEN}🚀 Starting Web UI server...${NC}"
echo -e "${YELLOW}📝 Server output will appear below. Press Ctrl+C to stop.${NC}"
echo "=================================================="

$PYTHON_CMD -m http.server $WEB_PORT &
SERVER_PID=$!

# Wait a moment for the server to start
sleep 2

# Check if the server is running
if ! kill -0 $SERVER_PID 2>/dev/null; then
    echo -e "${RED}❌ Server failed to start.${NC}"
    exit 1
fi

echo ""
echo -e "${GREEN}✅ Web UI server is running!${NC}"
echo ""
echo -e "${BLUE}🌐 Access the Web UI:${NC}"
echo "  📱 Open your browser and go to: ${GREEN}http://localhost:$WEB_PORT${NC}"
echo ""
echo -e "${YELLOW}⚠️  Note: This is a static demo. WebSocket connections will fail${NC}"
echo -e "${YELLOW}   until the Sutantra node is running with WebSocket support.${NC}"
echo ""
echo -e "${BLUE}🎨 Web UI Features:${NC}"
echo "  • Modern responsive design"
echo "  • Video streaming interface"
echo "  • Real-time statistics display"
echo "  • Stream quality controls"
echo "  • Blockchain integration UI"
echo ""
echo -e "${YELLOW}📋 Press Ctrl+C to stop the demo${NC}"

# Wait for the server process
wait $SERVER_PID

