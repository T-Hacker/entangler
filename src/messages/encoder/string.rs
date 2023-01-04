use tokio_util::codec::Encoder;

pub struct StringEncoder;

impl Encoder<&str> for StringEncoder {
    type Error = std::io::Error;

    fn encode(&mut self, item: &str, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        // Reserve space as an optimization.
        dst.reserve(4 + item.len());

        // Serialize string size.
        let length = item.len() as u32;
        let length = u32::to_le_bytes(length);
        dst.extend_from_slice(&length);

        // Serialize string data.
        let data = item.as_bytes();
        dst.extend_from_slice(data);

        Ok(())
    }
}
