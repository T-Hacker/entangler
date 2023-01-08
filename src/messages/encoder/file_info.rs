use super::StringEncoder;
use crate::index::FileInfo;
use std::io::ErrorKind;
use tokio_util::codec::Encoder;

pub struct FileInfoEncoder;

impl Encoder<&FileInfo> for FileInfoEncoder {
    type Error = std::io::Error;

    fn encode(&mut self, item: &FileInfo, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        let mut string_encoder = StringEncoder;

        // Write file path.
        let path = item.path();
        let path = path.to_str().ok_or_else(|| {
            std::io::Error::new(ErrorKind::Other, "Fail to convert path to string: {path:?}")
        })?;
        string_encoder.encode(path, dst)?;

        // Write file blocks.
        for block in item.blocks() {
            todo!();
        }

        Ok(())
    }
}
