use core::fmt;
use std::fmt::{Formatter, write};
use std::num::ParseIntError;

/// UNSAFE!!!!!!!!!!!!!!!!!
pub struct ReadonlyUri {
    _raw: *const str,

    _scheme: *const str,
    _username: *const str,
    _password: *const str,
    _host: *const str,
    _port: *const str,
    _path: *const str,
    _raw_query: *const str,
    _fragment: *const str,

    _parsed: bool,
    _query_parsed: bool,
    _port_num: i32,
}

unsafe impl Send for ReadonlyUri {}

impl fmt::Debug for ReadonlyUri {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if !self._parsed {
            return write!(f, "Uri(unparsed){{raw: `{}`}}", unsafe { &*self._raw });
        }
        write!(f, "Uri{{raw: `{}`, path: `{}`}}", unsafe { &*self._raw }, unsafe { &*self._path })
    }
}

macro_rules! make_uri_getter {
    ($name:ident, $field:ident) => {
        pub fn $name(&mut self) -> &str {
            if !self._parsed {
                self.parse();
            }
            unsafe{ &*self.$field }
        }
    };
}

impl ReadonlyUri {
    pub fn new(raw: *const str) -> Self {
        Self {
            _raw: raw,

            _scheme: "",
            _username: "",
            _password: "",
            _host: "",
            _port: "",
            _path: "",
            _raw_query: "",
            _fragment: "",
            _port_num: -1,

            _parsed: false,
            _query_parsed: false,
        }
    }

    make_uri_getter!(scheme, _scheme);
    make_uri_getter!(username, _username);
    make_uri_getter!(password, _password);
    make_uri_getter!(host, _host);
    make_uri_getter!(path, _path);
    make_uri_getter!(rawquery, _raw_query);
    make_uri_getter!(fragment, _fragment);

    pub fn port(&mut self) -> u32 {
        if !self._parsed {
            self.parse()
        }

        if self._port_num < 0 {
            let v = unsafe { (&*self._port) };
            if v.is_empty() {
                self._port_num = 0;
            } else {
                match v.parse::<u32>() {
                    Ok(n) => {
                        self._port_num = n as i32;
                    }
                    Err(_) => {
                        self._port_num = 0;
                    }
                }
            }
        }

        self._port_num as u32
    }

    pub fn parse(&mut self) {
        if self._parsed {
            return;
        }
        self._parsed = true;

        let mut tmp = unsafe { &*self._raw };

        if tmp.is_empty() {
            return;
        }

        match tmp.find("://") {
            None => {}
            Some(idx) => {
                tmp = &(tmp[idx + 3..]);
                self._scheme = &(tmp[0..idx]);
            }
        }

        if tmp.is_empty() {
            return;
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

        if tmp.is_empty() {
            return;
        }

        match tmp.rfind('#') {
            None => {}
            Some(idx) => {
                self._fragment = &(tmp[idx..]);
                tmp = &(tmp[0..idx]);
            }
        }

        if tmp.is_empty() {
            return;
        }

        match tmp.rfind('?') {
            None => {}
            Some(idx) => {
                self._raw_query = &(tmp[idx..]);
                tmp = &(tmp[0..idx]);
            }
        }

        if tmp.is_empty() {
            return;
        }

        match tmp.find('/') {
            None => {}
            Some(idx) => {
                self._path = &(tmp[idx..]);
                tmp = &(tmp[0..idx])
            }
        }

        if tmp.is_empty() {
            return;
        }

        match tmp.find(':') {
            None => {
                self._host = tmp;
            }
            Some(idx) => {
                self._host = &tmp[0..idx];
                self._port = &tmp[idx + 1..];
            }
        }
    }
}


pub struct Uri {
    scheme: String,
    username: String,
    password: String,
    host: String,
    port: u32,
    path: String,
}
