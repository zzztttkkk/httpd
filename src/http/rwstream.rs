use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpStream,
};
use tokio_rustls::client::TlsStream as CliTlsStream;
use tokio_rustls::server::TlsStream;

pub trait RwStream: AsyncRead + AsyncWrite + Unpin + Send + Sync {}

impl RwStream for TcpStream {}

impl RwStream for TlsStream<TcpStream> {}

impl RwStream for CliTlsStream<TcpStream> {}
