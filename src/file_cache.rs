use crate::{
    messages::{BlockInfo, FileInfo},
    scraper::scrape,
};
use std::{path::PathBuf, sync::Arc};
use tokio::sync::{mpsc, RwLock};
use tracing::*;

#[derive(Default)]
pub struct FileCache {
    file_info: Vec<FileInfo>,
    block_info: Vec<BlockInfo>,
}

impl FileCache {
    pub async fn new(path: PathBuf) -> Arc<RwLock<Self>> {
        let file_cache = Arc::new(RwLock::new(FileCache::default()));

        let (file_info_tx, file_info_rx) = mpsc::channel(16);
        let (block_info_tx, block_info_rx) = mpsc::channel(16);

        // Start scrapping the file system.
        tokio::spawn(scrape(path, file_info_tx, block_info_tx));

        // Start caching file info results.
        tokio::spawn(Self::cache_file_info(file_cache.clone(), file_info_rx));

        // Start caching block info results.
        tokio::spawn(Self::cache_block_info(file_cache.clone(), block_info_rx));

        file_cache
    }

    async fn cache_file_info(
        file_cache: Arc<RwLock<Self>>,
        mut file_info_rx: mpsc::Receiver<Result<FileInfo, std::io::Error>>,
    ) {
        while let Some(file_info) = file_info_rx.recv().await {
            let file_info = match file_info {
                Ok(file_info) => file_info,
                Err(e) => {
                    error!("Fail to retrive file infomation: {e:?}");

                    continue;
                }
            };
            info!("Caching file info: {file_info:?}");

            let mut file_cache = file_cache.write().await;
            file_cache.file_info.push(file_info);
        }
    }

    async fn cache_block_info(
        file_cache: Arc<RwLock<Self>>,
        mut block_info_rx: mpsc::Receiver<Result<BlockInfo, std::io::Error>>,
    ) {
        while let Some(block_info) = block_info_rx.recv().await {
            let block_info = match block_info {
                Ok(block_info) => block_info,
                Err(e) => {
                    error!("Fail to retrive block information: {e:?}");

                    continue;
                }
            };
            // info!("Caching block info: {block_info:?}");

            let mut file_cache = file_cache.write().await;
            file_cache.block_info.push(block_info);
        }
    }
}
