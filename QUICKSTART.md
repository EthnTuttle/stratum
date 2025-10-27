# Quick Start: SV2 Runtime Stats Dashboard

Get up and running with live mining metrics visualization in 2 minutes.

## What You'll See

```
╔═══════════════════════════════════════════════════╗
║   SV2 Mining Dashboard - Share Accounting         ║
╚═══════════════════════════════════════════════════╝

┌─────────────────────── 📊 Share Accounting ─────────────────────────┐
│ DS ID  CH ID  Type      Shares     Work Sum       Seq #     Blocks  │
│ 1      1      standard   42         150000.50     42        1       │
│ 1      2      extended   128        425123.75     128       0       │
│ 2      1      standard   96         298444.25     96        0       │
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

## Step 1: Start the Pool

```bash
# Terminal 1
cd /home/ethan/code/stratum-runtime-stats/roles/pool
cargo run -- -c your-config.toml
```

Look for this line in the output:
```
📊 Prometheus metrics server listening on http://0.0.0.0:9090/metrics
```

## Step 2: Launch the Dashboard

```bash
# Terminal 2 (new window/tab)
cd /home/ethan/code/stratum-runtime-stats/roles/ratzilla-dashboard
cargo run
```

That's it! The dashboard will connect automatically and start displaying metrics.

## Keyboard Controls

- `q` - Quit the dashboard
- `r` - Force refresh (manual update)

The dashboard auto-refreshes every 5 seconds by default.

## What the Metrics Mean

### Share Accounting Table

| Column | Meaning |
|--------|---------|
| **DS ID** | Downstream ID (each connected miner/translator) |
| **CH ID** | Channel ID (mining channel within that downstream) |
| **Type** | Channel type (standard=basic, extended=custom work) |
| **Shares** | Total accepted shares on this channel |
| **Work Sum** | Cumulative difficulty (sum of all share work) |
| **Seq #** | Last share's sequence number |
| **Blocks** | Blocks found on this channel |

### Connection Stats

- **Downstream Connections** - How many miners/translators are connected
- **Template Provider** - Is the block template source connected?
- **Channel counts** - How many of each channel type are active

## Troubleshooting

### "Connection refused"
- Pool isn't running or metrics server didn't start
- Check pool logs for the "Prometheus metrics server" message
- Verify port 9090 isn't blocked

### Empty tables
- No miners connected yet - wait for miners to join
- No shares submitted yet - give it a few seconds
- Metrics endpoint might be wrong - check URL with `curl http://localhost:9090/metrics`

### Dashboard not updating
- Press `r` to force refresh
- Check network connectivity
- Verify pool is still running

## Advanced Usage

### Custom Metrics URL

```bash
cargo run -- --metrics-url http://192.168.1.100:9090/metrics
```

### Faster Refresh Rate

```bash
cargo run -- --refresh-interval 2  # Update every 2 seconds
```

### View Raw Metrics

```bash
curl http://localhost:9090/metrics
```

## What's Happening Behind the Scenes

1. **Pool** collects share data from all connected miners
2. **Metrics Exporter** converts this into Prometheus format
3. **HTTP Server** exposes metrics at `/metrics` endpoint (port 9090)
4. **Dashboard** fetches and parses metrics every 5 seconds
5. **Ratatui** renders the beautiful terminal UI

## Next Steps

Once you have the dashboard running:

1. **Connect miners** to see share data populate
2. **Watch in real-time** as shares come in
3. **Monitor performance** - see which channels are most active
4. **Track work** - cumulative difficulty shows mining progress

## Files Created

```
roles/
├── metrics-exporter/       # Prometheus metrics library
│   ├── src/
│   │   ├── types.rs        # Data structures
│   │   ├── metrics.rs      # Prometheus conversion
│   │   ├── server.rs       # HTTP endpoint
│   │   └── lib.rs
│   └── README.md
│
├── pool/
│   └── src/lib/
│       └── metrics_integration.rs  # Pool → Metrics integration
│
└── ratzilla-dashboard/     # Terminal UI
    ├── src/
    │   ├── main.rs         # App entry point
    │   ├── app.rs          # State management
    │   ├── metrics_parser.rs  # Parse Prometheus format
    │   ├── ui.rs           # Ratatui rendering
    │   └── lib.rs
    └── README.md
```

## Resources

- Main docs: `RUNTIME_STATS.md`
- Dashboard docs: `roles/ratzilla-dashboard/README.md`
- Metrics docs: `roles/metrics-exporter/README.md`

---

**Enjoy monitoring your SV2 mining operation!** 🚀⛏️
