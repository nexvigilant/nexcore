use crate::error::{NexCloudError, Result};
use crate::manifest::TlsDef;
use rustls::ServerConfig;
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use std::io::BufReader;
use std::sync::Arc;

/// Load TLS configuration from PEM certificate and key files.
///
/// Tier: T2-P (∂ Boundary) — cryptographic boundary enforcement.
pub fn load_tls_config(tls_def: &TlsDef) -> Result<Arc<ServerConfig>> {
    // Load certificates
    let cert_file = std::fs::File::open(&tls_def.cert).map_err(|e| {
        NexCloudError::TlsConfig(format!(
            "cannot open cert '{}': {e}",
            tls_def.cert.display()
        ))
    })?;
    let mut cert_reader = BufReader::new(cert_file);
    let certs: Vec<CertificateDer<'static>> = rustls_pemfile::certs(&mut cert_reader)
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| NexCloudError::TlsConfig(format!("failed to parse certs: {e}")))?;

    if certs.is_empty() {
        return Err(NexCloudError::TlsConfig(
            "no certificates found in cert file".to_string(),
        ));
    }

    // Load private key
    let key_file = std::fs::File::open(&tls_def.key).map_err(|e| {
        NexCloudError::TlsConfig(format!("cannot open key '{}': {e}", tls_def.key.display()))
    })?;
    let mut key_reader = BufReader::new(key_file);
    let key: PrivateKeyDer<'static> = rustls_pemfile::private_key(&mut key_reader)
        .map_err(|e| NexCloudError::TlsConfig(format!("failed to parse key: {e}")))?
        .ok_or_else(|| NexCloudError::TlsConfig("no private key found in key file".to_string()))?;

    // Build server config with safe defaults
    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|e| NexCloudError::TlsConfig(format!("TLS config error: {e}")))?;

    Ok(Arc::new(config))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::TlsDef;
    use std::path::PathBuf;

    #[test]
    fn missing_cert_file_returns_error() {
        let def = TlsDef {
            cert: PathBuf::from("/nonexistent/cert.pem"),
            key: PathBuf::from("/nonexistent/key.pem"),
        };
        let result = load_tls_config(&def);
        assert!(result.is_err());
    }
}
