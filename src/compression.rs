use std::io::Write;

pub(crate) trait _WriteCompressionImpl {
    fn append(&mut self, buf: &[u8]) -> std::io::Result<usize>;
    fn _expose(self) -> std::io::Result<Box<bytebuffer::ByteBuffer>>;
}

pub trait BoxedWriteCompressionImpl: _WriteCompressionImpl {
    fn expose(self: Box<Self>) -> std::io::Result<Box<bytebuffer::ByteBuffer>>;
}

impl<T: _WriteCompressionImpl> BoxedWriteCompressionImpl for T
where
    T: Sized,
{
    fn expose(self: Box<Self>) -> std::io::Result<Box<bytebuffer::ByteBuffer>> {
        self._expose()
    }
}

macro_rules! compression_impl_for_flate2 {
    ($name:ident) => {
        impl _WriteCompressionImpl for flate2::write::$name<Box<bytebuffer::ByteBuffer>> {
            fn append(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                self.write(buf)
            }

            fn _expose(self) -> std::io::Result<Box<bytebuffer::ByteBuffer>> {
                self.finish()
            }
        }
    };
}

compression_impl_for_flate2!(GzEncoder);
compression_impl_for_flate2!(GzDecoder);
compression_impl_for_flate2!(DeflateEncoder);
compression_impl_for_flate2!(DeflateDecoder);

impl _WriteCompressionImpl for brotli::CompressorWriter<Box<bytebuffer::ByteBuffer>> {
    fn append(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.write(buf)
    }

    fn _expose(mut self) -> std::io::Result<Box<bytebuffer::ByteBuffer>> {
        match self.flush() {
            Ok(_) => Ok(self.into_inner()),
            Err(e) => Err(e),
        }
    }
}

impl _WriteCompressionImpl for brotli::DecompressorWriter<Box<bytebuffer::ByteBuffer>> {
    fn append(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.write(buf)
    }

    fn _expose(mut self) -> std::io::Result<Box<bytebuffer::ByteBuffer>> {
        _ = self.close()?;
        match self.into_inner() {
            Ok(v) => Ok(v),
            Err(v) => Ok(v),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BoxedWriteCompressionImpl;

    #[test]
    fn test_br() {
        let input = "HelloWorld".repeat(1024);

        let mut cw: Box<dyn BoxedWriteCompressionImpl> = Box::new(brotli::CompressorWriter::new(
            Box::new(bytebuffer::ByteBuffer::default()),
            4096,
            7,
            22,
        ));
        cw.append(input.as_bytes()).unwrap();
        let buf = cw.expose().unwrap();
        println!("{}", buf.len());

        let mut dw: Box<dyn BoxedWriteCompressionImpl> = Box::new(brotli::DecompressorWriter::new(
            Box::new(bytebuffer::ByteBuffer::default()),
            4096,
        ));
        dw.append(buf.as_bytes()).unwrap();
        let buf = dw.expose().unwrap();
        println!("{} {}", buf.len(), buf.as_bytes() == input.as_bytes());
    }

    #[test]
    fn test_gzip() {
        let input = "HelloWorld".repeat(1024);

        let mut cw: Box<dyn BoxedWriteCompressionImpl> = Box::new(flate2::write::GzEncoder::new(
            Box::new(bytebuffer::ByteBuffer::default()),
            flate2::Compression::new(7),
        ));
        cw.append(input.as_bytes()).unwrap();
        let buf = cw.expose().unwrap();
        println!("{}", buf.len());

        let mut dw: Box<dyn BoxedWriteCompressionImpl> = Box::new(flate2::write::GzDecoder::new(
            Box::new(bytebuffer::ByteBuffer::default()),
        ));

        let mut pos = 0;
        while pos < buf.as_bytes().len() {
            let v = dw.append(&buf.as_bytes()[pos..]).unwrap();
            pos += v;
        }

        let buf = dw.expose().unwrap();
        println!("{} {}", buf.len(), buf.as_bytes() == input.as_bytes());
    }

    #[test]
    fn test_deflate() {
        let input = "HelloWorld".repeat(1024);

        let mut cw: Box<dyn BoxedWriteCompressionImpl> =
            Box::new(flate2::write::DeflateEncoder::new(
                Box::new(bytebuffer::ByteBuffer::default()),
                flate2::Compression::new(7),
            ));
        cw.append(input.as_bytes()).unwrap();
        let buf = cw.expose().unwrap();
        println!("{}", buf.len());

        let mut dw: Box<dyn BoxedWriteCompressionImpl> = Box::new(
            flate2::write::DeflateDecoder::new(Box::new(bytebuffer::ByteBuffer::default())),
        );

        let mut pos = 0;
        while pos < buf.as_bytes().len() {
            let v = dw.append(&buf.as_bytes()[pos..]).unwrap();
            pos += v;
        }

        let buf = dw.expose().unwrap();
        println!("{} {}", buf.len(), buf.as_bytes() == input.as_bytes());
    }
}
