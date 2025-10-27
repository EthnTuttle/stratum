# Ratzilla Dashboard - SV2 Metrics Viewer

Terminal-style UI for visualizing Stratum V2 mining metrics in real-time.

## Features

- 📊 **Share Accounting Display** - View accepted shares, work sum, sequence numbers per channel
- 🔌 **Connection Monitoring** - Track active downstreams and channel types
- 🔄 **Auto-refresh** - Updates every 5 seconds (configurable)
- ⌨️  **Keyboard Controls** - Simple key bindings for navigation
- 🎨 **Terminal Style** - Beautiful retro-styled UI powered by ratatui

## Installation

```bash
# Build the dashboard
cd roles/ratzilla-dashboard
cargo build --release

# Or build from workspace root
cargo build --release --package ratzilla-dashboard
```

## Usage

### Basic Usage

```bash
# Connect to local metrics server (default)
cargo run --bin ratzilla-dashboard

# Or with explicit URL
cargo run --bin ratzilla-dashboard -- --metrics-url http://localhost:9090/metrics
```

### Command Line Options

```bash
ratzilla-dashboard [OPTIONS]

Options:
  -m, --metrics-url <URL>        Prometheus metrics URL [default: http://localhost:9090/metrics]
  -r, --refresh-interval <SECS>  Refresh interval in seconds [default: 5]
  -h, --help                      Print help
```

## Keyboard Controls

- `q` - Quit the dashboard
- `r` - Manual refresh (force update)

## Dashboard Layout

```
╔═══════════════════════════════════════════════════╗
║   SV2 Mining Dashboard - Share Accounting         ║
╚═══════════════════════════════════════════════════╝

┌─────────────────────── 📊 Share Accounting ─────────────────────────┐
│ DS ID  CH ID  Type      Shares     Work Sum       Seq #     Blocks  │
│ 1      1      standard   42         150000.50     42        1       │
│ 1      2      extended   128        425123.75     128       0       │
└────────────────────────────────────────────────────────────────────┘

┌─────────────────────── 🔌 Connections ──────────────────────────────┐
│ Downstream Connections: 3                                            │
│ Template Provider: ✓ Connected                                      │
│   standard channels: 5                                               │
│   extended channels: 2                                               │
└────────────────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────────────────┐
│ Press q to quit  •  r to refresh  •  Updates every 5s              │
└────────────────────────────────────────────────────────────────────┘
```

## Example: Full Workflow

### 1. Start the Pool with Metrics

```bash
# Terminal 1: Start the SV2 pool
cd roles/pool
cargo run -- -c config.toml

# Pool starts metrics server on http://0.0.0.0:9090
```

### 2. Launch the Dashboard

```bash
# Terminal 2: Start the dashboard
cd roles/ratzilla-dashboard
cargo run

# Dashboard connects to http://localhost:9090/metrics
```

### 3. View Live Mining Stats

The dashboard will display:
- Share submission rates per channel
- Cumulative work (difficulty sum)
- Connection status
- Real-time updates every 5 seconds

## Metrics Displayed

### Share Accounting Table

| Column | Description |
|--------|-------------|
| DS ID | Downstream ID (translator/miner connection) |
| CH ID | Channel ID (mining channel within downstream) |
| Type | Channel type (standard/extended/group) |
| Shares | Total accepted shares |
| Work Sum | Cumulative difficulty (share work sum) |
| Seq # | Last share sequence number |
| Blocks | Blocks found on this channel |

### Connection Stats

- **Downstream Connections** - Number of active connections
- **Template Provider** - Connection status to block template provider
- **Channel Counts** - Active channels by type

## Development

### Run in Development

```bash
cargo run --bin ratzilla-dashboard
```

### Run Tests

```bash
cargo test --package ratzilla-dashboard
```

### Project Structure

```
ratzilla-dashboard/
├── src/
│   ├── main.rs             # Entry point, terminal setup
│   ├── lib.rs              # Public API
│   ├── app.rs              # Application state, metrics fetching
│   ├── metrics_parser.rs   # Prometheus text format parser
│   └── ui.rs               # Ratatui UI rendering
└── README.md
```

## Troubleshooting

### Connection Refused

If you see "Failed to fetch metrics: Connection refused":
1. Ensure the pool is running with metrics enabled
2. Check that metrics server is on the correct port (default: 9090)
3. Verify firewall settings if connecting remotely

### No Data Displayed

If the dashboard shows empty tables:
1. Confirm miners are connected to the pool
2. Check that shares are being submitted
3. Wait a few seconds for the first metrics update

### Parse Errors

If you see "Failed to parse metrics":
1. Verify the metrics URL is correct
2. Check that the endpoint returns Prometheus text format
3. Try fetching manually: `curl http://localhost:9090/metrics`

## Technical Details

### Dependencies

- **ratatui** - Terminal UI framework
- **crossterm** - Terminal manipulation
- **reqwest** - HTTP client for fetching metrics
- **tokio** - Async runtime
- **serde/serde_json** - Serialization

### Metrics Format

Parses standard Prometheus text format:

```
sv2_pool_shares_accepted_total{downstream_id="1",channel_id="1",channel_type="standard"} 42
sv2_pool_share_work_sum{downstream_id="1",channel_id="1",channel_type="standard"} 150000.5
sv2_pool_downstream_connections_active 3
```

## Future Enhancements

- [ ] Graphical charts for share rate over time
- [ ] Per-miner hashrate estimation
- [ ] Alert notifications for issues
- [ ] Export metrics to CSV/JSON
- [ ] Multiple pool monitoring
- [ ] WebSocket support for real-time updates

## License

MIT OR Apache-2.0
