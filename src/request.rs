use crate::message::Message;

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
