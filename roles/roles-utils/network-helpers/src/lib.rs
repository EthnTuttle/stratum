pub mod noise_connection;
pub mod noise_stream;
pub mod plain_connection;
#[cfg(feature = "sv1")]
pub mod sv1_connection;
pub mod iroh_node;

use async_channel::{RecvError, SendError};
use codec_sv2::Error as CodecError;
use iroh::{NodeId, endpoint::{ConnectionError, ReadError, WriteError}};

pub use codec_sv2;

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
