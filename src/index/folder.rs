use super::IndexEntry;
use std::{collections::HashMap, path::PathBuf};

#[derive(Debug)]
pub struct FolderIndex {
    entries: HashMap<PathBuf, IndexEntry>,
}

impl FolderIndex {
    pub fn with_entires(entries: HashMap<PathBuf, IndexEntry>) -> Self {
        Self { entries }
    }
}
