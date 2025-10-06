# Ethan's Unofficial Guide to Stratum V2

An in-depth architectural guide to the Stratum V2 Reference Implementation, using the C4 model and mdBook.

## 📖 View Online

Once deployed: https://stratum-mining.github.io/stratum/ (or your GitHub Pages URL)

## 🏗️ Building Locally

### Prerequisites
- Rust 1.75.0+
- mdbook: `cargo install mdbook`
- mdbook-mermaid: `cargo install mdbook-mermaid`

### Build & Serve
```bash
mdbook serve --open
```

The book will be available at http://localhost:3000

### Build Only
```bash
mdbook build
```

Output will be in `book/`

## 📚 Contents

- **Architecture**: C4 model diagrams (Context, Container, Component)
- **Protocols**: Deep dive into Stratum V2 protocols
- **Development**: Contributing and getting started

## 🤝 Contributing

This is an unofficial guide! Contributions, corrections, and improvements are welcome.

## ⚠️ Disclaimer

This is an **unofficial** community-created guide. For official documentation:
- [SRI Repository](https://github.com/stratum-mining/stratum)
- [SV2 Specification](https://github.com/stratum-mining/sv2-spec)
- [Stratum Protocol Website](https://stratumprotocol.org)

## 📝 License

This documentation follows the same license as the SRI project: Apache 2.0 or MIT, at your option.

---

Made with ❤️ for the Stratum V2 community
