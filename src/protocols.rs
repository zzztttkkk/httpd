pub(crate) enum Protocol {
    Current { keep_alive: bool },
    WebSocket,
    Http2,
}
