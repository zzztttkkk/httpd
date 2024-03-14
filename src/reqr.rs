use crate::{
    internal::multi_map::MultiMap,
    message::{Message, MessageBody},
};

pub(crate) struct RequestReader<'a> {
    msg: &'a Message,
}

impl<'a> From<&'a Message> for RequestReader<'a> {
    fn from(value: &'a Message) -> Self {
        Self { msg: value }
    }
}

impl<'a> RequestReader<'a> {
    #[inline]
    pub fn method(&self) -> &String {
        &self.msg.firstline.0
    }

    #[inline]
    pub fn rawuri(&self) -> &String {
        &self.msg.firstline.1
    }

    pub fn version(&self) -> Result<(u8, u8), ()> {
        match (&self.msg.firstline.2).find("/") {
            None => Err(()),
            Some(idx) => {
                if !(&(self.msg.firstline.2.as_str()[..idx]).eq_ignore_ascii_case("http")) {
                    return Err(());
                }

                let versions = &(self.msg.firstline.2.as_str()[idx + 1..]);
                match versions.find(".") {
                    None => {
                        return Err(());
                    }
                    Some(idx) => {
                        let major;
                        match versions[..idx].parse::<u8>() {
                            Err(_) => {
                                return Err(());
                            }
                            Ok(num) => {
                                major = num;
                            }
                        }

                        let minor;
                        match versions[..idx].parse::<u8>() {
                            Err(_) => {
                                return Err(());
                            }
                            Ok(num) => {
                                minor = num;
                            }
                        }
                        return Ok((major, minor));
                    }
                }
            }
        }
    }

    pub fn headers(&self) -> &MultiMap {
        &self.msg.headers
    }

    pub fn body(&self) -> &MessageBody {
        &self.msg.body
    }
}
