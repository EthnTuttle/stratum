use std::{net::{SocketAddrV4, SocketAddrV6}, path::PathBuf, sync::Arc};

use async_channel::{Receiver, Sender};
use codec_sv2::{
    binary_sv2::{Deserialize, GetSize, Serialize},
    StandardEitherFrame, HandshakeRole,
};
use iroh::{NodeId, Endpoint, RelayMode, SecretKey};
use tracing::{debug, info, warn};

use crate::{Error, noise_iroh_connection::IrohConnection};

/// Configuration for an Iroh node
#[derive(Debug, Clone)]
pub struct IrohNodeConfig {
    /// Optional path to store the secret key for persistent identity
    /// If provided, the node will save/load its secret key from this file
    pub secret_key_path: Option<PathBuf>,
    /// Optional explicit secret key (takes precedence over secret_key_path)
    /// If neither is provided, a new random key is generated each time
    pub secret_key: Option<SecretKey>,
    /// Relay mode configuration for NAT traversal
    /// Uses Iroh's default relay servers if not specified
    pub relay_mode: Option<RelayMode>,
    /// Optional IPv4 bind address (defaults to 0.0.0.0:0)
    pub bind_addr_v4: Option<SocketAddrV4>,
    /// Optional IPv6 bind address (defaults to [::]:0)
    pub bind_addr_v6: Option<SocketAddrV6>,
    /// ALPN protocol identifier for Stratum V2
    pub alpn: Vec<u8>,
}

impl Default for IrohNodeConfig {
    fn default() -> Self {
        Self {
            secret_key_path: None,
            secret_key: None,
            relay_mode: Some(RelayMode::Default),
            bind_addr_v4: None,
            bind_addr_v6: None,
            alpn: b"stratum-v2".to_vec(),
        }
    }
}

/// Manager for Iroh node lifecycle and operations
#[derive(Debug)]
pub struct IrohNodeManager {
    endpoint: Arc<Endpoint>,
    config: IrohNodeConfig,
}

impl IrohNodeManager {
    /// Create a new IrohNodeManager with the given configuration
    pub async fn new(config: IrohNodeConfig) -> Result<Self, Error> {
        debug!("Initializing Iroh endpoint with config: {:?}", config);

        // Validate configuration
        Self::validate_config(&config)?;

        // Determine the secret key to use (priority: explicit > file > generate)
        let secret_key = if let Some(key) = config.secret_key.clone() {
            debug!("Using explicitly provided secret key");
            key
        } else if let Some(path) = &config.secret_key_path {
            Self::load_or_create_secret_key(path)?
        } else {
            debug!("Generating new ephemeral secret key");
            SecretKey::generate(&mut rand::thread_rng())
        };

        // Build endpoint with configuration
        let mut builder = Endpoint::builder()
            .secret_key(secret_key)
            .alpns(vec![config.alpn.clone()]);

        // Configure relay mode
        if let Some(relay_mode) = &config.relay_mode {
            builder = builder.relay_mode(relay_mode.clone());
            debug!("Configured relay mode: {:?}", relay_mode);
        }

        // Configure bind addresses if specified
        if let Some(addr_v4) = config.bind_addr_v4 {
            builder = builder.bind_addr_v4(addr_v4);
            debug!("Configured IPv4 bind address: {}", addr_v4);
        }
        if let Some(addr_v6) = config.bind_addr_v6 {
            builder = builder.bind_addr_v6(addr_v6);
            debug!("Configured IPv6 bind address: {}", addr_v6);
        }

        // Add discovery (using default n0 DNS discovery)
        builder = builder.discovery_n0();

        // Bind the endpoint
        let endpoint = builder
            .bind()
            .await
            .map_err(|e| {
                let boxed_error: Box<dyn std::error::Error + Send + Sync> = e.into();
                Error::IrohNodeInitialization(boxed_error)
            })?;

        info!("Iroh endpoint initialized with Node ID: {}", endpoint.node_id());

        Ok(Self {
            endpoint: Arc::new(endpoint),
            config,
        })
    }

