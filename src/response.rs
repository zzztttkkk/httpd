use crate::message::Message;

#[derive(Default)]
pub(crate) struct Response {
    pub(crate) msg: Message,
}

impl Response {
    pub(crate) fn new() -> Self {
        Self {
            msg: Default::default(),
        }
    }
}

#[derive(Default)]
pub struct ResponseBuilder {
    inner: Message,
}

impl ResponseBuilder {
    pub fn new() -> Self {
        Self {
            inner: Default::default(),
        }
    }

    pub fn finish(self) -> Message {
        self.inner
    }

    pub fn with_code(&mut self, code: u32, reason: &str) -> &mut Self {
        self.inner.firstline.0.push_str(code.to_string().as_str());
        self.inner.firstline.1.push_str(reason);
        self
    }

    pub fn with_version(&mut self, major: u8, minor: u8) -> &mut Self {
        self.inner
            .firstline
            .2
            .push_str(format!("HTTP/{}.{}", major, minor).as_str());
        self
    }

    pub fn append_header(&mut self, k: &str, v: &str) -> &mut Self {
        self.inner.headers.append(k, v);
        self
    }

    pub fn set_header(&mut self, k: &str, v: &str) -> &mut Self {
        self.inner.headers.set(k, v);
        self
    }
}
