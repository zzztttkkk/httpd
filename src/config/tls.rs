use crate::uitls::anyhow;
use serde::Deserialize;

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
    pub(crate) fn autofix(&mut self, root: Option<&Self>) -> anyhow::Result<()> {
        match root {
            Some(root) => {
                if self.timeout.is_zero() {
                    self.timeout = root.timeout.clone();
                }
            }
            None => {}
        }

        if self.timeout.is_zero() {
            self.timeout = DurationInMillis(std::time::Duration::from_secs(15));
        }
        Ok(())
    }

    pub(crate) fn load(&self) -> anyhow::Result<Option<tokio_rustls::rustls::ServerConfig>> {
        if self.cert.is_empty() && self.key.is_empty() {
            return Ok(None);
        }

        let mut certs = vec![];
        for v in rustls_pemfile::certs(&mut std::io::BufReader::new(anyhow::result(
            std::fs::File::open(&self.cert),
        )?)) {
            certs.push(anyhow::result(v)?);
        }

        let key = anyhow::result(rustls_pemfile::private_key(&mut std::io::BufReader::new(
            anyhow::result(std::fs::File::open(&self.key))?,
        )))?;
        let key = anyhow::option(key, "none key")?;

        let cfg = anyhow::result(
            tokio_rustls::rustls::ServerConfig::builder()
                .with_no_client_auth()
                .with_single_cert(certs, Into::into(key)),
        )?;
        Ok(Some(cfg))
    }
}
