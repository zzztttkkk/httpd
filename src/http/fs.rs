use std::future::Future;
use std::io::{ErrorKind, Write};

use async_trait::async_trait;
use tokio::fs::ReadDir;

use crate::http::context::Context;
use crate::http::handler::Handler;

pub struct FsHandler {
    root: String,
    prefix: String,
    disable_index: bool,
    index_render: Option<Box<dyn (Fn(&Vec<tokio::fs::DirEntry>) -> String) + Send + Sync>>,
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
        println!("Index: {}", fp);

        let index_path = format!("{}/index.html", fp);
        if let Ok(index_metadata) = (tokio::fs::metadata(&index_path).await) {
            self.file(&index_path, &index_metadata, ctx).await;
            return;
        }

        match tokio::fs::read_dir(fp).await {
            Err(_) => {
                ctx.response().statuscode(404);
                return;
            }
            Ok(ref mut iter) => {
                let mut ents = Vec::with_capacity(10);
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
                        rfn(&ents);
                    }
                    None => {
                        let mut resp = ctx.response();
                        resp.headers().set_content_type("text/html");

                        _ = resp.write("<html><document><body><ol>".as_bytes());
                        for ele in &ents {
                            if let Some(filename) = ele.file_name().to_str() {
                                if let Ok(metadate) = (ele.metadata().await) {
                                    if metadata.is_file() {
                                        _ = resp.write(
                                            format!("<li>./{} {}</li>", filename, metadata.len())
                                                .as_bytes(),
                                        );
                                        continue;
                                    }
                                    if metadata.is_dir() {
                                        _ = resp
                                            .write(format!("<li>./{}/</li>", filename).as_bytes());
                                        continue;
                                    }
                                }
                            }
                        }
                        _ = resp.write("</ol></body></document></html>".as_bytes());
                    }
                }
            }
        }
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
