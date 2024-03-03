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

    pub(crate) fn load(&self) -> anyhow::Result<Option<boring::ssl::SslAcceptor>> {
        let server = boring::ssl::SslMethod::tls_server();
        let mut builder = anyhow::result(boring::ssl::SslAcceptor::mozilla_modern(server))?;
        _ = anyhow::result(
            builder.set_certificate_file(self.cert.as_str(), boring::ssl::SslFiletype::PEM),
        )?;
        _ = anyhow::result(
            builder.set_private_key_file(self.key.as_str(), boring::ssl::SslFiletype::PEM),
        )?;
        builder.set_verify(boring::ssl::SslVerifyMode::PEER);
        Ok(Some(builder.build()))
    }
}
