use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream as ClientTlsStream;
use tokio_rustls::server::TlsStream as ServerTlsStream;

pub trait AsyncStream: AsyncRead + AsyncWrite + Send + Unpin {}

impl AsyncStream for TcpStream {}

impl AsyncStream for ServerTlsStream<TcpStream> {}

impl AsyncStream for ClientTlsStream<TcpStream> {}
