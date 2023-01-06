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
        cert_filename: Option<String>,
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

        Command::Listen {
            address,
            cert_filename,
            private_key_filename,
        } => listen(&address, cert_filename, private_key_filename).await?,

        Command::Connect { address } => connect(&address).await?,
    }

    Ok(())
}
