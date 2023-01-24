use std::io::Write;
use std::pin::Pin;

use bytebuffer::ByteBuffer;
use tokio::io::{AsyncBufReadExt, AsyncReadExt};

use crate::headers::Headers;

#[derive(Debug)]
pub struct Request {
    pub method: String,
    pub rawpath: String,
    pub version: String,
    pub headers: Headers,
    pub bodybuf: Option<ByteBuffer>,
}

enum ReadStatus {
    None,
    Headers,
    Body,
    Ok,
}

impl Request {
    pub async fn from11<Reader: AsyncBufReadExt + Unpin + Send>(mut reader: Reader, buf: &mut String) -> Result<Pin<Box<Self>>, i32> {
        let mut status = ReadStatus::None;
        let mut req = Request {
            method: "".to_string(),
            rawpath: "".to_string(),
            version: "".to_string(),
            headers: Headers::new(),
            bodybuf: None,
        };
        let mut body_remains: i64 = 0;
        let mut is_chunked = false;

        loop {
            match status {
                ReadStatus::None => {
                    buf.clear();
                    match reader.read_line(buf).await {
                        Ok(s) => {
                            if s <= 2 {
                                continue;
                            }

                            let mut fls = 0;
                            for rune in buf.chars() {
                                match fls {
                                    0 => {
                                        if rune == ' ' {
                                            fls += 1;
                                            continue;
                                        }
                                        req.method.push(rune);
                                    }
                                    1 => {
                                        if rune == ' ' {
                                            fls += 1;
                                            continue;
                                        }
                                        req.rawpath.push(rune);
                                    }
                                    2 => {
                                        if rune == '\r' {
                                            break;
                                        }
                                        req.version.push(rune);
                                    }
                                    _ => {
                                        break;
                                    }
                                }
                            }
                            status = ReadStatus::Headers;
                        }
                        Err(e) => {
                            println!("{}", e);
                            return Err(400);
                        }
                    }
                }
                ReadStatus::Headers => {
                    buf.clear();
                    match reader.read_line(buf).await {
                        Ok(s) => {
                            if s <= 2 {
                                status = ReadStatus::Body;

                                match req.headers.contentlength() {
                                    None => {}
                                    Some(cl) => {
                                        if cl < 0 {
                                            if !req.headers.ischunked() {
                                                return Err(400);
                                            }
                                            is_chunked = true;
                                        } else {
                                            body_remains = cl;
                                            if body_remains > 0 {
                                                req.bodybuf = Some(ByteBuffer::new());
                                                buf.clear();
                                                for _ in 0..buf.capacity() {
                                                    buf.push(0 as char);
                                                }
                                            }
                                        }
                                    }
                                }
                                continue;
                            }

                            let mut parts = buf.splitn(2, ':');
                            let key: &str;
                            match parts.next() {
                                None => {
                                    return Err(400);
                                }
                                Some(v) => {
                                    key = v;
                                }
                            }

                            match parts.next() {
                                None => {
                                    return Err(400);
                                }
                                Some(v) => {
                                    req.headers.add(key, v.trim());
                                }
                            }
                        }
                        Err(e) => {
                            println!("{}", e);
                            return Err(400);
                        }
                    }
                }
                ReadStatus::Body => {
                    if is_chunked {} else {
                        if body_remains > 0 {
                            let bytes: &mut [u8];
                            unsafe {
                                bytes = buf.as_bytes_mut();
                            }
                            match reader.read(bytes).await {
                                Ok(s) => {
                                    println!("{} {}", s, body_remains);
                                    ByteBuffer::write(req.bodybuf.as_mut().unwrap(), &bytes[0..s]).unwrap();
                                    body_remains -= s as i64;
                                }
                                Err(e) => {
                                    println!("{}", e);
                                    return Err(400);
                                }
                            };
                        }

                        if body_remains <= 0 {
                            status = ReadStatus::Ok;
                        }
                    }
                }
                ReadStatus::Ok => {
                    return Ok(Box::pin(req));
                }
            }
        }
    }
}


