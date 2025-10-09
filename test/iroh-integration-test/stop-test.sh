#!/bin/bash
# Stop all Iroh integration test services

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PID_FILE="$SCRIPT_DIR/test.pids"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

if [ ! -f "$PID_FILE" ]; then
    echo -e "${RED}Error: No test PIDs found. Are services running?${NC}"
    exit 1
fi

# Source the PID file
source "$PID_FILE"

echo -e "${YELLOW}Stopping all test services...${NC}"

# Function to stop a process gracefully
stop_process() {
    local pid=$1
    local name=$2

    if ps -p $pid > /dev/null 2>&1; then
        echo "Stopping $name (PID $pid)..."
        kill $pid 2>/dev/null || true

        # Wait for process to stop (max 5 seconds)
        for i in {1..5}; do
            if ! ps -p $pid > /dev/null 2>&1; then
                echo -e "${GREEN}$name stopped${NC}"
                return 0
            fi
            sleep 1
        done

        # Force kill if still running
        if ps -p $pid > /dev/null 2>&1; then
            echo -e "${YELLOW}Force killing $name...${NC}"
            kill -9 $pid 2>/dev/null || true
        fi
    else
        echo "$name (PID $pid) not running"
    fi
}

# Stop all services
stop_process $TRANSLATOR_PID "Translator"
stop_process $JDC_PID "JD Client"
stop_process $JDS_PID "JD Server"
stop_process $POOL_PID "Pool"

# Restore config backups
echo -e "\n${YELLOW}Restoring config backups...${NC}"
if [ -f "$SCRIPT_DIR/jdc-iroh.toml.bak" ]; then
    mv "$SCRIPT_DIR/jdc-iroh.toml.bak" "$SCRIPT_DIR/jdc-iroh.toml"
    echo "JDC config restored"
fi
if [ -f "$SCRIPT_DIR/translator-iroh.toml.bak" ]; then
    mv "$SCRIPT_DIR/translator-iroh.toml.bak" "$SCRIPT_DIR/translator-iroh.toml"
    echo "Translator config restored"
fi

# Clean up PID file
rm -f "$PID_FILE"

echo -e "\n${GREEN}All test services stopped${NC}"
