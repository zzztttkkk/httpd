use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt};

use crate::{ctx::ConnContext, uitls::multi_map::MultiMap};

enum ReadState {
    None,
    FirstLine0,
    FirstLine1,
    FirstLine2,
    HeadersDone,
}

#[derive(Default)]
pub(crate) struct MessageBody {
    pub(crate) internal: bytebuffer::ByteBuffer,
    pub(crate) w: Option<Box<dyn std::io::Write + Send>>,
}

// TODO remove this unsafe
pub(crate) struct BytesBufferProxy(u64);

impl BytesBufferProxy {
    fn ptr(&self) -> &mut bytebuffer::ByteBuffer {
        unsafe { std::mem::transmute(self.0) }
    }
}

impl std::io::Write for BytesBufferProxy {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.ptr().write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.ptr().flush()
    }
}

enum CompressionType {
    Gzip,
    Deflate,
}

impl MessageBody {
    pub(crate) fn enable_compression(&mut self, ct: CompressionType) {
        match ct {
            CompressionType::Gzip => {
                self.w = Some(Box::new(flate2::write::GzEncoder::new(
                    BytesBufferProxy(unsafe { std::mem::transmute(&self.internal) }),
                    flate2::Compression::default(),
                )))
            }
            CompressionType::Deflate => {
                self.w = Some(Box::new(flate2::write::DeflateEncoder::new(
                    BytesBufferProxy(unsafe { std::mem::transmute(&self.internal) }),
                    flate2::Compression::default(),
                )))
            }
        }
    }
}

#[derive(Default)]
pub(crate) struct Message {
    firstline: (String, String, String),
    headers: MultiMap,
    body: MessageBody,
}

pub(crate) enum MessageReadCode {
    Ok,
    ConnReadError,
    BadDatagram,
    ReachMaxBodySize,
    BadContentLength,
}

impl Message {
    fn get_content_length(&self) -> Result<usize, ()> {
        match self.headers.get("content-length") {
            Some(vs) => match vs.parse::<usize>() {
                Ok(num) => Ok(num),
                Err(_) => Err(()),
            },
            None => Ok(0),
        }
    }

    pub(crate) async fn from11<R: AsyncBufReadExt + Unpin, W: AsyncWriteExt + Unpin>(
        &mut self,
        ctx: &mut ConnContext<R, W>,
    ) -> MessageReadCode {
        let mut state = ReadState::None;
        let reader = &mut ctx.reader;
        let buf = &mut ctx.buf;
        let config = &(ctx.config.http);

        loop {
            match state {
                ReadState::None => {
                    let dest = unsafe { (&mut self.firstline.0).as_mut_vec() };
                    match reader.take(128).read_until(b' ', dest).await {
                        Ok(size) => {
                            if size < 1 {
                                return MessageReadCode::BadDatagram;
                            }
                            state = ReadState::FirstLine0;
                            continue;
                        }
                        Err(_) => {
                            return MessageReadCode::ConnReadError;
                        }
                    }
                }
                ReadState::FirstLine0 => {
                    let dest = unsafe { (&mut self.firstline.1).as_mut_vec() };
                    match reader
                        .take(config.max_url_size.u64())
                        .read_until(b' ', dest)
                        .await
                    {
                        Ok(size) => {
                            if size < 1 {
                                return MessageReadCode::BadDatagram;
                            }
                            state = ReadState::FirstLine1;
                            continue;
                        }
                        Err(_) => {
                            return MessageReadCode::ConnReadError;
                        }
                    }
                }
                ReadState::FirstLine1 => {
                    match reader.take(128).read_line(&mut self.firstline.2).await {
                        Ok(size) => {
                            if size < 1 {
                                return MessageReadCode::BadDatagram;
                            }
                            state = ReadState::FirstLine2;
                            continue;
                        }
                        Err(_) => {
                            return MessageReadCode::ConnReadError;
                        }
                    }
                }
                ReadState::FirstLine2 => loop {
                    match reader
                        .take(config.max_header_line_size.u64())
                        .read_until(b'\n', buf)
                        .await
                    {
                        Ok(size) => {
                            if size < 2 {
                                return MessageReadCode::BadDatagram;
                            }
                            if size == 2 {
                                state = ReadState::HeadersDone;
                                break;
                            }
                            format!("{}", std::str::from_utf8(&buf[..size]).unwrap());
                        }
                        Err(_) => {
                            return MessageReadCode::ConnReadError;
                        }
                    }
                },
                ReadState::HeadersDone => match self.get_content_length() {
                    Ok(mut remain_size) => {
                        if remain_size > config.max_body_size.u64() as usize {
                            return MessageReadCode::ReachMaxBodySize;
                        }

                        let cap = buf.capacity();
                        unsafe { buf.set_len(cap) };

                        loop {
                            let mut buf = buf.as_mut_slice();
                            if remain_size < cap {
                                buf = &mut buf[..remain_size];
                            }
                            match reader.read(buf).await {
                                Ok(size) => {
                                    remain_size -= size;
                                    if remain_size < 1 {
                                        return MessageReadCode::Ok;
                                    }
                                }
                                Err(_) => {
                                    return MessageReadCode::ConnReadError;
                                }
                            }
                        }
                    }
                    Err(_) => {
                        return MessageReadCode::BadContentLength;
                    }
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_unsafe_steing() {
        let mut name = String::new();
        let bytes = unsafe { (&mut name).as_mut_vec() };
        bytes.push(211);
        bytes.push(241);

        println!("{:?}", name.len());
    }
}
