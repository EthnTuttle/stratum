# Iroh Network Integration Test

This directory contains configuration files and test instructions for testing Stratum V2 roles communicating via Iroh network with mDNS local discovery.

## Test Topology

```
SV1 Miner (CPU Miner)
         |
         | SV1 (TCP)
         v
   Translator -----(Iroh/mDNS)-----> Pool
                                       ^
                                       |
                                 (Iroh/mDNS)
                                       |
   JD Client -----(Iroh/mDNS)-----> JD Server
         |                             |
         +-----------------------------+
                  (Iroh/mDNS)
```

## Components

1. **Pool** (`pool-iroh.toml`): Mining pool accepting downstream connections via Iroh
2. **JD Server** (`jds-iroh.toml`): Job Declarator Server accepting JDC connections via Iroh
3. **JD Client** (`jdc-iroh.toml`): Job Declarator Client connecting to both Pool and JDS via Iroh
4. **Translator** (`translator-iroh.toml`): SV1-to-SV2 translator connecting to Pool via Iroh

## Prerequisites

1. Build all roles:
   ```bash
   cd /home/ethan/code/stratum
   cargo build --release
   ```

2. Start a Bitcoin Core regtest node:
   ```bash
   bitcoind -regtest -rpcuser=user -rpcpassword=password -rpcport=18443
   ```

3. Start Template Provider:
   ```bash
   cd roles/template-provider
   cargo run --release -- -c ../../test/config/tproxy-config-local-tp-example.toml
   ```

## Test Execution Steps

### Step 1: Start Pool and Capture NodeID

```bash
cd /home/ethan/code/stratum/roles/pool
cargo run --release -- -c ../../test/iroh-integration-test/pool-iroh.toml
```

Look for the log line:
```
Iroh node initialized with Node ID: <POOL_NODE_ID>
```

Copy the Pool's NodeID for use in later steps.

### Step 2: Start JD Server and Capture NodeID

```bash
cd /home/ethan/code/stratum/roles/jd-server
cargo run --release -- -c ../../test/iroh-integration-test/jds-iroh.toml
```

Look for the log line:
```
Iroh node initialized with Node ID: <JDS_NODE_ID>
```

Copy the JDS's NodeID for use in the next step.

### Step 3: Update JD Client Config with NodeIDs

Edit `jdc-iroh.toml` and uncomment/fill in the NodeID fields:

```toml
[[upstreams]]
authority_pubkey = "9auqWEzQDVyd2oe1JVGFLMLHZtCo2FFqZwtKA5gd9xbuEu7PH72"
pool_address = "127.0.0.1"
pool_port = 34254
pool_iroh_node_id = "<POOL_NODE_ID_FROM_STEP_1>"
jds_address = "127.0.0.1"
jds_port = 34255
jds_iroh_node_id = "<JDS_NODE_ID_FROM_STEP_2>"
```

### Step 4: Start JD Client

```bash
cd /home/ethan/code/stratum/roles/jd-client
cargo run --release -- -c ../../test/iroh-integration-test/jdc-iroh.toml
```

Look for log lines indicating successful Iroh connections:
```
Connecting to JD Server via Iroh at NodeID: <JDS_NODE_ID>
Iroh connection established with JD Server
Connecting to Pool via Iroh at NodeID: <POOL_NODE_ID>
Iroh connection established with Pool
```

### Step 5: Update Translator Config with Pool NodeID

Edit `translator-iroh.toml` and uncomment/fill in the Pool's NodeID:

```toml
[[upstreams]]
address = "127.0.0.1"
port = 34254
authority_pubkey = "9auqWEzQDVyd2oe1JVGFLMLHZtCo2FFqZwtKA5gd9xbuEu7PH72"
iroh_node_id = "<POOL_NODE_ID_FROM_STEP_1>"
```

### Step 6: Start Translator

```bash
cd /home/ethan/code/stratum/roles/translator
cargo run --release -- -c ../../test/iroh-integration-test/translator-iroh.toml
```

Look for log lines indicating successful Iroh connection:
```
Attempting Iroh connection to upstream 0 at NodeID: <POOL_NODE_ID>
Successfully connected to upstream via Iroh
```

### Step 7: Connect a SV1 Miner

Use cpuminer to connect to the Translator:

```bash
./minerd -a sha256d -o stratum+tcp://127.0.0.1:34255 -u user -p password -D
```

## Verification

### Check mDNS Discovery

All Iroh nodes should discover each other via mDNS on the local network. You should see log messages indicating:
- Peer discovery events
- Direct connections being established without needing explicit IP addresses

### Check Iroh Connections

Verify that all connections are using Iroh:
1. Pool should show Iroh connections from JDC and Translator
2. JDS should show Iroh connection from JDC
3. JDC should show Iroh connections to Pool and JDS
4. Translator should show Iroh connection to Pool

### Check Mining Flow

1. Miner connects to Translator via SV1
2. Translator converts to SV2 and communicates with Pool via Iroh
3. Pool provides work templates
4. Shares flow back through the chain
5. If JDC is involved, custom block templates are negotiated via Iroh

## Troubleshooting

### Connection Fallback to TCP

If you see "falling back to TCP" messages, it means the Iroh connection failed and the system is using TCP instead. Common causes:
- NodeID not configured or incorrect
- Network firewall blocking UDP/QUIC
- mDNS not working on the network

### mDNS Not Working

If peers aren't discovering each other:
- Check that multicast DNS is enabled on your network
- Verify firewall allows mDNS (UDP port 5353)
- Check that all services are on the same local network

### NAT Traversal Issues

If connections fail due to NAT:
- Verify `relay_mode = "default"` is set in all Iroh configs
- Check that services can reach Iroh's default relay servers
- Consider using a custom relay server for testing

## Configuration Files

- `pool-iroh.toml`: Pool with Iroh support, ALPN "mining"
- `jds-iroh.toml`: JD Server with Iroh support, ALPN "jd"
- `jdc-iroh.toml`: JD Client with Iroh support, ALPN "jd"
- `translator-iroh.toml`: Translator with Iroh support, ALPN "mining"

All configs use:
- Persistent secret keys in `/tmp/*-iroh-secret` for stable NodeIDs
- Default relay mode for NAT traversal
- Standard test keys and addresses
- mDNS enabled by default

## Expected Behavior

1. **Zero Configuration Discovery**: Services should find each other via mDNS without needing to know IP addresses in advance
2. **Encrypted Connections**: All Iroh connections use QUIC with encryption
3. **NAT Traversal**: Services should connect even if behind NAT using Iroh's relay infrastructure
4. **Fallback Support**: If Iroh fails, services fall back to TCP for reliability
5. **Protocol Separation**: Different ALPN protocols ensure proper routing (mining vs job_declarator)
