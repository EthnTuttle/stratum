#![allow(clippy::new_ret_no_self)]
use async_channel::{bounded, Receiver, Sender};
use codec_sv2::{
    binary_sv2::{Deserialize, GetSize, Serialize},
    Error::MissingBytes, StandardDecoder, StandardEitherFrame,
};
use core::convert::TryInto;
use iroh::endpoint::{RecvStream, SendStream};
use tokio::task;
use tracing::{error, trace};

#[derive(Debug)]
pub struct PlainIrohConnection {}

impl PlainIrohConnection {
    /// Creates a new plain (unencrypted) Iroh connection that returns the same interface as PlainConnection
    ///
    /// # Arguments
    ///
    /// * `send_stream` - Iroh SendStream for outgoing messages
    /// * `recv_stream` - Iroh RecvStream for incoming messages
    ///
    /// # Returns
    ///
    /// A tuple of (Receiver, Sender) for StandardEitherFrame<Message>
    #[allow(clippy::new_ret_no_self)]
    pub async fn new<'a, Message: Serialize + Deserialize<'a> + GetSize + Send + 'static>(
        send_stream: SendStream,
        recv_stream: RecvStream,
    ) -> (
        Receiver<StandardEitherFrame<Message>>,
        Sender<StandardEitherFrame<Message>>,
    ) {
        const NOISE_HANDSHAKE_SIZE_HINT: usize = 3363412;

        let (sender_incoming, receiver_incoming): (
            Sender<StandardEitherFrame<Message>>,
            Receiver<StandardEitherFrame<Message>>,
        ) = bounded(10); // TODO caller should provide this param
        let (sender_outgoing, receiver_outgoing): (
            Sender<StandardEitherFrame<Message>>,
            Receiver<StandardEitherFrame<Message>>,
        ) = bounded(10); // TODO caller should provide this param

        // RECEIVE AND PARSE INCOMING MESSAGES FROM IROH STREAM
        task::spawn(async move {
            let mut decoder = StandardDecoder::<Message>::new();
            let mut recv_stream = recv_stream;

            loop {
                let writable = decoder.writable();
                // Read from Iroh RecvStream using read_exact
                match recv_stream.read_exact(writable).await {
                    Ok(()) => {
                        match decoder.next_frame() {
                            Ok(frame) => {
                                if let Err(e) = sender_incoming.send(frame.into()).await {
                                    error!("Failed to send incoming message: {}", e);
                                    task::yield_now().await;
                                    break;
                                }
                            }
                            Err(MissingBytes(size)) => {
                                // Only disconnect if we get noise handshake message - this
                                // shouldn't happen in plain_iroh_connection
                                if size == NOISE_HANDSHAKE_SIZE_HINT {
                                    error!("Got noise message on unencrypted connection - disconnecting");
                                    break;
                                } else {
                                    trace!("MissingBytes({}) on incoming message - ignoring", size);
                                }
                            }
                            Err(e) => {
                                error!("Failed to read from stream: {}", e);
                                sender_incoming.close();
                                task::yield_now().await;
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        // Just fail and force to reinitialize everything
                        error!("Failed to read from Iroh stream: {}", e);
                        sender_incoming.close();
                        task::yield_now().await;
                        break;
                    }
                }
            }
        });

        // ENCODE AND SEND INCOMING MESSAGES TO IROH STREAM
        task::spawn(async move {
            let mut encoder = codec_sv2::Encoder::<Message>::new();
            let mut send_stream = send_stream;

            loop {
                let received = receiver_outgoing.recv().await;
                match received {
                    Ok(frame) => {
                        let b = encoder.encode(frame.try_into().unwrap()).unwrap();

                        match send_stream.write_all(b).await {
                            Ok(_) => (),
                            Err(_) => {
                                let _ = send_stream.finish();
                            }
                        }
                    }
                    Err(_) => {
                        // Just fail and force to reinitialize everything
                        let _ = send_stream.finish();
                        error!("Failed to read from channel - terminating connection");
                        task::yield_now().await;
                        break;
                    }
                };
            }
        });

        (receiver_incoming, sender_outgoing)
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

    /// Test plain message flow over Iroh
    #[tokio::test]
    async fn test_plain_iroh_connection_bidirectional() {
        // Create two Iroh endpoints for testing
        let endpoint1 = Endpoint::builder().bind().await.unwrap();
        let endpoint2 = Endpoint::builder().bind().await.unwrap();

        let node2_id = endpoint2.node_id();

        // Spawn a task to accept connections on endpoint2
        let accept_task = tokio::spawn(async move {
            let incoming = endpoint2.accept().await.unwrap();
            let connecting = incoming.await.unwrap();
            let (send, recv) = connecting.accept_bi().await.unwrap();

            PlainIrohConnection::new::<TestMessage>(send, recv).await
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

        let (receiver1, sender1) = PlainIrohConnection::new::<TestMessage>(send, recv).await;

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

    /// Test interface compatibility with PlainConnection
    #[tokio::test]
    async fn test_plain_iroh_connection_interface_compatibility() {
        // Create two Iroh endpoints for testing
        let endpoint1 = Endpoint::builder().bind().await.unwrap();
        let endpoint2 = Endpoint::builder().bind().await.unwrap();

        let node2_id = endpoint2.node_id();

        // Spawn a task to accept connections on endpoint2
        let accept_task = tokio::spawn(async move {
            let incoming = endpoint2.accept().await.unwrap();
            let connecting = incoming.await.unwrap();
            let (send, recv) = connecting.accept_bi().await.unwrap();

            PlainIrohConnection::new::<TestMessage>(send, recv).await
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

        // Verify that PlainIrohConnection::new returns the same interface as PlainConnection::new
        // (Receiver, Sender) tuple
        let (_receiver1, sender1): (
            Receiver<StandardEitherFrame<TestMessage>>,
            Sender<StandardEitherFrame<TestMessage>>,
        ) = PlainIrohConnection::new::<TestMessage>(send, recv).await;

        let (receiver2, _sender2) = accept_task.await.unwrap();

        // If we get here, both connections succeeded and returned the correct interface
        // Test teardown by closing channels
        drop(sender1);

        // The receiver should eventually detect the closed channel
        assert!(receiver2.recv().await.is_err());
    }

    /// Test error handling for Iroh-specific failures
    #[tokio::test]
    async fn test_plain_iroh_connection_error_handling() {
        // Create two Iroh endpoints for testing
        let endpoint1 = Endpoint::builder().bind().await.unwrap();
        let endpoint2 = Endpoint::builder().bind().await.unwrap();

        let node2_id = endpoint2.node_id();

        // Spawn a task to accept connections on endpoint2
        let accept_task = tokio::spawn(async move {
            let incoming = endpoint2.accept().await.unwrap();
            let connecting = incoming.await.unwrap();
            let (send, recv) = connecting.accept_bi().await.unwrap();

            PlainIrohConnection::new::<TestMessage>(send, recv).await
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

        let (_receiver1, sender1) = PlainIrohConnection::new::<TestMessage>(send, recv).await;

        let (receiver2, _sender2) = accept_task.await.unwrap();

        // Close the sender to trigger error handling
        drop(sender1);

        // The receiver on the other end should detect the closed stream
        // This tests that connection termination is properly handled
        let result = receiver2.recv().await;
        assert!(result.is_err());
    }
}
