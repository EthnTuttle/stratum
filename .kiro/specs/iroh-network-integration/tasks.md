# Implementation Plan

- [x] 1. Add Iroh dependencies to network-helpers
  - Add Iroh dependencies to network-helpers Cargo.toml as regular dependencies
  - Create IrohNodeConfig and IrohNodeManager structures
  - Add Iroh-specific error types to network-helpers Error enum
  - _Requirements: 3.1, 3.2, 7.1_

- [x] 2. Implement NoiseIrohStream (equivalent to NoiseTcpStream)
  - [x] 2.1 Create NoiseIrohStream struct with Iroh transport
    - Implement NoiseIrohReadHalf using iroh::endpoint::RecvStream
    - Implement NoiseIrohWriteHalf using iroh::endpoint::SendStream
    - Replace TcpStream usage with Iroh BiStream connections
    - _Requirements: 8.1, 8.2, 8.3_

  - [x] 2.2 Implement Noise handshake over Iroh streams
    - Port existing Noise handshake logic from NoiseTcpStream
    - Ensure same handshake process works over Iroh transport
    - Maintain identical encryption and authentication guarantees
    - _Requirements: 8.1, 8.2, 8.4_

  - [x]* 2.3 Write unit tests for NoiseIrohStream
    - Test Noise handshake over Iroh connections
    - Test message encryption/decryption over Iroh
    - Test error handling for Iroh connection failures
    - Tests implemented with graceful skip when Iroh discovery not configured
    - _Requirements: 8.1, 8.3_

- [x] 3. Implement IrohConnection (equivalent to noise_connection.rs)
  - [x] 3.1 Create IrohConnection with same interface as Connection
    - Implement IrohConnection::new() returning (Receiver, Sender) interface
    - Use NoiseIrohStream instead of NoiseTcpStream internally
    - Follow same async task spawning pattern as noise_connection.rs
    - _Requirements: 3.2, 3.3, 4.1_

  - [x] 3.2 Implement spawn_reader and spawn_writer for Iroh
    - Port reader/writer task logic from noise_connection.rs
    - Adapt for NoiseIrohReadHalf and NoiseIrohWriteHalf
    - Maintain same message handling and error recovery
    - _Requirements: 4.2, 4.3_

  - [x]* 3.3 Write unit tests for IrohConnection
    - Test bidirectional message flow over Iroh
    - Test connection establishment and teardown
    - Test interface compatibility with existing Connection
    - _Requirements: 3.2, 4.1, 4.2_

- [x] 4. Implement PlainIrohConnection (equivalent to plain_connection.rs)
  - [x] 4.1 Create PlainIrohConnection for unencrypted Iroh
    - Implement PlainIrohConnection::new() with same interface
    - Use Iroh BiStream directly without Noise encryption
    - Follow same async task pattern as plain_connection.rs
    - _Requirements: 2.3, 3.2, 4.1_

  - [x] 4.2 Implement message handling over plain Iroh
    - Port message serialization/deserialization logic
    - Adapt StandardDecoder/Encoder for Iroh streams
    - Maintain same error handling and recovery
    - _Requirements: 4.2, 4.3_

  - [x]* 4.3 Write unit tests for PlainIrohConnection
    - Test plain message flow over Iroh
    - Test interface compatibility with PlainConnection
    - Test error handling for Iroh-specific failures
    - _Requirements: 2.3, 4.1_

- [x] 5. Implement IrohNodeManager for node lifecycle
  - [x] 5.1 Create IrohNodeManager for managing Iroh nodes
    - Implement node initialization with IrohNodeConfig
    - Add peer discovery and connection management
    - Implement persistent node identity across restarts
    - _Requirements: 7.2, 7.3, 7.4_

  - [x] 5.2 Add Iroh listening capabilities for servers
    - Implement server-side Iroh connection acceptance (no fallback needed)
    - Add ALPN protocol handling for Stratum V2
    - Support multiple concurrent Iroh connections
    - _Requirements: 1.1, 6.1, 6.2_

  - [x] 5.3 Add configuration validation and error handling
    - Validate relay servers and STUN server configurations
    - Implement clear error messages for configuration issues
    - Add logging for node initialization and peer discovery
    - _Requirements: 5.4, 7.4_

  - [x]* 5.4 Write unit tests for IrohNodeManager
    - Test node initialization with various configurations
    - Test peer discovery and connection establishment
    - Test error handling for invalid configurations
    - _Requirements: 7.1, 7.2, 7.4_

