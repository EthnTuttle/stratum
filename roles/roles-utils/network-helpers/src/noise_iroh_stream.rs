//! A Noise-encrypted wrapper around an Iroh BiStream, providing framed read/write I/O using the SV2
//! protocol and a stateful Noise handshake.
//!
//! This module provides `NoiseIrohStream`, which wraps an Iroh `BiStream` and performs a Noise-based
//! authenticated key exchange based on the provided [`HandshakeRole`].
//!
//! After a successful handshake, the stream can be split into a `NoiseIrohReadHalf` and
//! `NoiseIrohWriteHalf`, which support frame-based encoding/decoding of SV2 messages with optional
//! non-blocking behavior.

use crate::Error;
use codec_sv2::{
    binary_sv2::{Deserialize, GetSize, Serialize},
    noise_sv2::INITIATOR_EXPECTED_HANDSHAKE_MESSAGE_SIZE,
    HandshakeRole, NoiseEncoder, StandardNoiseDecoder, State,
};
use iroh::endpoint::{RecvStream, SendStream};

use codec_sv2::{noise_sv2::ELLSWIFT_ENCODING_SIZE, HandShakeFrame, StandardEitherFrame};
use std::convert::TryInto;
use tracing::{debug, error};

/// A Noise-secured duplex stream over Iroh that wraps an Iroh `BiStream`
/// and provides secure read/write capabilities using the Noise protocol.
///
/// This stream performs the full Noise handshake during construction
/// and returns a bidirectional encrypted stream split into read and write halves.
///
/// **Note:** This struct is **not cancellation-safe**.
/// If `read_frame()` or `write_frame()` is canceled mid-way,
/// internal state may be left in an inconsistent state, which can lead to
/// protocol errors or dropped frames.
pub struct NoiseIrohStream<Message: Serialize + Deserialize<'static> + GetSize + Send + 'static> {
    reader: NoiseIrohReadHalf<Message>,
    writer: NoiseIrohWriteHalf<Message>,
}

/// The reading half of a `NoiseIrohStream`.
///
/// It buffers incoming encrypted bytes, attempts to decode full Noise frames,
/// and exposes a method to retrieve structured messages of type `Message`.
pub struct NoiseIrohReadHalf<Message: Serialize + Deserialize<'static> + GetSize + Send + 'static> {
    reader: RecvStream,
    decoder: StandardNoiseDecoder<Message>,
    state: State,
    current_frame_buf: Vec<u8>,
    bytes_read: usize,
}

/// The writing half of a `NoiseIrohStream`.
///
/// It accepts structured messages, encodes them via the Noise protocol,
/// and writes the result to the Iroh stream.
pub struct NoiseIrohWriteHalf<Message: Serialize + Deserialize<'static> + GetSize + Send + 'static> {
    writer: SendStream,
    encoder: NoiseEncoder<Message>,
    state: State,
}

