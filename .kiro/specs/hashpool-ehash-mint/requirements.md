# Requirements Document

## Introduction

This feature implements hashpool.dev, a system that integrates Cashu ecash minting and wallet functionality into the Stratum v2 reference implementation. The implementation involves refactoring the existing Pool role's share persistence into a Mint daemon (mool), and creating a new Wallet component in the TProxy role (walloxy). The system integrates with the CDK (Cashu Development Kit) from https://github.com/cashubtc/cdk to enable miners to earn ecash tokens (eHash) based on their mining shares, supporting both traditional sats/BTC units and custom "hash" units.

## Requirements

### Requirement 1: Mint Implementation

**User Story:** As a mining pool operator, I want to refactor the existing share persistence into a Mint daemon (mool) so that I can issue ecash tokens to miners based on their submitted shares.

#### Acceptance Criteria

1. WHEN the Pool role starts THEN the system SHALL initialize a Mint daemon thread using the CDK crate from https://github.com/cashubtc/cdk
2. WHEN a share persistence event occurs THEN the system SHALL convert it to a ShareEvent and send it to the Mint thread
3. WHEN the Mint receives a ShareEvent THEN it SHALL evaluate payment conditions for eHash issuance
4. WHEN payment conditions are met THEN the Mint SHALL mint ecash tokens in the appropriate unit type (sats/btc or hash)
5. WHEN minting operations occur THEN the system SHALL maintain proper accounting and audit trails
6. WHEN the Mint operates THEN it SHALL support both Bitcoin (sats/BTC) and hash unit types for token issuance

### Requirement 2: SV2 eHash Extension Implementation

**User Story:** As a miner using a translator proxy, I want the proxy to negotiate eHash extension support and include locking pubkey information in channel setup for external wallet redemption.

#### Acceptance Criteria

1. WHEN the TProxy role connects to a pool THEN it SHALL negotiate eHash extension support (0x0003) following SV2 extension protocols
2. WHEN eHash extension is supported THEN the TProxy SHALL include locking pubkey TLV fields in channel open messages
3. WHEN eHash extension is not supported THEN the TProxy SHALL continue normal operation without eHash functionality
4. WHEN opening mining channels THEN the TProxy SHALL include the configured locking pubkey as TLV field 0x0003|0x01
5. WHEN receiving SubmitSharesSuccess messages THEN the TProxy SHALL process any eHash-related TLV fields
6. WHEN extension negotiation fails THEN the system SHALL implement proper error handling and fallback mechanisms

### Requirement 3: ShareEvent Integration

**User Story:** As a system architect, I want a standardized ShareEvent system that enables communication between mining components and ecash operations.

#### Acceptance Criteria

1. WHEN share persistence occurs in the Pool THEN the system SHALL generate ShareEvent messages
2. WHEN ShareEvents are created THEN they SHALL contain all necessary data for mint payment evaluation
3. WHEN ShareEvents are transmitted THEN the system SHALL ensure reliable delivery to the Mint
4. WHEN the TProxy receives SubmitSharesSuccess THEN it SHALL generate corresponding ShareEvents for wallet operations
5. WHEN ShareEvents are processed THEN the system SHALL rely on existing Stratum v2 share deduplication and validation logic

### Requirement 4: CDK Integration

**User Story:** As a developer, I want to integrate the CDK crate from the cashubtc/cdk repository to provide robust Cashu ecash functionality.

#### Acceptance Criteria

1. WHEN the system initializes THEN it SHALL use the CDK crate from https://github.com/cashubtc/cdk
2. WHEN CDK operations are performed THEN the system SHALL handle all CDK-specific error conditions using the existing Status system from the Stratum v2 roles
3. WHEN mint operations occur THEN the system SHALL comply with Cashu protocol specifications
4. WHEN wallet operations occur THEN the system SHALL maintain compatibility with standard Cashu wallets
5. WHEN CDK dependencies are updated THEN the system SHALL maintain backward compatibility with existing tokens

### Requirement 5: Configuration and Deployment

**User Story:** As a system administrator, I want configurable deployment options that integrate CDK mint and wallet configurations with existing Stratum v2 TOML configuration structures.

#### Acceptance Criteria

