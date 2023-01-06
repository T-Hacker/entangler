use color_eyre::eyre::Result;
use rustls::PrivateKey;
use std::{fs::File, io::BufReader, path::Path};

pub async fn generate_self_signed_cert(
    server_name: String,
    cert_filename: Option<String>,
    private_key_filename: Option<String>,
) -> Result<()> {
    // Get certificate filenames.
    let cert_filename = certificate_filename_or_default(cert_filename);
    let private_key_filename = private_key_filename_or_default(private_key_filename);

    // Generate certificate and private key.
    let cert = rcgen::generate_simple_self_signed(vec![server_name])?;
    let private_key = cert.serialize_private_key_pem();

    // Save certificate.
    let cert = cert.serialize_pem()?;
    tokio::fs::write(cert_filename, &cert).await?;

    // Save private key file.
    tokio::fs::write(private_key_filename, &private_key).await?;

    Ok(())
}

pub fn certificate_filename_or_default(cert_filename: Option<String>) -> String {
    cert_filename.unwrap_or_else(|| "cert".to_string())
}

pub fn private_key_filename_or_default(private_key_filename: Option<String>) -> String {
    private_key_filename.unwrap_or_else(|| "private_key".to_string())
}

pub fn read_certs_from_file<P>(certificate_path: P) -> Result<Vec<rustls::Certificate>>
where
    P: AsRef<Path>,
{
    let mut cert_chain_reader = BufReader::new(File::open(certificate_path)?);
    let certs = rustls_pemfile::certs(&mut cert_chain_reader)?
        .into_iter()
        .map(rustls::Certificate)
        .collect();

    Ok(certs)
}

pub fn read_private_key_from_file<P>(private_key_path: P) -> Result<PrivateKey>
where
    P: AsRef<Path>,
{
    let mut key_reader = BufReader::new(File::open(private_key_path)?);
    // if the file starts with "BEGIN RSA PRIVATE KEY"
    // let mut keys = rustls_pemfile::rsa_private_keys(&mut key_reader)?;
    // if the file starts with "BEGIN PRIVATE KEY"
    let mut keys = rustls_pemfile::pkcs8_private_keys(&mut key_reader)?;

    assert_eq!(keys.len(), 1);
    let key = rustls::PrivateKey(keys.remove(0));

    Ok(key)
}
