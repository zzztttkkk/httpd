use std::io::Write;

#[derive(Debug, Copy, Clone)]
pub enum CompressType {
    Gzip,
    Deflate,
}

pub trait CompressWriter: Write {
    fn finish(&mut self) -> std::io::Result<()>;
}

pub struct Gzip<W: Write>(flate2::write::GzEncoder<W>);

pub struct Deflate<W: Write>(flate2::write::DeflateEncoder<W>);

macro_rules! impl_compress_encoder {
    ($name:ty, $make:expr) => {
        impl<W> $name
        where
            W: Write,
        {
            #[inline(always)]
            pub fn new(w: W) -> Self {
                Self($make(w, flate2::Compression::default()))
            }

            #[inline(always)]
            pub fn with_level(w: W, level: flate2::Compression) -> Self {
                Self($make(w, level))
            }
        }

        impl<W> Write for $name
        where
            W: Write,
        {
            #[inline(always)]
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                self.0.write(buf)
            }

            #[inline(always)]
            fn flush(&mut self) -> std::io::Result<()> {
                self.0.flush()
            }
        }

        impl<W> $crate::http::compress::CompressWriter for $name
        where
            W: Write,
        {
            #[inline(always)]
            fn finish(&mut self) -> std::io::Result<()> {
                self.0.try_finish()
            }
        }
    };
}

impl_compress_encoder!(Gzip<W>, flate2::write::GzEncoder::new);
impl_compress_encoder!(Deflate<W>, flate2::write::DeflateEncoder::new);
