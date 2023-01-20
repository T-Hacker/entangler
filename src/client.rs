use crate::{
    certificate::{certificate_filename_or_default, read_certs_from_file},
    messages::{Message, MessageEncoder},
};
use color_eyre::eyre::Result;
use futures::SinkExt;
use notify::{RecursiveMode, Watcher};
use quinn::{ClientConfig, Endpoint};
use rustls::RootCertStore;
use std::{path::PathBuf, sync::Arc};
use tokio::sync::mpsc;
use tokio_util::codec::FramedWrite;
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
    let endpoint = Arc::new(Endpoint::client(local_address)?);

    // Setup file watcher.
    let (watcher_tx, mut watcher_rx) = mpsc::channel(16);
    let mut watcher = notify::recommended_watcher(move |res: Result<notify::Event, _>| {
        dbg!(&res);
        let event = match res {
            Ok(event) => event,
            Err(e) => {
                error!("Watcher error: {e:?}");

                return;
            }
        };

        // Send event.
        watcher_tx.blocking_send(event).unwrap();
    })?;

    watcher.watch(&source_path, RecursiveMode::Recursive)?;

    //
    let remote_address = address.parse()?;
    while let Some(event) = watcher_rx.recv().await {
        // Connect to the server.
        let connection = endpoint
            .connect_with(client_config.clone(), remote_address, &server_name)?
            .await?;
        let (send, _) = connection.open_bi().await?;

        // Wrap connection with codecs.
        let mut write_framed = FramedWrite::new(send, MessageEncoder);
        // let read_framed = FramedRead::new(recv, MessageDecoder);

        // Send event message.
        write_framed
            .send(&Message::WatcherEvent(event))
            .await
            .unwrap();

        write_framed.close().await.unwrap();
    }

    // //
    // tokio::signal::ctrl_c().await?;

    // Wait for server to clean up.
    endpoint.wait_idle().await;

    Ok(())
}
