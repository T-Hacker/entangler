mod certificate;
mod client;
mod messages;
mod server;

use certificate::generate_self_signed_cert;
use clap::Parser;
use client::connect;
use color_eyre::eyre::Result;
use server::listen;
use tracing::*;

#[derive(Debug, Parser)]
#[command(author, version, about)]
enum Command {
    /// Generate self signed certificate.
    GenerateCertificate {
        /// Name of the server. The client must use the same name when connecting to the server.
        server_name: String,

        /// Optional path to output the certificate.
        cert_filename: Option<String>,

        /// Optional path to output the certificate private key.
        private_key_filename: Option<String>,
    },

    /// Listen for incoming connections.
    Listen {
        /// Address to listen on.
        address: String,

        /// Connection certificate.
        cert_filename: Option<String>,

        /// Private certificate key.
        private_key_filename: Option<String>,
    },

    /// Connect to another client.
    Connect {
        /// Server name. Must match the name on the certificate.
        server_name: String,

        /// Client address.
        address: String,

        /// Connection certificate.
        cert_filename: Option<String>,
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
            server_name,
            cert_filename,
            private_key_filename,
        } => generate_self_signed_cert(server_name, cert_filename, private_key_filename).await?,

        Command::Listen {
            address,
            cert_filename,
            private_key_filename,
        } => listen(&address, cert_filename, private_key_filename).await?,

        Command::Connect {
            server_name,
            address,
            cert_filename,
        } => connect(&server_name, &address, cert_filename).await?,
    }

    Ok(())
}
