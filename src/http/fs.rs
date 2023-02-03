// Go SDK 1.19.4 src/net/http/fs/go

use std::future::Future;
use std::io::{ErrorKind, Write};
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use tokio::fs::ReadDir;
use tokio::io::AsyncReadExt;

use crate::http::context::Context;
use crate::http::handler::Handler;
use crate::utils::{self, Time, UtcTime};

use super::headers::Headers;
use super::request::Request;

pub type FsIndexRenderFuncType =
    Box<dyn (Fn(&mut Context, &Vec<tokio::fs::DirEntry>) -> String) + Send + Sync>;

pub struct FsHandler {
    root: String,
    prefix: String,
    disable_index: bool,
    index_render: Option<FsIndexRenderFuncType>,
}

#[derive(Clone, Copy, PartialEq)]
enum CheckCond {
    None,
    False,
    True,
}

impl FsHandler {
    pub fn new(root: &str, prefix: &str) -> Box<Self> {
        let mut root = root.to_string();
        if !root.ends_with("/") {
            root.push('/')
        }

        let metadata = std::fs::metadata(root.as_str()).unwrap();
        if !metadata.is_dir() {
            panic!("`{}` is not a dir", root)
        }

        let mut prefix = prefix.to_string();

        if !prefix.starts_with("/") {
            let mut tmp = "/".to_string();
            tmp.push_str(prefix.as_str());
            prefix = tmp;
        }

        if !prefix.ends_with("/") {
            prefix.push('/');
        }

        Box::new(Self {
            root,
            prefix,
            disable_index: false,
            index_render: None,
        })
    }

