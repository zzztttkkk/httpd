use std::io::Write;

pub(crate) trait CompressionImpl<W> {
    fn append(&mut self, buf: &[u8]) -> std::io::Result<usize>;
    fn expose(self) -> std::io::Result<W>;
}

impl CompressionImpl<Box<bytebuffer::ByteBuffer>>
    for flate2::write::GzEncoder<Box<bytebuffer::ByteBuffer>>
{
    fn append(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.write(buf)
    }

    fn expose(self) -> std::io::Result<Box<bytebuffer::ByteBuffer>> {
        self.finish()
    }
}

impl CompressionImpl<Box<bytebuffer::ByteBuffer>>
    for flate2::write::DeflateEncoder<Box<bytebuffer::ByteBuffer>>
{
    fn append(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.write(buf)
    }

    fn expose(self) -> std::io::Result<Box<bytebuffer::ByteBuffer>> {
        self.finish()
    }
}

impl CompressionImpl<Box<bytebuffer::ByteBuffer>>
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
