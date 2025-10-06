# Connection Flow

This page describes the connection lifecycle and message flows in Stratum V2.

> **Note**: This page is a work in progress. Detailed flows will be added as the documentation evolves.

## Standard Miner Connection

```mermaid
sequenceDiagram
    participant M as Miner
    participant P as Pool

    M->>P: TCP Connect
    M->>P: Noise Handshake
    M->>P: SetupConnection
    P->>M: SetupConnection.Success
    M->>P: OpenStandardMiningChannel
    P->>M: OpenStandardMiningChannel.Success
    P->>M: NewMiningJob
    M->>P: SubmitSharesStandard
    P->>M: SetNewPrevHash (on new block)
```

## Topics to be covered:

- Initial connection & handshake
- Channel opening
- Job reception
- Share submission
- Difficulty adjustment
- Reconnection handling
- Error scenarios

---

More details coming soon!
