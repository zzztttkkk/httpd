use std::io::Write;

pub(crate) trait WriteCompressionImpl<W> {
    fn append(&mut self, buf: &[u8]) -> std::io::Result<usize>;
    fn expose(self) -> std::io::Result<W>;
}

macro_rules! compression_impl_for_flate2 {
    ($name:ident) => {
        impl WriteCompressionImpl<Box<bytebuffer::ByteBuffer>>
            for flate2::write::$name<Box<bytebuffer::ByteBuffer>>
        {
            fn append(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                self.write(buf)
            }

            fn expose(self) -> std::io::Result<Box<bytebuffer::ByteBuffer>> {
                self.finish()
            }
        }
    };
}

compression_impl_for_flate2!(GzEncoder);
compression_impl_for_flate2!(GzDecoder);
compression_impl_for_flate2!(DeflateEncoder);
compression_impl_for_flate2!(DeflateDecoder);

impl WriteCompressionImpl<Box<bytebuffer::ByteBuffer>>
    for brotli::CompressorWriter<Box<bytebuffer::ByteBuffer>>
{
    fn append(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.write(buf)
    }

    fn expose(mut self) -> std::io::Result<Box<bytebuffer::ByteBuffer>> {
        match self.flush() {
            Ok(_) => Ok(self.into_inner()),
            Err(e) => Err(e),
        }
    }
}

impl WriteCompressionImpl<Box<bytebuffer::ByteBuffer>>
    for brotli::DecompressorWriter<Box<bytebuffer::ByteBuffer>>
{
    fn append(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.write(buf)
    }

    fn expose(mut self) -> std::io::Result<Box<bytebuffer::ByteBuffer>> {
        _ = self.close()?;
        match self.into_inner() {
            Ok(v) => Ok(v),
            Err(v) => Ok(v),
        }
    }
}