impl<Message> NoiseIrohStream<Message>
where
    Message: Serialize + Deserialize<'static> + GetSize + Send + 'static,
{
    /// Constructs a new `NoiseIrohStream` over the given Iroh BiStream,
    /// performing the Noise handshake in the given `role`.
    ///
    /// On success, returns a stream with encrypted communication channels.
    pub async fn new(
        send_stream: SendStream,
        recv_stream: RecvStream,
        role: HandshakeRole,
    ) -> Result<Self, Error> {
        let mut reader = recv_stream;
        let mut writer = send_stream;

        let mut decoder = StandardNoiseDecoder::<Message>::new();
        let mut encoder = NoiseEncoder::<Message>::new();
        let mut state = State::initialized(role.clone());

        match role {
            HandshakeRole::Initiator(_) => {
                let mut responder_state = codec_sv2::State::not_initialized(&role);
                let first_msg = state.step_0()?;
                send_message(&mut writer, first_msg.into(), &mut state, &mut encoder).await?;
                debug!("First handshake message sent");

                loop {
                    match receive_message(&mut reader, &mut responder_state, &mut decoder).await {
                        Ok(second_msg) => {
                            debug!("Second handshake message received");
                            let handshake_frame: HandShakeFrame = second_msg
                                .try_into()
                                .map_err(|_| Error::HandshakeRemoteInvalidMessage)?;
                            let payload: [u8; INITIATOR_EXPECTED_HANDSHAKE_MESSAGE_SIZE] =
                                handshake_frame
                                    .get_payload_when_handshaking()
                                    .try_into()
                                    .map_err(|_| Error::HandshakeRemoteInvalidMessage)?;
                            let transport_state = state.step_2(payload)?;
                            state = transport_state;
                            break;
                        }
                        Err(Error::CodecError(codec_sv2::Error::MissingBytes(_))) => {
                            debug!("Waiting for more bytes during handshake");
                        }
                        Err(e) => {
                            error!("Handshake failed with upstream: {:?}", e);
                            return Err(e);
                        }
                    }
                }
            }
            HandshakeRole::Responder(_) => {
                let mut initiator_state = codec_sv2::State::not_initialized(&role);

                loop {
                    match receive_message(&mut reader, &mut initiator_state, &mut decoder).await {
                        Ok(first_msg) => {
                            debug!("First handshake message received");
                            let handshake_frame: HandShakeFrame = first_msg
                                .try_into()
                                .map_err(|_| Error::HandshakeRemoteInvalidMessage)?;
                            let payload: [u8; ELLSWIFT_ENCODING_SIZE] = handshake_frame
                                .get_payload_when_handshaking()
                                .try_into()
                                .map_err(|_| Error::HandshakeRemoteInvalidMessage)?;
                            let (second_msg, transport_state) = state.step_1(payload)?;
                            send_message(&mut writer, second_msg.into(), &mut state, &mut encoder)
                                .await?;
                            debug!("Second handshake message sent");
                            state = transport_state;
                            break;
                        }
                        Err(Error::CodecError(codec_sv2::Error::MissingBytes(_))) => {
                            debug!("Waiting for more bytes during handshake");
                        }
                        Err(e) => {
                            error!("Handshake failed with downstream: {:?}", e);
                            return Err(e);
                        }
                    }
                }
            }
        };
        Ok(Self {
            reader: NoiseIrohReadHalf {
                reader,
                decoder,
                state: state.clone(),
                current_frame_buf: vec![],
                bytes_read: 0,
            },
            writer: NoiseIrohWriteHalf {
                writer,
                encoder,
                state,
            },
        })
    }

    /// Consumes the stream and returns its reader and writer halves.
    pub fn into_split(self) -> (NoiseIrohReadHalf<Message>, NoiseIrohWriteHalf<Message>) {
        (self.reader, self.writer)
    }
}

impl<Message> NoiseIrohWriteHalf<Message>
where
    Message: Serialize + Deserialize<'static> + GetSize + Send + 'static,
{
    /// Encrypts and writes a full message frame to the Iroh stream.
    ///
    /// Returns an error if the stream is closed or the message cannot be encoded.
    ///
    /// Not cancellation-safe: A canceled write may cause partial writes or state corruption.
    pub async fn write_frame(&mut self, frame: StandardEitherFrame<Message>) -> Result<(), Error> {
        let buf = self.encoder.encode(frame, &mut self.state)?;

        // Write all bytes to the stream
        let mut offset = 0;
        while offset < buf.len() {
            let n = self.writer
                .write(&buf[offset..])
                .await
                .map_err(Error::IrohWriteError)?;
            offset += n;
        }
        Ok(())
    }

    /// Attempts to write a message without blocking.
    ///
    /// Returns:
    /// - `Ok(true)` if the entire frame was written successfully.
    /// - `Ok(false)` if the stream is not ready (would block).
    /// - `Err(_)` on stream or encoding errors.
    ///
    /// Note: Iroh streams don't have a non-blocking write API like TCP.
    /// This method currently always returns false, indicating the caller should use write_frame.
    pub fn try_write_frame(&mut self, _frame: StandardEitherFrame<Message>) -> Result<bool, Error> {
        // Iroh's SendStream doesn't have a direct try_write method
        // For now, we indicate that non-blocking writes are not supported
        // Callers should use write_frame() instead
        Ok(false)
    }

    /// Gracefully finishes the writing half of the stream.
    ///
    /// Returns an error if finishing fails.
    pub async fn shutdown(&mut self) -> Result<(), Error> {
        self.writer
            .finish()
            .map_err(|_| Error::SocketClosed)?;
        Ok(())
    }
}

