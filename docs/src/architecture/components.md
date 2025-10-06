# Components Overview

This section provides detailed component-level architecture for each major role in the Stratum V2 Reference Implementation.

## Component Diagrams

The following pages break down the internal architecture of each role:

### Role Components

- **[Pool Role](./components/pool.md)** - Internal components of the pool server
- **[Mining Proxy](./components/proxy.md)** - Proxy aggregation components
- **[Translator Proxy](./components/translator.md)** - SV1 to SV2 translation components
- **[Template Provider](./components/template-provider.md)** - Block template management

## Common Component Patterns

Across all roles, you'll find these recurring patterns:

### Connection Manager
Handles incoming and outgoing network connections:
- TCP/TLS listener setup
- Connection state management
- Reconnection logic
- Connection pooling

### Message Handler
Processes protocol messages:
- Message parsing and validation
- Routing to appropriate handlers
- Response generation
- Error handling

### State Manager
Maintains application state:
- Active channels/sessions
- Mining jobs
- Share history
- Configuration

### Job Dispatcher
Distributes mining work:
- Job creation from templates
- Target difficulty calculation
- Work assignment to miners
- Job tracking

## Navigation

Choose a component to explore:

| Component | Description | Complexity |
|-----------|-------------|------------|
| [Pool](./components/pool.md) | Full pool server implementation | High |
| [Proxy](./components/proxy.md) | Mining aggregation proxy | Medium |
| [Translator](./components/translator.md) | SV1 ↔ SV2 bridge | Medium |
| [Template Provider](./components/template-provider.md) | Block template fetching | Low |

---

**Note**: These component diagrams reflect the current architecture as of the latest codebase analysis. Implementation details may evolve as the project develops.
