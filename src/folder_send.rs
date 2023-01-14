use color_eyre::Result;
use std::path::PathBuf;
use tracing::*;
use walkdir::WalkDir;

pub fn send_folder(path: PathBuf) -> Result<()> {
    for entry in WalkDir::new(path) {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                error!("Fail to walk entry: {e:?}");

                continue;
            }
        };

        let path = entry.path();
        if path.is_dir() {
            continue;
        }

        tokio::spawn(send_file(path.to_owned()));
    }

    Ok(())
}

pub async fn send_file(path: PathBuf) -> Result<()> {
    todo!();
}
