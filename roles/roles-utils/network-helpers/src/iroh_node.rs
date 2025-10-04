use std::{path::PathBuf, sync::Arc};

use async_channel::{Receiver, Sender};
use codec_sv2::{
    binary_sv2::{Deserialize, GetSize, Serialize},
    StandardEitherFrame,
};
use iroh::{NodeId, Endpoint};
use tracing::{debug, info};

use crate::Error;

/// Configuration for an Iroh node
#[derive(Debug, Clone)]
pub struct IrohNodeConfig {
    /// Optional storage path for the Iroh node data
    pub storage_path: Option<PathBuf>,
    /// List of relay servers for NAT traversal
    pub relay_servers: Vec<String>,
    /// List of STUN servers for NAT traversal
    pub stun_servers: Vec<String>,
    /// Optional bind port for the Iroh node
    pub bind_port: Option<u16>,
}

impl Default for IrohNodeConfig {
    fn default() -> Self {
        Self {
            storage_path: None,
            relay_servers: vec!["https://relay.iroh.computer".to_string()],
            stun_servers: vec!["stun.l.google.com:19302".to_string()],
            bind_port: None,
        }
    }
}

/// Manager for Iroh node lifecycle and operations
pub struct IrohNodeManager {
    endpoint: Arc<Endpoint>,
    config: IrohNodeConfig,
}

impl IrohNodeManager {
    /// Create a new IrohNodeManager with the given configuration
    pub async fn new(config: IrohNodeConfig) -> Result<Self, Error> {
        debug!("Initializing Iroh endpoint with config: {:?}", config);

        // Create endpoint with default configuration
        // TODO: Add proper relay and STUN server configuration in later tasks
        let endpoint = Endpoint::builder()
            .bind()
            .await
            .map_err(|e| {
                let boxed_error: Box<dyn std::error::Error + Send + Sync> = e.into();
                Error::IrohNodeInitialization(boxed_error)
            })?;

        info!("Iroh endpoint initialized with ID: {}", endpoint.node_id());

        Ok(Self {
            endpoint: Arc::new(endpoint),
            config,
        })
    }

    /// Get a reference to the underlying Iroh endpoint
    pub fn endpoint(&self) -> &Arc<Endpoint> {
        &self.endpoint
    }

    /// Get the node ID of this Iroh endpoint
    pub fn node_id(&self) -> NodeId {
        self.endpoint.node_id()
    }

    /// Get the configuration used by this node manager
    pub fn config(&self) -> &IrohNodeConfig {
        &self.config
    }

    /// Accept an incoming connection on the specified ALPN protocol
    /// This will be implemented in later tasks when we create the connection types
    pub async fn accept_connection<Message>(
        &self,
        _alpn: &[u8],
    ) -> Result<
        (
            Receiver<StandardEitherFrame<Message>>,
            Sender<StandardEitherFrame<Message>>,
        ),
        Error,
    >
    where
        Message: Serialize + Deserialize<'static> + GetSize + Send + 'static,
    {
        // This is a placeholder - will be implemented in task 5.2
        todo!("accept_connection will be implemented in task 5.2")
    }
}
