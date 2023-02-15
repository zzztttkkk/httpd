use std::fmt::Formatter;
use std::io::{Read, Write};

use bytebuffer::ByteBuffer;
use flate2::Compression;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWriteExt};

use crate::config::Config;
use crate::http::compress::{CompressType, CompressWriter, Deflate, Gzip};
use crate::http::headers::Headers;

pub struct ByteBufferWrapper {
    ptr: *mut Option<ByteBuffer>,
}

unsafe impl Send for ByteBufferWrapper {}

impl ByteBufferWrapper {
    #[inline(always)]
    fn bufref(&mut self) -> &mut ByteBuffer {
        unsafe { self.ptr.as_mut().unwrap().as_mut().unwrap() }
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

    #[inline]
    pub fn writeraw(&mut self, buf: &[u8]) {
        ByteBuffer::write(self.raw.as_mut().unwrap(), buf).unwrap();
    }

    pub(crate) fn clear(&mut self) {
        if self.decoder.is_some() {
            self.decoder = None;
        }

        if self.encoder.is_some() {
            if !self._encoder_finished {
                let _ = self.finishcompress();
            }
            self.encoder = None;
            self._encoder_finished = false;
        }

        if let Some(buf) = &mut self.raw {
            buf.clear();
        }
    }

    #[inline(always)]
    pub fn decompress(&mut self, ct: CompressType) {
        match ct {
            CompressType::Gzip => {
                self.decoder = Some(Box::new(flate2::read::GzDecoder::new(ByteBufferWrapper {
                    ptr: &mut self.raw,
                })));
            }
            CompressType::Deflate => {
                self.decoder = Some(Box::new(flate2::read::DeflateDecoder::new(
                    ByteBufferWrapper { ptr: &mut self.raw },
                )));
            }
        }
    }

    #[inline(always)]
    pub fn begincompress(&mut self, ct: CompressType, level: flate2::Compression) {
        match ct {
            CompressType::Gzip => {
                self.encoder = Some(Box::new(Gzip::with_level(
                    ByteBufferWrapper { ptr: &mut self.raw },
                    level,
                )));
            }
            CompressType::Deflate => {
                self.encoder = Some(Box::new(Deflate::with_level(
                    ByteBufferWrapper { ptr: &mut self.raw },
                    level,
                )));
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
            None => Ok(()),
            Some(encoder) => encoder.finish(),
        }
    }

    #[inline]
    pub fn raw(&self) -> Option<&ByteBuffer> {
        self.raw.as_ref()
    }
}

impl Read for BodyBuf {
    #[inline(always)]
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match &mut self.decoder {
            None => self.raw.as_mut().unwrap().read(buf),
            Some(decoder) => decoder.read(buf),
        }
    }
}

impl Write for BodyBuf {
    #[inline(always)]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match &mut self.encoder {
            None => self.raw.as_mut().unwrap().write(buf),
            Some(encoder) => encoder.write(buf),
        }
    }

    #[inline(always)]
    fn flush(&mut self) -> std::io::Result<()> {
        match &mut self.encoder {
            None => self.raw.as_mut().unwrap().flush(),
            Some(encoder) => encoder.flush(),
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
                write!(
                    f,
                    " len:{}, rpos:{}, wpos:{}",
                    v.len(),
                    v.get_rpos(),
                    v.get_wpos()
                )
                .unwrap();
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
        let _ = self.finishcompress();
    }
}

#[derive(Default)]
pub struct Range {
    pub begin: u64,
    pub length: u64,
}

pub struct Message {
    pub(crate) f0: String,
    pub(crate) f1: String,
    pub(crate) f2: String,
    pub(crate) headers: Headers,
    pub(crate) bodybuf: Option<BodyBuf>,

    pub(crate) output_compress_type: Option<CompressType>,
    pub(crate) output_readobj: Option<Box<dyn AsyncRead + Send>>,
    pub(crate) output_ranges: Option<Vec<Range>>,
    pub(crate) output_content_type: String,
}

#[derive(Debug, Clone, Copy)]
enum ReadStatus {
    None,
    Headers,
    Body,
    Ok,
}

