use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{Duration, SystemTime};

use tokio::io::{AsyncWriteExt, BufStream};
use tokio::net::{TcpListener, TcpStream};

mod router;
mod request;
mod headers;
mod response;
mod handler;

struct AliveCounter {
    counter: Arc<AtomicI64>,
}

impl AliveCounter {
    fn new(counter: Arc<AtomicI64>) -> Self {
        counter.fetch_add(1, Ordering::Relaxed);
        Self {
            counter
        }
    }
}

impl Drop for AliveCounter {
    fn drop(&mut self) {
        self.counter.fetch_sub(1, Ordering::Relaxed);
    }
}


async fn http11(stream: TcpStream, counter: Arc<AtomicI64>) {
    let mut stream = Box::pin(BufStream::new(stream));
    let mut buf = String::with_capacity(4096);
    loop {
        match request::Request::from11(stream.as_mut(), &mut buf).await {
            Ok(req) => {
                let _ = AliveCounter::new(counter.clone());

                println!("[{}] Request: {} {}", chrono::Local::now(), &req.method, &req.rawpath);
                let _ = stream.write("HTTP/1.0 200 OK\r\nContent-Length: 12\r\n\r\nHello World!".as_bytes()).await;
                let _ = stream.flush().await;
            }
            Err(v) => {
                if v == 0 {
                    return;
                }
                let _ = stream.write(format!("HTTP/1.0 {} Bad Request\r\nContent-Length: 12\r\n\r\nHello World!", v).as_bytes()).await;
                let _ = stream.flush().await;
                break;
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    println!("[{}] httpd listening @ 127.0.0.1:8080, Pid: {}", chrono::Local::now(), std::process::id());
    let alive_counter: Arc<AtomicI64> = Arc::new(AtomicI64::new(0));

    loop {
        tokio::select! {
            ar = listener.accept() => {
                match ar {
                    Err(_) => {
                        continue;
                    }
                    Ok((stream, _)) => {
                        let counter = alive_counter.clone();
                        tokio::spawn(async move {
                            http11(stream, counter).await;
                        });
                    }
                }
            }
            _ = tokio::signal::ctrl_c() => {
                println!("[{}] httpd is preparing to shutdown", chrono::Local::now());
                loop {
                    if alive_counter.load(Ordering::Relaxed) < 1 {
                        break;
                    }
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
                println!("[{}] httpd is gracefully shutdown", chrono::Local::now());
                return Ok(());
            }
        }
    }
}
