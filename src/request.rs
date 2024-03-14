use crate::{
    ctx::ConnContext,
    internal::multi_map::MultiMap,
    message::{Message, MessageBody, MessageReadCode},
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

    pub fn headers(&self) -> &MultiMap {
        &self.msg.headers
    }

    pub fn body(&self) -> &MessageBody {
        &self.msg.body
    }
}