pub static ERR_BAD_STREAM: u32 = 0;
pub static ERR_FIRST_LINE_TOO_LARGE: u32 = 414;
pub static ERR_NON_ASCII_IN_FIRST_LINE: u32 = 400;
pub static ERR_HEADER_LINE_TOOL_LARGE: u32 = 431;
pub static ERR_HEADERS_COUNT_TOO_MANY: u32 = 431;
pub static ERR_NON_ASCII_IN_HEADERS: u32 = 2;
pub static ERR_MAYBE_HTTP2: u32 = 99;

impl Message {
    pub(crate) fn new() -> Self {
        Self {
            f0: "".to_string(),
            f1: "".to_string(),
            f2: "".to_string(),
            headers: Headers::new(),
            bodybuf: None,
            output_compress_type: None,
            output_content_type: "".to_string(),
            output_readobj: None,
            output_ranges: None,
        }
    }

    pub(crate) fn clear(&mut self) {
        self.f0.clear();
        self.f1.clear();
        self.f2.clear();
        self.headers.clear();
        if let Some(body) = &mut self.bodybuf {
            body.clear();
        }
    }

    // todo safe read_line
    pub(crate) async fn from11<'a, R: AsyncBufReadExt + Unpin>(
        reader: &'a mut R,
        buf: &'a mut String,
        cfg: &'static Config,
    ) -> Result<Box<Self>, u32> {
        buf.clear();
        match reader
            .take((cfg.message.max_first_line_size) as u64)
            .read_line(buf)
            .await
        {
            Ok(line_size) => {
                if (line_size <= 2) {
                    return Err(ERR_BAD_STREAM);
                }
                if line_size >= cfg.message.max_first_line_size {
                    return Err(ERR_FIRST_LINE_TOO_LARGE);
                }
            }
            Err(_) => return Err(ERR_BAD_STREAM),
        }

        let mut status = ReadStatus::None;
        let mut msg = Box::new(Message::new());

        let mut body_remains: i64 = 0;
        let mut is_chunked = false;
        let mut header_count = 0;

        loop {
            match status {
                ReadStatus::None => {
                    if buf.starts_with("PRI * HTTP/2.0") {
                        return Err(ERR_MAYBE_HTTP2);
                    }

                    let mut fls = 0;
                    for rune in buf.chars() {
                        if rune > 127 as char {
                            return Err(ERR_NON_ASCII_IN_FIRST_LINE);
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
                ReadStatus::Headers => {
                    buf.clear();
                    match reader
                        .take((cfg.message.max_header_line_size) as u64)
                        .read_line(buf)
                        .await
                    {
                        Ok(line_size) => {
                            if line_size < 2 {
                                return Err(ERR_BAD_STREAM);
                            }

                            if line_size == 2 {
                                if msg.headers.ischunked() {
                                    is_chunked = true;
                                    msg.bodybuf = Some(BodyBuf::new(Some(ByteBuffer::new())));
                                } else {
                                    let cl = msg.headers.content_length();
                                    if cl > 0 {
                                        if cfg.message.max_incoming_body_size > 0
                                            && cl > cfg.message.max_incoming_body_size
                                        {
                                            return Err(0);
                                        }

                                        body_remains = cl as i64;
                                        let mut bbuf = ByteBuffer::new();
                                        bbuf.resize(body_remains as usize);
                                        msg.bodybuf = Some(BodyBuf::new(Some(bbuf)));

                                        unsafe {
                                            let vec = buf.as_mut_vec();
                                            vec.resize(vec.capacity(), 0);
                                        }
                                    }
                                }

                                if let Some(ct) = msg.headers.content_encoding() {
                                    msg.bodybuf.as_mut().unwrap().decompress(ct);
                                }

                                status = ReadStatus::Body;
                                continue;
                            }

                            if line_size >= cfg.message.max_header_line_size {
                                return Err(ERR_HEADER_LINE_TOOL_LARGE);
                            }

                            if header_count > cfg.message.max_header_count {
                                return Err(ERR_HEADERS_COUNT_TOO_MANY);
                            }

                            if !buf.is_ascii() {
                                return Err(ERR_NON_ASCII_IN_HEADERS);
                            }

                            let mut parts = buf.splitn(2, ':');
                            let key = match parts.next() {
                                None => {
                                    return Err(ERR_BAD_STREAM);
                                }
                                Some(v) => v,
                            };

                            match parts.next() {
                                None => {
                                    return Err(ERR_BAD_STREAM);
                                }
                                Some(v) => {
                                    msg.headers.append(key, v.trim());
                                    header_count += 1;
                                }
                            }
                        }
                        Err(e_) => {
                            return Err(ERR_BAD_STREAM);
                        }
                    }
                }
                ReadStatus::Body => {
                    if is_chunked {
                        loop {
                            buf.clear();
                            match reader.take(36).read_line(buf).await {
                                Ok(line_size) => {
                                    if line_size < 2 || line_size >= 36 {
                                        return Err(ERR_BAD_STREAM);
                                    }

                                    if line_size == 2 {
                                        status = ReadStatus::Ok;
                                        break;
                                    }

                                    let mut remain_chunk_size =
                                        match buf.as_str()[0..line_size - 2].parse::<usize>() {
                                            Ok(v) => v,
                                            Err(e) => {
                                                return Err(ERR_BAD_STREAM);
                                            }
                                        };

                                    unsafe {
                                        let vec = buf.as_mut_vec();
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
                                                    return Err(ERR_BAD_STREAM);
                                                }

                                                msg.bodybuf
                                                    .as_mut()
                                                    .unwrap()
                                                    .writeraw(&bytes[0..rbs]);

                                                if remain_chunk_size <= rbs {
                                                    break;
                                                }
                                                remain_chunk_size -= rbs;
                                            }
                                            Err(e) => {
                                                return Err(ERR_BAD_STREAM);
                                            }
                                        }
                                    }

                                    match reader.read_u8().await {
                                        Ok(v) => {
                                            if v != b'\r' {
                                                return Err(ERR_BAD_STREAM);
                                            }
                                        }
                                        Err(_) => {
                                            return Err(ERR_BAD_STREAM);
                                        }
                                    }

                                    match reader.read_u8().await {
                                        Ok(v) => {
                                            if v != b'\n' {
                                                return Err(ERR_BAD_STREAM);
                                            }
                                        }
                                        Err(_) => {
                                            return Err(ERR_BAD_STREAM);
                                        }
                                    }
                                }
                                Err(_) => {
                                    return Err(ERR_BAD_STREAM);
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
                                        return Err(ERR_BAD_STREAM);
                                    }
                                    msg.bodybuf.as_mut().unwrap().writeraw(&bytes[0..s]);
                                    body_remains -= s as i64;
                                }
                                Err(_) => {
                                    return Err(ERR_BAD_STREAM);
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

    fn body_buf_size(&self) -> usize {
        match &self.bodybuf {
            None => 0,
            Some(buf) => match &buf.raw {
                None => 0,
                Some(buf) => buf.len(),
            },
        }
    }

    pub(crate) async fn to11<Writer: AsyncWriteExt + Unpin>(
        &mut self,
        writer: &mut Writer,
    ) -> std::io::Result<()> {
        (Writer::write(
            writer,
            format!("HTTP/1.1 {} {}\r\n", self.f1, self.f2).as_bytes(),
        )
        .await)?;

        let body_buf_size = self.body_buf_size();
        if self.output_readobj.is_none() {
            self.headers.set_content_length(body_buf_size);
        } else if self.output_ranges.is_none() {
        }

        if let Some(map) = self.headers.map() {
            for (key, vals) in map {
                for val in vals {
                    (Writer::write(writer, format!("{}: {}\r\n", key, val).as_bytes()).await)?;
                }
            }
        }
        (Writer::write(writer, "\r\n".as_bytes()).await)?;

        match &mut self.output_readobj {
            Some(readobj) => match self.output_ranges.as_ref() {
                Some(ranges) => {
                    todo!()
                }
                None => {
                    todo!()
                }
            },
            None => {
                if body_buf_size > 0 {
                    let buf = self.bodybuf.as_ref().unwrap().raw().unwrap();
                    (Writer::write(writer, buf.as_bytes()).await)?;
                }
            }
        }

        Ok(())
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
            if let Some(ct) = self.output_compress_type {
                body.begincompress(ct, Compression::default());
                self.headers.set_content_encoding(ct);
            }
        }

        body.write(buf)
    }

    #[inline]
    fn flush(&mut self) -> std::io::Result<()> {
        if self.bodybuf.is_none() {
            return Ok(());
        }

        let body = self.bodybuf.as_mut().unwrap();
        let result = body.flush();
        if self.output_compress_type.is_some() {
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
