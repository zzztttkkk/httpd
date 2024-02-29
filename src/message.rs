use std::io::Write;

use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt};

use crate::compression::CompressionImpl;
use crate::{ctx::ConnContext, uitls::multi_map::MultiMap};

enum ReadState {
    None,
    FirstLine0,
    FirstLine1,
    FirstLine2,
    HeadersDone,
}

pub(crate) struct MessageBody {
    pub(crate) internal: Option<Box<bytebuffer::ByteBuffer>>,
    pub(crate) cw: Option<Box<dyn CompressionImpl<Box<bytebuffer::ByteBuffer>> + Send>>,
}

impl Default for MessageBody {
    fn default() -> Self {
        Self {
            internal: Some(Box::new(Default::default())),
            cw: None,
        }
    }
}

impl std::io::Write for MessageBody {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self.internal.as_mut() {
            Some(inner) => inner.write(buf),
            None => self.cw.as_mut().unwrap().append(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub(crate) enum CompressionType {
    Brotil,
    Deflate,
    Gzip,
}

impl MessageBody {
    pub(crate) fn enable_compression(mut self, ct: CompressionType, level: u32) {
        let buf = self.internal.unwrap();
        self.internal = None;

        match ct {
            CompressionType::Brotil => {
                let mut params = brotli::enc::BrotliEncoderParams::default();
                params.quality = level as i32;
                self.cw = Some(Box::new(brotli::CompressorWriter::with_params(
                    buf, 4096, &params,
                )));
            }
            CompressionType::Deflate => {
                self.cw = Some(Box::new(flate2::write::DeflateEncoder::new(
                    buf,
                    flate2::Compression::new(std::cmp::min(level, 9)),
                )));
            }
            CompressionType::Gzip => {
                self.cw = Some(Box::new(flate2::write::GzEncoder::new(
                    buf,
                    flate2::Compression::new(std::cmp::min(level, 9)),
                )));
            }
        }
    }
}

#[derive(Default)]
pub(crate) struct Message {
    pub(crate) firstline: (String, String, String),
    pub(crate) headers: MultiMap,
    pub(crate) body: MessageBody,
}

#[derive(Debug)]
pub(crate) enum MessageReadCode {
    Ok,
    ConnReadError,
    BadDatagram,
    ReachMaxBodySize,
    BadContentLength,
}

const MAX_HEADER_NAME_LENGTH: usize = 256;

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
                            // trim last space
                            unsafe { dest.set_len(size - 1) };
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
                            unsafe { dest.set_len(size - 1) };
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
                            if size < 3 {
                                return MessageReadCode::BadDatagram;
                            }
                            unsafe { ((&mut self.firstline.2).as_mut_vec()).set_len(size - 2) };
                            state = ReadState::FirstLine2;
                            continue;
                        }
                        Err(_) => {
                            return MessageReadCode::ConnReadError;
                        }
                    }
                }
                ReadState::FirstLine2 => {
                    // TODO test
                    let mut keytmp = [0 as u8; MAX_HEADER_NAME_LENGTH];
                    let mut keyidx: usize;

                    'readlines: loop {
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
                                // trim `\r\n`
                                unsafe { buf.set_len(size - 2) };

                                keyidx = 0;
                                for idx in 0..buf.len() {
                                    let c = buf[idx];
                                    if !c.is_ascii() {
                                        return MessageReadCode::BadDatagram;
                                    }

                                    if c == b':' {
                                        let key = unsafe {
                                            std::str::from_utf8_unchecked(&keytmp[..keyidx])
                                        }
                                        .trim();

                                        let value =
                                            unsafe { std::str::from_utf8_unchecked(&buf[idx..]) }
                                                .trim();
                                        
                                        println!("{}: {}", key, value);
                                        self.headers.append(key, value);
                                        break 'readlines;
                                    }

                                    if keyidx >= MAX_HEADER_NAME_LENGTH {
                                        // header name too long
                                        return MessageReadCode::BadDatagram;
                                    }
                                    unsafe {
                                        *(keytmp.get_unchecked_mut(keyidx)) = c;
                                    }
                                    keyidx += 1;
                                }
                            }
                            Err(_) => {
                                return MessageReadCode::ConnReadError;
                            }
                        }
                    }
                }
                ReadState::HeadersDone => match self.get_content_length() {
                    Ok(mut remain_size) => {
                        if remain_size < 1 {
                            return MessageReadCode::Ok;
                        }

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
                                    _ = self.body.internal.as_mut().unwrap().write(&buf[..size]);
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
        let a = 00;
        let b = [0 as u8; 4096 * 10];
        let c = 00;
        println!("{:p} {:p} {:p}", &a, &b, &c);
    }
}
