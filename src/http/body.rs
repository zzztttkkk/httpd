use std::io::Write;
use bytes::BufMut;
use flate2::Compression;
use flate2::write::{DeflateEncoder, GzEncoder};
use flate2::read::{DeflateDecoder, GzDecoder};
use brotli2::read::BrotliDecoder;
use brotli2::write::BrotliEncoder;

pub(crate) struct BytesBuf {
    raw: bytes::BytesMut,
    cursor: usize,
}

impl BytesBuf {
    pub fn new() -> Self {
        return Self {
            raw: bytes::BytesMut::new(),
            cursor: 0,
        };
    }

    pub fn with_capacity(cap: usize) -> Self {
        return Self {
            raw: bytes::BytesMut::with_capacity(cap),
            cursor: 0,
        };
    }

    pub fn rseek(&mut self, idx: usize) -> &mut Self {
        if idx > self.raw.len() - 1 {
            panic!("bad idx");
        }
        self.cursor = idx;
        return self;
    }

    fn ptr(&self) -> BytesBufPtr {
        return BytesBufPtr(unsafe { std::mem::transmute(self) });
    }
}

impl std::io::Write for BytesBuf {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.raw.put_slice(buf);
        return Ok(buf.len());
    }

    #[inline]
    fn flush(&mut self) -> std::io::Result<()> {
        return Ok(());
    }
}

impl std::io::Read for BytesBuf {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if buf.len() < 1 || self.cursor >= self.raw.len() {
            return Ok(0);
        }

        let remain = self.raw.len() - self.cursor;
        if remain < 1 {
            return Ok(0);
        }

        let rl = std::cmp::min(remain, buf.len());
        let mut idx: usize = 0;
        loop {
            buf[idx] = self.raw[self.cursor];
            idx += 1;
            self.cursor += 1;
            if idx == rl {
                break;
            }
        }
        return Ok(rl);
    }
}

pub struct Body {
    buf: BytesBuf,

    _compression_type: Option<CompressionType>,
    _as_encoder: bool,
    encoder: Option<Box<dyn std::io::Write>>,
    flushed: bool,
    decoder: Option<Box<dyn std::io::Read>>,
}

unsafe impl Send for Body {}

unsafe impl Sync for Body {}

#[derive(Copy, Clone)]
struct BytesBufPtr(usize);

impl std::io::Write for BytesBufPtr {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mref: &mut BytesBuf = unsafe { std::mem::transmute(self.0) };
        mref.write(buf)
    }

    #[inline]
    fn flush(&mut self) -> std::io::Result<()> {
        let mref: &mut BytesBuf = unsafe { std::mem::transmute(self.0) };
        mref.flush()
    }
}

impl std::io::Read for BytesBufPtr {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mref: &mut BytesBuf = unsafe { std::mem::transmute(self.0) };
        mref.read(buf)
    }
}

#[derive(Clone, Copy)]
pub enum CompressionType {
    Gzip,
    Deflate,
    Br,
}

fn make_encoder(ctype: CompressionType, w: impl std::io::Write + 'static) -> Box<dyn std::io::Write + 'static> {
    match ctype {
        CompressionType::Gzip => {
            Box::new(GzEncoder::new(w, Compression::default()))
        }
        CompressionType::Deflate => {
            Box::new(DeflateEncoder::new(w, Compression::default()))
        }
        CompressionType::Br => {
            Box::new(BrotliEncoder::new(w, 6))
        }
    }
}

fn make_decoder(ctype: CompressionType, r: impl std::io::Read + 'static) -> Box<dyn std::io::Read + 'static> {
    match ctype {
        CompressionType::Gzip => {
            Box::new(GzDecoder::new(r))
        }
        CompressionType::Deflate => {
            Box::new(DeflateDecoder::new(r))
        }
        CompressionType::Br => {
            Box::new(BrotliDecoder::new(r))
        }
    }
}

impl Body {
    fn new(cap: usize, is_outgoing: bool) -> Self {
        return Self {
            buf: BytesBuf::with_capacity(cap),
            encoder: None,
            decoder: None,
            flushed: false,
            _compression_type: None,
            _as_encoder: is_outgoing,
        };
    }

    pub fn new_for_incoming(cap: usize) -> Self {
        return Self::new(cap, false);
    }

    pub fn new_for_outgoing(cap: usize) -> Self {
        return Self::new(cap, true);
    }

    pub fn set_compression_type(&mut self, ct: CompressionType) {
        assert!(self._compression_type.is_none(), "compression is not none");
        self._compression_type = Some(ct);
    }

    #[inline]
    pub fn get_compression_type(&self) -> Option<CompressionType> {
        return self._compression_type;
    }

    #[inline]
    pub fn len(&self) -> usize {
        return self.buf.raw.len();
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        return self.buf.raw.is_empty();
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        return self.buf.raw.capacity();
    }

    #[inline]
    pub fn bytes(&self) -> &[u8] {
        return &self.buf.raw[..];
    }
}

impl std::io::Write for Body {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if self._as_encoder {
            if let Some(ct) = self._compression_type {
                if self.encoder.is_none() {
                    self.encoder = Some(make_encoder(ct, self.buf.ptr()));
                }
            }
        }

        return match self.encoder.as_mut() {
            None => {
                self.buf.write(buf)
            }
            Some(encoder) => {
                encoder.write(buf)
            }
        };
    }

    #[inline]
    fn flush(&mut self) -> std::io::Result<()> {
        self.encoder = None;
        Ok(())
    }
}

impl std::io::Read for Body {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if !self._as_encoder {
            if let Some(ct) = self._compression_type {
                if self.decoder.is_none() {
                    self.decoder = Some(make_decoder(ct, self.buf.ptr()));
                }
            }
        }

        match self.decoder.as_mut() {
            None => {
                self.buf.read(buf)
            }
            Some(decoder) => {
                decoder.read(buf)
            }
        }
    }
}

impl Drop for Body {
    #[inline]
    fn drop(&mut self) {
        _ = self.flush();
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use bytes::BufMut;
    use flate2::Compression;
    use flate2::read::{DeflateDecoder, GzDecoder};
    use flate2::write::GzEncoder;
    use crate::http::body::{Body, BytesBuf, CompressionType, make_encoder};
    use crate::http::body::CompressionType::Gzip;
    use crate::utils;
    use brotli2::read::BrotliDecoder;
    use brotli2::write::BrotliEncoder;

    #[test]
    fn test_encode_gzip() {
        let input = "-----------hello world---------------";

        let mut body = Body::new_for_outgoing(1024);
        body.set_compression_type(CompressionType::Deflate);
        body.write(input.as_bytes());
        body.flush();

        let mut decoder = DeflateDecoder::new(&body.buf.raw[..]);
        let mut dest = String::new();
        decoder.read_to_string(&mut dest).unwrap();
        assert_eq!(input, dest.as_str());
    }

    #[test]
    fn test_encode_br() {
        let input = "-----------hello world---------------";

        let mut body = Body::new_for_outgoing(1024);
        body.set_compression_type(CompressionType::Br);
        body.write(input.as_bytes());
        body.flush();

        let mut decoder = BrotliDecoder::new(&mut body.buf);
        let vs = String::from_utf8(utils::read_all(&mut decoder, 0).unwrap()).unwrap();
        println!("{}", vs);
    }

    #[test]
    fn test_read() {
        let mut buf = BytesBuf::new();
        buf.write("hello world!".as_bytes());
        let data = utils::read_all(&mut buf, 0).unwrap();
        println!("{}", String::from_utf8(data).unwrap());
    }
}
