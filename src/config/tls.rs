use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;

use super::duration_in_millis::DurationInMillis;

#[derive(Deserialize, Clone, Default, Debug)]
pub(crate) struct TlsConfig {
    #[serde(default)]
    pub cert: String,

    #[serde(default)]
    pub key: String,

    #[serde(default)]
    pub timeout: DurationInMillis,
}

impl TlsConfig {
    pub(crate) fn autofix(&mut self) {}

    pub(crate) fn load(&self) -> Option<tokio_rustls::rustls::ServerConfig> {
        if self.cert.is_empty() && self.key.is_empty() {
            return None;
        }

        let msg = format!(
            "httpd.config: failed to load tls key from `{}`, `{}`",
            self.cert, self.key
        );
        let msg = &msg;

        let certs = rustls_pemfile::certs(&mut BufReader::new(File::open(&self.cert).expect(msg)))
            .map(|v| v.expect(msg))
            .collect();

        let key =
            rustls_pemfile::private_key(&mut BufReader::new(File::open(&self.key).expect(msg)))
                .expect(msg)
                .expect(msg);

        return Some(
            tokio_rustls::rustls::ServerConfig::builder()
                .with_no_client_auth()
                .with_single_cert(certs, Into::into(key))
                .expect(msg),
        );
    }
}
