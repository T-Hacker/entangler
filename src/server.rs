use crate::{
    certificate::{
        certificate_filename_or_default, private_key_filename_or_default, read_certs_from_file,
        read_private_key_from_file,
    },
    messages::{decoders::HelloMessageDecoder, encoder::HelloMessageEncoder, HelloMessage},
    server,
};
use color_eyre::eyre::{eyre, Result};
use futures::{SinkExt, TryStreamExt};
use quinn::{Endpoint, RecvStream, SendStream, ServerConfig};
use std::net::SocketAddr;
use tokio_util::codec::{FramedRead, FramedWrite};
use tracing::*;

pub async fn listen(
    address: &str,
    cert_filename: Option<String>,
    private_key_filename: Option<String>,
) -> Result<()> {
    // Create server connection configuration.
    let cert_filename = certificate_filename_or_default(cert_filename);
    let private_key_filename = private_key_filename_or_default(private_key_filename);
    let certs = read_certs_from_file(cert_filename)?;
    let private_key = read_private_key_from_file(private_key_filename)?;
    let server_config = ServerConfig::with_single_cert(certs, private_key)?;

    // Create the endpoint to start receiving connections.
    let endpoint = Endpoint::server(server_config, address.parse()?)?;

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
        let (send, recv) = match connection.open_bi().await {
            Ok(ch) => ch,
            Err(e) => {
                error!("Fail to open incoming connection: {e:?}");

                continue;
            }
        };

        // Create a task to handle client requests.
        let remote_address = connection.remote_address();
        tokio::spawn(async move {
            match handle_client(remote_address, send, recv).await {
                Ok(()) => info!("Client closed connection {}.", remote_address),
                Err(e) => error!(
                    "Error handling client {}: {}",
                    connection.remote_address(),
                    e
                ),
            }
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

    info!("Received hello message from client ({remote_address}): {hello_message:?}");

    // Respond to the client with an hello message.
    let mut framed = FramedWrite::new(send, HelloMessageEncoder);
    let hello_message = HelloMessage::new(123, "server_test".to_string(), "0.0.1".to_string());
    framed.send(&hello_message).await?;
    framed.flush().await?;

    Ok(())
}
