use std::fmt::Formatter;
use std::io::{Read, Write};
use std::num::ParseIntError;

use bytebuffer::ByteBuffer;
use flate2::Compression;
use tokio::io::{AsyncBufReadExt, AsyncReadExt};

use crate::compress::{CompressType, CompressWriter, Deflate, Gzip};
use crate::config::Config;
use crate::error::StatusCodeError;
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
    _encoder_finished: bool,
}

impl BodyBuf {
    #[inline(always)]
    pub fn new(buf: Option<ByteBuffer>) -> Self {
        Self {
            raw: buf,
            decoder: None,
            encoder: None,
            _encoder_finished: false,
        }
    }

    pub fn writeraw(&mut self, buf: &[u8]) {
        ByteBuffer::write(self.raw.as_mut().unwrap(), buf).unwrap();
    }

    #[inline(always)]
    pub fn decompress(&mut self, ct: CompressType) {
        match ct {
            CompressType::Gzip => {
                self.decoder = Some(Box::new(flate2::read::GzDecoder::new(ByteBufferWrapper { ptr: &mut self.raw })));
            }
            CompressType::Deflate => {
                self.decoder = Some(Box::new(flate2::read::DeflateDecoder::new(ByteBufferWrapper { ptr: &mut self.raw })));
            }
        }
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
        }
    }

    #[inline(always)]
    pub fn finishcompress(&mut self) -> std::io::Result<()> {
        if self._encoder_finished {
            return Ok(());
        }
        self._encoder_finished = true;

        match &mut self.encoder {
            None => {
                Ok(())
            }
            Some(encoder) => {
                encoder.finish()
            }
        }
    }

    pub fn raw(&self) -> Option<&ByteBuffer> { self.raw.as_ref() }
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

impl Drop for BodyBuf {
    fn drop(&mut self) {
        self.finishcompress();
    }
}

#[derive(Debug)]
pub struct Message {
    pub(crate) f0: String,
    pub(crate) f1: String,
    pub(crate) f2: String,
    pub(crate) headers: Headers,
    pub(crate) bodybuf: Option<BodyBuf>,

    pub(crate) _compress_type: Option<CompressType>,
}

#[derive(Debug, Clone, Copy)]
enum ReadStatus {
    None,
    Headers,
    Body(i64),
    Ok,
}

impl Message {
    pub fn new() -> Self {
        Self {
            f0: "".to_string(),
            f1: "".to_string(),
            f2: "".to_string(),
            headers: Headers::new(),
            bodybuf: None,
            _compress_type: None,
        }
    }

