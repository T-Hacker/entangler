use bytes::{Buf, BufMut};
use color_eyre::Result;
use md5::Digest;
use std::{
    io::ErrorKind,
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};
use tokio_util::codec::{Decoder, Encoder};

pub type PathId = [u8; 32];

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FileInfo {
    path: PathBuf,
    size: u64,
    number_blocks: u32,
    block_size: u32,
    last_modified: SystemTime,
}

impl FileInfo {
    pub fn new(
        path: PathBuf,
        size: u64,
        number_blocks: u32,
        block_size: u32,
        last_modified: SystemTime,
    ) -> Self {
        Self {
            path,
            size,
            number_blocks,
            block_size,
            last_modified,
        }
    }

    pub fn with_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let metadata = path.metadata()?;
        let size = metadata.len();
        let last_modified = metadata.modified()?;
        let block_size: u32 = if size < 250 * 1024 * 1024 {
            128 * 1024
        } else if size < 500 * 1024 * 1024 {
            256 * 1024
        } else if size < 1 * 1024 * 1024 * 1024 {
            512 * 1024
        } else if size < 2 * 1024 * 1024 * 1024 {
            1 * 1024 * 1024
        } else if size < 4 * 1024 * 1024 * 1024 {
            2 * 1024 * 1024
        } else if size < 8 * 1024 * 1024 * 1024 {
            4 * 1024 * 1024
        } else if size < 16 * 1024 * 1024 * 1024 {
            8 * 1024 * 1024
        } else {
            16 * 1024 * 1024
        };

        let number_blocks = (size as f32 / block_size as f32).ceil() as u32;

        Ok(FileInfo::new(
            path.to_owned(),
            size,
            number_blocks,
            block_size,
            last_modified,
        ))
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

    pub fn block_size(&self) -> u32 {
        self.block_size
    }

    pub fn last_modified(&self) -> &SystemTime {
        &self.last_modified
    }

    pub fn calculate_hash_path(&self) -> PathId {
        let mut hasher = sha3::Sha3_256::new();

        let path = self.path.to_str().unwrap();
        hasher.update(path);

        let path_id = hasher.finalize();

        path_id.try_into().unwrap()
    }
}

pub struct FileInfoEncoder;

impl Encoder<&FileInfo> for FileInfoEncoder {
    type Error = std::io::Error;

    fn encode(&mut self, item: &FileInfo, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
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

        // Write block size.
        dst.put_u32_le(item.block_size);

        // Write last modified date.
        let last_modified = item
            .last_modified
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        let last_modified = last_modified.as_nanos() as u64;
        dst.put_u64_le(last_modified);

        Ok(())
    }
}

pub struct FileInfoDecoder;

impl Decoder for FileInfoDecoder {
    type Item = FileInfo;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
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

        // Read block size.
        if src.len() < 4 {
            src.reserve(4_usize.saturating_sub(src.len()));

            return Ok(None);
        }

        let block_size = src.get_u32_le();

        // Read last modified date.
        if src.len() < 8 {
            src.reserve(8_usize.saturating_sub(src.len()));

            return Ok(None);
        }

        let last_modified = src.get_u64_le();
        let last_modified = Duration::from_nanos(last_modified);
        let last_modified = SystemTime::UNIX_EPOCH.checked_add(last_modified).unwrap();

        // Return object.
        Ok(Some(FileInfo {
            path,
            size,
            number_blocks,
            block_size,
            last_modified,
        }))
    }
}
