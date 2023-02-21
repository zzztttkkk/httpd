use std::{future::Future, pin::Pin, sync::Arc};

use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::Mutex,
};

use super::{
    conn::{Conn, WsConn},
    message::Message,
};

pub trait WebSocketHandler: Sync + Send {
    fn handle<'a: 'c, 'c>(
        &'a self,
        conn: &'c mut Box<dyn WsConn>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'c>>;
}
