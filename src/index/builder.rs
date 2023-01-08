use super::{FileInfo, FolderIndex};
use futures::{stream::FuturesUnordered, StreamExt};
use jwalk::WalkDir;
use std::{collections::HashMap, path::PathBuf};
use tracing::*;

pub struct IndexBuilder {
    path: PathBuf,
}

impl IndexBuilder {
    pub fn from_path<P>(folder_path: P) -> Self
    where
        P: Into<PathBuf>,
    {
        Self {
            path: folder_path.into(),
        }
    }

    pub async fn build(&self) -> FolderIndex {
        // Walk the file system and create index entries.
        let mut entries: FuturesUnordered<_> = WalkDir::new(&self.path)
            .into_iter()
            .filter_map(|entry| {
                // Report file entry errors.
                let entry = match entry {
                    Ok(entry) => entry,
                    Err(e) => {
                        error!("Fail to read folder entry: {e:?}");

                        return None;
                    }
                };

                // Skip folders.
                let path = entry.path();
                if path.is_dir() {
                    return None;
                }

                // Create index entry.
                Some(FileInfo::new(path))
            })
            .collect();

        // Create folder index.
        let mut index_entries = HashMap::with_capacity(entries.len());
        while let Some(entry) = entries.next().await {
            let entry = match entry {
                Ok(entry) => entry,
                Err(e) => {
                    error!("Fail to process index entry: {e:?}");

                    continue;
                }
            };

            let file_path = entry.path();
            let entry = match FileInfo::new(file_path).await {
                Ok(entry) => entry,
                Err(e) => {
                    error!("Fail to create index entry: {e:?}");

                    continue;
                }
            };

            index_entries.insert(file_path.to_path_buf(), entry);
        }

        FolderIndex::with_entires(index_entries)
    }
}
