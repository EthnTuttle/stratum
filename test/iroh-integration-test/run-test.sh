#!/bin/bash
# Automated Iroh Integration Test Runner
# This script helps automate the test execution and NodeID discovery process

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== Iroh Network Integration Test ===${NC}\n"

# Function to extract NodeID from log output
extract_node_id() {
    local log_file=$1
    timeout 30 bash -c "
        tail -f '$log_file' 2>/dev/null | while IFS= read -r line; do
            if [[ \$line =~ 'Iroh node initialized with Node ID: '([a-zA-Z0-9]+) ]]; then
                echo \${BASH_REMATCH[1]}
                break
            fi
        done
    " || echo ""
}

# Check prerequisites
echo -e "${YELLOW}Checking prerequisites...${NC}"

if ! command -v bitcoind &> /dev/null; then
    echo -e "${RED}Error: bitcoind not found. Please install Bitcoin Core.${NC}"
    exit 1
fi

if ! pgrep -f "bitcoind.*regtest" > /dev/null; then
    echo -e "${YELLOW}Warning: Bitcoin Core regtest not running.${NC}"
    echo "Starting Bitcoin Core regtest..."
    bitcoind -regtest -rpcuser=user -rpcpassword=password -rpcport=18443 -daemon
    sleep 3
fi

echo -e "${GREEN}Prerequisites OK${NC}\n"

# Build all roles
echo -e "${YELLOW}Building all roles...${NC}"
cd "$PROJECT_ROOT"
cargo build --release --bin pool_sv2 --bin jd_server --bin jd_client --bin translator_sv2
echo -e "${GREEN}Build complete${NC}\n"

