use crate::messages::{BlockInfo, FileInfo, Message, MessageDecoder, MessageEncoder};
use color_eyre::{eyre::eyre, Result};
use futures::{SinkExt, TryStreamExt};
use quinn::{RecvStream, SendStream};
use std::path::Path;
use tokio::{
    fs::File,
    io::{AsyncReadExt, BufReader},
};
use tokio_util::codec::{FramedRead, FramedWrite};
use tracing::*;

pub async fn start_file_sync(
    path: impl AsRef<Path>,
    write_framed: &mut FramedWrite<SendStream, MessageEncoder>,
    read_framed: &mut FramedRead<RecvStream, MessageDecoder>,
) -> Result<()> {
    // Send file info.
    let file_info = FileInfo::with_file(path)?;
    info!("Sending file information: {file_info:?}");

    write_framed
        .send(&Message::FileInfo(file_info.clone()))
        .await?;

    // Receive file info.
    let Some(Message::FileInfo(received_file_info)) = read_framed.try_next().await? else {
       return Err(eyre!("Did not receive file info.")) ;
    };
    info!("Received file information: {received_file_info:?}");

    // Check if we should send or receive the file based on modification date.
    if file_info.last_modified() > received_file_info.last_modified() {
        send_file_blocks(&file_info, write_framed).await?;
    } else if file_info.last_modified() < received_file_info.last_modified() {
        receive_file_blocks().await?;
    }

    Ok(())
}

pub async fn handle_file_sync(file_info: &FileInfo) -> Result<()> {
    todo!();
}

async fn send_file_blocks(
    file_info: &FileInfo,
    write_framed: &mut FramedWrite<SendStream, MessageEncoder>,
) -> Result<()> {
    info!("Star sending file blocks: {file_info:?}");

    // Open file with buffered reads.
    let block_size = file_info.block_size() as usize;
    let file = File::open(file_info.path()).await?;
    let mut file = BufReader::with_capacity(block_size, file);

    // Send block info.
    let path_id = file_info.calculate_hash_path();
    let mut buffer = vec![0u8; block_size as usize];
    let mut offset = 0;
    loop {
        let bytes_read = file.read(&mut buffer).await?;

        // Check if we reached the end of the file.
        if bytes_read == 0 {
            break;
        }

        // Create block info object.
        let block_info = BlockInfo::from_buffer(&buffer[0..bytes_read], path_id, offset);

        // Send block info.
        write_framed.send(&Message::BlockInfo(block_info));

        // Incremente offset by bytes read.
        offset += bytes_read as u64;
    }

    Ok(())
}

async fn receive_file_blocks() -> Result<()> {
    todo!();
}
