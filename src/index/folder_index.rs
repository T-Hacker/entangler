use super::FileInfo;
use std::{collections::HashMap, path::PathBuf};

#[derive(Debug, PartialEq, Eq)]
pub struct FolderIndex {
    entries: HashMap<PathBuf, FileInfo>,
}

impl FolderIndex {
    pub fn with_entires(entries: HashMap<PathBuf, FileInfo>) -> Self {
        Self { entries }
    }

    pub fn entries(&self) -> &HashMap<PathBuf, FileInfo> {
        &self.entries
    }
}
