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
    pub(crate) fn autofix(&mut self) -> Option<String> {
        None
    }

    #[cfg(feature = "rustls")]
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

    #[cfg(feature = "nativetls")]
    pub(crate) fn load(&self) -> anyhow::Result<Option<native_tls::TlsAcceptor>> {
        use std::io::Read;

        if self.cert.is_empty() && self.key.is_empty() {
            return Ok(None);
        }

        let mut certbytes = vec![];
        _ = anyhow::result(
            anyhow::result(std::fs::File::open(&self.cert))?.read_to_end(&mut certbytes),
        )?;

        let mut keybytes = vec![];
        _ = anyhow::result(
            anyhow::result(std::fs::File::open(&self.key))?.read_to_end(&mut keybytes),
        )?;

        let ident = anyhow::result(native_tls::Identity::from_pkcs8(&certbytes, &keybytes))?;
        let acceptor = anyhow::result(native_tls::TlsAcceptor::builder(ident).build())?;
        Ok(Some(acceptor))
    }
}
