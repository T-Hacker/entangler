use bytes::{Buf, BufMut};
use std::io::ErrorKind;
use tokio_util::codec::{Decoder, Encoder};

#[derive(Debug, PartialEq, Eq)]
pub struct HelloMessage {
    magic_number: u32,
    name: String,
    version: String,
}

impl HelloMessage {
    pub fn new(magic_number: u32, name: String, version: String) -> Self {
        Self {
            magic_number,
            name,
            version,
        }
    }

    pub fn magic_number(&self) -> u32 {
        self.magic_number
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn version(&self) -> &str {
        &self.version
    }
}

pub struct HelloMessageEncoder;

impl Encoder<&HelloMessage> for HelloMessageEncoder {
    type Error = std::io::Error;

    fn encode(
        &mut self,
        item: &HelloMessage,
        dst: &mut bytes::BytesMut,
    ) -> Result<(), Self::Error> {
        // Write magic number.
        dst.put_u32_le(item.magic_number);

        // Write name.
        if item.name.len() >= 255 {
            return Err(std::io::Error::new(
                ErrorKind::Other,
                "Hello name is too large.",
            ));
        }
        dst.put_u8(item.name.len() as u8);
        dst.put(item.name.as_bytes());

        // Write version.
        if item.version.len() >= 255 {
            return Err(std::io::Error::new(
                ErrorKind::Other,
                "Hello version is too large.",
            ));
        }
        dst.put_u8(item.version.len() as u8);
        dst.put(item.version.as_bytes());

        Ok(())
    }
}

pub struct HelloMessageDecoder;

impl Decoder for HelloMessageDecoder {
    type Item = HelloMessage;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Read magic number.
        if src.len() < 4 {
            src.reserve(4_usize.saturating_sub(src.len()));

            return Ok(None);
        }

        let magic_number = src.get_u32_le();

        // Read name.
        if src.len() < 1 {
            src.reserve(1_usize.saturating_sub(src.len()));

            return Ok(None);
        }

        let name_len = src.get_u8() as usize;
        if src.len() < name_len {
            src.reserve(name_len.saturating_sub(src.len()));

            return Ok(None);
        }

        let name = src.split_to(name_len);
        let name = name.to_vec();
        let name = String::from_utf8(name).map_err(|e| {
            std::io::Error::new(
                ErrorKind::Other,
                format!("Unable to encode name string: {e:?}"),
            )
        })?;

        // Read version.
        if src.len() < 1 {
            src.reserve(1_usize.saturating_sub(src.len()));

            return Ok(None);
        }

        let version_len = src.get_u8() as usize;
        if src.len() < version_len {
            src.reserve(version_len.saturating_sub(src.len()));

            return Ok(None);
        }

        let version = src.split_to(version_len);
        let version = version.to_vec();
        let version = String::from_utf8(version).map_err(|e| {
            std::io::Error::new(
                ErrorKind::Other,
                format!("Unable to encode version string: {e:?}"),
            )
        })?;

        // Return object.
        Ok(Some(HelloMessage {
            magic_number,
            name,
            version,
        }))
    }
}