    async fn index(&self, fp: &str, metadata: &std::fs::Metadata, ctx: &mut Context) {
        let index_path = format!("{}/index.html", fp);
        if let Ok(index_metadata) = (tokio::fs::metadata(&index_path).await) {
            self.file(&index_path, &index_metadata, ctx).await;
            return;
        }

        let mut current_dir_name = "";
        if let Some(idx) = fp.rfind('/') {
            current_dir_name = &fp[idx + 1..];
        }

        match tokio::fs::read_dir(fp).await {
            Err(_) => {
                ctx.response().statuscode(404);
                return;
            }
            Ok(ref mut iter) => {
                let mut ents = Vec::with_capacity(8);
                loop {
                    let item = iter.next_entry().await;
                    match item {
                        Err(_) => {
                            break;
                        }
                        Ok(opt) => match opt {
                            Some(ent) => {
                                ents.push(ent);
                            }
                            None => {
                                break;
                            }
                        },
                    }
                }

                match &self.index_render {
                    Some(rfn) => {
                        rfn(ctx, &ents);
                    }
                    None => {
                        let mut resp = ctx.response();
                        resp.headers().set_content_type("text/html");
                        for ele in &ents {
                            if let Some(filename) = ele.file_name().to_str() {
                                if let Ok(ref metadata) = (ele.metadata().await) {
                                    if metadata.is_file() || metadata.is_dir() {
                                        _ = resp.write(
                                            format!(
                                                "<li><a href=\"./{}/{}\">{}</a></li>",
                                                current_dir_name, filename, filename,
                                            )
                                            .as_bytes(),
                                        );
                                        continue;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    async fn _whole_file(&self, fp: &str, metadata: &std::fs::Metadata, ctx: &mut Context) {
        match tokio::fs::File::open(fp).await {
            Err(_) => {
                ctx.response().statuscode(404);
                return;
            }
            Ok(mut f) => {
                if metadata.len() <= 200 * 1024 {
                    let mut buf = Vec::<u8>::with_capacity(200 * 1024);
                    buf.resize(buf.capacity(), 0);
                    let ptr = unsafe { buf.as_mut_slice() };
                    match f.read(ptr).await {
                        Ok(read_size) => {
                            if read_size as u64 != metadata.len() {
                                ctx.response().statuscode(404);
                                return;
                            }
                            ctx.response().write(&ptr[0..read_size]);
                        }
                        Err(_) => {
                            ctx.response().statuscode(404);
                        }
                    }
                } else {
                    ctx.response().msg._output_readobj = Some(Box::new(f));
                }
            }
        }
    }

    async fn _part_file(
        &self,
        fp: &str,
        metadata: &std::fs::Metadata,
        ctx: &mut Context,
        begin: usize,
        end: usize,
    ) {
    }

    fn scan_etag<'a, 'b: 'a>(mut s: &'b str) -> (&'a str, &'a str) {
        s = s.trim();
        let mut start: usize = 0;
        if s.starts_with("W/") {
            start = 2;
        }
        if s.len() <= 2 || s.as_bytes()[start] != '"' as u8 {
            return ("", "");
        }
        s = &s[start + 1..];
        match s.find('"') {
            Some(idx) => {
                return (&s[..idx], &s[idx + 1..]);
            }
            None => {
                return ("", "");
            }
        }
    }

    fn etag_strong_match(etag: &str, etag_header_val: &str) -> bool {
        return etag == etag_header_val && etag.as_bytes()[0] == '"' as u8;
    }

    fn etag_weak_match(etag: &str, etag_header_val: &str) -> bool {
        return etag.trim_start_matches("W/") == etag_header_val.trim_start_matches("W/");
    }

    fn check_if_match(ctx: &mut Context) -> CheckCond {
        match ctx.request().headers().get("if-match") {
            None => {
                return CheckCond::None;
            }
            Some(val) => {
                let mut req = ctx.request();
                let etag_header_val = req.headers().get("etag");
                if etag_header_val.is_none() {
                    return CheckCond::False;
                }
                let etag_header_val = etag_header_val.unwrap().as_str();

                let mut val = val.as_str();
                loop {
                    val = val.trim();
                    if val.is_empty() {
                        return CheckCond::None;
                    }

                    let fb = val.as_bytes()[0];
                    if fb == ',' as u8 {
                        val = &val[1..];
                        continue;
                    }
                    if fb == '*' as u8 {
                        return CheckCond::True;
                    }

                    let (etag, remain) = Self::scan_etag(val);
                    if etag.is_empty() {
                        break;
                    }

                    if Self::etag_strong_match(etag, etag_header_val) {
                        return CheckCond::True;
                    }
                    val = remain;
                }
            }
        }

        return CheckCond::False;
    }

    fn check_if_unmodified_since(ctx: &mut Context, modified_time: Option<UtcTime>) -> CheckCond {
        if modified_time.is_none() {
            return CheckCond::None;
        }
        let modified_time = modified_time.unwrap();

        match ctx.request().headers().get("if-unmodified-since") {
            None => {
                return CheckCond::None;
            }
            Some(val) => match utils::Time::parse_from_header_value(val) {
                None => {
                    return CheckCond::None;
                }
                Some(iust) => {
                    if modified_time <= iust {
                        return CheckCond::True;
                    }
                    return CheckCond::False;
                }
            },
        }
    }

    fn check_if_none_match(ctx: &mut Context) -> CheckCond {
        match ctx.request().headers().get("if-none-match") {
            Some(inm) => {
                let mut req = ctx.request();
                let etag_header_val = req.headers().get("etag");
                if etag_header_val.is_none() {
                    return CheckCond::False;
                }
                let etag_header_val = etag_header_val.unwrap().as_str();

                let mut tmp = inm.as_str();
                loop {
                    tmp = tmp.trim();
                    if tmp.is_empty() {
                        break;
                    }
                    let fb = tmp.as_bytes()[0];
                    if fb == ',' as u8 {
                        tmp = &tmp[1..];
                        continue;
                    }
                    if fb == '*' as u8 {
                        return CheckCond::False;
                    }

                    let (etag, remain) = Self::scan_etag(tmp);
                    if etag.is_empty() {
                        return CheckCond::False;
                    }
                    if Self::etag_weak_match(etag, etag_header_val) {
                        return CheckCond::False;
                    }
                    tmp = remain;
                }
                return CheckCond::True;
            }
            None => {
                return CheckCond::None;
            }
        }
    }

    fn write_not_modified(ctx: &mut Context) {
        let mut resp = ctx.response();
        resp.headers().remove("content-type");
        resp.headers().remove("content-length");
        resp.headers().remove("content-encoding");
        if let Some(v) = resp.headers().get("etag") {
            if v.is_empty() {
                resp.headers().remove("last-modified");
            }
        }
        resp.statuscode(304);
    }

    fn check_if_modified_since(ctx: &mut Context, modified_time: Option<UtcTime>) -> CheckCond {
        if modified_time.is_none() {
            return CheckCond::None;
        }
        let modified_time = modified_time.unwrap();

        let mut req = ctx.request();
        if req.method() != "GET" && req.method() != "HEAD" {
            return CheckCond::None;
        }

        match req.headers().get("if-modified-since") {
            None => {
                return CheckCond::None;
            }
            Some(ims) => match Time::parse_from_header_value(ims) {
                None => {
                    return CheckCond::None;
                }
                Some(t) => {
                    if modified_time <= t {
                        return CheckCond::False;
                    }
                    return CheckCond::True;
                }
            },
        }
    }

    fn check_if_range(ctx: &mut Context, modified_time: Option<UtcTime>) -> CheckCond {
        let mut req = ctx.request();
        let req_ptr: usize = unsafe { std::mem::transmute(req.ptr()) };

        if req.method() != "GET" && req.method() != "HEAD" {
            return CheckCond::False;
        }

        match req.headers().get("if-range") {
            None => {
                return CheckCond::None;
            }
            Some(ir) => {
                let (etag, _) = Self::scan_etag(ir);
                if !etag.is_empty() {
                    let tmp_req_ref: &mut Request = unsafe { std::mem::transmute(req_ptr) };
                    let etag_header_val = tmp_req_ref.headers().get("etag");
                    if etag_header_val.is_none() {
                        return CheckCond::False;
                    }
                    let etag_header_val = etag_header_val.unwrap().as_str();
                    if Self::etag_strong_match(etag, etag_header_val) {
                        return CheckCond::True;
                    }
                    return CheckCond::False;
                }

                if modified_time.is_none() {
                    return CheckCond::False;
                }
                let modified_time = modified_time.unwrap();

                match Time::parse_from_header_value(ir) {
                    None => {
                        return CheckCond::False;
                    }
                    Some(t) => {
                        if modified_time <= t {
                            return CheckCond::True;
                        }
                        return CheckCond::False;
                    }
                }
            }
        }

        CheckCond::False
    }

    fn check_preconditions<'a, 'b: 'a>(
        ctx: &'b mut Context,
        headers: &'b Headers,
        modified_time: Option<UtcTime>,
    ) -> (bool, &'a str) {
        let mut ch = Self::check_if_match(ctx);
        if ch == CheckCond::None {
            ch = Self::check_if_unmodified_since(ctx, modified_time);
        }
        if ch == CheckCond::False {
            ctx.response().statuscode(412);
            return (true, "");
        }

        match Self::check_if_none_match(ctx) {
            CheckCond::False => {
                let req = ctx.request();
                if req.method() == "GET" || req.method() == "HEAD" {
                    Self::write_not_modified(ctx);
                    return (true, "");
                }
                ctx.response().statuscode(412);
                return (true, "");
            }
            CheckCond::None => {
                if Self::check_if_modified_since(ctx, modified_time) == CheckCond::False {
                    Self::write_not_modified(ctx);
                    return (true, "");
                }
            }
            _ => {}
        }

        match headers.get("range") {
            None => {
                return (false, "");
            }
            Some(rh) => {
                if Self::check_if_range(ctx, modified_time) == CheckCond::False {
                    return (false, "");
                }
                return (false, rh);
            }
        }
    }

    async fn file(&self, fp: &str, metadata: &std::fs::Metadata, ctx: &mut Context) {
        let mut modified_time: Option<UtcTime> = None;
        if let Ok(mt) = metadata.modified() {
            if !is_zero_time(&mt) {
                ctx.response()
                    .headers()
                    .set("last-modified", &to_time_string(&mt));
                modified_time = Some(Time::utc_from(&mt));
            }
        }

        let mut req = ctx.request();
        Self::check_preconditions(ctx, req.headers(), modified_time);

        self._whole_file(fp, metadata, ctx).await;
    }
}

#[async_trait]
impl Handler for FsHandler {
    async fn handle(&self, ctx: &mut Context) {
        let mut req = ctx.request();
        let rpath = req.uri().path();
        if !rpath.starts_with(self.prefix.as_str()) {
            ctx.response().statuscode(404);
            return;
        }

        let remain_path = &rpath[self.prefix.len()..];
        let fs_path = format!("{}{}", &self.root, remain_path);

        match tokio::fs::metadata(fs_path.as_str()).await {
            Err(_) => {
                ctx.response().statuscode(404);
                return;
            }
            Ok(ref metadata) => {
                if metadata.is_dir() {
                    if self.disable_index {
                        ctx.response().statuscode(404);
                    } else {
                        self.index(fs_path.as_str(), metadata, ctx).await;
                    }
                    return;
                }
                if metadata.is_file() {
                    self.file(fs_path.as_str(), metadata, ctx).await;
                    return;
                }
                ctx.response().statuscode(404);
            }
        }
    }
}

#[inline]
fn is_zero_time(st: &SystemTime) -> bool {
    return match st.duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_secs() == 0,
        Err(_) => true,
    };
}

fn to_time_string(st: &SystemTime) -> String {
    let ut = utils::Time::utc_from(st);
    return ut.format("%a, %d %b %Y %H:%M:%S GMT").to_string();
}
