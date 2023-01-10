use super::block_info::BlockInfo;
use color_eyre::eyre::{eyre, Result};
use std::{
    fs::File,
    io::{BufReader, Read},
    path::{Path, PathBuf},
};
use tracing::*;

#[derive(Debug, PartialEq, Eq)]
pub struct FileInfo {
    path: PathBuf,
    size: u64,
    block_size: u32,
    blocks: Vec<BlockInfo>,
}

impl FileInfo {
    pub fn new(path: PathBuf, size: u64, block_size: u32, blocks: Vec<BlockInfo>) -> Self {
        Self {
            path,
            size,
            block_size,
            blocks,
        }
    }

    pub fn with_file_path<P>(file_path: P) -> Result<Self>
    where
        P: Into<PathBuf>,
    {
        // Skip folders.
        let file_path = file_path.into();
        if file_path.is_dir() {
            return Err(eyre!("Not a file path: {file_path:?}"));
        }

        // Calculate block size.
        let file = File::open(&file_path)?;
        let metadata = file.metadata()?;
        let file_size = metadata.len();
        let block_size: u32 = if file_size < 250 * 1024 * 1024 {
            128 * 1024
        } else if file_size < 500 * 1024 * 1024 {
            256 * 1024
        } else if file_size < 1 * 1024 * 1024 * 1024 {
            512 * 1024
        } else if file_size < 2 * 1024 * 1024 * 1024 {
            1 * 1024 * 1024
        } else if file_size < 4 * 1024 * 1024 * 1024 {
            2 * 1024 * 1024
        } else if file_size < 8 * 1024 * 1024 * 1024 {
            4 * 1024 * 1024
        } else if file_size < 16 * 1024 * 1024 * 1024 {
            8 * 1024 * 1024
        } else {
            16 * 1024 * 1024
        };

        // Calculate block signatures.
        let num_blocks = (file_size / block_size as u64 + 1) as usize;
        let mut blocks = Vec::with_capacity(num_blocks);
        let mut file_buffer = BufReader::with_capacity(block_size as usize, file);
        let mut buffer = vec![0u8; block_size as usize];
        let mut total_bytes_to_read = file_size;
        loop {
            // Read file block.
            let bytes_to_read = usize::min(block_size as usize, total_bytes_to_read as usize);
            if bytes_to_read == 0 {
                break;
            }
            file_buffer.read_exact(&mut buffer[..bytes_to_read])?;

            // Create block.
            let offset = file_size - total_bytes_to_read;
            blocks.push(BlockInfo::with_buffer(offset, &buffer[..bytes_to_read]));

            // Decrement the number of bytes to read.
            total_bytes_to_read -= bytes_to_read as u64;
        }

        // Return index.
        Ok(Self {
            path: file_path,
            size: file_size,
            block_size,
            blocks,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn block_size(&self) -> u32 {
        self.block_size
    }

    pub fn blocks(&self) -> &[BlockInfo] {
        &self.blocks
    }
}
