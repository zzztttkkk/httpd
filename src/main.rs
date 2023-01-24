#![allow(dead_code)]


use tokio::io::{AsyncWriteExt, BufStream};
use tokio::net::{TcpListener, TcpStream};

mod router;
mod request;
mod headers;
mod response;
mod handler;


async fn http11(stream: TcpStream) {
    let mut stream = Box::pin(BufStream::new(stream));
    let mut buf = String::with_capacity(4096);
    loop {
        match request::Request::from11(stream.as_mut(), &mut buf).await {
            Ok(req) => {
                println!("Request: {} {}", &req.method, &req.rawpath);
                let _ = stream.write("HTTP/1.0 200 OK\r\nContent-Length: 12\r\n\r\nHello World!".as_bytes()).await;
                let _ = stream.flush().await;
                break;
            }
            Err(v) => {
                let _ = stream.write(format!("HTTP/1.0 {} Bad Request\r\nContent-Length: 12\r\n\r\nHello World!", v).as_bytes()).await;
                let _ = stream.flush().await;
                if true {
                    return;
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    println!("httpd listening @ 127.0.0.1:8080");
    loop {
        match listener.accept().await {
            Err(_) => {
                continue;
            }
            Ok((stream, _)) => {
                tokio::spawn(async move {
                    http11(stream).await;
                });
            }
        }
    }
}