    pub async fn from11<Reader: AsyncBufReadExt + Unpin + Send>(mut reader: Reader, buf: &mut String, cfg: &Config) -> Result<Self, StatusCodeError> {
        let mut status = ReadStatus::None;
        let mut msg = Message::new();

        let mut body_remains: i64 = 0;
        let mut is_chunked = false;

        loop {
            match status {
                ReadStatus::None => {
                    buf.clear();
                    match reader.read_line(buf).await {
                        Ok(line_size) => {
                            if line_size < 2 {
                                return Err(StatusCodeError::new(0));
                            }

                            if line_size == 2 {
                                continue;
                            }

                            let mut fls = 0;
                            for rune in buf.chars() {
                                if rune > 127 as char {
                                    return Err(StatusCodeError::new(0));
                                }

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
                        }
                        Err(e) => {
                            println!("{}", e);
                            return Err(StatusCodeError::new(0));
                        }
                    }
                }
                ReadStatus::Headers => {
                    buf.clear();
                    match reader.read_line(buf).await {
                        Ok(line_size) => {
                            if line_size < 2 {
                                return Err(StatusCodeError::new(0));
                            }

                            if line_size == 2 {
                                match msg.headers.content_length() {
                                    None => {}
                                    Some(cl) => {
                                        if cl < 0 {
                                            if !msg.headers.ischunked() {
                                                return Err(StatusCodeError::new(0));
                                            }
                                            is_chunked = true;
                                            msg.bodybuf = Some(BodyBuf::new(Some(ByteBuffer::new())));
                                        } else {
                                            body_remains = cl;
                                            if body_remains > 0 {
                                                let mut bbuf = ByteBuffer::new();
                                                bbuf.resize(body_remains as usize);
                                                msg.bodybuf = Some(BodyBuf::new(Some(bbuf)));

                                                unsafe {
                                                    let mut vec = buf.as_mut_vec();
                                                    vec.resize(vec.capacity(), 0);
                                                }
                                            }
                                        }
                                    }
                                }
                                status = ReadStatus::Body(body_remains);
                                continue;
                            }

                            if !buf.is_ascii() {
                                return Err(StatusCodeError::new(0));
                            }

                            let mut parts = buf.splitn(2, ':');
                            let key = match parts.next() {
                                None => {
                                    return Err(StatusCodeError::new(0));
                                }
                                Some(v) => {
                                    v
                                }
                            };

                            match parts.next() {
                                None => {
                                    return Err(StatusCodeError::new(0));
                                }
                                Some(v) => {
                                    msg.headers.append(key, v.trim());
                                }
                            }
                        }
                        Err(e) => {
                            println!("{}", e);
                            return Err(StatusCodeError::new(0));
                        }
                    }
                }
                ReadStatus::Body(_) => {
                    if is_chunked {
                        loop {
                            buf.clear();

                            match reader.read_line(buf).await {
                                Ok(line_size) => {
                                    if line_size < 2 {
                                        return Err(StatusCodeError::new(0));
                                    }

                                    if line_size == 2 {
                                        status = ReadStatus::Ok;
                                        break;
                                    }

                                    let mut remain_chunk_size = match buf.as_str()[0..line_size - 2].parse::<usize>() {
                                        Ok(v) => { v }
                                        Err(e) => {
                                            println!("ParseChunkSizeFailed: {:?}", e);
                                            return Err(StatusCodeError::new(0));
                                        }
                                    };

                                    unsafe {
                                        let mut vec = buf.as_mut_vec();
                                        vec.resize(vec.capacity(), 0);
                                    }

                                    let bytes: &mut [u8];
                                    unsafe {
                                        bytes = buf.as_bytes_mut();
                                    }

                                    loop {
                                        match reader.read(bytes).await {
                                            Ok(rbs) => {
                                                if rbs < 1 {
                                                    return Err(StatusCodeError::new(0));
                                                }

                                                msg.bodybuf.as_mut().unwrap().writeraw(&bytes[0..rbs]);

                                                if remain_chunk_size <= rbs {
                                                    break;
                                                }
                                                remain_chunk_size -= rbs;
                                            }
                                            Err(e) => {
                                                println!("{}", e);
                                                return Err(StatusCodeError::new(0));
                                            }
                                        }
                                    }

                                    match reader.read_u8().await {
                                        Ok(v) => {
                                            if v != b'\r' {
                                                return Err(StatusCodeError::new(0));
                                            }
                                        }
                                        Err(_) => {
                                            return Err(StatusCodeError::new(0));
                                        }
                                    }

                                    match reader.read_u8().await {
                                        Ok(v) => {
                                            if v != b'\n' {
                                                return Err(StatusCodeError::new(0));
                                            }
                                        }
                                        Err(_) => {
                                            return Err(StatusCodeError::new(0));
                                        }
                                    }
                                }
                                Err(e) => {
                                    println!("{:?}", e);
                                    return Err(StatusCodeError::new(0));
                                }
                            }
                        }
                    } else {
                        if body_remains > 0 {
                            let bytes: &mut [u8];
                            unsafe {
                                bytes = buf.as_bytes_mut();
                            }
                            match reader.read(bytes).await {
                                Ok(s) => {
                                    if s < 1 {
                                        return Err(StatusCodeError::new(0));
                                    }
                                    msg.bodybuf.as_mut().unwrap().writeraw(&bytes[0..s]);
                                    body_remains -= s as i64;
                                }
                                Err(e) => {
                                    println!("{}", e);
                                    return Err(StatusCodeError::new(0));
                                }
                            };
                        }

                        if body_remains <= 0 {
                            status = ReadStatus::Ok;
                        }
                    }
                }
                ReadStatus::Ok => {
                    return Ok(msg);
                }
            }
        }
    }
}

impl Write for Message {
    #[inline(always)]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut init = false;
        if self.bodybuf.is_none() {
            self.bodybuf = Some(BodyBuf::new(Some(ByteBuffer::new())));
            init = true;
        }

        let body = self.bodybuf.as_mut().unwrap();
        if init {
            if let Some(ct) = self._compress_type {
                body.begincompress(ct, Compression::default());
                self.headers.set_content_encoding(ct);
            }
        }

        body.write(buf)
    }

    #[inline(always)]
    fn flush(&mut self) -> std::io::Result<()> {
        let body = self.bodybuf.as_mut().unwrap();
        let result = body.flush();
        if self._compress_type.is_some() {
            match body.finishcompress() {
                Ok(_) => {}
                Err(e) => {
                    return Err(e);
                }
            };
        }
        result
    }
}
