use crate::{
    certificate::*,
    file_cache::FileCache,
    messages::{HelloMessage, HelloMessageDecoder, HelloMessageEncoder},
    MAGIC_NUMBER, NAME, VERSION,
};
use color_eyre::eyre::{eyre, Result};
use futures::{SinkExt, TryStreamExt};
use notify::{RecursiveMode, Watcher};
use quinn::{Endpoint, RecvStream, SendStream, ServerConfig};
use std::{net::SocketAddr, path::PathBuf};
use tokio_util::codec::{FramedRead, FramedWrite};
use tracing::*;

pub async fn listen(
    address: &str,
    cert_filename: Option<String>,
    private_key_filename: Option<String>,
    source_path: PathBuf,
) -> Result<()> {
    // Try to resolve relative source paths.
    let source_path = source_path.canonicalize()?;

    // Create server connection configuration.
    let cert_filename = certificate_filename_or_default(cert_filename);
    let private_key_filename = private_key_filename_or_default(private_key_filename);
    let certs = read_certs_from_file(cert_filename)?;
    let private_key = read_private_key_from_file(private_key_filename)?;
    let server_config = ServerConfig::with_single_cert(certs, private_key)?;

    // Create the endpoint to start receiving connections.
    let endpoint = Endpoint::server(server_config, address.parse()?)?;

    // Start indexing files.
    let file_cache = FileCache::new(source_path.clone()).await;

    // Setup file watcher.
    let mut watcher = notify::recommended_watcher(|res| match res {
        Ok(event) => info!("Watch event: {event:?}"),
        Err(e) => error!("File watcher error: {e:?}"),
    })?;
    watcher.watch(&source_path, RecursiveMode::Recursive)?;

    // Process incoming connections.
    info!("Waiting for connections...");
    while let Some(connection) = endpoint.accept().await {
        // Accept incoming connection.
        let connection = match connection.await {
            Ok(connection) => connection,
            Err(e) => {
                error!("Unable to accept incoming connection: {e:?}");

                continue;
            }
        };

        // Open incoming connection.
        let (send, recv) = match connection.accept_bi().await {
            Ok(ch) => ch,
            Err(e) => {
                error!("Fail to open incoming connection: {e:?}");

                continue;
            }
        };

        // Create a task to handle client requests.
        tokio::spawn(async move {
            let remote_address = connection.remote_address();

            match handle_client(remote_address, send, recv).await {
                Ok(()) => info!("Client closed connection {}.", remote_address),
                Err(e) => error!("Error handling client {}: {}", remote_address, e),
            }

            connection.closed().await;
        });
    }

    Ok(())
}

async fn handle_client(
    remote_address: SocketAddr,
    send: SendStream,
    recv: RecvStream,
) -> Result<()> {
    // Await the hello message.
    let mut framed = FramedRead::new(recv, HelloMessageDecoder);
    let hello_message = framed
        .try_next()
        .await?
        .ok_or_else(|| eyre!("Didn't receive hello message from client: {remote_address}."))?;

    info!("Received hello message from client ({remote_address}): {hello_message:#?}");

    // Check if version match.
    let client_version = hello_message.version();
    if client_version != VERSION {
        warn!("Version mismatched between server ({VERSION}) and client ({client_version}).");
    }

    // Respond to the client with an hello message.
    let mut framed = FramedWrite::new(send, HelloMessageEncoder);
    let hello_message = HelloMessage::new(MAGIC_NUMBER, NAME.to_string(), VERSION.to_string());

    info!("Send hello message to client ({remote_address}): {hello_message:#?}");
    framed.send(&hello_message).await?;

    Ok(())
}
