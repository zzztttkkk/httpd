use std::collections::HashMap;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;

use crate::headers::Headers;
use crate::message::Message;
use crate::request::Request;

pub struct Response {
    pub(crate) msg: Message,

    pub(crate) _status_code: u32,
}

static mut _REASONS_MAP_INIT_LOCK: AtomicBool = AtomicBool::new(false);
static mut _REASONS_MAP_FREEZE_LOCK: AtomicBool = AtomicBool::new(false);
static mut _REASONS_MAP: Option<RwLock<HashMap<u32, String>>> = None;

fn all_std_status_codes() -> Vec<&'static str> {
    vec![
        "100 Continue",
        "101 Switching Protocols",
        "103 Early Hints",
        "200 OK",
        "201 Created",
        "202 Accepted",
        "203 Non-Authoritative Information",
        "204 No Content",
        "205 Reset Content",
        "206 Partial Content",
        "300 Multiple Choices",
        "301 Moved Permanently",
        "302 Found",
        "303 See Other",
        "304 Not Modified",
        "307 Temporary Redirect",
        "308 Permanent Redirect",
        "400 Bad Request",
        "401 Unauthorized",
        "402 Payment Required",
        "403 Forbidden",
        "404 Not Found",
        "405 Method Not Allowed",
        "406 Not Acceptable",
        "407 Proxy Authentication Required",
        "408 Request Timeout",
        "409 Conflict",
        "410 Gone",
        "411 Length Required",
        "412 Precondition Failed",
        "413 Payload Too Large",
        "414 URI Too Long",
        "415 Unsupported Media Type",
        "416 Range Not Satisfiable",
        "417 Expectation Failed",
        "418 I'm a teapot",
        "422 Unprocessable Entity",
        "425 Too Early",
        "426 Upgrade Required",
        "428 Precondition Required",
        "429 Too Many Requests",
        "431 Request Header Fields Too Large",
        "451 Unavailable For Legal Reasons",
        "500 Internal Server Error",
        "501 Not Implemented",
        "502 Bad Gateway",
        "503 Service Unavailable",
        "504 Gateway Timeout",
        "505 HTTP Version Not Supported",
        "506 Variant Also Negotiates",
        "507 Insufficient Storage",
        "508 Loop Detected",
        "510 Not Extended",
        "511 Network Authentication Required",
    ]
}

async unsafe fn get_reasons_map() -> &'static mut RwLock<HashMap<u32, String>> {
    while _REASONS_MAP.is_none() {
        let exchange_result = _REASONS_MAP_INIT_LOCK.compare_exchange(
            false,
            true,
            Ordering::SeqCst,
            Ordering::SeqCst,
        );
        if exchange_result == Ok(false) {
            let mut map: HashMap<u32, String> = HashMap::new();
            for item in all_std_status_codes() {
                if let Some(idx) = item.find(' ') {
                    map.insert(
                        item[..idx].parse::<u32>().unwrap(),
                        item[idx + 1..].to_string(),
                    );
                }
            }
            _REASONS_MAP = Some(RwLock::new(map));
        } else {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }
    _REASONS_MAP.as_mut().unwrap()
}

pub async fn register_status_code(code: u32, reason: &str) {
    unsafe {
        assert!(
            !(_REASONS_MAP_FREEZE_LOCK.load(Ordering::SeqCst)),
            "can not register after first read"
        );
    }
    let rw = unsafe { get_reasons_map() }.await;
    let mut map = rw.write().await;
    map.insert(code, reason.to_string());
}

async fn get_status_code_reason(code: u32) -> Option<&'static String> {
    unsafe {
        let _ = _REASONS_MAP_FREEZE_LOCK.compare_exchange(
            false,
            true,
            Ordering::SeqCst,
            Ordering::SeqCst,
        );
    }

    let rw = unsafe { get_reasons_map().await };
    let map = rw.read().await;
    match map.get(&code) {
        None => None,
        Some(v) => {
            let ptr: *const String = v;
            let nref: &'static String = unsafe { &*ptr };
            Some(nref)
        }
    }
}

impl Response {
    pub fn new() -> Self {
        Self {
            msg: Message::new(),
            _status_code: 0,
        }
    }

    pub fn default(req: &mut Request) -> Box<Self> {
        let mut resp = Box::new(Self::new());
        resp.msg._compress_type = req.headers().compress_type("accept-encoding");
        resp
    }

    #[inline(always)]
    pub fn statuscode(&mut self, code: u32) -> &mut Self {
        self._status_code = code;
        self
    }

    #[inline(always)]
    pub fn headers(&mut self) -> &mut Headers {
        &mut self.msg.headers
    }

    async fn tomsg(&mut self) {
        let _ = self.msg.flush();

        if self._status_code == 0 {
            self._status_code = 200;
        }

        self.msg.f1 = self._status_code.to_string();
        match get_status_code_reason(self._status_code).await {
            None => {
                self.msg.f2 = "Undefined".to_string();
            }
            Some(reason) => {
                self.msg.f2 = reason.to_string();
            }
        };
    }

    pub async fn to11<Writer: AsyncWriteExt + Unpin + Send>(
        &mut self,
        mut writer: Writer,
    ) -> std::io::Result<()> {
        self.tomsg().await;
        self.msg.to11(writer).await
    }
}

impl Write for Response {
    #[inline(always)]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.msg.write(buf)
    }

    #[inline(always)]
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use crate::compress::CompressType;
    use crate::response::{
        get_reasons_map, get_status_code_reason, register_status_code, Response,
    };

    #[test]
    fn resp_wf() {
        let mut resp = Response::new();
        resp.msg._compress_type = Some(CompressType::Gzip);

        let _ = resp.write("hello".repeat(10).as_bytes()).unwrap();
        resp.flush().unwrap();
    }

    #[test]
    fn status() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        runtime.block_on(async {
            register_status_code(200, "OK").await;
            register_status_code(404, "NOT FOUND").await;
            println!("{:?}", get_status_code_reason(200).await);
        });
    }
}
