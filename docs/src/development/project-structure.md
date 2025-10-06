# Project Structure

Understanding the organization of the SRI codebase.

> **Note**: This page is a work in progress.

## Directory Layout

```
stratum/
├── benches/          # Benchmarks
├── protocols/        # Protocol library crates
│   ├── v1/          # Stratum V1
│   └── v2/          # Stratum V2
│       ├── binary-sv2/
│       ├── codec-sv2/
│       ├── framing-sv2/
│       ├── noise-sv2/
│       └── subprotocols/
│           ├── common-messages/
│           ├── job-negotiation/
│           ├── mining/
│           └── template-distribution/
├── roles/           # Application roles
│   ├── pool/
│   ├── mining-proxy/
│   ├── translator/
│   ├── template-provider/
│   └── roles-utils/
│       └── network-helpers/
├── utils/           # Utility crates
│   ├── rpc-client/
│   └── ...
├── examples/        # Example implementations
└── test/           # Integration tests
```

## Crate Organization

### Protocols Layer
Low-level protocol implementation:
- **Beta stability**: Core protocol mostly stable
- **Library crates**: Used by roles
- **Focus**: Message encoding, framing, encryption

### Roles Layer
Application binaries:
- **Alpha stability**: Still evolving
- **Binary crates**: Deployable applications
- **Focus**: Business logic, connection management

### Utils Layer
Shared utilities:
- RPC clients
- Configuration helpers
- Common functionality

---

More details coming soon!
