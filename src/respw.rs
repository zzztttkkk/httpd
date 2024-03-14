use crate::message::Message;

pub(crate) struct ResponseWriter<'a> {
    msg: &'a mut Message,
}

impl<'a> From<&'a mut Message> for ResponseWriter<'a> {
    fn from(value: &'a mut Message) -> Self {
        Self { msg: value }
    }
}

impl<'a> ResponseWriter<'a> {
    #[inline]
    pub fn version(&mut self, major: u8, minor: u8) -> &mut Self {
        self.msg
            .firstline
            .0
            .push_str(format!("HTTP/{}.{}", major, minor).as_str());
        self
    }

    #[inline]
    pub fn code(&mut self, code: u16, reason: &str) -> &mut Self {
        self.msg.firstline.1.push_str(code.to_string().as_str());
        self.msg.firstline.2.push_str(reason);
        self
    }

    #[inline]
    pub fn header(&mut self, k: &str, v: &str) -> &mut Self {
        self.msg.headers.append(k, v);
        self
    }

    #[inline]
    pub fn setheader(&mut self, k: &str, v: &str) -> &mut Self {
        self.msg.headers.set(k, v);
        self
    }

    #[inline]
    pub fn delheader(&mut self, k: &str) -> &mut Self {
        self.msg.headers.delete(k);
        self
    }

    #[inline]
    pub fn end(self) -> &'a mut Message {
        self.msg
    }
}
