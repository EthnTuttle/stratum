#![allow(clippy::new_ret_no_self)]
use crate::{
    noise_iroh_stream::{NoiseIrohReadHalf, NoiseIrohStream, NoiseIrohWriteHalf},
    Error,
};
use async_channel::{unbounded, Receiver, Sender};
use codec_sv2::{
    binary_sv2::{Deserialize, GetSize, Serialize},
    HandshakeRole, StandardEitherFrame,
};
use iroh::endpoint::{RecvStream, SendStream};
use std::sync::Arc;
use tokio::task;
use tracing::{debug, error};

pub struct IrohConnection;

struct IrohConnectionState<Message> {
    sender_incoming: Sender<StandardEitherFrame<Message>>,
    receiver_incoming: Receiver<StandardEitherFrame<Message>>,
    sender_outgoing: Sender<StandardEitherFrame<Message>>,
    receiver_outgoing: Receiver<StandardEitherFrame<Message>>,
}

impl<Message> IrohConnectionState<Message> {
    fn close_all(&self) {
        self.sender_incoming.close();
        self.receiver_incoming.close();
        self.sender_outgoing.close();
        self.receiver_outgoing.close();
    }
}

impl IrohConnection {
    pub async fn new<Message>(
        send_stream: SendStream,
        recv_stream: RecvStream,
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
        let (sender_incoming, receiver_incoming) = unbounded();
        let (sender_outgoing, receiver_outgoing) = unbounded();

        let conn_state = Arc::new(IrohConnectionState {
            sender_incoming,
            receiver_incoming: receiver_incoming.clone(),
            sender_outgoing: sender_outgoing.clone(),
            receiver_outgoing,
        });

        let (read_half, write_half) =
            NoiseIrohStream::<Message>::new(send_stream, recv_stream, role)
                .await?
                .into_split();

        Self::spawn_reader(read_half, Arc::clone(&conn_state));
        Self::spawn_writer(write_half, conn_state);

        Ok((receiver_incoming, sender_outgoing))
    }

    fn spawn_reader<Message>(
        mut read_half: NoiseIrohReadHalf<Message>,
        conn_state: Arc<IrohConnectionState<Message>>,
    ) -> task::JoinHandle<()>
    where
        Message: Serialize + Deserialize<'static> + GetSize + Send + 'static,
    {
        let sender_incoming = conn_state.sender_incoming.clone();

        task::spawn(async move {
            loop {
                tokio::select! {
                    _ = tokio::signal::ctrl_c() => {
                        debug!("Reader received shutdown signal.");
                        break;
                    }
                    res = read_half.read_frame() => match res {
                        Ok(frame) => {
                            if sender_incoming.send(frame).await.is_err() {
                                error!("Reader: channel closed, shutting down.");
                                break;
                            }
                        }
                        Err(e) => {
                            error!("Reader: error while reading frame: {e:?}");
                            break;
                        }
                    }
                }
            }

            conn_state.close_all();
        })
    }