- [ ] 6. Update network-helpers lib.rs exports
  - [ ] 6.1 Add Iroh connection exports to lib.rs
    - Export IrohConnection and PlainIrohConnection
    - Export IrohNodeManager and IrohNodeConfig
    - Add Iroh-specific error types to Error enum
    - _Requirements: 3.1, 3.2_
  
  - [ ] 6.2 Update documentation and examples
    - Document transport selection between TCP and Iroh
    - Add usage examples for IrohConnection::new()
    - Document configuration options for Iroh nodes
    - _Requirements: 7.1, 7.2_
  
  - [ ]* 6.3 Write integration tests for network-helpers
    - Test all connection types return same interface
    - Test mixed transport scenarios (TCP + Iroh)
    - Test transport selection and fallback logic
    - _Requirements: 4.1, 4.3, 6.4_

- [ ] 7. Update role configurations to support Iroh
  - [ ] 7.1 Extend Pool configuration for Iroh transport
    - Add Iroh transport options to PoolConfig
    - Support dual listeners (TCP + Iroh) in pool configuration
    - Add Iroh node configuration to pool settings
    - _Requirements: 1.1, 6.1, 7.1_
  
  - [ ] 7.2 Extend JD Client configuration for Iroh transport
    - Add Iroh transport options to JobDeclaratorClientConfig
    - Support Iroh upstream connections in JD client
    - Add peer discovery configuration for JD client
    - _Requirements: 2.1, 2.2, 7.1_
  
  - [ ] 7.3 Extend Translator configuration for Iroh transport
    - Add Iroh transport options to translator configuration
    - Support Iroh upstream and downstream connections
    - Add transport selection logic to translator
    - _Requirements: 2.1, 2.4_
  
  - [ ]* 7.4 Write configuration tests for roles
    - Test configuration parsing for Iroh transport options
    - Test validation of Iroh-specific settings
    - Test fallback behavior when Iroh is unavailable
    - _Requirements: 2.4, 7.4_

- [ ] 8. Update roles to use Iroh connections
  - [ ] 8.1 Update Pool to support IrohConnection
    - Modify pool connection acceptance to use IrohConnection::new()
    - Support both Connection::new() and IrohConnection::new() based on config
    - Maintain same downstream handling logic for both transports
    - _Requirements: 1.1, 1.2, 6.2_
  
  - [ ] 8.2 Update JD Client to support IrohConnection
    - Modify upstream connections to use IrohConnection::new() when configured
    - Support peer discovery for connecting to JD servers
    - Maintain same channel management logic for both transports
    - _Requirements: 2.1, 2.2, 2.3_
  
  - [ ] 8.3 Update Translator to support IrohConnection
    - Support IrohConnection for both upstream and downstream connections
    - Add transport selection logic based on configuration
    - Maintain same translation logic for both transports
    - _Requirements: 2.1, 2.3_
  
  - [ ]* 8.4 Write integration tests for role updates
    - Test end-to-end Stratum V2 communication over Iroh
    - Test mixed transport scenarios (TCP clients + Iroh clients)
    - Test transport fallback and error handling
    - _Requirements: 1.2, 2.3, 6.2_

- [ ] 9. Add comprehensive error handling and logging
  - [ ] 9.1 Implement Iroh-specific error handling
    - Add detailed error messages for Iroh connection failures
    - Implement peer discovery error handling and logging
    - Add connection establishment logging with peer information
    - _Requirements: 1.4, 5.1, 5.2_
  
  - [ ] 9.2 Add client-side transport fallback mechanisms
    - Implement automatic fallback from Iroh to TCP for client connections when configured
    - Add exponential backoff for Iroh connection retries
    - Log transport selection and fallback events for client connections
    - _Requirements: 2.4, 5.3_
  
  - [ ]* 9.3 Write error handling tests
    - Test error propagation from Iroh to application layer
    - Test fallback behavior under various failure scenarios
    - Test logging output for different error conditions
    - _Requirements: 1.4, 2.4, 5.1_

- [ ] 10. Create configuration examples and documentation
  - [ ] 10.1 Create example configuration files
    - Add Iroh transport examples for all roles
    - Add dual transport examples (TCP + Iroh listeners)
    - Add peer discovery and relay server configuration examples
    - _Requirements: 7.1, 7.2_
  
  - [ ] 10.2 Update README files with Iroh integration guide
    - Document transport selection through connection constructors
    - Add network topology examples and NAT traversal use cases
    - Document Iroh node configuration and best practices
    - _Requirements: 1.1, 2.1, 7.1_
  
  - [ ]* 10.3 Write end-to-end integration tests
    - Test complete mining workflow over Iroh transport
    - Test NAT traversal scenarios with relay servers
    - Test performance comparison between TCP and Iroh transports
    - _Requirements: 1.1, 1.2, 1.3_