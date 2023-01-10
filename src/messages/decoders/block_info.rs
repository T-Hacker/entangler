use crate::index::BlockInfo;
use bytes::{Buf, BytesMut};
use tokio_util::codec::Decoder;

pub struct BlockInfoDecoder;

impl Decoder for BlockInfoDecoder {
    type Item = BlockInfo;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Decode block offset.
        const OFFSET_SIZE: usize = std::mem::size_of::<u64>();
        if src.len() < OFFSET_SIZE {
            src.reserve(OFFSET_SIZE.saturating_sub(src.len()));

            return Ok(None);
        }

        let offset = src.get_u64_le();

        // Decode block size.
        const BLOCK_SIZE: usize = std::mem::size_of::<u32>();
        if src.len() < BLOCK_SIZE {
            src.reserve(BLOCK_SIZE.saturating_sub(src.len()));

            return Ok(None);
        }

        let block_size = src.get_u32_le();

        // Decode CRC32.
        const CRC32_SIZE: usize = std::mem::size_of::<u32>();
        if src.len() < CRC32_SIZE {
            src.reserve(CRC32_SIZE.saturating_sub(src.len()));

            return Ok(None);
        }

        let crc32 = src.get_u32_le();

        // Decode SHA3.
        if src.len() < 32 {
            src.reserve(32_usize.saturating_sub(src.len()));

            return Ok(None);
        }

        let mut sha3 = [0u8; 32];
        src.copy_to_slice(&mut sha3);

        // Return object.
        Ok(Some(BlockInfo::new(offset, block_size, crc32, sha3)))
    }
}
