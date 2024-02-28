use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt};
use toml::de;

use crate::{config::Config, ctx::ConnContext, uitls::multi_map::MultiMap};

enum ReadState {
    None,
    FirstLine0,
    FirstLine1,
    FirstLine2,
    HeadersDone,
}

#[derive(Debug, Default)]
pub(crate) struct Message {
    firstline: (String, String, String),
    headers: MultiMap,
    body: bytebuffer::ByteBuffer,
}

pub(crate) enum MessageReadCode {
    Ok,
    ConnReadError,
    BadDatagram,
}

impl Message {
    #[inline]
    pub(crate) fn new() -> Self {
        Default::default()
    }

    pub(crate) async fn from11<R: AsyncBufReadExt + Unpin, W: AsyncWriteExt + Unpin>(
        &mut self,
        ctx: &mut ConnContext<R, W>,
    ) -> MessageReadCode {
        let mut state = ReadState::None;
        let reader = &mut ctx.reader;
        let buf = &mut ctx.buf;

        loop {
            match state {
                ReadState::None => {
                    let dest = unsafe { (&mut self.firstline.0).as_mut_vec() };
                    match reader.take(128).read_until(b' ', dest).await {
                        Ok(size) => {
                            if size < 1 {
                                return MessageReadCode::BadDatagram;
                            }
                            state = ReadState::FirstLine0;
                            continue;
                        }
                        Err(_) => {
                            return MessageReadCode::ConnReadError;
                        }
                    }
                }
                ReadState::FirstLine0 => {
                    let dest = unsafe { (&mut self.firstline.1).as_mut_vec() };
                    match reader.take(128).read_until(b' ', dest).await {
                        Ok(size) => {
                            if size < 1 {
                                return MessageReadCode::BadDatagram;
                            }
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
                            if size < 1 {
                                return MessageReadCode::BadDatagram;
                            }
                            state = ReadState::FirstLine2;
                            continue;
                        }
                        Err(_) => {
                            return MessageReadCode::ConnReadError;
                        }
                    }
                }
                ReadState::FirstLine2 => {
                    loop {
                    }
                }
                ReadState::HeadersDone => todo!(),
            }
        }

        MessageReadCode::Ok
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_unsafe_steing() {
        let mut name = String::new();
        let bytes = unsafe { (&mut name).as_mut_vec() };
        bytes.push(211);
        bytes.push(241);

        println!("{:?}", name.len());
    }
}
