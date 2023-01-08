use crate::index::BlockInfo;
use bytes::BufMut;
use tokio_util::codec::Encoder;

pub struct BlockInfoEncoder;

impl Encoder<&BlockInfo> for BlockInfoEncoder {
    type Error = std::io::Error;

    fn encode(&mut self, item: &BlockInfo, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        // Reserve space for performance reasons.
        const SIZE: usize = std::mem::size_of::<BlockInfo>();
        dst.reserve(SIZE.saturating_sub(dst.len()));

        // Write the block offset.
        dst.put_u64_le(item.offset());

        // Write block size.
        dst.put_u32_le(item.block_size());

        // Write block CRC32.
        dst.put_u32_le(item.crc32());

        // Write block SHA3.
        dst.put_slice(item.sha3());

        Ok(())
    }
}
