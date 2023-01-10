use super::MAX_FRAME_SIZE;
use bytes::{Buf, BytesMut};
use std::io::ErrorKind;
use tokio_util::codec::Decoder;

pub struct StringDecoder;

impl Decoder for StringDecoder {
    type Item = String;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 4 {
            // Not enough data.
            src.reserve(4_usize.saturating_sub(src.len()));

            return Ok(None);
        }

        // Read length marker.
        let mut length_bytes = [0u8; 4];
        src.copy_to_slice(&mut length_bytes);
        let length = u32::from_le_bytes(length_bytes) as usize;

        // Check that the length is not too large to avoid a denial of
        // service attack where the server runs out of memory.
        if length > MAX_FRAME_SIZE {
            return Err(std::io::Error::new(
                ErrorKind::InvalidData,
                format!("Frame of length {} is too large.", length),
            ));
        }

        if src.len() < length {
            // The full string has not yet arrived.
            //
            // We reserve more space in the buffer. This is not strictly
            // necessary, but is a good idea performance-wise.
            src.reserve(length.saturating_sub(src.len()));

            // We inform the Framed that we need more bytes to form the next
            // frame.
            return Ok(None);
        }

        // Use advance to modify src such that it no longer contains
        // this frame.
        let data = src[..length].to_vec();
        src.advance(length);

        // Convert the data to a string, or fail if it is not valid utf-8.
        match String::from_utf8(data) {
            Ok(string) => Ok(Some(string)),
            Err(utf8_error) => Err(std::io::Error::new(
                ErrorKind::InvalidData,
                utf8_error.utf8_error(),
            )),
        }
    }
}
