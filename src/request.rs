use crate::{
    ctx::ConnContext,
    message::{Message, MessageReadCode},
};

pub struct RequestQueryer<'a> {
    msg: &'a Message,
}

impl<'a> RequestQueryer<'a> {
    pub fn new(msg: &'a Message) -> Self {
        Self { msg }
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