    fn spawn_writer<Message>(
        mut write_half: NoiseIrohWriteHalf<Message>,
        conn_state: Arc<IrohConnectionState<Message>>,
    ) -> task::JoinHandle<()>
    where
        Message: Serialize + Deserialize<'static> + GetSize + Send + 'static,
    {
        let receiver_outgoing = conn_state.receiver_outgoing.clone();

        task::spawn(async move {
            loop {
                tokio::select! {
                    _ = tokio::signal::ctrl_c() => {
                        debug!("Writer received shutdown signal.");
                        break;
                    }
                    res = receiver_outgoing.recv() => match res {
                        Ok(frame) => {
                            if let Err(e) = write_half.write_frame(frame).await {
                                error!("Writer: error while writing frame: {e:?}");
                                break;
                            }
                        }
                        Err(_) => {
                            debug!("Writer: channel closed, shutting down.");
                            break;
                        }
                    }
                }
            }

            if let Err(e) = write_half.shutdown().await {
                error!("Writer: error during shutdown: {e:?}");
            }

            conn_state.close_all();
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use codec_sv2::{
        binary_sv2::{
            self,
            decodable::{DecodableField, FieldMarker},
            Seq0255,
        },
        noise_sv2::{Initiator, Responder},
        Sv2Frame,
    };
    use iroh::Endpoint;

    // Define a minimal test message type that implements all necessary traits
    #[derive(Debug, Clone, PartialEq)]
    struct TestMessage {
        data: Seq0255<'static, u8>,
    }

    impl binary_sv2::Serialize for TestMessage {
        fn to_bytes(self, dst: &mut [u8]) -> Result<usize, binary_sv2::Error> {
            self.data.to_bytes(dst)
        }
    }

    impl<'a> binary_sv2::Deserialize<'a> for TestMessage {
        fn get_structure(_data: &[u8]) -> Result<Vec<FieldMarker>, binary_sv2::Error> {
            Ok(vec![])
        }

        fn from_decoded_fields(_data: Vec<DecodableField<'a>>) -> Result<Self, binary_sv2::Error> {
            Ok(TestMessage {
                data: Seq0255::new(vec![]).unwrap(),
            })
        }
    }

    impl binary_sv2::GetSize for TestMessage {
        fn get_size(&self) -> usize {
            self.data.get_size()
        }
    }

    /// Helper to create a test message
    fn create_test_message() -> TestMessage {
        TestMessage {
            data: Seq0255::new(vec![1u8, 2, 3, 4, 5]).unwrap(),
        }
    }

    /// Test bidirectional message flow over IrohConnection
    #[tokio::test]
    async fn test_iroh_connection_bidirectional() {
        // Create two Iroh endpoints for testing
        let endpoint1 = Endpoint::builder().bind().await.unwrap();
        let endpoint2 = Endpoint::builder().bind().await.unwrap();

        let node2_id = endpoint2.node_id();

        // Generate keypair for responder
        use secp256k1::{Secp256k1, SecretKey};
        let secp = Secp256k1::new();
        let secret_key = SecretKey::new(&mut rand::thread_rng());
        let key_pair = secp256k1::Keypair::from_secret_key(&secp, &secret_key);
        let responder_pub_key = key_pair.public_key();

        // Spawn a task to accept connections on endpoint2 (responder)
        let accept_task = tokio::spawn(async move {
            let incoming = endpoint2.accept().await.unwrap();
            let connecting = incoming.await.unwrap();
            let (send, recv) = connecting.accept_bi().await.unwrap();

            let responder_role = HandshakeRole::Responder(Responder::new(key_pair, 31536000));

            IrohConnection::new::<TestMessage>(send, recv, responder_role)
                .await
                .unwrap()
        });

        // Give the accept task time to start
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // Connect using the node ID - Iroh will handle address discovery locally
        let connection = match endpoint1.connect(node2_id, b"sv2-test").await {
            Ok(conn) => conn,
            Err(_) => {
                // Skip test if Iroh networking isn't available
                println!("Skipping test - Iroh discovery not configured");
                return;
            }
        };
        let (send, recv) = connection.open_bi().await.unwrap();

        let initiator_role =
            HandshakeRole::Initiator(Initiator::new(Some(responder_pub_key.into())));

        let (receiver1, sender1) =
            IrohConnection::new::<TestMessage>(send, recv, initiator_role)
                .await
                .unwrap();

        let (receiver2, sender2) = accept_task.await.unwrap();

        // Test message flow: endpoint1 -> endpoint2
        let test_message = create_test_message();
        let test_frame = Sv2Frame::from_message(test_message, 0x01, 0, false).unwrap();
        let frame_to_send: StandardEitherFrame<TestMessage> = test_frame.into();

        sender1.send(frame_to_send).await.unwrap();

        // Read on endpoint2
        let received_frame = receiver2.recv().await.unwrap();

        // Verify we received an SV2 frame
        match received_frame {
            StandardEitherFrame::Sv2(frame) => {
                assert_eq!(frame.get_header().unwrap().msg_type(), 0x01);
            }
            _ => panic!("Expected Sv2 frame"),
        }

        // Test message flow in reverse: endpoint2 -> endpoint1
        let test_message2 = create_test_message();
        let test_frame2 = Sv2Frame::from_message(test_message2, 0x02, 0, false).unwrap();
        let frame_to_send2: StandardEitherFrame<TestMessage> = test_frame2.into();

        sender2.send(frame_to_send2).await.unwrap();

        // Read on endpoint1
        let received_frame2 = receiver1.recv().await.unwrap();

        // Verify we received an SV2 frame
        match received_frame2 {
            StandardEitherFrame::Sv2(frame) => {
                assert_eq!(frame.get_header().unwrap().msg_type(), 0x02);
            }
            _ => panic!("Expected Sv2 frame"),
        }
    }

    /// Test connection establishment and teardown
    #[tokio::test]
    async fn test_iroh_connection_establishment() {
        // Create two Iroh endpoints for testing
        let endpoint1 = Endpoint::builder().bind().await.unwrap();
        let endpoint2 = Endpoint::builder().bind().await.unwrap();

        let node2_id = endpoint2.node_id();

        // Generate keypair for responder
        use secp256k1::{Secp256k1, SecretKey};
        let secp = Secp256k1::new();
        let secret_key = SecretKey::new(&mut rand::thread_rng());
        let key_pair = secp256k1::Keypair::from_secret_key(&secp, &secret_key);
        let responder_pub_key = key_pair.public_key();

        // Spawn a task to accept connections on endpoint2 (responder)
        let accept_task = tokio::spawn(async move {
            let incoming = endpoint2.accept().await.unwrap();
            let connecting = incoming.await.unwrap();
            let (send, recv) = connecting.accept_bi().await.unwrap();

            let responder_role = HandshakeRole::Responder(Responder::new(key_pair, 31536000));

            IrohConnection::new::<TestMessage>(send, recv, responder_role)
                .await
                .unwrap()
        });

        // Give the accept task time to start
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // Connect using the node ID - Iroh will handle address discovery locally
        let connection = match endpoint1.connect(node2_id, b"sv2-test").await {
            Ok(conn) => conn,
            Err(_) => {
                // Skip test if Iroh networking isn't available
                println!("Skipping test - Iroh discovery not configured");
                return;
            }
        };
        let (send, recv) = connection.open_bi().await.unwrap();

        let initiator_role =
            HandshakeRole::Initiator(Initiator::new(Some(responder_pub_key.into())));

        let (receiver1, sender1) =
            IrohConnection::new::<TestMessage>(send, recv, initiator_role)
                .await
                .unwrap();

        let (receiver2, sender2) = accept_task.await.unwrap();

        // If we get here, both connections succeeded
        // Test teardown by closing channels
        drop(sender1);
        drop(sender2);

        // The receivers should eventually detect the closed channels
        assert!(receiver1.recv().await.is_err());
        assert!(receiver2.recv().await.is_err());
    }

    /// Test interface compatibility with existing Connection
    #[tokio::test]
    async fn test_iroh_connection_interface_compatibility() {
        // Create two Iroh endpoints for testing
        let endpoint1 = Endpoint::builder().bind().await.unwrap();
        let endpoint2 = Endpoint::builder().bind().await.unwrap();

        let node2_id = endpoint2.node_id();

        // Generate keypair for responder
        use secp256k1::{Secp256k1, SecretKey};
        let secp = Secp256k1::new();
        let secret_key = SecretKey::new(&mut rand::thread_rng());
        let key_pair = secp256k1::Keypair::from_secret_key(&secp, &secret_key);
        let responder_pub_key = key_pair.public_key();

        // Spawn a task to accept connections on endpoint2 (responder)
        let accept_task = tokio::spawn(async move {
            let incoming = endpoint2.accept().await.unwrap();
            let connecting = incoming.await.unwrap();
            let (send, recv) = connecting.accept_bi().await.unwrap();

            let responder_role = HandshakeRole::Responder(Responder::new(key_pair, 31536000));

            IrohConnection::new::<TestMessage>(send, recv, responder_role)
                .await
                .unwrap()
        });

        // Give the accept task time to start
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // Connect using the node ID - Iroh will handle address discovery locally
        let connection = match endpoint1.connect(node2_id, b"sv2-test").await {
            Ok(conn) => conn,
            Err(_) => {
                // Skip test if Iroh networking isn't available
                println!("Skipping test - Iroh discovery not configured");
                return;
            }
        };
        let (send, recv) = connection.open_bi().await.unwrap();

        let initiator_role =
            HandshakeRole::Initiator(Initiator::new(Some(responder_pub_key.into())));

        // Verify that IrohConnection::new returns the same interface as Connection::new
        // (Receiver, Sender) tuple
        let result: Result<
            (
                Receiver<StandardEitherFrame<TestMessage>>,
                Sender<StandardEitherFrame<TestMessage>>,
            ),
            Error,
        > = IrohConnection::new::<TestMessage>(send, recv, initiator_role).await;

        assert!(result.is_ok());
        let (_receiver1, _sender1) = result.unwrap();

        accept_task.await.unwrap();
    }
}
