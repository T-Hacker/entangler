use super::{BlockInfoDecoder, StringDecoder};
use crate::index::FileInfo;
use bytes::{Buf, BytesMut};
use tokio_util::codec::Decoder;

pub struct FileInfoDecoder;

impl Decoder for FileInfoDecoder {
    type Item = FileInfo;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let mut string_decoder = StringDecoder;

        // Decode file path.
        let Some(path) = string_decoder.decode(src)? else {
            return Ok(None);
        };

        // Decode file size.
        if src.len() < 8 {
            src.reserve(8_usize.saturating_sub(src.len()));

            return Ok(None);
        }

        let size = src.get_u64_le();

        // Decode block size.
        if src.len() < 4 {
            src.reserve(4_usize.saturating_sub(src.len()));

            return Ok(None);
        }

        let block_size = src.get_u32_le();

        // Decode blocks.
        let num_blocks = size as usize / block_size as usize;

        let block_size_bytes = num_blocks * block_size as usize;
        src.reserve(block_size_bytes.saturating_sub(src.len()));

        let mut block_info_decoder = BlockInfoDecoder;
        let mut blocks = Vec::with_capacity(num_blocks);
        for _ in 0..num_blocks {
            let Some(block_info) = block_info_decoder.decode(src)? else {
               return Ok(None);
            };

            blocks.push(block_info);
        }

        // Return object.
        Ok(Some(FileInfo::new(path.into(), size, block_size, blocks)))
    }
}
