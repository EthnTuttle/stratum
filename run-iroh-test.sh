#!/usr/bin/env bash

# Script to run Pool and Translator Proxy with Iroh networking
# This demonstrates the SV2 stack with Iroh transport layer

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

# Check if binaries are built
check_binaries() {
    print_header "Checking Binaries"

    if [ ! -f "target/debug/pool" ]; then
        print_warning "Pool binary not found. Building..."
        cargo build --manifest-path=roles/pool/Cargo.toml
    else
        print_success "Pool binary found"
    fi

    if [ ! -f "target/debug/translator" ]; then
        print_warning "Translator binary not found. Building..."
        cargo build --manifest-path=roles/translator/Cargo.toml
    else
        print_success "Translator binary found"
    fi
}

# Create example config files
create_configs() {
    print_header "Creating Configuration Files"

    # Pool config with Iroh
    cat > iroh-pool-config.toml <<'EOF'
# SRI Pool Configuration with Iroh Network Support
authority_public_key = "9auqWEzQDVyd2oe1JVGFLMLHZtCo2FFqZwtKA5gd9xbuEu7PH72"
authority_secret_key = "mkDLTBBRxdBv998612qipDYoTK3YUrqLe8uWw7gu3iXbSrn2n"
cert_validity_sec = 3600
listen_address = "0.0.0.0:34254"

# Coinbase Configuration
coinbase_reward_script = "addr(tb1qa0sm0hxzj0x25rh8gw5xlzwlsfvvyz8u96w3p8)"
server_id = 1
pool_signature = "Stratum V2 SRI Pool (Iroh Test)"

# Template Provider (must be running separately)
tp_address = "127.0.0.1:48336"
shares_per_minute = 1.0
share_batch_size = 10

# Iroh Network Configuration
iroh_listen_address = "iroh://auto"

[iroh_node_config]
secret_key_path = "./pool-iroh-secret.key"
relay_mode = "default"
alpn_protocol = "mining"
EOF

    # Translator config (will be updated with pool's Node ID)
    cat > iroh-tproxy-config.toml <<'EOF'
# Translator Proxy Configuration with Iroh
downstream_address = "0.0.0.0"
downstream_port = 34255
max_supported_version = 2
min_supported_version = 2
downstream_extranonce2_size = 4
user_identity = "iroh_test_miner"
aggregate_channels = true

[downstream_difficulty_config]
min_individual_miner_hashrate = 10_000_000_000_000.0
shares_per_minute = 6.0
enable_vardiff = true

[iroh_node_config]
secret_key_path = "./tproxy-iroh-secret.key"
relay_mode = "default"
alpn_protocol = "mining"

[[upstreams]]
address = "127.0.0.1"
port = 34254
iroh_node_id = "REPLACE_WITH_POOL_NODE_ID"
authority_pubkey = "9auqWEzQDVyd2oe1JVGFLMLHZtCo2FFqZwtKA5gd9xbuEu7PH72"
EOF

    print_success "Configuration files created"
}

# Start the pool and extract its Node ID
start_pool() {
    print_header "Starting Pool with Iroh"

    print_info "Starting pool in background..."
    print_warning "Make sure you have a Template Provider running on 127.0.0.1:48336"
    print_warning "Otherwise the pool will fail to start!"
    echo ""

    # Start pool and capture output
    cargo run --manifest-path=roles/pool/Cargo.toml -- -c iroh-pool-config.toml > pool.log 2>&1 &
    POOL_PID=$!

    print_info "Pool starting (PID: $POOL_PID)..."
    print_info "Waiting for Pool to initialize and print Node ID..."

    # Wait for Node ID to appear in logs (max 30 seconds)
    for i in {1..30}; do
        if grep -q "Node ID:" pool.log 2>/dev/null; then
            break
        fi
        sleep 1
        echo -n "."
    done
    echo ""

    # Extract Node ID from logs
    if grep -q "Node ID:" pool.log 2>/dev/null; then
        POOL_NODE_ID=$(grep "Node ID:" pool.log | tail -1 | sed 's/.*Node ID: \([^ ]*\).*/\1/')
        print_success "Pool started successfully!"
        echo ""
        print_header "POOL NODE ID"
        echo -e "${GREEN}${POOL_NODE_ID}${NC}"
        echo ""

        # Save for later use
        echo "$POOL_NODE_ID" > .pool_node_id
        echo "$POOL_PID" > .pool_pid

        return 0
    else
        print_error "Failed to get Pool Node ID"
        print_info "Pool log output:"
        tail -20 pool.log
        kill $POOL_PID 2>/dev/null || true
        return 1
    fi
}

