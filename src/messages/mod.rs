mod hello;

pub use hello::{HelloMessage, HelloMessageDecoder, HelloMessageEncoder};

#[cfg(test)]
mod tests {
    use super::{HelloMessage, HelloMessageDecoder, HelloMessageEncoder};
    use bytes::BytesMut;
    use tokio_util::codec::{Decoder, Encoder};

    #[test]
    fn hello_message() {
        // Create object.
        let hello_message = HelloMessage::new(123, "test".to_string(), "0.1.0".to_string());

        // Encode object.
        let mut hello_message_encoder = HelloMessageEncoder;
        let mut buffer = BytesMut::new();
        hello_message_encoder
            .encode(&hello_message, &mut buffer)
            .unwrap();

        // Decode object.
        let mut hello_message_decoder = HelloMessageDecoder;
        let decoded_hello_message = hello_message_decoder.decode(&mut buffer).unwrap().unwrap();

        // Make sure that we don't have unused bytes on the buffer.
        assert!(buffer.is_empty());

        // Make sure both objects are equal.
        assert_eq!(decoded_hello_message, hello_message);
    }
}
