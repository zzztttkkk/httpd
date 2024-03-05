use std::io::Write;

use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt};

use crate::compression::WriteCompressionImpl;
use crate::config::http::HttpConfig;
use crate::uitls::header;
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
    pub(crate) cw: Option<Box<dyn WriteCompressionImpl<Box<bytebuffer::ByteBuffer>> + Send>>,
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
    pub(crate) fn compression(mut self, ct: CompressionType, level: u32) {
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

    pub(crate) fn decompression(mut self, ct: CompressionType) {
        let buf = self.internal.unwrap();
        self.internal = None;

        match ct {
            CompressionType::Brotil => {
                self.cw = Some(Box::new(brotli::DecompressorWriter::new(buf, 4096)));
            }
            CompressionType::Deflate => {
                self.cw = Some(Box::new(flate2::write::DeflateDecoder::new(buf)));
            }
            CompressionType::Gzip => {
                self.cw = Some(Box::new(flate2::write::GzDecoder::new(buf)));
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
    ReachMaxHeadersCount,
    BadChunkSize,
}

const MAX_HEADER_NAME_LENGTH: usize = 256;

macro_rules! read_const_length_body_impl {
    ($self:ident, $reader:ident, $buf:ident, $remain_size:ident, $write:ident ) => {
        if $remain_size < 1 {
            return MessageReadCode::Ok;
        }

        let bufcap = $buf.capacity();
        loop {
            let mut _buf = $buf.as_mut_slice();
            if $remain_size < bufcap {
                _buf = &mut _buf[..$remain_size];
            }
            match $reader.read(_buf).await {
                Ok(size) => {
                    $self.$write(&_buf[..size]);
                    $remain_size -= size;
                    if $remain_size < 1 {
                        return MessageReadCode::Ok;
                    }
                }
                Err(_) => {
                    return MessageReadCode::ConnReadError;
                }
            }
        }
    };
}

macro_rules! read_chunked_body_impl {
    ($self:ident, $reader:ident, $buf:ident, $max_body_size:ident, $write:ident) => {
        let bufcap = $buf.capacity();
        let mut lenline = String::with_capacity(128);
        let mut remain_size: usize;
        let mut read_size: usize = 0;
        loop {
            lenline.clear();
            match $reader.take(128).read_line(&mut lenline).await {
                Ok(_) => match lenline.as_str().trim().parse::<usize>() {
                    Ok(num) => {
                        remain_size = num;
                    }
                    Err(_) => {
                        return MessageReadCode::BadChunkSize;
                    }
                },
                Err(_) => {
                    return MessageReadCode::ConnReadError;
                }
            }

            read_size += remain_size;
            if read_size > $max_body_size {
                return MessageReadCode::ReachMaxBodySize;
            }

            loop {
                if remain_size < 1 {
                    break;
                }

                let mut _buf = $buf.as_mut_slice();
                if remain_size < bufcap {
                    _buf = &mut _buf[..remain_size];
                }

                match $reader.read(_buf).await {
                    Ok(size) => {
                        $self.$write(&_buf[..size]);
                        remain_size -= size;
                        if remain_size < 1 {
                            break;
                        }
                    }
                    Err(_) => {
                        return MessageReadCode::ConnReadError;
                    }
                }
            }

            match $reader.take(2).read_line(&mut lenline).await {
                Ok(_) => return MessageReadCode::Ok,
                Err(_) => return MessageReadCode::BadDatagram,
            }
        }
    };
}

#[inline]
fn is_latin_1_graphic(b: u8) -> bool {
    matches!(b, b' '..=b'~' | b'\xa0'..=b'\xff')
}

impl Message {
    pub(crate) fn get_content_length(&self) -> Result<usize, ()> {
        match self.headers.get("content-length") {
            Some(vs) => match vs.parse::<usize>() {
                Ok(num) => Ok(num),
                Err(_) => Err(()),
            },
            None => Ok(0),
        }
    }

    fn write_raw(&mut self, v: &[u8]) {
        _ = self.body.internal.as_mut().unwrap().write(v);
    }

    fn write_compression(&mut self, v: &[u8]) {
        _ = self.body.write(v);
    }

    async fn _read_const_length_body<R: AsyncBufReadExt + Unpin>(
        &mut self,
        reader: &mut R,
        buf: &mut Vec<u8>,
        mut remain_size: usize,
    ) -> MessageReadCode {
        read_const_length_body_impl!(self, reader, buf, remain_size, write_raw);
    }

    #[inline]
    pub(crate) async fn read_const_length_body<
        R: AsyncBufReadExt + Unpin,
        W: AsyncWriteExt + Unpin,
    >(
        &mut self,
        ctx: &mut ConnContext<R, W>,
    ) -> MessageReadCode {
        match self.get_content_length() {
            Ok(size) => {
                self._read_const_length_body(&mut ctx.reader, &mut ctx.buf, size)
                    .await
            }
            Err(_) => MessageReadCode::BadContentLength,
        }
    }

    pub(crate) async fn read_const_length_body_decompression<R: AsyncBufReadExt + Unpin>(
        &mut self,
        reader: &mut R,
        buf: &mut Vec<u8>,
        mut remain_size: usize,
    ) -> MessageReadCode {
        read_const_length_body_impl!(self, reader, buf, remain_size, write_compression);
    }

    pub(crate) async fn read_chunked_body<R: AsyncBufReadExt + Unpin>(
        &mut self,
        reader: &mut R,
        buf: &mut Vec<u8>,
        max_body_size: usize,
    ) -> MessageReadCode {
        read_chunked_body_impl!(self, reader, buf, max_body_size, write_raw);
    }

    pub(crate) async fn read_chunked_body_decompression<R: AsyncBufReadExt + Unpin>(
        &mut self,
        reader: &mut R,
        buf: &mut Vec<u8>,
        max_body_size: usize,
    ) -> MessageReadCode {
        read_chunked_body_impl!(self, reader, buf, max_body_size, write_compression);
    }

    pub(crate) async fn read_body_normal<R: AsyncBufReadExt + Unpin>(
        &mut self,
        reader: &mut R,
        buf: &mut Vec<u8>,
        config: &'static HttpConfig,
    ) -> MessageReadCode {
        let cap = buf.capacity();
        unsafe { buf.set_len(cap) }; // safety: just bytes array, no ref

        if header::contains(self.headers.getall("transfer-encoding"), "chunked") {
            return self
                .read_chunked_body(reader, buf, config.max_body_size.0)
                .await;
        }

        match self.get_content_length() {
            Ok(remain_size) => {
                if remain_size > config.max_body_size.0 {
                    return MessageReadCode::ReachMaxBodySize;
                }
                self._read_const_length_body(reader, buf, remain_size).await
            }
            Err(_) => MessageReadCode::BadContentLength,
        }
    }

    pub(crate) async fn read_headers<R: AsyncBufReadExt + Unpin, W: AsyncWriteExt + Unpin>(
        &mut self,
        ctx: &mut ConnContext<R, W>,
    ) -> MessageReadCode {
        let mut state = ReadState::None;
        let reader = &mut ctx.reader;
        let buf = &mut ctx.buf;
        let config = &(ctx.config.http);

        macro_rules! ensure_lantin_1 {
            ($bytes:expr) => {
                for b in ($bytes) {
                    if !is_latin_1_graphic(*b) {
                        return MessageReadCode::BadDatagram;
                    }
                }
            };
        }

        let max_header_line_size = config.max_header_line_size.u64();
        let max_headers_count = config.max_headers_count;

        loop {
            match state {
                ReadState::None => {
                    let dest = unsafe { (&mut self.firstline.0).as_mut_vec() }; // safety: no double copy
                    match reader.take(128).read_until(b' ', dest).await {
                        Ok(size) => {
                            if size < 1 {
                                return MessageReadCode::BadDatagram;
                            }
                            unsafe { dest.set_len(size - 1) }; // safety: trim last space and len check in front
                            ensure_lantin_1!(dest);
                            state = ReadState::FirstLine0;
                            continue;
                        }
                        Err(_) => {
                            return MessageReadCode::ConnReadError;
                        }
                    }
                }
                ReadState::FirstLine0 => {
                    let dest = unsafe { (&mut self.firstline.1).as_mut_vec() }; // safety: no double copy
                    match reader
                        .take(config.max_url_size.u64())
                        .read_until(b' ', dest)
                        .await
                    {
                        Ok(size) => {
                            if size < 1 {
                                return MessageReadCode::BadDatagram;
                            }
                            unsafe { dest.set_len(size - 1) }; // safety: trim last space and len check in front
                            ensure_lantin_1!(dest);
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
                            let bytes = unsafe { (&mut self.firstline.2).as_mut_vec() }; // safety: trim `\r\n` and len check in front
                            unsafe { bytes.set_len(size - 2) };
                            ensure_lantin_1!(bytes);
                            state = ReadState::FirstLine2;
                            continue;
                        }
                        Err(_) => {
                            return MessageReadCode::ConnReadError;
                        }
                    }
                }
                ReadState::FirstLine2 => {
                    let mut keytmp = [0 as u8; MAX_HEADER_NAME_LENGTH];
                    let mut keyidx: usize;

                    'readlines: loop {
                        buf.clear();
                        let mut hc = 0;
                        match reader
                            .take(max_header_line_size)
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
                                unsafe { buf.set_len(size - 2) }; // safety: trim `\r\n` and len check in front

                                keyidx = 0;
                                for idx in 0..buf.len() {
                                    let c = buf[idx];
                                    if !is_latin_1_graphic(c) {
                                        return MessageReadCode::BadDatagram;
                                    }

                                    if c == b':' {
                                        // safety: all bytes in `keytmp` is_ascii_graphic
                                        let key = unsafe {
                                            std::str::from_utf8_unchecked(&keytmp[..keyidx])
                                        }
                                        .trim();

                                        if idx + 1 >= buf.len() {
                                            return MessageReadCode::BadDatagram;
                                        }

                                        // safety: not calling `std::str::from_utf8`, because i only want ascii chars in the header value
                                        ensure_lantin_1!(&buf[(idx + 1)..]);
                                        let value: &str = unsafe {
                                            std::str::from_utf8_unchecked(&buf[(idx + 1)..])
                                        }
                                        .trim();

                                        self.headers.append(key, value);
                                        hc += 1;
                                        if hc > max_headers_count {
                                            return MessageReadCode::ReachMaxHeadersCount;
                                        }
                                        break 'readlines;
                                    }

                                    if keyidx >= MAX_HEADER_NAME_LENGTH {
                                        return MessageReadCode::BadDatagram;
                                    }
                                    // safety: boundary check in front
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
                ReadState::HeadersDone => {
                    return MessageReadCode::Ok;
                }
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