# Update translator config with pool's Node ID
update_tproxy_config() {
    POOL_NODE_ID="$1"
    print_header "Updating Translator Config"

    print_info "Setting Pool Node ID: $POOL_NODE_ID"

    # Update the config file with actual Node ID
    sed -i "s/REPLACE_WITH_POOL_NODE_ID/$POOL_NODE_ID/" iroh-tproxy-config.toml

    print_success "Translator config updated"
}

# Start the translator proxy
start_tproxy() {
    print_header "Starting Translator Proxy with Iroh"

    print_info "Starting translator in background..."

    cargo run --manifest-path=roles/translator/Cargo.toml -- -c iroh-tproxy-config.toml > tproxy.log 2>&1 &
    TPROXY_PID=$!

    echo "$TPROXY_PID" > .tproxy_pid

    print_info "Translator starting (PID: $TPROXY_PID)..."
    sleep 3

    if ps -p $TPROXY_PID > /dev/null; then
        print_success "Translator started successfully!"
    else
        print_error "Translator failed to start"
        print_info "Translator log output:"
        tail -20 tproxy.log
        return 1
    fi
}

# Show status and logs
show_status() {
    print_header "Status"

    if [ -f .pool_pid ]; then
        POOL_PID=$(cat .pool_pid)
        if ps -p $POOL_PID > /dev/null; then
            print_success "Pool is running (PID: $POOL_PID)"
            if [ -f .pool_node_id ]; then
                echo -e "  ${BLUE}Node ID:${NC} $(cat .pool_node_id)"
            fi
        else
            print_error "Pool is not running"
        fi
    fi

    if [ -f .tproxy_pid ]; then
        TPROXY_PID=$(cat .tproxy_pid)
        if ps -p $TPROXY_PID > /dev/null; then
            print_success "Translator is running (PID: $TPROXY_PID)"
            echo -e "  ${BLUE}Listening on:${NC} 0.0.0.0:34255 (SV1)"
        else
            print_error "Translator is not running"
        fi
    fi

    echo ""
    print_info "To view logs in real-time:"
    echo "  Pool:       tail -f pool.log"
    echo "  Translator: tail -f tproxy.log"
}

# Stop all services
stop_all() {
    print_header "Stopping Services"

    if [ -f .tproxy_pid ]; then
        TPROXY_PID=$(cat .tproxy_pid)
        if ps -p $TPROXY_PID > /dev/null 2>&1; then
            print_info "Stopping Translator (PID: $TPROXY_PID)..."
            kill $TPROXY_PID
            sleep 1
            print_success "Translator stopped"
        fi
        rm -f .tproxy_pid
    fi

    if [ -f .pool_pid ]; then
        POOL_PID=$(cat .pool_pid)
        if ps -p $POOL_PID > /dev/null 2>&1; then
            print_info "Stopping Pool (PID: $POOL_PID)..."
            kill $POOL_PID
            sleep 1
            print_success "Pool stopped"
        fi
        rm -f .pool_pid
    fi

    rm -f .pool_node_id
}

# Cleanup function
cleanup() {
    echo ""
    print_warning "Received interrupt signal, cleaning up..."
    stop_all
    exit 0
}

# Main function
main() {
    # Handle Ctrl+C
    trap cleanup INT TERM

    case "${1:-start}" in
        start)
            print_header "Stratum V2 Iroh Network Test"

            check_binaries
            create_configs

            # Start pool first
            if ! start_pool; then
                print_error "Failed to start pool. Exiting."
                exit 1
            fi

            # Update translator config with pool's Node ID
            POOL_NODE_ID=$(cat .pool_node_id)
            update_tproxy_config "$POOL_NODE_ID"

            # Start translator
            if ! start_tproxy; then
                print_error "Failed to start translator. Cleaning up..."
                stop_all
                exit 1
            fi

            show_status

            print_header "Setup Complete!"
            echo ""
            print_success "Services are running!"
            echo ""
            echo "Connect your SV1 miner to: 0.0.0.0:34255"
            echo ""
            echo "Commands:"
            echo "  $0 status  - Show status"
            echo "  $0 logs    - Show logs"
            echo "  $0 stop    - Stop all services"
            echo ""
            ;;

        stop)
            stop_all
            ;;

        status)
            show_status
            ;;

        logs)
            print_header "Recent Logs"
            echo ""
            echo -e "${BLUE}=== Pool Logs ===${NC}"
            tail -30 pool.log 2>/dev/null || echo "No pool logs found"
            echo ""
            echo -e "${BLUE}=== Translator Logs ===${NC}"
            tail -30 tproxy.log 2>/dev/null || echo "No translator logs found"
            ;;

        *)
            echo "Usage: $0 {start|stop|status|logs}"
            echo ""
            echo "Commands:"
            echo "  start  - Start pool and translator with Iroh"
            echo "  stop   - Stop all services"
            echo "  status - Show status of services"
            echo "  logs   - Show recent logs"
            exit 1
            ;;
    esac
}

main "$@"
