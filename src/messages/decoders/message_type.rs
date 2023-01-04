use crate::messages::MessageType;
use tokio_util::codec::Decoder;

pub struct MessageTypeDecoder;

impl Decoder for MessageTypeDecoder {
    type Item = MessageType;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.is_empty() {
            // No data to process.
            src.reserve(1 - src.len());

            return Ok(None);
        }

        // Deserialize the message type.
        let src = src.split_to(1);
        let message_type = &src[0];

        let message_type = match message_type {
            0 => Ok(MessageType::Hello),

            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Unknown message type.".to_string(),
            )),
        }?;

        // Return object.
        Ok(Some(message_type))
    }
}
