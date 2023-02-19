use core::fmt;
use std::fmt::{format, Formatter};

use once_cell::sync::Lazy;

use crate::utils::MultiValuesMap;

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
    _safe_path: Option<String>,
}

unsafe impl Send for ReadonlyUri {}

fn read_str_ptr(ptr: *const str) -> &'static str {
    if (ptr.is_null()) {
        return "";
    }
    unsafe { &*ptr }
}

impl fmt::Debug for ReadonlyUri {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if !self._parsed {
            return write!(f, "ReadonlyUri(unparsed){{raw: `{}`}}", unsafe {
                &*self._raw
            });
        }
        write!(
            f,
            r#"ReadonlyUri{{
	raw: `{}`, 
	scheme: `{}`,
	username: `{}`,
	password: `{}`,
	host: `{}`,
	port: `{}`,
	path: `{}`,
	raw_query: `{}`,
	fragment: `{}`
}}"#,
            read_str_ptr(self._raw),
            read_str_ptr(self._scheme),
            read_str_ptr(self._username),
            read_str_ptr(self._password),
            read_str_ptr(self._host),
            read_str_ptr(self._port),
            read_str_ptr(self._path),
            read_str_ptr(self._raw_query),
            read_str_ptr(self._fragment)
        )
    }
}

macro_rules! make_uri_getter {
    ($name:ident, $field:ident) => {
        pub fn $name(&mut self) -> &str {
            if !self._parsed {
                self._parse();
            }
            unsafe { &*self.$field }
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
            _safe_path: None,
        }
    }

    pub fn parse(raw: *const str) -> Self {
        let mut v = Self::new(raw);
        v._parse();
        v
    }

    make_uri_getter!(scheme, _scheme);
    make_uri_getter!(username, _username);
    make_uri_getter!(password, _password);
    make_uri_getter!(host, _host);
    make_uri_getter!(rawquery, _raw_query);
    make_uri_getter!(fragment, _fragment);

    pub fn path(&mut self) -> &String {
        if self._safe_path.is_none() {
            if !self._parsed {
                self._parse();
            }

            let raw_path = unsafe { &*self._path };
            let tmp = raw_path
                .replace('~', "")
                .replace('\\', "/")
                .replace("..", ".");
            let parts = tmp
                .split('/')
                .map(|v| v.trim())
                .filter(|&v| !v.is_empty() && v != ".");

            let mut result = String::new();

            for part in parts {
                result.push('/');
                result.push_str(part);
            }

            if result.is_empty() {
                result.push('/');
            }

            if raw_path.ends_with('/') && !result.ends_with('/') {
                result.push('/');
            }
            self._safe_path = Some(result);
        }

        self._safe_path.as_ref().unwrap()
    }

    pub fn port(&mut self) -> u32 {
        if !self._parsed {
            self._parse()
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

    fn _parse(&mut self) {
        if self._parsed {
            return;
        }
        self._parsed = true;

        let mut tmp = (unsafe { &*self._raw }).trim();

        if tmp.is_empty() {
            return;
        }

        match tmp.find("://") {
            None => {}
            Some(idx) => {
                self._scheme = (&(tmp[0..idx])).trim();
                tmp = &(tmp[idx + 3..]);
            }
        }

        if tmp.is_empty() {
            return;
        }

        match tmp.find('@') {
            None => {}
            Some(idx) => {
                let authinfo = &tmp[..idx];
                tmp = &(tmp[idx + 1..]);
                match authinfo.find(':') {
                    None => {}
                    Some(idx) => {
                        self._username = (&(authinfo[..idx])).trim();
                        self._password = (&(authinfo[idx + 1..])).trim();
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
                self._fragment = (&(tmp[idx + 1..])).trim();
                tmp = &(tmp[0..idx]);
            }
        }

        if tmp.is_empty() {
            return;
        }

        match tmp.rfind('?') {
            None => {}
            Some(idx) => {
                self._raw_query = (&(tmp[idx + 1..])).trim();
                tmp = &(tmp[0..idx]);
            }
        }

        if tmp.is_empty() {
            return;
        }

        match tmp.find('/') {
            None => {}
            Some(idx) => {
                self._path = (&(tmp[idx..])).trim();
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
                self._host = (&tmp[0..idx]).trim();
                self._port = (&tmp[idx + 1..]).trim();
            }
        }
    }
}

pub struct Uri {
    pub scheme: String,
    pub username: String,
    pub password: String,
    pub host: String,
    pub port: u32,
    pub path: String,
    pub query: Option<MultiValuesMap>,
}

static ENCODE_TABLE: Lazy<[bool; 128]> = Lazy::new(|| {
    let mut table: [bool; 128] = [false; 128];
    for c in b'a'..=b'z' {
        table[c as usize] = true;
    }
    for c in b'A'..=b'Z' {
        table[c as usize] = true;
    }
    for c in b'0'..=b'9' {
        table[c as usize] = true;
    }
    for c in "-_.!~*'();/?:@&=+$,#".as_bytes() {
        table[(*c) as usize] = true;
    }
    table
});

static ENCODE_COMPONENT_TABLE: Lazy<[bool; 128]> = Lazy::new(|| {
    let mut table: [bool; 128] = [false; 128];
    for c in b'a'..=b'z' {
        table[c as usize] = true;
    }
    for c in b'A'..=b'Z' {
        table[c as usize] = true;
    }
    for c in b'0'..=b'9' {
        table[c as usize] = true;
    }
    for c in "-_.!~*'()".as_bytes() {
        table[(*c) as usize] = true;
    }
    table
});

impl Uri {
    pub fn new() -> Self {
        Self {
            scheme: "".to_string(),
            username: "".to_string(),
            password: "".to_string(),
            host: "".to_string(),
            port: 0,
            path: "".to_string(),
            query: None,
        }
    }

    fn _encode_to(input: &[u8], output: &mut Vec<u8>, table: &'static Lazy<[bool; 128]>) {
        for c in input {
            let c = *c;
            if c < 128 && table[c as usize] {
                output.push(c);
                continue;
            }

            output.push(b'%');
            output.extend_from_slice(format!("{:X}", c).as_bytes());
        }
    }

    pub fn encode_component_to(input: &[u8], output: &mut Vec<u8>) {
        Self::_encode_to(input, output, &ENCODE_TABLE);
    }

    pub fn encode_to(input: &[u8], output: &mut Vec<u8>) {
        Self::_encode_to(input, output, &ENCODE_COMPONENT_TABLE);
    }

    pub fn encode(input: &[u8]) -> String {
        let mut txt = String::with_capacity((input.len() as f32 * 1.5) as usize);
        unsafe { Self::encode_to(input, txt.as_mut_vec()) };
        txt
    }

    pub fn encode_component(input: &[u8]) -> String {
        let mut txt = String::with_capacity((input.len() as f32 * 1.5) as usize);
        unsafe { Self::encode_component_to(input, txt.as_mut_vec()) };
        txt
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::{ReadonlyUri, Uri};

    #[test]
    fn parse() {
        // std
        let mut uri =
            ReadonlyUri::parse("http://ztk:123456@httpd.local:8080/static/index/?hello=world#main");

        assert_eq!(uri.scheme(), "http");
        assert_eq!(uri.username(), "ztk");
        assert_eq!(uri.password(), "123456");
        assert_eq!(uri.host(), "httpd.local");
        assert_eq!(uri.port(), 8080);
        assert_eq!(uri.path(), "/static/index/");
        assert_eq!(uri.rawquery(), "hello=world");
        assert_eq!(uri.fragment(), "main");

        println!("{:?}", ReadonlyUri::parse("ztk:123456@foo.com:80/"));

        println!("{:?}", ReadonlyUri::parse(""));

        println!("{:?}", ReadonlyUri::parse("/ABCD"));

        println!("{:?}", ReadonlyUri::parse("ABCD"));
    }

    #[test]
    fn safe_path() {
        let mut v = ReadonlyUri::new("");
        v._path = "~/../.\\\\./../././.a.r/key.txt/";
        println!("{}", v.path());
    }

    #[test]
    fn uri_encode() {
        println!("{}", Uri::encode("/我好".as_bytes()))
    }
}
