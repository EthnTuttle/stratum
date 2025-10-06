//! # Network Helpers for Stratum V2
//!
//! This crate provides network connection abstractions for Stratum V2 roles,
//! supporting both TCP and Iroh transports with optional Noise encryption.
//!
//! ## Transport Options
//!
//! ### TCP Connections
//! - [`noise_connection::Connection`] - TCP with Noise encryption
//! - [`plain_connection::PlainConnection`] - TCP without encryption
//!
//! ### Iroh Connections
//! - [`IrohConnection`] - Iroh transport with Noise encryption
//! - [`PlainIrohConnection`] - Iroh transport without encryption
//!
//! ## Usage Examples
//!
//! ### Using Iroh Node Manager (Server-side)
//!
//! ```rust,ignore
//! use network_helpers_sv2::{IrohNodeManager, IrohNodeConfig};
//! use codec_sv2::{HandshakeRole, noise_sv2::Responder};
//! use secp256k1::Keypair;
//!
//! # async fn example() -> Result<(), network_helpers_sv2::Error> {
//! // Configure and initialize an Iroh node
//! let config = IrohNodeConfig {
//!     secret_key_path: Some("./iroh_key".into()),
//!     alpn: b"stratum-v2".to_vec(),
//!     ..Default::default()
//! };
//!
//! let manager = IrohNodeManager::new(config).await?;
//! println!("Node ID: {}", manager.node_id());
//!
//! // Accept incoming connections (returns same interface as TCP Connection)
//! // Note: You need a keypair for the Responder role
//! // let keypair = Keypair::new(...);
//! // let (receiver, sender) = manager.accept_connection(
//! //     HandshakeRole::Responder(Box::new(Responder::new(keypair, 31536000)))
//! // ).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Using Iroh Connection (Client-side)
//!
//! ```rust,ignore
//! use network_helpers_sv2::{IrohNodeManager, IrohNodeConfig, IrohConnection};
//! use codec_sv2::{HandshakeRole, noise_sv2::Initiator};
//! use iroh::NodeId;
//!
//! # async fn example() -> Result<(), network_helpers_sv2::Error> {
//! // Initialize your Iroh node
//! let config = IrohNodeConfig::default();
//! let manager = IrohNodeManager::new(config).await?;
//!
//! // Connect to a remote peer by NodeId (you would get this from the server)
//! let peer_id: NodeId = unimplemented!("Get from server");
//! let connection = manager.endpoint()
//!     .connect(peer_id, b"stratum-v2")
//!     .await?;
//!
//! // Open a bidirectional stream
//! let (send_stream, recv_stream) = connection.open_bi().await?;
//!
//! // Create connection (returns same interface as TCP Connection)
//! let (receiver, sender) = IrohConnection::new(
//!     send_stream,
//!     recv_stream,
//!     HandshakeRole::Initiator(Box::new(Initiator::new(None)))
//! ).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Transport Selection
//!
//! Both TCP and Iroh connections provide the same `(Receiver, Sender)` interface,
//! allowing roles to work transparently with either transport:
//!
//! - **TCP**: Direct connection, requires publicly accessible IP or port forwarding
//! - **Iroh**: NAT traversal via STUN/TURN, peer-to-peer with relay fallback
//!
//! Choose TCP for:
//! - Data center deployments with direct connectivity
//! - Low-latency requirements on local networks
//! - Simple network topology
//!
//! Choose Iroh for:
//! - NAT traversal without port forwarding
//! - Peer-to-peer connections across different networks
//! - Resilient connections with automatic relay fallback
//! - Mobile or residential network deployments
//!
//! ## Configuration
//!
//! ### Iroh Node Configuration
//!
//! ```rust
//! use network_helpers_sv2::{IrohNodeConfig, RelayMode};
//! use std::net::{Ipv4Addr, SocketAddrV4};
//!
//! let config = IrohNodeConfig {
//!     // Persistent identity (optional)
//!     secret_key_path: Some("./my_node_key".into()),
//!
//!     // Relay configuration for NAT traversal
//!     relay_mode: Some(RelayMode::Default),
//!
//!     // Optional bind addresses
//!     bind_addr_v4: Some(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)),
//!
//!     // ALPN protocol identifier
//!     alpn: b"stratum-v2".to_vec(),
//!
//!     ..Default::default()
//! };
//! ```

pub mod noise_connection;
pub mod noise_stream;
pub mod noise_iroh_stream;
pub mod noise_iroh_connection;
pub mod plain_connection;
pub mod plain_iroh_connection;
#[cfg(feature = "sv1")]
pub mod sv1_connection;
pub mod iroh_node;

use async_channel::{RecvError, SendError};
use codec_sv2::Error as CodecError;
use iroh::{NodeId, endpoint::{ConnectionError, ReadError, WriteError}};

pub use codec_sv2;

// Export Iroh connection types for public use
pub use noise_iroh_connection::IrohConnection;
pub use plain_iroh_connection::PlainIrohConnection;

// Export Iroh node management types
pub use iroh_node::{IrohNodeConfig, IrohNodeManager};

// Re-export commonly used Iroh types for convenience
pub use iroh::{NodeId as IrohNodeId, RelayMode};

#[derive(Debug)]
pub enum Error {
    HandshakeRemoteInvalidMessage,
    CodecError(CodecError),
    RecvError,
    SendError,
    // This means that a socket that was supposed to be opened have been closed, likley by the
    // peer
    SocketClosed,
    /// Iroh node initialization failed
    IrohNodeInitialization(Box<dyn std::error::Error + Send + Sync>),
    /// Connection to Iroh peer failed
    IrohConnectionFailed {
        peer_id: NodeId,
        source: ConnectionError,
    },
    /// Peer discovery failed
    IrohPeerDiscoveryFailed {
        peer_id: NodeId,
        reason: String,
    },
    /// Iroh connection error
    IrohConnectionError(ConnectionError),
    /// Iroh stream read error
    IrohReadError(ReadError),
    /// Iroh stream write error
    IrohWriteError(WriteError),
}

impl From<CodecError> for Error {
    fn from(e: CodecError) -> Self {
        Error::CodecError(e)
    }
}
impl From<RecvError> for Error {
    fn from(_: RecvError) -> Self {
        Error::RecvError
    }
}
impl<T> From<SendError<T>> for Error {
    fn from(_: SendError<T>) -> Self {
        Error::SendError
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for Error {
    fn from(e: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Error::IrohNodeInitialization(e)
    }
}

impl From<ConnectionError> for Error {
    fn from(e: ConnectionError) -> Self {
        Error::IrohConnectionError(e)
    }
}

impl From<ReadError> for Error {
    fn from(e: ReadError) -> Self {
        Error::IrohReadError(e)
    }
}

impl From<WriteError> for Error {
    fn from(e: WriteError) -> Self {
        Error::IrohWriteError(e)
    }
}
