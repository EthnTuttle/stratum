# SV2 Metrics Exporter

Prometheus metrics exporter for Stratum V2 roles (pool, translator).

## Features

- **Share Accounting Metrics**: Track accepted shares, work sum, sequence numbers per channel
- **Connection Metrics**: Monitor active downstreams and channels by type
- **Template Provider Metrics**: Template provider connection status
- **Prometheus Format**: Standard `/metrics` endpoint for scraping
- **Trait-based Design**: Easy integration with any SV2 role

## Metrics Exposed

### Pool Metrics

```prometheus
# Share accounting (per downstream/channel)
sv2_pool_shares_accepted_total{downstream_id="1", channel_id="1", channel_type="standard"} 42
sv2_pool_share_work_sum{downstream_id="1", channel_id="1", channel_type="standard"} 150000.5
sv2_pool_share_sequence_number{downstream_id="1", channel_id="1", channel_type="standard"} 42
sv2_pool_blocks_found_total{downstream_id="1", channel_id="1", channel_type="standard"} 1

# Connections
sv2_pool_downstream_connections_active 3
sv2_pool_channels_active{channel_type="standard"} 5
sv2_pool_channels_active{channel_type="extended"} 2

# Template Provider
sv2_pool_template_provider_connected 1
```

### Translator Metrics (planned)

```prometheus
sv2_translator_upstream_connected{pool_address="pool.example.com:3333"} 1
sv2_translator_downstream_connections_active 10
```

## Usage

### As a Library

```rust
use metrics_exporter::{MetricsServer, MetricsProvider};

// Implement MetricsProvider for your role
impl MetricsProvider for MyRole {
    type Snapshot = PoolMetricsSnapshot;

    fn get_metrics_snapshot(&self) -> Self::Snapshot {
        // Collect your metrics
    }
}

// Start the metrics server
let server = MetricsServer::new("0.0.0.0:9090".to_string());
let exporter = server.exporter();

// Spawn server
tokio::spawn(async move {
    server.start().await.unwrap();
});

// Periodically update metrics
loop {
    let snapshot = my_role.get_metrics_snapshot();
    exporter.update_from_pool_snapshot(&snapshot);
    tokio::time::sleep(Duration::from_secs(5)).await;
}
```

### Integration with Pool

The pool role automatically starts a metrics server on port 9090:

```bash
# Start the pool
cargo run --bin pool_sv2 -- -c config.toml

# Access metrics
curl http://localhost:9090/metrics
```

## Prometheus Configuration

Add to your `prometheus.yml`:

```yaml
scrape_configs:
  - job_name: 'sv2_pool'
    scrape_interval: 15s
    static_configs:
      - targets: ['localhost:9090']
```

## Visualization with Ratzilla

Ratzilla provides a terminal-style web UI for metrics visualization:

```bash
# Coming soon: ratzilla dashboard
cargo run --bin ratzilla-dashboard -- --metrics-url http://localhost:9090/metrics
```

## Architecture

```
┌─────────────────┐
│   SV2 Pool      │
│  (implements    │
│ MetricsProvider)│
└────────┬────────┘
         │ get_metrics_snapshot()
         │ every 5s
         ▼
┌─────────────────┐
│ PrometheusExpor │
│ ter             │
│ - Converts      │
│   snapshots to  │
│   Prometheus    │
│   format        │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ MetricsServer   │
│ HTTP :9090      │
│ GET /metrics    │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Prometheus     │
│  (scraper)      │
└─────────────────┘
```

## Development

```bash
# Build
cargo build --package metrics-exporter

# Test
cargo test --package metrics-exporter

# Run pool with metrics
cargo run --bin pool_sv2
```

## TODO

- [ ] Implement translator metrics
- [ ] Track blocks found per channel
- [ ] Track template count
- [ ] Add ratzilla dashboard
- [ ] Add Grafana dashboard examples
- [ ] Add alerting examples
