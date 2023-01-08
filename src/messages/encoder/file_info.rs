use super::{BlockInfoEncoder, StringEncoder};
use crate::index::FileInfo;
use std::io::ErrorKind;
use tokio_util::codec::Encoder;

pub struct FileInfoEncoder;

impl Encoder<&FileInfo> for FileInfoEncoder {
    type Error = std::io::Error;

    fn encode(&mut self, item: &FileInfo, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        let mut string_encoder = StringEncoder;
        let mut block_info_encoder = BlockInfoEncoder;

        // Write file path.
        let path = item.path();
        let path = path.to_str().ok_or_else(|| {
            std::io::Error::new(ErrorKind::Other, "Fail to convert path to string: {path:?}")
        })?;
        string_encoder.encode(path, dst)?;

        // Reserve space for all blocks for performance reasons.
        let block_size_bytes = item.blocks().len() * item.block_size() as usize;
        dst.reserve(block_size_bytes.saturating_sub(dst.len()));

        // Write file blocks.
        for block in item.blocks() {
            block_info_encoder.encode(block, dst)?;
        }

        Ok(())
    }
}
