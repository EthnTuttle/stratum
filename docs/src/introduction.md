# Introduction

Welcome to **Ethan's Unofficial Guide to Stratum V2**! This guide provides an in-depth look at the architecture and implementation of the Stratum V2 Reference Implementation (SRI).

## What is Stratum V2?

[Stratum V2](https://stratumprotocol.org) is a next-generation Bitcoin mining protocol designed to enhance:

- **Efficiency**: Reduced bandwidth usage and faster share submission
- **Security**: Encrypted connections and authenticated messages
- **Flexibility**: Support for custom block template construction
- **Decentralization**: Miners can choose their own transactions

## What is SRI?

The Stratum V2 Reference Implementation is a fully open-source, Rust-based implementation of the Stratum V2 protocol. It provides:

1. **Protocol Primitives**: Low-level Rust library crates for building SV2 applications
2. **Role Implementations**: Ready-to-use implementations of pools, proxies, and translators
3. **Modular Architecture**: Clean separation between protocol handling and application logic

## Who is This Guide For?

This guide is intended for:

- Developers implementing Stratum V2 in mining software
- Pool operators deploying SV2 infrastructure
- Anyone wanting to understand the SV2 architecture and codebase
- Contributors to the SRI project

## How to Use This Guide

The guide is organized into three main sections:

### Architecture
Explore the system design using the C4 model:
- **Context**: How SRI fits into the Bitcoin mining ecosystem
- **Containers**: High-level modules and their interactions
- **Components**: Detailed component breakdowns for each role

### Protocols
Understand the Stratum V2 protocol implementation:
- Message types and encoding
- Connection establishment and lifecycle
- Job negotiation and template distribution

### Development
Practical information for working with the codebase:
- Project structure and organization
- Building and testing
- Contributing guidelines

## Disclaimer

This is an **unofficial guide** created to help understand the Stratum V2 Reference Implementation. For official documentation and specifications, please refer to:

- [Official SRI Repository](https://github.com/stratum-mining/stratum)
- [Stratum V2 Specification](https://github.com/stratum-mining/sv2-spec)
- [Stratum Protocol Website](https://stratumprotocol.org)

Let's dive in! 🦀⛏️
