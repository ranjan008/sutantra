#!/bin/bash

# Sutantra Two-Node Streaming Demo
# Node 1: Streaming node with WebRTC and Web UI
# Node 2: Viewer node that connects to streams

set -e

echo "🚀 Starting Sutantra Two-Node Streaming Demo"
echo "============================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Configuration
NODE1_PORT=30333
NODE1_WEB_PORT=8080
NODE2_PORT=30334
NODE2_WEB_PORT=8082
LOG_LEVEL=info

echo -e "${BLUE}📋 Configuration:${NC}"
echo "  • Node 1 (Streamer): Port $NODE1_PORT, Web UI: $NODE1_WEB_PORT"
echo "  • Node 2 (Viewer): Port $NODE2_PORT, Web UI: $NODE2_WEB_PORT"
echo "  • Log Level: $LOG_LEVEL"
echo ""

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}❌ Cargo not found. Please install Rust: https://rustup.rs/${NC}"
    exit 1
fi

# Build the project if needed
if [ ! -f "target/release/sutantra-node" ]; then
    echo -e "${YELLOW}🔨 Building Sutantra node...${NC}"
    if ! cargo build --release; then
        echo -e "${RED}❌ Build failed. Please check the errors above.${NC}"
        exit 1
    fi
    echo -e "${GREEN}✅ Build completed successfully${NC}"
fi

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
    if [ ! -z "$NODE1_PID" ]; then
        echo -e "${YELLOW}  Stopping Node 1...${NC}"
        kill $NODE1_PID 2>/dev/null || true
        wait $NODE1_PID 2>/dev/null || true
    fi
    
    if [ ! -z "$NODE2_PID" ]; then
        echo -e "${YELLOW}  Stopping Node 2...${NC}"
        kill $NODE2_PID 2>/dev/null || true
        wait $NODE2_PID 2>/dev/null || true
    fi
    
    echo -e "${GREEN}✅ Cleanup completed${NC}"
    exit 0
}

# Set up signal handlers
trap cleanup SIGINT SIGTERM

# Set environment variables
export RUST_LOG=$LOG_LEVEL
export RUST_BACKTRACE=1

echo -e "${PURPLE}🎬 Starting Node 1 (Streaming Node)...${NC}"
echo "  • Role: Stream Creator & WebRTC Engine"
echo "  • Port: $NODE1_PORT"
echo "  • Web UI: http://localhost:$NODE1_WEB_PORT"
echo ""

# Start Node 1 (Streaming node with Web UI)
./target/release/sutantra-node start \
    --port $NODE1_PORT \
    --web-ui \
    --web-port $NODE1_WEB_PORT \
    --streaming \
    --validator > node1.log 2>&1 &

NODE1_PID=$!

# Wait for Node 1 to start
sleep 3

# Check if Node 1 is running
if ! kill -0 $NODE1_PID 2>/dev/null; then
    echo -e "${RED}❌ Node 1 failed to start. Check node1.log for details.${NC}"
    cat node1.log
    exit 1
fi

echo -e "${GREEN}✅ Node 1 started successfully (PID: $NODE1_PID)${NC}"

echo ""
echo -e "${PURPLE}👥 Starting Node 2 (Viewer Node)...${NC}"
echo "  • Role: Stream Viewer & Light Client"
echo "  • Port: $NODE2_PORT"
echo "  • Web UI: http://localhost:$NODE2_WEB_PORT"
echo "  • Bootstrap: Node 1"
echo ""

# Start Node 2 (Light client node with Web UI)
./target/release/sutantra-node start-light \
    --port $NODE2_PORT \
    --web-ui \
    --web-port $NODE2_WEB_PORT \
    --bootnodes "/ip4/127.0.0.1/tcp/$NODE1_PORT" > node2.log 2>&1 &

NODE2_PID=$!

# Wait for Node 2 to start
sleep 3

# Check if Node 2 is running
if ! kill -0 $NODE2_PID 2>/dev/null; then
    echo -e "${RED}❌ Node 2 failed to start. Check node2.log for details.${NC}"
    cat node2.log
    exit 1
fi

echo -e "${GREEN}✅ Node 2 started successfully (PID: $NODE2_PID)${NC}"

echo ""
echo -e "${GREEN}🎉 Both nodes are running successfully!${NC}"
echo ""

echo -e "${BLUE}🌐 Web UI Access:${NC}"
echo -e "  📡 ${GREEN}Node 1 (Streamer):${NC} http://localhost:$NODE1_WEB_PORT"
echo -e "  👥 ${GREEN}Node 2 (Viewer):${NC}   http://localhost:$NODE2_WEB_PORT"
echo ""

echo -e "${BLUE}🎥 How to Test Streaming:${NC}"
echo -e "  ${YELLOW}1. Streamer Setup:${NC}"
echo "     • Open http://localhost:$NODE1_WEB_PORT in your browser"
echo "     • Allow camera/microphone access when prompted"
echo "     • Click 'Start Stream' button"
echo "     • Configure quality settings in the side panel"
echo ""
echo -e "  ${YELLOW}2. Viewer Setup:${NC}"
echo "     • Open http://localhost:$NODE2_WEB_PORT in another browser/tab"
echo "     • The viewer should automatically discover available streams"
echo "     • Click on a stream to start watching"
echo ""

echo -e "${BLUE}🔧 API Endpoints:${NC}"
echo "  • Node 1 Health: http://localhost:$NODE1_WEB_PORT/health"
echo "  • Node 2 Health: http://localhost:$NODE2_WEB_PORT/health"
echo "  • Node 1 Info: http://localhost:$NODE1_WEB_PORT/api/node/info"
echo "  • Node 2 Info: http://localhost:$NODE2_WEB_PORT/api/node/info"
echo ""

echo -e "${BLUE}📊 Real-time Monitoring:${NC}"
echo "  • Stream statistics update live in the web UI"
echo "  • Network metrics show actual WebRTC performance"
echo "  • Blockchain integration displays node status"
echo "  • STUN servers show decentralized connectivity"
echo ""

echo -e "${BLUE}🔍 Log Files:${NC}"
echo "  • Node 1 logs: ${GREEN}node1.log${NC}"
echo "  • Node 2 logs: ${GREEN}node2.log${NC}"
echo ""

echo -e "${YELLOW}📋 Demo Features:${NC}"
echo "  ✅ Real WebRTC streaming between nodes"
echo "  ✅ Decentralized STUN server infrastructure"
echo "  ✅ Blockchain-integrated streaming platform"
echo "  ✅ Beautiful web UI for both streamer and viewer"
echo "  ✅ Real-time statistics and monitoring"
echo "  ✅ P2P network discovery and connection"
echo ""

echo -e "${GREEN}🎊 Ready for live streaming! Press Ctrl+C to stop both nodes.${NC}"
echo ""

# Monitor both nodes
echo -e "${BLUE}📈 Node Status Monitor:${NC}"
while true; do
    sleep 10
    
    # Check if both nodes are still running
    if ! kill -0 $NODE1_PID 2>/dev/null; then
        echo -e "${RED}⚠️  Node 1 stopped unexpectedly${NC}"
        break
    fi
    
    if ! kill -0 $NODE2_PID 2>/dev/null; then
        echo -e "${RED}⚠️  Node 2 stopped unexpectedly${NC}"
        break
    fi
    
    # Show brief status
    echo -e "${GREEN}✅ Both nodes running...${NC} ($(date '+%H:%M:%S'))"
done

# If we get here, one of the nodes stopped
echo -e "${YELLOW}🔄 Monitoring stopped. Check log files for details.${NC}"
cleanup

