use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

use crate::{
    ctx::ConnContext,
    message::{Message, MessageReadCode},
};

#[derive(Default)]
pub struct Request {
    pub(crate) msg: Message,
}

impl Request {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn method(&self) -> &String {
        &self.msg.firstline.0
    }

    pub fn url(&self) -> &String {
        &self.msg.firstline.1
    }

    pub fn version(&self) -> &String {
        &self.msg.firstline.2
    }
}
