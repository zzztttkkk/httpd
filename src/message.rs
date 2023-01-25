use std::fmt::Formatter;
use std::io::{Read, Write};
use std::time::Duration;

use bytebuffer::ByteBuffer;
use tokio::io::{AsyncBufReadExt, AsyncReadExt};

use crate::compress::{CompressWriter, Deflate, Gzip, Zlib};
use crate::headers::Headers;

pub struct ByteBufferWrapper {
    ptr: *mut Option<ByteBuffer>,
}

unsafe impl Send for ByteBufferWrapper {}

impl ByteBufferWrapper {
    #[inline(always)]
    fn bufref(&mut self) -> &mut ByteBuffer {
        unsafe {
            self.ptr.as_mut().unwrap().as_mut().unwrap()
        }
    }
}

impl Read for ByteBufferWrapper {
    #[inline(always)]
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.bufref().read(buf)
    }
}

impl Write for ByteBufferWrapper {
    #[inline(always)]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.bufref().write(buf)
    }

    #[inline(always)]
    fn flush(&mut self) -> std::io::Result<()> {
        self.bufref().flush()
    }
}

pub struct BodyBuf {
    raw: Option<ByteBuffer>,
    decoder: Option<Box<dyn Read + Send>>,
    encoder: Option<Box<dyn CompressWriter + Send>>,
}

pub enum CompressType {
    Gzip,
    Deflate,
    Zlib,
}

impl BodyBuf {
    #[inline(always)]
    fn new(buf: Option<ByteBuffer>) -> Self {
        Self {
            raw: buf,
            decoder: None,
            encoder: None,
        }
    }

    pub fn writeraw(&mut self, buf: &[u8]) {
        ByteBuffer::write(self.raw.as_mut().unwrap(), buf).unwrap();
    }

    #[inline(always)]
    pub fn decompress(&mut self) {
        self.decoder = Some(Box::new(flate2::read::GzDecoder::new(ByteBufferWrapper { ptr: &mut self.raw })));
    }

    #[inline(always)]
    pub fn begincompress(&mut self, ct: CompressType, level: flate2::Compression) {
        match ct {
            CompressType::Gzip => {
                self.encoder = Some(Box::new(Gzip::with_level(ByteBufferWrapper { ptr: &mut self.raw }, level)));
            }
            CompressType::Deflate => {
                self.encoder = Some(Box::new(Deflate::with_level(ByteBufferWrapper { ptr: &mut self.raw }, level)));
            }
            CompressType::Zlib => {
                self.encoder = Some(Box::new(Zlib::with_level(ByteBufferWrapper { ptr: &mut self.raw }, level)));
            }
        }
    }

    #[inline(always)]
    pub fn finishcompress(&mut self) -> std::io::Result<()> {
        match &mut self.encoder {
            None => {
                Ok(())
            }
            Some(encoder) => {
                encoder.finish()
            }
        }
    }
}

impl Read for BodyBuf {
    #[inline(always)]
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match &mut self.decoder {
            None => {
                self.raw.as_mut().unwrap().read(buf)
            }
            Some(decoder) => {
                decoder.read(buf)
            }
        }
    }
}

impl Write for BodyBuf {
    #[inline(always)]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match &mut self.encoder {
            None => {
                self.raw.as_mut().unwrap().write(buf)
            }
            Some(encoder) => {
                encoder.write(buf)
            }
        }
    }

    #[inline(always)]
    fn flush(&mut self) -> std::io::Result<()> {
        match &mut self.encoder {
            None => {
                self.raw.as_mut().unwrap().flush()
            }
            Some(encoder) => {
                encoder.flush()
            }
        }
    }
}

impl std::fmt::Debug for BodyBuf {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "BodyBuf{{").unwrap();
        match &self.raw {
            None => {
                write!(f, " len:0").unwrap();
            }
            Some(v) => {
                write!(f, " len:{}, rpos:{}, wpos:{}", v.len(), v.get_rpos(), v.get_wpos()).unwrap();
            }
        }

        if self.decoder.is_some() {
            write!(f, ", In Decompressing").unwrap();
        }

        if self.encoder.is_some() {
            write!(f, ", In Compressing").unwrap();
        }

        write!(f, "}}")
    }
}