# Create log directory
LOG_DIR="$SCRIPT_DIR/logs"
mkdir -p "$LOG_DIR"
rm -f "$LOG_DIR"/*.log

# Start Pool
echo -e "${YELLOW}Starting Pool...${NC}"
cd "$PROJECT_ROOT/roles/pool"
cargo run --release -- -c "$SCRIPT_DIR/pool-iroh.toml" > "$LOG_DIR/pool.log" 2>&1 &
POOL_PID=$!
echo "Pool started with PID $POOL_PID"

# Wait for Pool to initialize and get NodeID
sleep 3
POOL_NODE_ID=$(grep -oP 'Iroh node initialized with Node ID: \K[a-zA-Z0-9]+' "$LOG_DIR/pool.log" | head -1)
if [ -z "$POOL_NODE_ID" ]; then
    echo -e "${RED}Error: Failed to get Pool NodeID${NC}"
    kill $POOL_PID 2>/dev/null || true
    exit 1
fi
echo -e "${GREEN}Pool NodeID: $POOL_NODE_ID${NC}\n"

# Start JD Server
echo -e "${YELLOW}Starting JD Server...${NC}"
cd "$PROJECT_ROOT/roles/jd-server"
cargo run --release -- -c "$SCRIPT_DIR/jds-iroh.toml" > "$LOG_DIR/jds.log" 2>&1 &
JDS_PID=$!
echo "JD Server started with PID $JDS_PID"

# Wait for JDS to initialize and get NodeID
sleep 3
JDS_NODE_ID=$(grep -oP 'Iroh node initialized with Node ID: \K[a-zA-Z0-9]+' "$LOG_DIR/jds.log" | head -1)
if [ -z "$JDS_NODE_ID" ]; then
    echo -e "${RED}Error: Failed to get JDS NodeID${NC}"
    kill $POOL_PID $JDS_PID 2>/dev/null || true
    exit 1
fi
echo -e "${GREEN}JDS NodeID: $JDS_NODE_ID${NC}\n"

# Update JDC config with NodeIDs
echo -e "${YELLOW}Updating JD Client config with discovered NodeIDs...${NC}"
JDC_CONFIG="$SCRIPT_DIR/jdc-iroh.toml"
cp "$JDC_CONFIG" "$JDC_CONFIG.bak"
sed -i "s/# pool_iroh_node_id = \"<POOL_NODE_ID_HERE>\"/pool_iroh_node_id = \"$POOL_NODE_ID\"/" "$JDC_CONFIG"
sed -i "s/# jds_iroh_node_id = \"<JDS_NODE_ID_HERE>\"/jds_iroh_node_id = \"$JDS_NODE_ID\"/" "$JDC_CONFIG"
echo -e "${GREEN}JDC config updated${NC}\n"

# Start JD Client
echo -e "${YELLOW}Starting JD Client...${NC}"
cd "$PROJECT_ROOT/roles/jd-client"
cargo run --release -- -c "$JDC_CONFIG" > "$LOG_DIR/jdc.log" 2>&1 &
JDC_PID=$!
echo "JD Client started with PID $JDC_PID"
sleep 3

# Check if JDC connected via Iroh
if grep -q "Iroh connection established" "$LOG_DIR/jdc.log"; then
    echo -e "${GREEN}JD Client successfully connected via Iroh${NC}\n"
else
    echo -e "${YELLOW}Warning: JD Client may have fallen back to TCP${NC}\n"
fi

# Update Translator config with Pool NodeID
echo -e "${YELLOW}Updating Translator config with Pool NodeID...${NC}"
TRANSLATOR_CONFIG="$SCRIPT_DIR/translator-iroh.toml"
cp "$TRANSLATOR_CONFIG" "$TRANSLATOR_CONFIG.bak"
sed -i "s/# iroh_node_id = \"<POOL_NODE_ID_HERE>\"/iroh_node_id = \"$POOL_NODE_ID\"/" "$TRANSLATOR_CONFIG"
echo -e "${GREEN}Translator config updated${NC}\n"

# Start Translator
echo -e "${YELLOW}Starting Translator...${NC}"
cd "$PROJECT_ROOT/roles/translator"
cargo run --release -- -c "$TRANSLATOR_CONFIG" > "$LOG_DIR/translator.log" 2>&1 &
TRANSLATOR_PID=$!
echo "Translator started with PID $TRANSLATOR_PID"
sleep 3

# Check if Translator connected via Iroh
if grep -q "Successfully connected to upstream via Iroh" "$LOG_DIR/translator.log"; then
    echo -e "${GREEN}Translator successfully connected via Iroh${NC}\n"
else
    echo -e "${YELLOW}Warning: Translator may have fallen back to TCP${NC}\n"
fi

# Summary
echo -e "${GREEN}=== Test Environment Running ===${NC}"
echo -e "Pool:       PID $POOL_PID, NodeID: $POOL_NODE_ID"
echo -e "JD Server:  PID $JDS_PID, NodeID: $JDS_NODE_ID"
echo -e "JD Client:  PID $JDC_PID"
echo -e "Translator: PID $TRANSLATOR_PID"
echo ""
echo -e "Log files in: $LOG_DIR"
echo -e "Pool log:       $LOG_DIR/pool.log"
echo -e "JDS log:        $LOG_DIR/jds.log"
echo -e "JDC log:        $LOG_DIR/jdc.log"
echo -e "Translator log: $LOG_DIR/translator.log"
echo ""
echo -e "${YELLOW}To connect a miner:${NC}"
echo -e "  ./minerd -a sha256d -o stratum+tcp://127.0.0.1:34255 -u user -p password -D"
echo ""
echo -e "${YELLOW}To stop all services:${NC}"
echo -e "  kill $POOL_PID $JDS_PID $JDC_PID $TRANSLATOR_PID"
echo ""
echo -e "${YELLOW}To monitor logs:${NC}"
echo -e "  tail -f $LOG_DIR/*.log"
echo ""

# Save PIDs for cleanup script
cat > "$SCRIPT_DIR/test.pids" <<EOF
POOL_PID=$POOL_PID
JDS_PID=$JDS_PID
JDC_PID=$JDC_PID
TRANSLATOR_PID=$TRANSLATOR_PID
EOF

echo -e "${GREEN}Test environment ready!${NC}"
echo -e "Press Ctrl+C to stop monitoring, or run './stop-test.sh' to stop all services."

# Monitor logs
trap "echo -e '\n${YELLOW}To stop services, run: ./stop-test.sh${NC}'; exit 0" INT
tail -f "$LOG_DIR"/*.log
