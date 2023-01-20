use color_eyre::Result;
use tokio::fs;

pub async fn sync_file_request(file: &mut fs::File) -> Result<()> {
    // Find out when was the file last modified.
    let metadata = file.metadata().await?;
    let last_modified = metadata.modified()?;

    Ok(())
}

pub async fn sync_file_response(file: &mut fs::File) -> Result<()> {
    todo!();
}
