# Runtime Stats Integration for SV2

This worktree implements Prometheus metrics export and visualization for Stratum V2 roles.

## Project Goal

Provide real-time runtime statistics and monitoring for SV2 mining infrastructure with:
- **Share accounting metrics** (accepted shares, work sum, sequence numbers)
- **Connection monitoring** (downstreams, channels, upstream pools)
- **Prometheus export** (standard `/metrics` endpoint)
- **Ratzilla dashboard** (terminal-style web UI)

## Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                       SV2 Roles                               │
├──────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌─────────────┐              ┌──────────────────┐          │
│  │    Pool     │              │   Translator     │          │
│  │             │              │                  │          │
│  │ Implements  │              │  Implements      │          │
│  │ Metrics     │              │  Metrics         │          │
│  │ Provider    │              │  Provider        │          │
│  └──────┬──────┘              └────────┬─────────┘          │
│         │                               │                    │
│         │ get_metrics_snapshot()        │                    │
│         │ every 5s                      │                    │
│         ▼                               ▼                    │
│  ┌──────────────────────────────────────────────────┐       │
│  │         metrics-exporter crate                    │       │
│  │                                                    │       │
│  │  ┌─────────────────┐  ┌──────────────────────┐  │       │
│  │  │Prometheus       │  │  MetricsServer       │  │       │
│  │  │Exporter         │  │  HTTP :9090          │  │       │
│  │  │                 │  │  /metrics endpoint   │  │       │
│  │  └─────────────────┘  └──────────────────────┘  │       │
│  └────────────────┬──────────────────────────────┘          │
│                   │                                          │
└───────────────────┼──────────────────────────────────────────┘
                    │
                    │ HTTP /metrics (Prometheus format)
                    ▼
┌──────────────────────────────────────────────────────────────┐
│                    Monitoring Stack                           │
├──────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌──────────────┐         ┌──────────────────┐             │
│  │  Prometheus  │         │    Ratzilla      │             │
│  │   Scraper    │         │   Dashboard      │             │
│  │              │         │  (Terminal UI)   │             │
│  │  Scrapes     │         │                  │             │
│  │  every 15s   │         │  Fetches metrics │             │
│  └──────┬───────┘         └────────┬─────────┘             │
│         │                           │                        │
│         │ Time series DB            │ Real-time display     │
│         ▼                           ▼                        │
│  ┌──────────────┐         ┌──────────────────┐             │
│  │   Grafana    │         │    Browser       │             │
│  │  Dashboards  │         │  localhost:8080  │             │
│  └──────────────┘         └──────────────────┘             │
│                                                               │
└──────────────────────────────────────────────────────────────┘
```

## Implementation Status

### ✅ Completed

1. **metrics-exporter crate** (`roles/metrics-exporter/`)
   - Prometheus exporter with share accounting metrics
   - HTTP server exposing `/metrics` endpoint
   - Trait-based design (MetricsProvider)
   - Full test coverage (7/7 tests passing)

2. **Pool integration** (`roles/pool/`)
   - Implemented MetricsProvider for ChannelManager
   - Automatic metrics server startup on port 9090
   - Collects share accounting from all channels
   - Updates metrics every 5 seconds

3. **Ratzilla dashboard** (`roles/ratzilla-dashboard/`)
   - Terminal-style UI using ratatui
   - Prometheus metrics parser
   - Real-time share accounting display
   - Connection status monitoring
   - Auto-refresh every 5 seconds
   - Keyboard controls (q=quit, r=refresh)

### 🚧 Pending

4. **Translator integration** (`roles/translator/`)
   - Need to implement MetricsProvider
   - Track upstream pool connections
   - Track downstream SV1 miner connections

## Key Metrics

### Share Accounting (Most Important!)

These metrics track the core mining activity:

```prometheus
# Total shares accepted per channel
sv2_pool_shares_accepted_total{downstream_id="1", channel_id="1", channel_type="standard"} 42

# Cumulative work (difficulty sum)
sv2_pool_share_work_sum{downstream_id="1", channel_id="1", channel_type="standard"} 150000.5

# Last share sequence number
sv2_pool_share_sequence_number{downstream_id="1", channel_id="1", channel_type="standard"} 42

# Blocks found
sv2_pool_blocks_found_total{downstream_id="1", channel_id="1", channel_type="standard"} 1
```

## Quick Start

### 1. Start Pool with Metrics

```bash
cd /home/ethan/code/stratum-runtime-stats/roles/pool
cargo run -- -c config.toml
```

The pool automatically starts a Prometheus metrics server on `http://0.0.0.0:9090`

