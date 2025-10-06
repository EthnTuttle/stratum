# Contributing Guide

How to contribute to the Stratum V2 Reference Implementation.

> **Note**: This page is a work in progress.

## Getting Started

1. Read the [official CONTRIBUTING.md](https://github.com/stratum-mining/stratum/blob/main/CONTRIBUTING.md)
2. Join the [Discord community](https://discord.gg/fsEW23wFYs)
3. Look for [good first issues](https://github.com/stratum-mining/stratum/labels/good%20first%20issue)

## Development Workflow

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `cargo test`
5. Run formatter: `cargo fmt`
6. Run clippy: `cargo clippy`
7. Submit a pull request

## Code Style

- Follow Rust standard conventions
- Run `cargo fmt` before committing
- Address `cargo clippy` warnings
- Add tests for new functionality
- Document public APIs

## Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture
```

## Documentation

- Use rustdoc comments for public APIs
- Add examples where helpful
- Update this guide when adding features!

## Community

- Discord: [SV2 Community](https://discord.gg/fsEW23wFYs)
- GitHub: [Issues](https://github.com/stratum-mining/stratum/issues)
- Website: [stratumprotocol.org](https://stratumprotocol.org)

---

Thank you for contributing to SRI! 🦀
