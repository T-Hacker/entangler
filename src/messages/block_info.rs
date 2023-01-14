use bytes::{Buf, BufMut};
use std::{
    io::ErrorKind,
    path::{Path, PathBuf},
};
use tokio_util::codec::{Decoder, Encoder};

#[derive(Debug, PartialEq, Eq)]
pub struct BlockInfo {
    path: PathBuf,
    offset: u64,
    block_size: u32,
    hash: u128,
}

impl BlockInfo {
    pub fn new(path: PathBuf, offset: u64, block_size: u32, hash: u128) -> Self {
        Self {
            path,
            offset,
            block_size,
            hash,
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
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
        // Write path.
        let path = item.path.to_str().ok_or_else(|| {
            std::io::Error::new(ErrorKind::Other, "Fail to convert path to string.")
        })?;
        dst.put_u16_le(path.len() as u16);
        dst.put(path.as_bytes());

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
        // Read path.
        if src.len() < 2 {
            src.reserve(2_usize.saturating_sub(src.len()));

            return Ok(None);
        }

        let path_len = src.get_u16_le() as usize;
        if src.len() < path_len {
            src.reserve(path_len.saturating_sub(src.len()));

            return Ok(None);
        }

        let path = src.split_to(path_len as usize);
        let path = path.to_vec();
        let path = String::from_utf8(path).map_err(|e| {
            std::io::Error::new(
                ErrorKind::Other,
                format!("Unable to encode path string: {e:?}"),
            )
        })?;
        let path = PathBuf::from(path);

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
            path,
            offset,
            block_size,
            hash,
        }))
    }
}