### 2. Launch the Dashboard

In a new terminal:

```bash
cd /home/ethan/code/stratum-runtime-stats/roles/ratzilla-dashboard
cargo run

# Or with custom URL
cargo run -- --metrics-url http://localhost:9090/metrics --refresh-interval 3
```

The dashboard will display:
- 📊 Share accounting table (shares, work sum, sequence numbers)
- 🔌 Connection status (downstreams, template provider)
- ⚡ Real-time updates every 5 seconds
- ⌨️  Keyboard controls: `q` to quit, `r` to refresh

### 3. View Raw Metrics (Optional)

```bash
# Raw Prometheus format
curl http://localhost:9090/metrics

# Or open in browser
open http://localhost:9090
```

### 4. Set Up Prometheus (Optional)

Create `prometheus.yml`:

```yaml
scrape_configs:
  - job_name: 'sv2_pool'
    scrape_interval: 15s
    static_configs:
      - targets: ['localhost:9090']
```

Run Prometheus:

```bash
docker run -p 9091:9090 -v $(pwd)/prometheus.yml:/etc/prometheus/prometheus.yml prom/prometheus
```

## Example Metrics Output

```
# HELP sv2_pool_shares_accepted_total Total accepted shares per downstream/channel
# TYPE sv2_pool_shares_accepted_total gauge
sv2_pool_shares_accepted_total{downstream_id="1",channel_id="1",channel_type="standard"} 42

# HELP sv2_pool_share_work_sum Cumulative share work (difficulty sum) per channel
# TYPE sv2_pool_share_work_sum gauge
sv2_pool_share_work_sum{downstream_id="1",channel_id="1",channel_type="standard"} 150000.5

# HELP sv2_pool_downstream_connections_active Number of active downstream connections
# TYPE sv2_pool_downstream_connections_active gauge
sv2_pool_downstream_connections_active 3

# HELP sv2_pool_channels_active Number of active channels by type
# TYPE sv2_pool_channels_active gauge
sv2_pool_channels_active{channel_type="standard"} 5
sv2_pool_channels_active{channel_type="extended"} 2

# HELP sv2_pool_template_provider_connected Template provider connection status
# TYPE sv2_pool_template_provider_connected gauge
sv2_pool_template_provider_connected 1
```

## Next Steps

### Phase 1: Testing (Current)
- [ ] Test metrics with live pool
- [ ] Connect actual downstream miners
- [ ] Verify share accounting metrics update correctly
- [ ] Test with multiple downstreams

### Phase 2: Translator Integration
- [ ] Implement MetricsProvider for translator
- [ ] Track upstream SV2 pool connection
- [ ] Track downstream SV1 miner connections
- [ ] Test with SV1 miners

### Phase 3: Ratzilla Dashboard
- [ ] Create ratzilla-based web UI
- [ ] Fetch metrics from Prometheus endpoint
- [ ] Render terminal-style tables and charts
- [ ] Show real-time share accounting
- [ ] Display connection status

### Phase 4: Enhanced Features
- [ ] Track blocks found per channel
- [ ] Track template reception count
- [ ] Add per-miner hashrate estimation
- [ ] Add alerting for stale connections
- [ ] Create Grafana dashboard templates

## Development

### Build All

```bash
cargo build --workspace
```

### Test Metrics Exporter

```bash
cargo test --package metrics-exporter
```

### Run Pool with Metrics

```bash
cargo run --bin pool_sv2
# Metrics available at http://localhost:9090/metrics
```

## Project Structure

```
roles/
├── metrics-exporter/     # Prometheus metrics library
│   ├── src/
│   │   ├── types.rs      # MetricsProvider trait, snapshot types
│   │   ├── metrics.rs    # PrometheusExporter
│   │   ├── server.rs     # HTTP /metrics endpoint
│   │   └── lib.rs
│   └── README.md
├── pool/
│   ├── src/lib/
│   │   ├── metrics_integration.rs  # MetricsProvider impl
│   │   └── mod.rs                   # Start metrics server
│   └── Cargo.toml        # Added metrics-exporter dependency
└── translator/           # TODO: Add metrics integration
```

## Resources

- [Prometheus Documentation](https://prometheus.io/docs/)
- [Ratzilla GitHub](https://github.com/orhun/ratzilla)
- [Stratum V2 Spec](https://github.com/stratum-mining/sv2-spec)

## Contributing

This is a development worktree. When ready:

1. Test thoroughly with live mining
2. Document any issues found
3. Create PR to main branch
4. Include example dashboards and configs
