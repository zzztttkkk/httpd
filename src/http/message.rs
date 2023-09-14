use std::io::Write;
use bytes::{BufMut, BytesMut};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt};
use crate::http::body::{Body, CompressionType};
use crate::http::header::Header;
use crate::utils;

pub struct Message {
    pub(crate) fl: (String, String, String),
    pub(crate) header: Header,
    pub(crate) body: Option<Body>,
}

impl Message {
    pub(crate) fn new() -> Self {
        return Self {
            fl: (String::new(), String::new(), String::new()),
            header: Header::new(),
            body: None,
        };
    }

    pub(crate) fn get_content_encoding(&self) -> Result<Option<CompressionType>, String> {
        self.header.get_content_encoding()
    }

    pub(crate) fn set_content_encoding(&mut self, ct: CompressionType, for_outgoing: bool) {
        if self.body.is_some() {
            panic!("message's body is not none");
        }

        match ct {
            CompressionType::Gzip => {
                self.header.set("content-encoding", "gzip");
            }
            CompressionType::Deflate => {
                self.header.set("content-encoding", "deflate");
            }
            CompressionType::Br => {
                self.header.set("content-encoding", "br");
            }
        }

        if for_outgoing {
            self.body = Some(Body::new_for_outgoing(4096));
        } else {
            self.body = Some(Body::new_for_incoming(4096));
        }
    }

    async fn read_empty_line<T: AsyncBufReadExt + Unpin>(stream: &mut T, buf: &mut Vec<u8>) -> bool {
        return if let Ok(rel) = stream.take(2).read_until(b'\n', buf).await {
            rel == 2
        } else {
            false
        };
    }

    fn ensure_body(&mut self, size: Option<usize>) -> bool {
        if self.body.is_some() {
            return true;
        }

        self.body = Some(Body::new_for_incoming(size.unwrap_or(4096)));
        return if let Ok(ce_opt) = self.get_content_encoding() {
            if let Some(ct) = ce_opt {
                self.body.as_mut().unwrap().set_compression_type(ct);
            }
            true
        } else {
            false
        };
    }

    async fn read_chunked_body<T: AsyncBufReadExt + Unpin>(
        &mut self,
        stream: &mut T,
        buf: &mut Vec<u8>,
    ) -> u32 {
        if !self.ensure_body(None) {
            return 400;
        }

        let mut chunk_size: Option<usize> = None;
        loop {
            match chunk_size.as_ref() {
                None => {
                    buf.clear();
                    match stream.take(36).read_until(b'\n', buf).await {
                        Ok(rl) => {
                            if let Ok(line) = std::str::from_utf8(&buf[..rl - 2]) {
                                if let Ok(v) = line.parse::<usize>() {
                                    chunk_size = Some(v);
                                    continue;
                                } else {
                                    return 400;
                                }
                            } else {
                                return 400;
                            }
                        }
                        Err(_) => {
                            return 400;
                        }
                    }
                }
                Some(sizeref) => {
                    let mut current_chunk_remain_size = *sizeref;
                    if current_chunk_remain_size == 0 {
                        if Message::read_empty_line(stream, buf).await {
                            return 0;
                        }
                        return 400;
                    }

                    buf.resize(buf.capacity(), 0);

                    loop {
                        let current_read_length: usize;
                        if current_chunk_remain_size > buf.capacity() {
                            current_read_length = buf.capacity();
                        } else {
                            current_read_length = current_chunk_remain_size;
                        }

                        if let Ok(rl) = stream.read_exact(&mut buf[..current_read_length]).await {
                            if rl < 1 {
                                return 1;
                            }

                            self.body.as_mut().unwrap().write(&buf[..rl]);
                            current_chunk_remain_size -= rl;
                            if current_chunk_remain_size == 0 {
                                if Message::read_empty_line(stream, buf).await {
                                    break;
                                }
                                return 400;
                            }
                        } else {
                            return 400;
                        }
                    }
                }
            }
        }
    }

    pub(crate) async fn readfrom11<T: AsyncBufReadExt + Unpin>(
        &mut self,
        stream: &mut T,
        buf: &mut Vec<u8>,
        for_request: bool,
    ) -> u32 {
        buf.clear();
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

        // read header lines
        loop {
            buf.clear();
            if let Ok(rl) = stream.take(64 * 1024).read_until(b'\n', buf).await {
                if rl < 1 {
                    return 1;
                }

                let line_result = std::str::from_utf8(&buf[..rl - 1]);
                if line_result.is_err() {
                    return 400;
                }

                let line = line_result.unwrap().trim_end();
                if line.is_empty() {
                    break;
                }

                let mut split_iter = line.splitn(2, ':');
                let key = split_iter.next().unwrap_or("");
                if key.is_empty() {
                    return 400;
                }
                self.header.add(
                    key,
                    split_iter.next().unwrap_or("").trim_start(),
                );
            } else {
                return 400;
            }
        }

        if self.header.is_chunked() {
            return self.read_chunked_body(stream, buf).await;
        }

        if let Ok(mut size) = self.header.get_content_length() {
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
                    if rl < 1 {
                        return 1;
                    }
                    if !self.ensure_body(Some(size)) {
                        return 400;
                    }
                    self.body.as_mut().unwrap().write(&buf[..rl]);

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
        is_request: bool,
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
        if let Some(body) = self.body.as_mut() {
            _ = body.flush();
            body_length = body.len();
        }

        self.header
            .set("content-length", body_length.to_string().as_str());
        self.header.set("server", "httpd.rs");
        self.header.set(
            "date",
            utils::time::utc()
                .format(utils::time::DEFAULT_HTTP_HEADER_TIME_LAYOUT)
                .to_string()
                .as_str(),
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
            _ = stream.write(body.bytes()).await;
        }
        return stream.flush().await;
    }
}
