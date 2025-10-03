# Requirements Document

## Introduction

This feature adds Iroh peer-to-peer networking support as an alternative transport layer for Stratum V2 connections by extending the network-helpers crate. Iroh provides content-addressed networking with built-in NAT traversal, making it ideal for decentralized mining scenarios where direct TCP connections may not be feasible due to firewalls, NATs, or network topology constraints.

The integration extends the existing network-helpers abstraction by adding IrohConnection alongside the existing Connection types (noise_connection, plain_connection). This approach maintains full backward compatibility while providing transport choice through connection constructor selection. All existing channel management logic in channels-sv2 remains unchanged since it operates on the standardized (Receiver, Sender) interface provided by all connection types.

## Requirements

### Requirement 1

**User Story:** As a mining pool operator, I want to use Iroh networking for Stratum V2 connections, so that miners behind NATs can connect without port forwarding or VPN setup.

#### Acceptance Criteria

1. WHEN a pool server uses Iroh-based connections for listening THEN it SHALL accept incoming connections from Iroh peers
2. WHEN a mining client connects via Iroh THEN the connection SHALL support all existing Stratum V2 message types
3. WHEN using Noise over Iroh THEN the system SHALL maintain the same security guarantees as Noise over TCP
4. IF Iroh connection establishment fails THEN the system SHALL provide clear error messages indicating the failure reason

### Requirement 2

**User Story:** As a mining client developer, I want to configure Iroh as a transport option, so that I can choose between TCP and Iroh based on network conditions.

#### Acceptance Criteria

1. WHEN configuring a client connection THEN the system SHALL support both TCP-based and Iroh-based connections through configuration
2. WHEN using Iroh transport THEN the client SHALL be able to discover and connect to pool servers by their Iroh node ID
3. WHEN using either transport THEN the system SHALL support both Noise-encrypted and plain connections
4. IF Iroh connection establishment fails THEN the client SHALL gracefully fall back to TCP transport when configured with fallback option

### Requirement 3

**User Story:** As a system administrator, I want Iroh integration to replace TCP at the transport layer, so that existing deployments continue working without modification and I can choose transport through configuration.

#### Acceptance Criteria

1. WHEN using existing TCP-based connections THEN the system SHALL work identically to current behavior
2. WHEN using Iroh-based connections THEN the system SHALL support both Noise-encrypted and plain variants
3. WHEN creating any connection type THEN it SHALL return the same (Receiver, Sender) interface regardless of underlying transport
4. IF Iroh dependencies are missing THEN the system SHALL provide clear compilation errors when attempting to use Iroh-based connections

### Requirement 4

**User Story:** As a mining software developer, I want consistent connection abstractions, so that I can write transport-agnostic code for channel management.

#### Acceptance Criteria

1. WHEN implementing channel logic THEN the code SHALL work identically regardless of underlying connection type
2. WHEN reading/writing messages THEN the API SHALL be the same for all connection types
3. WHEN handling connection events THEN the same message types SHALL be received from all connection types
4. IF transport-specific configuration is needed THEN it SHALL be isolated in the connection constructor parameters

### Requirement 5

**User Story:** As a network administrator, I want proper error handling and logging for Iroh connections, so that I can troubleshoot connectivity issues.

#### Acceptance Criteria

1. WHEN IrohConnection establishment fails THEN the system SHALL log detailed error information including peer ID and failure reason
2. WHEN IrohConnection is established THEN the system SHALL log successful connection with peer information
3. WHEN peer discovery fails THEN the system SHALL provide actionable error messages
4. IF Iroh node configuration is invalid THEN the system SHALL validate and report configuration errors at startup

### Requirement 6

**User Story:** As a mining pool operator, I want to accept both TCP and Iroh connections simultaneously, so that I can support clients using different transport methods without running separate services.

#### Acceptance Criteria

1. WHEN configuring a server THEN the system SHALL support listening with both TCP-based and Iroh-based connections concurrently
2. WHEN clients connect via different transports THEN the server SHALL handle all connections through the same channel management logic
3. WHEN managing multiple transport listeners THEN the system SHALL provide unified connection status and metrics
4. IF one transport fails THEN the other transport SHALL continue operating normally

### Requirement 7

**User Story:** As a mining pool operator, I want to configure Iroh node settings, so that I can optimize performance for my specific deployment scenario.

#### Acceptance Criteria

1. WHEN configuring Iroh transport THEN the system SHALL support custom relay servers and STUN servers
2. WHEN setting up Iroh node THEN the system SHALL allow configuration of storage location and network policies
3. WHEN running in production THEN the system SHALL support persistent node identity across restarts
4. IF configuration is invalid THEN the system SHALL validate settings and provide clear error messages

### Requirement 8

**User Story:** As a security-conscious mining operator, I want to use Noise encryption over Iroh connections, so that I maintain the same security guarantees as TCP+Noise connections.

#### Acceptance Criteria

1. WHEN using Noise over Iroh THEN the system SHALL provide the same encryption and authentication as Noise over TCP
2. WHEN establishing encrypted Iroh connections THEN the system SHALL perform the same handshake process as TCP connections
3. WHEN using NoiseIrohStream THEN it SHALL be interchangeable with NoiseTcpStream from the application perspective
4. IF Noise handshake fails over Iroh THEN the system SHALL provide the same error handling as TCP failures