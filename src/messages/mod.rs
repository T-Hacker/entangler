pub mod decoders;
pub mod encoder;
mod hello;

pub use hello::HelloMessage;

#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum MessageType {
    Hello = 0,
    Index,
}

#[cfg(test)]
mod tests {
    use super::{decoders::HelloMessageDecoder, encoder::HelloMessageEncoder, hello::HelloMessage};
    use bytes::BytesMut;
    use tokio_util::codec::{Decoder, Encoder};

    #[test]
    fn hello_message() {
        // Create message object.
        let magic_number = 1234;
        let name = "test_client";
        let version = "0.1.0";
        let message = HelloMessage::new(magic_number, name.to_string(), version.to_string());

        // Encode message object.
        let mut encoder = HelloMessageEncoder;
        let mut buffer = BytesMut::new();
        encoder.encode(&message, &mut buffer).unwrap();

        // Decode message object.
        let mut decoder = HelloMessageDecoder;
        let decoded_message = decoder.decode(&mut buffer).unwrap().unwrap();

        // Assert that we down't have more bytes in the buffer.
        assert!(buffer.is_empty());

        // Assert that both message are equal.
        assert_eq!(decoded_message, message);
    }
}
