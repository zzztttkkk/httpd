use crate::message::Message;

#[derive(Default)]
pub(crate) struct Request {
    pub(crate) msg: Message,
}

impl Request {
    pub fn new() -> Self {
        Default::default()
    }
}