impl<Message> NoiseIrohReadHalf<Message>
where
    Message: Serialize + Deserialize<'static> + GetSize + Send + 'static,
{
    /// Reads and decodes a complete frame from the Iroh stream.
    ///
    /// This method blocks until a full frame is read and decoded,
    /// handling `MissingBytes` errors from the codec automatically.
    ///
    /// Not cancellation-safe: Cancellation may leave partially-read state behind.
    pub async fn read_frame(&mut self) -> Result<StandardEitherFrame<Message>, Error> {
        loop {
            let expected = self.decoder.writable_len();

            if self.current_frame_buf.len() != expected {
                self.current_frame_buf.resize(expected, 0);
                self.bytes_read = 0;
            }

            while self.bytes_read < expected {
                let n = self
                    .reader
                    .read(&mut self.current_frame_buf[self.bytes_read..])
                    .await
                    .map_err(Error::IrohReadError)?
                    .ok_or(Error::SocketClosed)?;

                if n == 0 {
                    return Err(Error::SocketClosed);
                }

                self.bytes_read += n;
            }

            self.decoder
                .writable()
                .copy_from_slice(&self.current_frame_buf[..]);

            self.bytes_read = 0;

            match self.decoder.next_frame(&mut self.state) {
                Ok(frame) => return Ok(frame),
                Err(codec_sv2::Error::MissingBytes(_)) => {
                    tokio::task::yield_now().await;
                    continue;
                }
                Err(e) => return Err(Error::CodecError(e)),
            }
        }
    }

    /// Attempts to read and decode a frame without blocking.
    ///
    /// Returns:
    /// - `Ok(Some(frame))` if a full frame is successfully decoded.
    /// - `Ok(None)` if not enough data is available yet.
    /// - `Err(_)` on stream or decoding errors.
    pub fn try_read_frame(&mut self) -> Result<Option<StandardEitherFrame<Message>>, Error> {
        let expected = self.decoder.writable_len();

        if self.current_frame_buf.len() != expected {
            self.current_frame_buf.resize(expected, 0);
            self.bytes_read = 0;
        }

        // Iroh's RecvStream doesn't have a direct try_read method
        // We'll need to handle this differently than TCP
        // For now, we return None to indicate would block
        // This will be refined in future iterations based on Iroh's async patterns
        Ok(None)
    }
}

async fn send_message<Message: Serialize + Deserialize<'static> + GetSize + Send + 'static>(
    writer: &mut SendStream,
    msg: StandardEitherFrame<Message>,
    state: &mut State,
    encoder: &mut NoiseEncoder<Message>,
) -> Result<(), Error> {
    let buffer = encoder.encode(msg, state)?;

    // Write all bytes to the stream
    let mut offset = 0;
    while offset < buffer.len() {
        let n = writer
            .write(&buffer[offset..])
            .await
            .map_err(Error::IrohWriteError)?;
        offset += n;
    }
    Ok(())
}

