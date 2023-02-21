use std::{future::Future, pin::Pin, sync::Arc};

use tokio::io::{AsyncRead, AsyncWrite, BufStream};

use super::message::Message;

pub trait ReadPart {
    fn read(&mut self) -> Pin<Box<dyn Future<Output = Message>>>;
}

pub trait WritePart {
    fn send(&mut self, msg: Message) -> Pin<Box<dyn Future<Output = ()>>>;
}

pub trait WsConn: ReadPart + WritePart {
    fn split(&mut self) -> (Box<dyn ReadPart>, Box<dyn WritePart>);
}

pub struct Conn<RW: AsyncRead + AsyncWrite + 'static> {
    stream: Option<BufStream<RW>>,
}

impl<RW: AsyncRead + AsyncWrite + 'static> Conn<RW> {}

impl<RW: AsyncRead + AsyncWrite + 'static> ReadPart for Conn<RW> {
    fn read(&mut self) -> Pin<Box<dyn Future<Output = Message>>> {
        Box::pin(async move { Message::Ping })
    }
}

impl<RW: AsyncRead + AsyncWrite + 'static> WritePart for Conn<RW> {
    fn send(&mut self, msg: Message) -> Pin<Box<dyn Future<Output = ()>>> {
        Box::pin(async move {})
    }
}

impl<RW: AsyncRead + AsyncWrite + 'static> WsConn for Conn<RW> {
    fn split(&mut self) -> (Box<dyn ReadPart>, Box<dyn WritePart>) {
        todo!()
    }
}
