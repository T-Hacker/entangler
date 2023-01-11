use crate::{
    certificate::{certificate_filename_or_default, read_certs_from_file},
    messages::{
        decoders::{FolderIndexDecoder, HelloMessageDecoder},
        encoder::HelloMessageEncoder,
        HelloMessage,
    },
    MAGIC_NUMBER, NAME, VERSION,
};
use color_eyre::eyre::{eyre, Result};
use futures::{SinkExt, TryStreamExt};
use quinn::{ClientConfig, Endpoint};
use rustls::RootCertStore;
use std::path::PathBuf;
use tokio_util::codec::{FramedRead, FramedWrite};
use tracing::*;

pub async fn connect(
    server_name: &str,
    address: &str,
    certificate_path: Option<String>,
    source_path: PathBuf,
) -> Result<()> {
    // Try to resolve relative source paths.
    let source_path = source_path.canonicalize()?;

    // Create client connection configuration.
    let certificate_path = certificate_filename_or_default(certificate_path);
    let certs = read_certs_from_file(certificate_path)?;

    let mut certificate_store = RootCertStore::empty();
    for cert in &certs {
        certificate_store.add(cert)?;
    }

    let client_config = ClientConfig::with_root_certificates(certificate_store);

    // Bind the client socket to an address.
    let local_address = "0.0.0.0:0".parse()?;
    let endpoint = Endpoint::client(local_address)?;

    // Connect to the server.
    let remote_address = address.parse()?;
    let connection = endpoint
        .connect_with(client_config, remote_address, server_name)?
        .await?;
    let (send, recv) = connection.open_bi().await?;

    // Send the hello message.
    info!("Sending hello message...");
    let mut framed = FramedWrite::new(send, HelloMessageEncoder);
    let hello_message = HelloMessage::new(MAGIC_NUMBER, NAME.to_string(), VERSION.to_string());
    framed.send(&hello_message).await?;

    // Receive the hello message from the server.
    info!("Receiving hello message...");
    let mut framed = FramedRead::new(recv, HelloMessageDecoder);
    let hello_message = framed
        .try_next()
        .await?
        .ok_or_else(|| eyre!("Hello message not received."))?;
    info!("Received hello message from server: {hello_message:?}");

    // Receive folder index from the server.
    info!("Receiving folder index...");
    let recv = framed.into_inner();
    let mut framed = FramedRead::new(recv, FolderIndexDecoder);
    let folder_index = framed
        .try_next()
        .await?
        .ok_or_else(|| eyre!("Folder index not received."))?;
    dbg!(folder_index);

    // Check if version match.
    let server_version = hello_message.version();
    if server_version != VERSION {
        warn!("Version mismatched between server ({server_version}) and client ({VERSION}).");
    }

    // Wait for server to clean up.
    endpoint.wait_idle().await;

    Ok(())
}
