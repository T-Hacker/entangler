use super::{FileInfo, FolderIndex};
use futures::{stream::FuturesUnordered, StreamExt};
use jwalk::rayon::prelude::*;
use jwalk::WalkDir;
use std::{collections::HashMap, path::PathBuf};
use tracing::*;

pub struct FolderIndexBuilder {
    path: PathBuf,
}

impl FolderIndexBuilder {
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
        let index_entries: HashMap<PathBuf, FileInfo> = WalkDir::new(&self.path)
            .into_iter()
            .par_bridge()
            .filter_map(|entry| {
                // Report file entry errors.
                let entry = match entry {
                    Ok(entry) => entry,
                    Err(_) => return None,
                };

                // Skip folders.
                let path = entry.path();
                if path.is_dir() {
                    return None;
                }

                // Create index entry.
                match FileInfo::with_file_path(path.clone()) {
                    Ok(file_info) => Some((path, file_info)),
                    Err(e) => {
                        error!("Fail to process file ({path:?}): {e:?}");

                        None
                    }
                }
            })
            .collect();

        FolderIndex::with_entires(index_entries)
    }
}
