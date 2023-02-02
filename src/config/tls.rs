use std::{fs::File, io::BufReader, path::Path};

use serde::Deserialize;
use tokio_rustls::rustls::{Certificate, PrivateKey, ServerConfig};

#[derive(Deserialize, Clone, Default)]
pub struct ConfigTLS {
    #[serde(default)]
    pub cert: String,

    #[serde(default)]
    pub key: String,
}

impl ConfigTLS {
    pub fn autofix(&mut self) {}

    pub fn load(&self) -> Option<ServerConfig> {
        if self.cert.is_empty() && self.key.is_empty() {
            return None;
        }

        let mut certs = Vec::new();
        for e in rustls_pemfile::certs(&mut BufReader::new(
            File::open(Path::new(self.cert.as_str())).unwrap(),
        ))
        .unwrap()
        {
            certs.push(Certificate(e));
        }
        let mut keys = Vec::new();
        for e in rustls_pemfile::pkcs8_private_keys(&mut BufReader::new(
            File::open(Path::new(self.key.as_str())).unwrap(),
        ))
        .unwrap()
        {
            keys.push(PrivateKey(e));
        }
        if keys.is_empty() {
            for e in rustls_pemfile::rsa_private_keys(&mut BufReader::new(
                File::open(Path::new(self.key.as_str())).unwrap(),
            ))
            .unwrap()
            {
                keys.push(PrivateKey(e));
            }
        }
        return Some(
            ServerConfig::builder()
                .with_safe_defaults()
                .with_no_client_auth()
                .with_single_cert(certs, keys.remove(0))
                .unwrap(),
        );
    }
}
