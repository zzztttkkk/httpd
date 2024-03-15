use crate::{request::RequestQueryer, respw::ResponseWriter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum WsOpCode {
    Text = 0x1,
    Binary = 0x2,
    Ping = 0x9,
    Pong = 0xA,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ExtOpCode {
    Continuation = 0x0,
    Close = 0x8,
}

pub(crate) fn upgrade(req: &mut RequestQueryer, resp: &mut ResponseWriter) -> Result<bool, ()> {
    static MAGIC_BYTES: &[u8] = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11".as_bytes();

    match req.headers().get("sec-websocket-key") {
        Some(key) => {
            if key.is_empty() {
                return Ok(false);
            }
            use base64::Engine;
            use sha1::Digest;

            let mut hasher = sha1::Sha1::new();
            hasher.update(key.as_bytes());
            hasher.update(MAGIC_BYTES);

            let hash = hasher.finalize();

            resp.header(
                "sec-websocket-accept",
                &base64::engine::general_purpose::STANDARD.encode(hash.as_slice()),
            );
        }
        None => {
            return Ok(false);
        }
    }

    resp.version(1, 1).code(101, "Switching Protocols");
    resp.header("sec-websocket-version", "13")
        .header("upgrade", "websocket")
        .header("connection", "Upgrade");
    Ok(true)
}