async fn receive_message<Message: Serialize + Deserialize<'static> + GetSize + Send + 'static>(
    reader: &mut RecvStream,
    state: &mut State,
    decoder: &mut StandardNoiseDecoder<Message>,
) -> Result<StandardEitherFrame<Message>, Error> {
    let mut buffer = vec![0u8; decoder.writable_len()];
    reader
        .read_exact(&mut buffer)
        .await
        .map_err(|_| Error::SocketClosed)?;
    decoder.writable().copy_from_slice(&buffer);
    decoder.next_frame(state).map_err(Error::CodecError)
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
            // Return an empty structure since we're using Seq0255 which has its own structure
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

    /// Test that NoiseIrohStream can perform a successful handshake between initiator and responder
    #[tokio::test]
    async fn test_noise_iroh_handshake() {
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

            // Create responder role
            let responder_role = HandshakeRole::Responder(Responder::new(key_pair, 31536000));

            NoiseIrohStream::<TestMessage>::new(send, recv, responder_role)
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

        // Create initiator role with responder's public key
        let initiator_role =
            HandshakeRole::Initiator(Initiator::new(Some(responder_pub_key.into())));

        let stream1 = NoiseIrohStream::<TestMessage>::new(send, recv, initiator_role)
            .await
            .unwrap();

        // Wait for responder handshake to complete
        let stream2 = accept_task.await.unwrap();

        // If we get here, both handshakes succeeded
        drop(stream1);
        drop(stream2);
    }

    /// Test bidirectional message flow over NoiseIrohStream
    #[tokio::test]
    async fn test_noise_iroh_bidirectional_message_flow() {
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

            let responder_role =
                HandshakeRole::Responder(Responder::new(key_pair, 31536000));

            NoiseIrohStream::<TestMessage>::new(send, recv, responder_role)
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

        let stream1 = NoiseIrohStream::<TestMessage>::new(send, recv, initiator_role)
            .await
            .unwrap();

        let stream2 = accept_task.await.unwrap();

        // Split streams
        let (mut reader1, mut writer1) = stream1.into_split();
        let (mut reader2, mut writer2) = stream2.into_split();

        // Test message flow: endpoint1 -> endpoint2
        let test_message = create_test_message();
        let test_frame = Sv2Frame::from_message(test_message, 0x01, 0, false).unwrap();
        let frame_to_send: StandardEitherFrame<TestMessage> = test_frame.into();

        // Spawn a task to write from endpoint1
        let write_task1 = tokio::spawn(async move {
            writer1.write_frame(frame_to_send).await.unwrap();
            writer1
        });

        // Read on endpoint2
        let received_frame = reader2.read_frame().await.unwrap();

        // Verify we received an SV2 frame
        match received_frame {
            StandardEitherFrame::Sv2(frame) => {
                assert_eq!(frame.get_header().unwrap().msg_type(), 0x01);
            }
            _ => panic!("Expected Sv2 frame"),
        }

        let mut writer1 = write_task1.await.unwrap();

        // Test message flow in reverse: endpoint2 -> endpoint1
        let test_message2 = create_test_message();
        let test_frame2 = Sv2Frame::from_message(test_message2, 0x02, 0, false).unwrap();
        let frame_to_send2: StandardEitherFrame<TestMessage> = test_frame2.into();

        // Write from endpoint2
        let write_task2 = tokio::spawn(async move {
            writer2.write_frame(frame_to_send2).await.unwrap();
            writer2
        });

        // Read on endpoint1
        let received_frame2 = reader1.read_frame().await.unwrap();

        // Verify we received an SV2 frame
        match received_frame2 {
            StandardEitherFrame::Sv2(frame) => {
                assert_eq!(frame.get_header().unwrap().msg_type(), 0x02);
            }
            _ => panic!("Expected Sv2 frame"),
        }

        write_task2.await.unwrap();
        writer1.shutdown().await.unwrap();
    }

    /// Test error handling when connection is closed prematurely
    #[tokio::test]
    async fn test_noise_iroh_connection_closed() {
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

        // Spawn a task to accept and immediately close
        let accept_task = tokio::spawn(async move {
            let incoming = endpoint2.accept().await.unwrap();
            let connecting = incoming.await.unwrap();
            let (send, recv) = connecting.accept_bi().await.unwrap();

            let responder_role =
                HandshakeRole::Responder(Responder::new(key_pair, 31536000));

            let stream = NoiseIrohStream::<TestMessage>::new(send, recv, responder_role)
                .await
                .unwrap();

            let (_, mut writer) = stream.into_split();
            // Close the writer
            writer.shutdown().await.unwrap();
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

        let stream1 = NoiseIrohStream::<TestMessage>::new(send, recv, initiator_role)
            .await
            .unwrap();

        let (mut reader1, _writer1) = stream1.into_split();

        // Wait for the responder to close
        accept_task.await.unwrap();

        // Give some time for the close to propagate
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Try to read, which should fail
        let result = reader1.read_frame().await;
        assert!(result.is_err());
    }
}
