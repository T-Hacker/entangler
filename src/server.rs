use crate::{
    certificate::*,
    file_cache::FileCache,
    messages::{Message, MessageDecoder, MessageEncoder},
};
use color_eyre::eyre::Result;
use futures::TryStreamExt;
use quinn::{Endpoint, RecvStream, SendStream, ServerConfig};
use std::{net::SocketAddr, path::PathBuf};
use tokio::sync::broadcast;
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
    let (watcher_tx, _) = broadcast::channel(16);
    let watcher_tx_clone = watcher_tx.clone();
    let mut watcher = notify::recommended_watcher(move |res: Result<notify::Event, _>| {
        dbg!(&res);
        let event = match res {
            Ok(event) => event,
            Err(e) => {
                error!("Watcher error: {e:?}");

                return;
            }
        };

        watcher_tx_clone.send(event).unwrap_or_default();
    })?;
    // watcher.watch(&source_path, RecursiveMode::Recursive)?;

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
        {
            let connection = connection.clone();
            tokio::spawn(async move {
                let remote_address = connection.remote_address();

                match handle_client(remote_address, send, recv).await {
                    Ok(()) => info!("Client closed connection {}.", remote_address),
                    Err(e) => error!("Error handling client {}: {}", remote_address, e),
                }
            });
        }

        connection.closed().await;
    }

    Ok(())
}

async fn handle_client(
    remote_address: SocketAddr,
    send: SendStream,
    recv: RecvStream,
) -> Result<()> {
    let mut write_framed = FramedWrite::new(send, MessageEncoder);
    let mut read_framed = FramedRead::new(recv, MessageDecoder);

    //
    loop {
        let Some(message) = read_framed.try_next().await? else {
            break;
        };

        tokio::spawn(async move {
            match message {
                Message::WatcherEvent(notify_event) => match notify_event.kind {
                    // notify::EventKind::Modify(_) => todo!(),
                    notify::EventKind::Remove(_) => {
                        for path in &notify_event.paths {
                            if path.is_dir() {
                                if let Err(e) = tokio::fs::remove_dir_all(path).await {
                                    error!("Fail to remove folder: {e:?}");
                                }
                            }
                        }
                    }

                    _ => warn!("Not handling this watcher event: {notify_event:#?}"),
                },
            }
        });
    }

    //
    info!("Client ({remote_address}) disconnected successfully.");

    Ok(())
}
