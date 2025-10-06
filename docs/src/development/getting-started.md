# Getting Started

Learn how to build, test, and contribute to the Stratum V2 Reference Implementation.

> **Note**: This page is a work in progress.

## Prerequisites

- Rust 1.75.0 or later
- Bitcoin Core (for Template Provider)
- Basic understanding of Bitcoin mining

## Building SRI

```bash
# Clone the repository
git clone https://github.com/stratum-mining/stratum.git
cd stratum

# Build all crates
cargo build --release

# Run tests
cargo test
```

## Running Components

### Pool
```bash
cargo run --release --bin pool
```

### Mining Proxy
```bash
cargo run --release --bin mining-proxy
```

### Translator
```bash
cargo run --release --bin translator
```

### Template Provider
```bash
cargo run --release --bin template-provider
```

## Configuration

Configuration files use TOML format. See example configurations in:
- `roles/pool/config.toml.example`
- `roles/mining-proxy/config.toml.example`
- etc.

---

For detailed setup instructions, visit:
[Official Getting Started Guide](https://stratumprotocol.org/blog/getting-started/)
