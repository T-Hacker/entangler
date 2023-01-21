use std::io::ErrorKind;

use super::file_info::PathId;
use bytes::{Buf, BufMut};
use md5::{Digest, Md5};
use tokio_util::codec::{Decoder, Encoder};

#[derive(Debug, PartialEq, Eq)]
pub struct BlockInfo {
    path_id: PathId,
    offset: u64,
    block_size: u32,
    hash: u128,
}

impl BlockInfo {
    pub fn new(path_id: PathId, offset: u64, block_size: u32, hash: u128) -> Self {
        Self {
            path_id,
            offset,
            block_size,
            hash,
        }
    }

    pub fn from_buffer(buffer: &[u8], path_id: PathId, offset: u64) -> Self {
        // Calculate block hash.
        let mut hasher = Md5::new();
        hasher.update(buffer);

        let hash = hasher.finalize();
        let hash = u128::from_le_bytes(hash.try_into().unwrap());

        // Return object.
        Self {
            path_id,
            offset,
            block_size: buffer.len() as u32,
            hash,
        }
    }

    pub fn path_id(&self) -> &PathId {
        &self.path_id
    }

    pub fn offset(&self) -> u64 {
        self.offset
    }

    pub fn block_size(&self) -> u32 {
        self.block_size
    }

    pub fn hash(&self) -> u128 {
        self.hash
    }
}

pub struct BlockInfoEncoder;

impl Encoder<&BlockInfo> for BlockInfoEncoder {
    type Error = std::io::Error;

    fn encode(&mut self, item: &BlockInfo, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        // Write file identifier.
        dst.put_slice(item.path_id());

        // Write offset.
        dst.put_u64_le(item.offset);

        // Write block size.
        dst.put_u32_le(item.block_size);

        // Write hash.
        dst.put_u128_le(item.hash);

        Ok(())
    }
}

pub struct BlockInfoDecoder;

impl Decoder for BlockInfoDecoder {
    type Item = BlockInfo;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Read path identifier.
        const PATH_ID_SIZE: usize = std::mem::size_of::<PathId>();
        if src.len() < PATH_ID_SIZE {
            src.reserve(PATH_ID_SIZE.saturating_sub(src.len()));

            return Ok(None);
        }

        let path_id = src.split_to(PATH_ID_SIZE);
        let path_id = path_id.to_vec();
        let path_id = path_id.try_into().map_err(|e| {
            std::io::Error::new(
                ErrorKind::InvalidData,
                format!("Fail to parse path id: {e:?}"),
            )
        })?;

        // Read offset.
        if src.len() < 8 {
            src.reserve(8_usize.saturating_sub(src.len()));

            return Ok(None);
        }

        let offset = src.get_u64_le();

        // Read block size.
        if src.len() < 4 {
            src.reserve(4_usize.saturating_sub(src.len()));

            return Ok(None);
        }

        let block_size = src.get_u32_le();

        // Read hash.
        if src.len() < 16 {
            src.reserve(16_usize.saturating_sub(src.len()));

            return Ok(None);
        }

        let hash = src.get_u128_le();

        // Return object.
        Ok(Some(BlockInfo {
            path_id,
            offset,
            block_size,
            hash,
        }))
    }
}
