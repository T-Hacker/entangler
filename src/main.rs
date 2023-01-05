mod messages;

use clap::Parser;
use color_eyre::eyre::Result;
use futures::SinkExt;
use messages::HelloMessage;
use tokio::net::UdpSocket;
use tokio_stream::StreamExt;
use tokio_util::udp::UdpFramed;
use tracing::*;

#[derive(Debug, Parser)]
#[command(author, version, about)]
enum Command {
    /// Generate self signed certificate.
    GenerateCertificate {
        cert_filename: Option<String>,
        private_key_filename: Option<String>,
    },

    /// Listen for incoming connections.
    Listen {
        /// Address to listen on.
        address: String,
    },

    /// Connect to another client.
    Connect {
        /// Client address.
        address: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging.
    tracing_subscriber::fmt().init();

    // Initialize error report.
    color_eyre::install()?;

    // Handle arguments.
    let command = Command::parse();
    match command {
        Command::GenerateCertificate {
            cert_filename,
            private_key_filename,
        } => generate_self_signed_cert(cert_filename, private_key_filename).await?,
        Command::Listen { address } => listen(&address).await?,
        Command::Connect { address } => connect(&address).await?,
    }

    Ok(())
}

async fn generate_self_signed_cert(
    cert_filename: Option<String>,
    private_key_filename: Option<String>,
) -> Result<()> {
    let cert_filename = cert_filename.unwrap_or_else(|| "cert".to_string());
    let private_key_filename = private_key_filename.unwrap_or_else(|| "private_key".to_string());

    // Generate certificate and private key.
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".to_string()])?;
    let private_key_der = cert.serialize_private_key_der();

    // Save certificate.
    let cert_der = cert.serialize_der()?;
    tokio::fs::write(cert_filename, &cert_der).await?;

    // Save private key file.
    tokio::fs::write(private_key_filename, &private_key_der).await?;

    Ok(())
}

async fn connect(address: &str) -> Result<()> {
    // Create a socket in a random port.
    let socket = UdpSocket::bind("0.0.0.0:0").await?;

    // Encapsulate the socket with a message encoder.
    let encoder = messages::encoder::HelloMessageEncoder;
    let mut framed = UdpFramed::new(socket, encoder);

    // Send hello message.
    let message = HelloMessage::new(123, "test_client".to_string(), "0.0.1".to_string());
    framed.send((&message, address.parse()?)).await?;

    Ok(())
}

async fn listen(address: &str) -> Result<()> {
    // Start listening for incoming connections.
    let socket = UdpSocket::bind(address).await?;

    // Encapsulate the socket with a message decoder.
    let decoder = messages::decoders::HelloMessageDecoder;
    let mut framed = UdpFramed::new(socket, decoder);

    // Process incoming messages.
    while let Some(frame) = framed.next().await {
        match frame {
            Ok((hello_frame, address)) => {
                dbg!(address);
                dbg!(hello_frame);
            }

            Err(e) => {
                error!("Fail to receive message: {e:?}");

                continue;
            }
        }
    }

    Ok(())
}
