pub struct Uri<'a> {
    _raw: &'a str,
    _scheme: &'a str,
    _username: &'a str,
    _password: &'a str,
    _host: &'a str,
    _port: &'a str,
    _path: &'a str,
    _raw_query: &'a str,
}

impl<'a> Uri<'a> {
    pub fn new(raw: &'a str) -> Self {
        Self {
            _raw: raw,
            _scheme: "",
            _username: "",
            _password: "",
            _host: "",
            _port: "",
            _path: "",
            _raw_query: "",
        }
    }

    pub fn parse(&mut self) {
        let mut tmp = self._raw;
        match tmp.find("://") {
            None => {}
            Some(idx) => {
                tmp = &(tmp[idx + 3..]);
                self._scheme = &(self._raw[0..idx]);
            }
        }

        match tmp.find('@') {
            None => {}
            Some(idx) => {
                tmp = &(tmp[idx + 1..]);
                let authinfo = &tmp[..idx];
                match authinfo.find(':') {
                    None => {}
                    Some(idx) => {
                        self._username = &(authinfo[..idx]);
                        self._password = &(authinfo[idx + 1..]);
                    }
                }
            }
        }

        let mut has_port = false;
        match tmp.find(':') {
            None => {}
            Some(idx) => {
                tmp = &(tmp[idx + 1..]);
                self._host = &(tmp[..idx]);
                has_port = true;
            }
        }

        match tmp.find('/') {
            None => {}
            Some(idx) => {
                tmp = &(tmp[idx + 1..]);
                self._path = &(tmp[..idx]);
                has_port = false;
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::uri::Uri;

    #[test]
    fn new() {
        let v = Uri::new("https://spk:123456@abc.com:5678/efg?f=45");
    }
}

