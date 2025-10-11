#!/usr/bin/env bash

# Script to run Mining Device with Iroh networking
# This demonstrates connecting a mining device to a pool via Iroh

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

print_header() {
    echo ""
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}========================================${NC}"
    echo ""
}

print_info() {
    echo -e "${BLUE}→ $1${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

# Check if binary is built
check_binary() {
    print_header "Checking Mining Device Binary"

    if [ ! -f "target/debug/mining-device" ]; then
        print_warning "Mining device binary not found. Building..."
        cargo build --manifest-path=roles/test-utils/mining-device/Cargo.toml
    else
        print_success "Mining device binary found"
    fi
}

# Get pool node ID
get_pool_node_id() {
    if [ -f .pool_node_id ]; then
        POOL_NODE_ID=$(cat .pool_node_id)
        print_success "Found Pool Node ID: $POOL_NODE_ID"
        return 0
    else
        print_error "Pool Node ID not found!"
        print_info "Make sure the pool is running with Iroh enabled."
        print_info "Run './run-iroh-test.sh start' first to start the pool."
        return 1
    fi
}

# Start mining device with TCP
start_tcp() {
    print_header "Starting Mining Device (TCP Mode)"

    POOL_ADDRESS="${1:-127.0.0.1:34254}"
    HANDICAP="${2:-1000}"

    print_info "Configuration:"
    echo "  Pool Address: $POOL_ADDRESS"
    echo "  Handicap: $HANDICAP (microseconds between hashes)"
    echo ""

    print_info "Starting mining device..."
    print_warning "Press Ctrl+C to stop"
    echo ""

    cargo run --manifest-path=roles/test-utils/mining-device/Cargo.toml -- \
        --address-pool "$POOL_ADDRESS" \
        --handicap "$HANDICAP"
}

# Start mining device with Iroh
start_iroh() {
    print_header "Starting Mining Device (Iroh Mode)"

    # Get pool node ID
    if ! get_pool_node_id; then
        exit 1
    fi

    POOL_ADDRESS="${1:-127.0.0.1:34254}"
    HANDICAP="${2:-1000}"
    SECRET_KEY_PATH="./mining-device-iroh-secret.key"

    print_info "Configuration:"
    echo "  Pool Address: $POOL_ADDRESS (fallback, not used for Iroh)"
    echo "  Pool Node ID: $POOL_NODE_ID"
    echo "  Handicap: $HANDICAP (microseconds between hashes)"
    echo "  Secret Key: $SECRET_KEY_PATH"
    echo ""

    print_info "Starting mining device with Iroh..."
    print_warning "Press Ctrl+C to stop"
    echo ""

    cargo run --manifest-path=roles/test-utils/mining-device/Cargo.toml -- \
        --address-pool "$POOL_ADDRESS" \
        --pool-iroh-node-id "$POOL_NODE_ID" \
        --iroh-secret-key-path "$SECRET_KEY_PATH" \
        --handicap "$HANDICAP"
}

# Show usage
show_usage() {
    cat <<EOF
Mining Device Iroh Test Runner

Usage: $0 <mode> [pool_address] [handicap]

Modes:
  tcp    - Connect via TCP (default: 127.0.0.1:34254)
  iroh   - Connect via Iroh network (requires pool Node ID)

Arguments:
  pool_address  - Pool address (default: 127.0.0.1:34254)
  handicap      - Microseconds between hashes (default: 1000)
                  Lower = faster mining, higher CPU usage
                  Higher = slower mining, lower CPU usage

Examples:
  # Connect via TCP to local pool
  $0 tcp

  # Connect via TCP to remote pool
  $0 tcp pool.example.com:34254

  # Connect via Iroh (requires pool running with Iroh)
  $0 iroh

  # Connect with custom handicap
  $0 iroh 127.0.0.1:34254 500

Advanced Options:
  The mining device supports many CLI options. To see all:
  cargo run --manifest-path=roles/test-utils/mining-device/Cargo.toml -- --help

Notes:
  - For Iroh mode, the pool must be running with Iroh enabled
  - The pool's Node ID is read from .pool_node_id file
  - Run './run-iroh-test.sh start' first to start the pool
  - Handicap affects mining speed: 1000 = reasonable CPU usage

EOF
}

# Main function
main() {
    MODE="${1:-tcp}"
    POOL_ADDRESS="${2:-127.0.0.1:34254}"
    HANDICAP="${3:-1000}"

    case "$MODE" in
        tcp)
            check_binary
            start_tcp "$POOL_ADDRESS" "$HANDICAP"
            ;;

        iroh)
            check_binary
            start_iroh "$POOL_ADDRESS" "$HANDICAP"
            ;;

        help|--help|-h)
            show_usage
            ;;

        *)
            print_error "Unknown mode: $MODE"
            echo ""
            show_usage
            exit 1
            ;;
    esac
}

main "$@"
