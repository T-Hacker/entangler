use super::block::Block;
use color_eyre::eyre::{eyre, Result};
use std::path::{Path, PathBuf};
use tokio::{
    fs::File,
    io::{AsyncReadExt, BufReader},
};
use tracing::info;

#[derive(Debug)]
pub struct IndexEntry {
    path: PathBuf,
    blocks: Vec<Block>,
}

impl IndexEntry {
    pub async fn new<P>(file_path: P) -> Result<Self>
    where
        P: Into<PathBuf>,
    {
        // Skip folders.
        let file_path = file_path.into();
        if file_path.is_dir() {
            return Err(eyre!("Not a file path: {file_path:?}"));
        }

        // Calculate block size.
        let file = loop {
            match File::open(&file_path).await {
                Ok(file) => break file,
                Err(e) => {
                    if let Some(error_code) = e.raw_os_error() {
                        if error_code == 24 {
                            tokio::task::yield_now().await;

                            continue;
                        }
                    };

                    return Err(eyre!("Fail to open file: {e:?}"));
                }
            }
        };
        let metadata = file.metadata().await?;
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
            let bytes_read = file_buffer.read_exact(&mut buffer[..bytes_to_read]).await?;
            info!("Path: {file_path:?} Bytes read: {bytes_read} Bytes to read: {bytes_to_read} File size: {file_size}");

            // Create block.
            let offset = file_size - total_bytes_to_read;
            blocks.push(Block::new(offset, &buffer[..bytes_read]));

            // Decrement the number of bytes to read.
            total_bytes_to_read -= bytes_read as u64;
        }

        // Return index.
        Ok(Self {
            path: file_path.into(),
            blocks,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}
