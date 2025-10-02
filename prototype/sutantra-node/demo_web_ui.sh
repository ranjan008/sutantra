#!/bin/bash

# Sutantra Web UI Demo Script
# Starts a Sutantra node with web UI enabled

set -e

echo "🚀 Starting Sutantra Web UI Demo"
echo "================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
NODE_PORT=30333
WEB_PORT=8080
LOG_LEVEL=info

echo -e "${BLUE}📋 Configuration:${NC}"
echo "  • Node Port: $NODE_PORT"
echo "  • Web UI Port: $WEB_PORT"
echo "  • Log Level: $LOG_LEVEL"
echo ""

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}❌ Cargo not found. Please install Rust: https://rustup.rs/${NC}"
    exit 1
fi

# Build the project
echo -e "${YELLOW}🔨 Building Sutantra node...${NC}"
if ! cargo build --release; then
    echo -e "${RED}❌ Build failed. Please check the errors above.${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Build completed successfully${NC}"
echo ""

# Create web-ui symlink if it doesn't exist
if [ ! -L "web-ui" ] && [ ! -d "web-ui" ]; then
    echo -e "${YELLOW}🔗 Creating web-ui symlink...${NC}"
    ln -s ../../web-ui web-ui
fi

# Function to cleanup on exit
cleanup() {
    echo ""
    echo -e "${YELLOW}🧹 Cleaning up...${NC}"
    
    # Kill any background processes
    if [ ! -z "$NODE_PID" ]; then
        kill $NODE_PID 2>/dev/null || true
        wait $NODE_PID 2>/dev/null || true
    fi
    
    echo -e "${GREEN}✅ Cleanup completed${NC}"
    exit 0
}

# Set up signal handlers
trap cleanup SIGINT SIGTERM

# Start the node with web UI
echo -e "${BLUE}🌐 Starting Sutantra node with Web UI...${NC}"
echo "  • Node endpoint: http://localhost:$NODE_PORT"
echo "  • Web UI: http://localhost:$WEB_PORT/ui/"
echo "  • WebSocket: ws://localhost:$WEB_PORT/ws"
echo ""

# Set environment variables
export RUST_LOG=$LOG_LEVEL
export RUST_BACKTRACE=1

# Start the node
echo -e "${GREEN}🚀 Launching Sutantra node...${NC}"
echo -e "${YELLOW}📝 Logs will appear below. Press Ctrl+C to stop.${NC}"
echo "=================================================="

./target/release/sutantra-node start \
    --port $NODE_PORT \
    --web-ui \
    --web-port $WEB_PORT \
    --streaming &

NODE_PID=$!

# Wait a moment for the node to start
sleep 3

# Check if the node is running
if ! kill -0 $NODE_PID 2>/dev/null; then
    echo -e "${RED}❌ Node failed to start. Check the logs above.${NC}"
    exit 1
fi

echo ""
echo -e "${GREEN}✅ Sutantra node is running!${NC}"
echo ""
echo -e "${BLUE}🌐 Access the Web UI:${NC}"
echo "  📱 Open your browser and go to: ${GREEN}http://localhost:$WEB_PORT/ui/${NC}"
echo ""
echo -e "${BLUE}🎥 To start streaming:${NC}"
echo "  1. Allow camera/microphone access when prompted"
echo "  2. Click 'Start Stream' button"
echo "  3. Configure quality settings in the side panel"
echo ""
echo -e "${BLUE}🔧 API Endpoints:${NC}"
echo "  • Health: http://localhost:$WEB_PORT/health"
echo "  • Node Info: http://localhost:$WEB_PORT/api/node/info"
echo "  • WebSocket: ws://localhost:$WEB_PORT/ws"
echo ""
echo -e "${YELLOW}📋 Press Ctrl+C to stop the demo${NC}"

# Wait for the node process
wait $NODE_PID

