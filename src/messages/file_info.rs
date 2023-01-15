use bytes::{Buf, BufMut};
use std::{
    io::ErrorKind,
    path::{Path, PathBuf},
};
use tokio_util::codec::{Decoder, Encoder};

#[derive(Debug, PartialEq, Eq)]
pub struct FileInfo {
    id: u32,
    path: PathBuf,
    size: u64,
    number_blocks: u32,
}

impl FileInfo {
    pub fn new(id: u32, path: PathBuf, size: u64, number_blocks: u32) -> Self {
        Self {
            id,
            path,
            size,
            number_blocks,
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn number_blocks(&self) -> u32 {
        self.number_blocks
    }
}

pub struct FileInfoEncoder;

impl Encoder<&FileInfo> for FileInfoEncoder {
    type Error = std::io::Error;

    fn encode(&mut self, item: &FileInfo, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        // Write file id.
        dst.put_u32_le(item.id);

        // Write file path.
        let path = item.path.to_str().ok_or_else(|| {
            std::io::Error::new(ErrorKind::Other, "Fail to convert path to string.")
        })?;
        dst.put_u16_le(path.len() as u16);
        dst.put(path.as_bytes());

        // Write file size.
        dst.put_u64_le(item.size);

        // Write number of blocks.
        dst.put_u32_le(item.number_blocks);

        Ok(())
    }
}

pub struct FileInfoDecoder;

impl Decoder for FileInfoDecoder {
    type Item = FileInfo;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Read file id.
        if src.len() < 4 {
            src.reserve(4_usize.saturating_sub(src.len()));

            return Ok(None);
        }

        let id = src.get_u32_le();

        // Read file path.
        if src.len() < 2 {
            src.reserve(2_usize.saturating_sub(src.len()));

            return Ok(None);
        }

        let path_len = src.get_u16_le() as usize;
        if src.len() < path_len {
            src.reserve(path_len.saturating_sub(src.len()));

            return Ok(None);
        }

        let path = src.split_to(path_len);
        let path = path.to_vec();
        let path = String::from_utf8(path).map_err(|e| {
            std::io::Error::new(
                ErrorKind::Other,
                format!("Unable to encode path string: {e:?}"),
            )
        })?;
        let path = PathBuf::from(path);

        // Read file size.
        if src.len() < 8 {
            src.reserve(8_usize.saturating_sub(src.len()));

            return Ok(None);
        }

        let size = src.get_u64_le();

        // Read number of blocks.
        if src.len() < 4 {
            src.reserve(4_usize.saturating_sub(src.len()));

            return Ok(None);
        }

        let number_blocks = src.get_u32_le();

        // Return object.
        Ok(Some(FileInfo {
            id,
            path,
            size,
            number_blocks,
        }))
    }
}
