# Message Types

Comprehensive reference of all Stratum V2 message types.

> **Note**: This page is a work in progress. Detailed message specifications will be added as the documentation evolves.

## Common Messages

### SetupConnection
Negotiate protocol version and features.

### SetupConnection.Success
Confirm successful connection setup.

### SetupConnection.Error
Report connection setup errors.

### ChannelEndpointChanged
Notify of upstream connection changes.

## Mining Protocol Messages

### OpenStandardMiningChannel
Request to open a standard mining channel.

### OpenStandardMiningChannel.Success
Confirm channel opened successfully.

### OpenExtendedMiningChannel
Request to open an extended mining channel (with job negotiation support).

### NewMiningJob
Distribute new mining work to miners.

### SetNewPrevHash
Update prevhash after new block found.

### SubmitSharesStandard
Submit found shares from standard channel.

### SubmitSharesExtended
Submit found shares from extended channel.

### SetTarget
Adjust mining difficulty target.

### Reconnect
Request miner to reconnect to different endpoint.

## Job Negotiation Messages

### AllocateMiningJobToken
Request token for custom job declaration.

### AllocateMiningJobToken.Success
Provide allocated job token.

### DeclareMiningJob
Declare custom mining job with specific transactions.

### DeclareMiningJob.Success
Confirm custom job accepted.

### DeclareMiningJob.Error
Report custom job rejected.

### IdentifyTransactions
Identify which transactions are known to pool.

### ProvideMissingTransactions
Provide transaction data for unknown transactions.

## Template Distribution Messages

### NewTemplate
Distribute new block template.

### SetNewPrevHash
Notify of new block (update prevhash).

### RequestTransactionData
Request detailed transaction data.

### RequestTransactionData.Success
Provide requested transaction data.

---

For detailed message field specifications, refer to:
- [SV2 Specification](https://github.com/stratum-mining/sv2-spec)
- SRI source code: `protocols/v2/subprotocols/*/src/`
