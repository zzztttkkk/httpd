use tokio::io::{AsyncBufReadExt, AsyncReadExt};

use crate::{config::Config, uitls::multi_map::MultiMap};

enum MessageReadState {
    None,
    FirstLine1,
    FirstLine2,
    FirstLine3,
    HeadersDone,
}

#[derive(Debug, Default)]
pub(crate) struct Message {
    firstline: (String, String, String),
    headers: MultiMap,
    body: bytebuffer::ByteBuffer,
}

impl Message {
    #[inline]
    pub(crate) fn new() -> Self {
        Default::default()
    }

    pub(crate) async fn from<R: AsyncBufReadExt + Unpin>(
        &mut self,
        src: &mut R,
        rbuf: &mut Vec<u8>,
        config: &'static Config,
    ) {
        let mut state = MessageReadState::None;

        loop {
            match state {
                MessageReadState::None => {
                    let len = src.take(128).read_until(b' ', rbuf).await;

                    state = MessageReadState::FirstLine1;
                }
                MessageReadState::FirstLine1 => todo!(),
                MessageReadState::FirstLine2 => todo!(),
                MessageReadState::FirstLine3 => todo!(),
                MessageReadState::HeadersDone => todo!(),
            }
        }
    }
}
