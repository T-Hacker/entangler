use crate::messages::{BlockInfo, FileInfo};
use std::{
    fs::FileType,
    path::Path,
    sync::atomic::{AtomicU32, Ordering},
};
use tokio::sync::mpsc;
use tracing::*;
use walkdir::WalkDir;

pub async fn scrape(
    path: impl AsRef<Path>,
    file_info_tx: mpsc::Sender<FileInfo>,
    block_info_tx: mpsc::Sender<BlockInfo>,
) {
    let next_file_id = AtomicU32::new(0);

    for entry in WalkDir::new(path).into_iter() {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                error!("Fail to walk entry: {e:?}");

                continue;
            }
        };

        // Filter anything that is not a file.
        if !entry.file_type().is_file() {
            continue;
        }

        let file_info_tx = file_info_tx.clone();
        tokio::spawn(async move {
            // Create and send file info.
            let id = next_file_id.fetch_add(1, Ordering::SeqCst);

            let path = entry.path();

            let metadata = match entry.metadata() {
                Ok(metadata) => metadata,
                Err(e) => {
                    error!("Fail to aquire metadata: {e:?}");

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
            let file_info = FileInfo::new(id, path.to_owned(), size, number_blocks);

            file_info_tx.send(file_info).await.unwrap();

            // Create and send block info data.
            todo!();
        });
    }

    todo!();
}
