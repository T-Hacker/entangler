pub mod decoders;
pub mod encoder;

mod hello;

// Exported types.
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
    use crate::{MAGIC_NUMBER, NAME, VERSION};
    use bytes::BytesMut;
    use tokio_util::codec::{Decoder, Encoder};

    #[test]
    fn hello_message() {
        // Create message object.
        let magic_number = MAGIC_NUMBER;
        let name = NAME.to_string();
        let version = VERSION.to_string();
        let message = HelloMessage::new(magic_number, name, version);

        // Encode message object.
        let mut encoder = HelloMessageEncoder;
        let mut buffer = BytesMut::new();
        encoder.encode(&message, &mut buffer).unwrap();

        // Decode message object.
        let mut decoder = HelloMessageDecoder;
        let decoded_message = decoder.decode(&mut buffer).unwrap().unwrap();

        // Assert that we don't have more bytes in the buffer.
        assert!(buffer.is_empty());

        // Assert that both message are equal.
        assert_eq!(decoded_message, message);
    }
}
