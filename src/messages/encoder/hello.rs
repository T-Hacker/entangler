use super::StringEncoder;
use crate::messages::hello::HelloMessage;
use bytes::BufMut;
use tokio_util::codec::Encoder;

pub struct HelloMessageEncoder;

impl Encoder<&HelloMessage> for HelloMessageEncoder {
    type Error = std::io::Error;

    fn encode(
        &mut self,
        item: &HelloMessage,
        dst: &mut bytes::BytesMut,
    ) -> Result<(), Self::Error> {
        // Reserve buffer space as an optimization.
        dst.reserve(1 + 4 + 4);

        // Deserialize message type.
        let message_type = *item.r#type() as u8;
        dst.put_u8(message_type);

        // Deserialize magic number.
        let magic_number = u32::to_le_bytes(item.magic_number());
        dst.extend_from_slice(&magic_number);

        // Deserialize name.
        let mut string_encoder = StringEncoder;
        string_encoder.encode(item.name(), dst)?;

        // Deserialize version.
        string_encoder.encode(item.version(), dst)?;

        Ok(())
    }
}
