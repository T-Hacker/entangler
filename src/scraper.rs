use crate::messages::{BlockInfo, FileInfo};
use rayon::prelude::*;
use std::{
    fs::File,
    io::{BufReader, ErrorKind, Read},
    path::PathBuf,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
};
use tokio::sync::mpsc;
use tracing::*;
use walkdir::WalkDir;

pub async fn scrape(
    path: PathBuf,
    file_info_tx: mpsc::Sender<Result<FileInfo, std::io::Error>>,
    block_info_tx: mpsc::Sender<Result<BlockInfo, std::io::Error>>,
) {
    info!("Starting to scrape: {path:?}");

    let next_file_id = Arc::new(AtomicU32::new(0));

    let entries = WalkDir::new(path).into_iter().par_bridge();
    entries.for_each(|entry| {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                error!("Fail to walk entry: {e:?}");

                return;
            }
        };

        // Filter anything that is not a file.
        if !entry.file_type().is_file() {
            return;
        }

        let file_info_tx = file_info_tx.clone();
        let block_info_tx = block_info_tx.clone();
        let next_file_id = next_file_id.clone();

        // Create and send file info.
        let id = next_file_id.fetch_add(1, Ordering::SeqCst);

        let path = entry.path();

        let metadata = match entry.metadata() {
            Ok(metadata) => metadata,
            Err(e) => {
                file_info_tx
                    .blocking_send(Err(std::io::Error::new(
                        ErrorKind::Other,
                        format!("Fail to read metadata for file {path:?}: {e:?}"),
                    )))
                    .unwrap();

                return;
            }
        };
        let size = metadata.len();

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

        let number_blocks = f32::ceil(size as f32 / block_size as f32) as u32;
        let file_info = FileInfo::new(
            path.to_owned(),
            size,
            number_blocks,
            block_size,
            metadata.modified().unwrap(),
        );
        let path_id = file_info.calculate_hash_path();

        file_info_tx.blocking_send(Ok(file_info)).unwrap();

        // Create and send block info data.
        let file = match File::open(path) {
            Ok(file) => file,
            Err(e) => {
                file_info_tx
                    .blocking_send(Err(std::io::Error::new(
                        ErrorKind::Other,
                        format!("Fail to open file {path:?}: {e:?}"),
                    )))
                    .unwrap();

                return;
            }
        };

        let mut file = BufReader::with_capacity(block_size as usize, file);
        let mut buffer = vec![0u8; block_size as usize];
        let mut offset = 0;
        loop {
            let bytes_read = match file.read(&mut buffer) {
                Ok(byte_read) => byte_read,
                Err(e) => {
                    block_info_tx.blocking_send(Err(std::io::Error::new(ErrorKind::Other, format!("Fail to process block with offset {offset} of file {path:?}: {e:?}")))).unwrap(); // Should never fail.

                    continue;
                }
            };

            // Check if we reached the end of the file.
            if bytes_read == 0 {
                break;
            }

            // Create block info object.
            let block_info = BlockInfo::from_buffer(&buffer[0..bytes_read], path_id, offset);

            // Send block info.
            block_info_tx.blocking_send(Ok(block_info)).unwrap();

            // Advance offset.
            offset += bytes_read as u64;
        }
    });
}
