use std::io::{ErrorKind, Write};

use async_trait::async_trait;

use crate::http::context::Context;
use crate::http::handler::Handler;

pub struct FsHandler {
    root: String,
    prefix: String,
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

        Box::new(Self { root, prefix })
    }

    async fn index(&self, fp: &str, metadata: &std::fs::Metadata, ctx: &mut Context) {
        println!("Index: {}", fp);
    }

    async fn file(&self, fp: &str, metadata: &std::fs::Metadata, ctx: &mut Context) {
        println!("File: {}", fp);

        match metadata.modified() {
            Ok(mt) => todo!(),
            _ => {}
        }
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
                    self.index(fs_path.as_str(), metadata, ctx).await;
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