    /// Load secret key from file, or create a new one if it doesn't exist
    fn load_or_create_secret_key(path: &PathBuf) -> Result<SecretKey, Error> {
        use std::fs;
        use std::io::{Read, Write};

        if path.exists() {
            debug!("Loading secret key from: {:?}", path);
            let mut file = fs::File::open(path).map_err(|e| {
                Error::IrohNodeInitialization(format!("Failed to open secret key file: {}", e).into())
            })?;

            let mut key_bytes = Vec::new();
            file.read_to_end(&mut key_bytes).map_err(|e| {
                Error::IrohNodeInitialization(format!("Failed to read secret key file: {}", e).into())
            })?;

            // Try to parse as hex string first (common format)
            let key = if key_bytes.len() == 64 {
                // Assume hex encoding
                let key_str = String::from_utf8(key_bytes).map_err(|e| {
                    Error::IrohNodeInitialization(format!("Invalid secret key file format: {}", e).into())
                })?;
                let bytes = hex::decode(key_str.trim()).map_err(|e| {
                    Error::IrohNodeInitialization(format!("Failed to decode hex secret key: {}", e).into())
                })?;

                // Ensure we have exactly 32 bytes
                if bytes.len() != 32 {
                    return Err(Error::IrohNodeInitialization(
                        format!("Invalid secret key length: expected 32 bytes, got {}", bytes.len()).into()
                    ));
                }
                let bytes_array: [u8; 32] = bytes.try_into().unwrap();
                SecretKey::from_bytes(&bytes_array)
            } else if key_bytes.len() == 32 {
                // Try as raw bytes
                let bytes_array: [u8; 32] = key_bytes.try_into().map_err(|_| {
                    Error::IrohNodeInitialization("Invalid secret key length".into())
                })?;
                SecretKey::from_bytes(&bytes_array)
            } else {
                return Err(Error::IrohNodeInitialization(
                    format!("Invalid secret key file length: expected 32 or 64 bytes, got {}", key_bytes.len()).into()
                ));
            };

            info!("Loaded persistent secret key from: {:?}", path);
            Ok(key)
        } else {
            debug!("Secret key file not found, creating new one at: {:?}", path);
            let key = SecretKey::generate(&mut rand::thread_rng());

            // Create parent directory if needed
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(|e| {
                    Error::IrohNodeInitialization(format!("Failed to create directory: {}", e).into())
                })?;
            }

            // Save the key as hex-encoded string
            let key_bytes = key.to_bytes();
            let key_hex = hex::encode(&key_bytes);

            let mut file = fs::File::create(path).map_err(|e| {
                Error::IrohNodeInitialization(format!("Failed to create secret key file: {}", e).into())
            })?;

            file.write_all(key_hex.as_bytes()).map_err(|e| {
                Error::IrohNodeInitialization(format!("Failed to write secret key file: {}", e).into())
            })?;

            info!("Created and saved new secret key to: {:?}", path);
            Ok(key)
        }
    }

    /// Validate the configuration
    fn validate_config(config: &IrohNodeConfig) -> Result<(), Error> {
        // Validate ALPN is not empty
        if config.alpn.is_empty() {
            return Err(Error::IrohNodeInitialization(
                "ALPN protocol identifier cannot be empty".into()
            ));
        }

        // Warn if both secret_key and secret_key_path are provided
        if config.secret_key.is_some() && config.secret_key_path.is_some() {
            warn!("Both secret_key and secret_key_path are provided. Using explicit secret_key.");
        }

        Ok(())
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

    /// Accept an incoming connection on the configured ALPN protocol
    ///
    /// This method waits for an incoming Iroh connection and returns the
    /// same (Receiver, Sender) interface as the TCP-based Connection::new(),
    /// allowing for transparent usage in server roles.
    pub async fn accept_connection<Message>(
        &self,
        role: HandshakeRole,
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
        debug!("Waiting for incoming Iroh connection on ALPN: {:?}",
               String::from_utf8_lossy(&self.config.alpn));

        // Accept an incoming connection on the configured ALPN
        let connecting = self.endpoint
            .accept()
            .await
            .ok_or_else(|| {
                Error::IrohNodeInitialization("Endpoint closed, no more connections".into())
            })?;

        // Wait for the connection to be established
        let connection = connecting.await.map_err(|e| {
            Error::IrohConnectionError(e)
        })?;

        let remote_node_id = connection.remote_node_id().expect("Connection should have remote node ID");
        info!("Accepted Iroh connection from peer: {}", remote_node_id);

        // Accept a bidirectional stream from the remote peer
        let (send_stream, recv_stream) = connection.accept_bi().await.map_err(|e| {
            warn!("Failed to accept bidirectional stream from {}: {}", remote_node_id, e);
            Error::IrohConnectionError(e)
        })?;

        debug!("Established bidirectional stream with peer: {}", remote_node_id);

        // Use IrohConnection to wrap the streams and perform handshake
        IrohConnection::new(send_stream, recv_stream, role).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Test default configuration
    #[test]
    fn test_default_config() {
        let config = IrohNodeConfig::default();

        assert!(config.secret_key_path.is_none());
        assert!(config.secret_key.is_none());
        assert!(config.relay_mode.is_some());
        assert!(config.bind_addr_v4.is_none());
        assert!(config.bind_addr_v6.is_none());
        assert_eq!(config.alpn, b"stratum-v2");
    }

    /// Test node initialization with default config
    #[tokio::test]
    async fn test_node_initialization_default() {
        let config = IrohNodeConfig::default();
        let result = IrohNodeManager::new(config).await;

        assert!(result.is_ok());
        let manager = result.unwrap();

        // Verify node ID was generated
        let node_id = manager.node_id();
        assert_ne!(node_id.to_string(), "");
    }

    /// Test node initialization with persistent secret key
    #[tokio::test]
    async fn test_node_initialization_with_persistent_key() {
        let temp_dir = TempDir::new().unwrap();
        let key_path = temp_dir.path().join("secret_key");

        let config1 = IrohNodeConfig {
            secret_key_path: Some(key_path.clone()),
            ..Default::default()
        };

        // First initialization should create the key file
        let manager1 = IrohNodeManager::new(config1).await.unwrap();
        let node_id1 = manager1.node_id();

        // Verify key file was created
        assert!(key_path.exists());

        // Read the key file
        let key_contents = fs::read_to_string(&key_path).unwrap();
        assert_eq!(key_contents.len(), 64); // Hex-encoded 32 bytes

        // Second initialization should reuse the same key
        let config2 = IrohNodeConfig {
            secret_key_path: Some(key_path.clone()),
            ..Default::default()
        };

        let manager2 = IrohNodeManager::new(config2).await.unwrap();
        let node_id2 = manager2.node_id();

        // Both managers should have the same node ID
        assert_eq!(node_id1, node_id2);
    }

    /// Test node initialization with explicit secret key
    #[tokio::test]
    async fn test_node_initialization_with_explicit_key() {
        let secret_key = SecretKey::generate(&mut rand::thread_rng());
        let expected_node_id = secret_key.public();

        let config = IrohNodeConfig {
            secret_key: Some(secret_key),
            ..Default::default()
        };

        let manager = IrohNodeManager::new(config).await.unwrap();
        let node_id = manager.node_id();

        // Should match the public key of the provided secret
        assert_eq!(node_id, expected_node_id);
    }

    /// Test configuration validation - empty ALPN
    #[tokio::test]
    async fn test_config_validation_empty_alpn() {
        let config = IrohNodeConfig {
            alpn: vec![],
            ..Default::default()
        };

        let result = IrohNodeManager::new(config).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            Error::IrohNodeInitialization(msg) => {
                assert!(msg.to_string().contains("ALPN"));
            }
            _ => panic!("Expected IrohNodeInitialization error"),
        }
    }

    /// Test configuration with custom bind addresses
    #[tokio::test]
    async fn test_config_with_custom_bind_addresses() {
        use std::net::{Ipv4Addr, Ipv6Addr};

        let config = IrohNodeConfig {
            bind_addr_v4: Some(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)),
            bind_addr_v6: Some(SocketAddrV6::new(Ipv6Addr::LOCALHOST, 0, 0, 0)),
            ..Default::default()
        };

        let result = IrohNodeManager::new(config).await;
        assert!(result.is_ok());
    }

    /// Test secret key persistence with nested directory
    #[tokio::test]
    async fn test_secret_key_nested_directory() {
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir.path().join("nested").join("dir").join("secret_key");

        let config = IrohNodeConfig {
            secret_key_path: Some(nested_path.clone()),
            ..Default::default()
        };

        let result = IrohNodeManager::new(config).await;
        assert!(result.is_ok());

        // Verify the nested directory structure was created
        assert!(nested_path.exists());
        assert!(nested_path.parent().unwrap().exists());
    }

    /// Test that explicit secret key takes precedence over file
    #[tokio::test]
    async fn test_explicit_key_precedence() {
        let temp_dir = TempDir::new().unwrap();
        let key_path = temp_dir.path().join("secret_key");

        // Create a key file first
        let file_key = SecretKey::generate(&mut rand::thread_rng());
        let file_key_hex = hex::encode(file_key.to_bytes());
        fs::write(&key_path, file_key_hex).unwrap();

        // Now create a different key to use explicitly
        let explicit_key = SecretKey::generate(&mut rand::thread_rng());
        let expected_node_id = explicit_key.public();

        let config = IrohNodeConfig {
            secret_key_path: Some(key_path),
            secret_key: Some(explicit_key),
            ..Default::default()
        };

        let manager = IrohNodeManager::new(config).await.unwrap();
        let node_id = manager.node_id();

        // Should use the explicit key, not the file key
        assert_eq!(node_id, expected_node_id);
        assert_ne!(node_id, file_key.public());
    }

    /// Test relay mode configuration
    #[tokio::test]
    async fn test_relay_mode_config() {
        let config = IrohNodeConfig {
            relay_mode: Some(RelayMode::Disabled),
            ..Default::default()
        };

        let result = IrohNodeManager::new(config).await;
        assert!(result.is_ok());
    }

    /// Test custom ALPN protocol
    #[tokio::test]
    async fn test_custom_alpn() {
        let custom_alpn = b"my-custom-protocol".to_vec();

        let config = IrohNodeConfig {
            alpn: custom_alpn.clone(),
            ..Default::default()
        };

        let manager = IrohNodeManager::new(config).await.unwrap();
        assert_eq!(manager.config().alpn, custom_alpn);
    }
}
