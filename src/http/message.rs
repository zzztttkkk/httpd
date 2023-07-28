use std::cell::RefCell;
use std::net::SocketAddr;
use bytes::{BufMut, BytesMut};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt};
use crate::utils;

use super::header::Header;

pub struct Message {
    pub fl: (String, String, String),
    pub header: Header,
    pub body: Option<BytesMut>,
}

impl Message {
    pub(crate) fn new() -> Self {
        return Self {
            fl: (String::new(), String::new(), String::new()),
            header: Header::new(),
            body: None,
        };
    }

    pub(crate) async fn readfrom11<T: AsyncBufReadExt + Unpin>(
        &mut self,
        stream: &mut T,
        buf: &mut Vec<u8>,
    ) -> u32 {
        self.fl.0.clear();
        if let Ok(rl) = stream.take(30).read_until(b' ', buf).await {
            if rl < 2 {
                return 1;
            }
            if let Ok(method) = std::str::from_utf8(&buf[..rl - 1]) {
                self.fl.0.push_str(method.to_ascii_uppercase().as_str());
            } else {
                return 400;
            }
        } else {
            return 400;
        }

        // todo on fl.0

        self.fl.1.clear();
        if let Ok(rl) = stream
            .take(64 * 1024)
            .read_until(b' ', unsafe { self.fl.1.as_mut_vec() })
            .await
        {
            if rl < 1 {
                return 1;
            }
            if rl == 1 {
                self.fl.1 = "/".to_string();
            } else {
                unsafe {
                    let vs = self.fl.1.as_mut_vec();
                    vs.remove(vs.len() - 1);
                }
            }
        } else {
            return 400;
        }

        // todo on fl.1

        self.fl.2.clear();
        buf.clear();
        if let Ok(rl) = stream.take(30).read_until(b'\n', buf).await {
            if rl < 2 {
                return 1;
            }
            if let Ok(version) = std::str::from_utf8(&buf[..rl - 1]) {
                self.fl.2.push_str(version.trim_end());
            } else {
                return 400;
            }
        } else {
            return 400;
        }

        // todo on fl.2

        loop {
            buf.clear();
            if let Ok(rl) = stream.take(64 * 1024).read_until(b'\n', buf).await {
                if rl < 1 {
                    return 1;
                }
                let line = unsafe { std::str::from_utf8_unchecked(&buf[..rl - 1]) }.trim_end();
                if line.is_empty() {
                    break;
                }

                let mut split_iter = line.splitn(2, ':');
                let key = split_iter.next().unwrap_or("");
                if key.is_empty() {
                    return 400;
                }
                self.header
                    .add(key, split_iter.next().unwrap_or("").trim_start());
            } else {
                return 400;
            }
        }

        // todo check chunked transfer-encoding

        let content_length_result = self.header.get_content_length();
        if let Ok(mut size) = content_length_result {
            if size == 0 {
                return 0;
            }

            buf.resize(buf.capacity(), 0);

            loop {
                let require_length: usize;
                if size > buf.capacity() {
                    require_length = buf.capacity();
                } else {
                    require_length = size;
                }
                if let Ok(rl) = stream.read_exact(&mut buf[..require_length]).await {
                    if rl < 1 || rl > require_length {
                        return 1;
                    }

                    if self.body.is_none() {
                        self.body = Some(BytesMut::new());
                    }
                    let bref = self.body.as_mut().unwrap();
                    bref.put_slice(&buf[..rl]);

                    size -= rl;
                    if size == 0 {
                        break;
                    }
                }
            }
        } else {
            return 400;
        }
        return 0;
    }

    pub(crate) async fn writeto11<T: AsyncWriteExt + Unpin>(
        &mut self,
        stream: &mut T,
        buf: &mut Vec<u8>,
    ) -> std::io::Result<()> {
        if self.fl.0.is_empty() {
            _ = stream.write("HTTP/1.1".as_bytes()).await;
        } else {
            _ = stream.write(self.fl.0.as_bytes()).await;
        }
        _ = stream.write(" ".as_bytes()).await;

        if self.fl.1.is_empty() || self.fl.2.is_empty() {
            _ = stream.write("200".as_bytes()).await;
            _ = stream.write(" ".as_bytes()).await;
            _ = stream.write("OK".as_bytes()).await;
            _ = stream.write("\r\n".as_bytes()).await;
        } else {
            _ = stream.write(self.fl.1.as_bytes()).await;
            _ = stream.write(" ".as_bytes()).await;
            _ = stream.write(self.fl.2.as_bytes()).await;
            _ = stream.write("\r\n".as_bytes()).await;
        }


        let mut body_length: usize = 0;
        if let Some(body) = self.body.as_ref() {
            body_length = body.len()
        }

        self.header.set("content-length", body_length.to_string().as_str());
        self.header.set("server", "httpd.rs");
        self.header.set(
            "date",
            utils::time::utc().format(
                utils::time::DEFAULT_HTTP_HEADER_TIME_LAYOUT
            ).to_string().as_str(),
        );

        buf.clear();
        self.header.each(|k, vs| {
            for v in vs {
                _ = buf.extend_from_slice(format!("{}: {}\r\n", k, v).as_bytes());
            }
        });
        _ = stream.write(buf.as_slice()).await;
        _ = stream.write("\r\n".as_bytes()).await;

        if let Some(body) = self.body.as_ref() {
            _ = stream.write(body).await;
        }
        return stream.flush().await;
    }
}

pub struct Request {
    pub(crate) msg: Message,
}

impl Request {
    pub(crate) fn new() -> Self {
        return Self {
            msg: Message::new(),
        };
    }

    pub fn method(&self) -> &String {
        return &self.msg.fl.0;
    }

    pub fn url(&self) -> &String {
        return &self.msg.fl.1;
    }

    pub fn version(&self) -> &String {
        return &self.msg.fl.2;
    }

    pub fn header(&self) -> &Header {
        return &self.msg.header;
    }

    pub fn body(&self) -> &Option<BytesMut> {
        return &self.msg.body;
    }
}

pub struct Response {
    pub(crate) msg: Message,

    _status_code: RefCell<Option<u32>>,
}

impl Response {
    pub(crate) fn new() -> Self {
        return Self {
            msg: Message::new(),
            _status_code: RefCell::new(None),
        };
    }
}

pub struct Context {
    _remote_addr: SocketAddr,
    pub(crate) req: Request,
    pub(crate) resp: Response,
}

impl Context {
    pub(crate) fn new(addr: SocketAddr) -> Self {
        return Self {
            _remote_addr: addr,
            req: Request::new(),
            resp: Response::new(),
        };
    }

    pub fn remote_addr(&self) -> &SocketAddr {
        return &self._remote_addr;
    }

    pub fn keep_alive(&self) -> bool {
        return false;
    }
}
