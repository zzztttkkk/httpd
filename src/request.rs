use crate::message::Message;

#[derive(Debug, Default)]
pub(crate) struct Request {
    pub(crate) msg: Message,
}

impl Request {
    pub fn new() -> Self {
        Default::default()
    }
}
