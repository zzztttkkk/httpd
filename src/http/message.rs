use bytes::{BufMut, BytesMut};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt};

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
                println!("{}", self.fl.1);
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
                        self.body = Some(bytes::BytesMut::new());
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
    ) {
        _ = stream.flush().await;
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
}

impl Response {
    pub(crate) fn new() -> Self {
        return Self {
            msg: Message::new(),
        };
    }
}

pub struct Context {
    pub(crate) req: Request,
    pub(crate) resp: Response,
}

impl Context {
    pub(crate) fn new() -> Self {
        return Self {
            req: Request::new(),
            resp: Response::new(),
        };
    }
}
