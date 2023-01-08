use super::{MessageTypeDecoder, StringDecoder};
use crate::messages::{hello::HelloMessage, MessageType};
use bytes::BytesMut;
use std::io::ErrorKind;
use tokio_util::codec::Decoder;

pub struct HelloMessageDecoder;

impl Decoder for HelloMessageDecoder {
    type Item = HelloMessage;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Try to deserialize to message type.
        let mut message_type_decoder = MessageTypeDecoder;
        let message_type = match message_type_decoder.decode(src) {
            Ok(Some(message_type)) => message_type,

            Ok(None) => return Ok(None),
            Err(e) => return Err(e),
        };

        // Check if this is a hello message.
        let MessageType::Hello = &message_type else {
            return Err(std::io::Error::new(ErrorKind::InvalidData,"Invalid hello message.".to_string()));
        };

        // Try to deserialize the magic number.
        if src.len() < 4 {
            // Not enough data. Lets just reserve some data for the next buffer.
            src.reserve(4_usize.saturating_sub(src.len()));

            return Ok(None);
        }

        let magic_number = {
            let src = src.split_to(4);
            let magic_number = src[..4].to_vec();
            let magic_number: [u8; 4] = magic_number.try_into().unwrap();

            u32::from_le_bytes(magic_number)
        };

        // Try to deserialize the name.
        let mut string_decoder = StringDecoder;
        let Some(name) = string_decoder.decode(src)? else {
            // Didn't get the name.
            return Ok(None);
        };

        // Try to deserialize the version.
        let Some(version) = string_decoder.decode(src)? else {
            // Didn't get the version.
            return Ok(None);
        };

        // Return the object.
        Ok(Some(HelloMessage::new(magic_number, name, version)))
    }
}