1. WHEN deploying the Pool role THEN the system SHALL merge CDK mint configuration options with existing Stratum v2 TOML configuration
2. WHEN deploying the TProxy role THEN the system SHALL merge CDK wallet configuration options with existing Stratum v2 TOML configuration
3. WHEN deploying the JDC role THEN the system SHALL merge CDK mint proxy configuration options with existing Stratum v2 TOML configuration
4. WHEN configuring ecash operations THEN the system SHALL maintain compatibility with existing Stratum v2 configuration patterns
5. WHEN configuration changes occur THEN the system SHALL validate both Stratum v2 and CDK settings and provide clear error messages

### Requirement 6: Error Handling and Reliability

**User Story:** As a mining operation manager, I want robust error handling to ensure continuous mining operations even when ecash components experience issues.

#### Acceptance Criteria

1. WHEN Mint operations fail THEN the Pool role SHALL continue normal mining operations
2. WHEN Wallet operations fail THEN the TProxy role SHALL continue normal translation operations
3. WHEN network connectivity to the mint is lost THEN the system SHALL queue operations for retry
4. WHEN CDK operations encounter errors THEN the system SHALL log detailed error information
5. WHEN recovery from failures occurs THEN the system SHALL resume operations without data loss

### Requirement 7: Unified Mint Implementation

**User Story:** As a developer, I want a single Mint implementation that works identically in both Pool and JDC roles since the mint logic is independent of share relay behavior.

#### Acceptance Criteria

1. WHEN implementing Mint functionality THEN the system SHALL create a single Mint implementation that works in both Pool and JDC contexts
2. WHEN the Pool role uses Mint THEN it SHALL instantiate the same Mint implementation
3. WHEN the JDC role uses Mint THEN it SHALL instantiate the same Mint implementation
4. WHEN Mint operates THEN the existing Stratum v2 role logic SHALL handle all share routing and relay responsibilities
5. WHEN the Mint implementation is updated THEN both Pool and JDC deployments SHALL automatically benefit from the changes

### Requirement 8: External Wallet Integration

**User Story:** As a mining operation manager, I want external Cashu wallets to be able to redeem eHash tokens using the locking pubkeys provided by TProxy.

#### Acceptance Criteria

1. WHEN external wallets connect to the mint THEN they SHALL be able to query quotes by locking pubkey
2. WHEN TProxy submits shares with locking pubkeys THEN the mint SHALL create PAID quotes associated with those pubkeys
3. WHEN external wallets query for quotes THEN they SHALL receive all PAID quotes matching their locking pubkeys
4. WHEN external wallets redeem tokens THEN the mint SHALL process redemptions using standard Cashu protocols
5. WHEN redemption occurs THEN the system SHALL maintain audit trails for all operations

### Requirement 9: eHash Keyset Lifecycle Management

**User Story:** As a mining pool operator, I want automated keyset lifecycle management that transitions eHash keysets through different states based on mining events.

#### Acceptance Criteria

1. WHEN the Mint receives a ShareAccountingEvent with block_found=true THEN it SHALL query the Template Provider for block reward details and transition the active keyset to QUANTIFYING state
2. WHEN the Mint detects a BOLT12 payment via LDK integration THEN it SHALL verify the payment is for mining rewards and transition the active keyset to QUANTIFYING state
3. WHEN a keyset transitions to QUANTIFYING state THEN the Mint SHALL create a new ACTIVE keyset first to ensure continuous eHash minting capability
4. WHEN quantification is complete THEN the Mint SHALL automatically transition the keyset to PAYOUT state with the calculated conversion rate
5. WHEN a keyset is in PAYOUT state THEN external wallets SHALL be able to swap eHash tokens for sats at the determined rate
6. WHEN all eHash tokens are redeemed or payout period expires THEN the Mint SHALL transition the keyset to EXPIRED state 

### Requirement 10: Monitoring and Observability

**User Story:** As a system operator, I want comprehensive monitoring of ecash operations to ensure proper system health and performance.

#### Acceptance Criteria

1. WHEN ecash operations occur THEN the system SHALL emit appropriate metrics and logs
2. WHEN mint operations are performed THEN the system SHALL track issuance rates and volumes
3. WHEN wallet operations are performed THEN the system SHALL track redemption success rates
4. WHEN keyset lifecycle events occur THEN the system SHALL log state transitions and timing
5. WHEN system health checks run THEN the system SHALL report on ecash component status
6. WHEN performance issues occur THEN the system SHALL provide diagnostic information for troubleshooting