#[derive(Debug)]
pub struct Message {
    pub f0: String,
    pub f1: String,
    pub f2: String,
    pub headers: Headers,
    pub bodybuf: Option<BodyBuf>,
}

#[derive(Debug, Clone, Copy)]
pub enum ReadStatus {
    None,
    Headers,
    Body(i64),
    Ok,
}

impl Message {
    pub async fn from11<Reader: AsyncBufReadExt + Unpin + Send>(mut reader: Reader, buf: &mut String, onstatus: fn(ReadStatus, &Message) -> Result<(), i32>) -> Result<Self, i32> {
        let mut status = ReadStatus::None;
        let mut msg = Message {
            f0: "".to_string(),
            f1: "".to_string(),
            f2: "".to_string(),
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
                            if s == 0 {
                                return Err(0);
                            }

                            if s == 2 {
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
                                        msg.f0.push(rune);
                                    }
                                    1 => {
                                        if rune == ' ' {
                                            fls += 1;
                                            continue;
                                        }
                                        msg.f1.push(rune);
                                    }
                                    2 => {
                                        if rune == '\r' {
                                            break;
                                        }
                                        msg.f2.push(rune);
                                    }
                                    _ => {
                                        break;
                                    }
                                }
                            }
                            status = ReadStatus::Headers;
                            onstatus(status, &msg)?;
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
                            if s == 0 {
                                return Err(0);
                            }

                            if s == 2 {
                                match msg.headers.contentlength() {
                                    None => {}
                                    Some(cl) => {
                                        if cl < 0 {
                                            if !msg.headers.ischunked() {
                                                return Err(400);
                                            }
                                            is_chunked = true;
                                        } else {
                                            body_remains = cl;
                                            if body_remains > 0 {
                                                let mut bbuf = ByteBuffer::new();
                                                bbuf.resize(body_remains as usize);
                                                msg.bodybuf = Some(BodyBuf::new(Some(bbuf)));
                                                buf.clear();
                                                for _ in 0..(std::cmp::min(buf.capacity(), body_remains as usize)) {
                                                    buf.push(0 as char);
                                                }
                                            }
                                        }
                                    }
                                }
                                status = ReadStatus::Body(body_remains);
                                onstatus(status, &msg)?;
                                continue;
                            }

                            let mut parts = buf.splitn(2, ':');
                            let key = match parts.next() {
                                None => {
                                    return Err(400);
                                }
                                Some(v) => {
                                    v
                                }
                            };

                            match parts.next() {
                                None => {
                                    return Err(400);
                                }
                                Some(v) => {
                                    msg.headers.add(key, v.trim());
                                }
                            }
                        }
                        Err(e) => {
                            println!("{}", e);
                            return Err(400);
                        }
                    }
                }
                ReadStatus::Body(_) => {
                    if is_chunked {
                        // todo read chunked body
                    } else {
                        if body_remains > 0 {
                            let bytes: &mut [u8];
                            unsafe {
                                bytes = buf.as_bytes_mut();
                            }
                            match reader.read(bytes).await {
                                Ok(s) => {
                                    if s < 1 {
                                        return Err(0);
                                    }
                                    msg.bodybuf.as_mut().unwrap().writeraw(&bytes[0..s]);
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
                            onstatus(status, &msg)?;
                        }
                    }
                }
                ReadStatus::Ok => {
                    return Ok(msg);
                }
            }
        }
    }

    pub async fn from11_with_timeout<Reader: AsyncBufReadExt + Unpin + Send>(mut reader: Reader, buf: &mut String, onstatus: fn(ReadStatus, &Message) -> Result<(), i32>, timeout: u64) -> Result<Self, i32> {
        tokio::select! {
            result = Self::from11(reader, buf, onstatus) => {
                result
            }
            _ = tokio::time::sleep(Duration::from_millis(timeout)) => {
                Err(2)
            }
        }
    }
}
