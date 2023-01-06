use color_eyre::eyre::Result;
use std::{fs::File, io::BufReader, path::Path};

pub async fn generate_self_signed_cert(
    cert_filename: Option<String>,
    private_key_filename: Option<String>,
) -> Result<()> {
    // Get certificate filenames.
    let cert_filename = certificate_filename_or_default(cert_filename);
    let private_key_filename = private_key_filename_or_default(private_key_filename);

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

pub fn certificate_filename_or_default(cert_filename: Option<String>) -> String {
    cert_filename.unwrap_or_else(|| "cert".to_string())
}

pub fn private_key_filename_or_default(private_key_filename: Option<String>) -> String {
    private_key_filename.unwrap_or_else(|| "private_key".to_string())
}

pub fn read_certs_from_file<P>(
    cert_filename: P,
    private_key_file: P,
) -> Result<(Vec<rustls::Certificate>, rustls::PrivateKey)>
where
    P: AsRef<Path>,
{
    let mut cert_chain_reader = BufReader::new(File::open(cert_filename)?);
    let certs = rustls_pemfile::certs(&mut cert_chain_reader)?
        .into_iter()
        .map(rustls::Certificate)
        .collect();

    let mut key_reader = BufReader::new(File::open(private_key_file)?);
    // if the file starts with "BEGIN RSA PRIVATE KEY"
    // let mut keys = rustls_pemfile::rsa_private_keys(&mut key_reader)?;
    // if the file starts with "BEGIN PRIVATE KEY"
    let mut keys = rustls_pemfile::pkcs8_private_keys(&mut key_reader)?;

    assert_eq!(keys.len(), 1);
    let key = rustls::PrivateKey(keys.remove(0));

    Ok((certs, key))
}
