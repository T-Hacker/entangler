use crate::messages::PathId;
use md5::Digest;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

#[derive(Default)]
pub struct PathIdCache {
    path_ids: HashMap<PathId, PathBuf>,
}

impl PathIdCache {
    pub fn add_path(&self, path: impl Into<PathBuf>) {
        todo!();
    }

    pub fn get_path(&self, path_id: &PathId) -> Option<&PathId> {
        todo!();
    }

    pub fn calculate_path_id(path: impl AsRef<Path>) -> PathId {
        let mut hasher = sha3::Sha3_256::new();

        let path = path.as_ref().to_str().unwrap();
        hasher.update(path);

        let path_id = hasher.finalize();

        path_id.try_into().unwrap()
    }
}
